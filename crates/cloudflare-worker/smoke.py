import gzip
from contextlib import suppress
import os
from pathlib import Path
import re
import signal
import socket
import subprocess
import tempfile
import threading
import time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

root = Path(__file__).resolve().parents[2]
payload = b'<wasm-probe>worker poison & marker</wasm-probe>' * 2048
requests = []

class Upstream(BaseHTTPRequestHandler):
    def do_GET(self):
        requests.append(self.headers.get('User-Agent'))
        body = gzip.compress(payload)
        self.send_response(200)
        self.send_header('Content-Encoding', 'gzip')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass

upstream = ThreadingHTTPServer(('127.0.0.1', 0), Upstream)
threading.Thread(target=upstream.serve_forever, daemon=True).start()
with socket.socket() as sock:
    sock.bind(('127.0.0.1', 0))
    port = sock.getsockname()[1]
env = dict(os.environ, WRANGLER_SEND_METRICS='false')
with tempfile.TemporaryFile(mode='w+') as log:
    worker = subprocess.Popen([
        'npm', 'exec', '--yes', '--package', 'wrangler@4.149.0', '--', 'wrangler',
        'dev', '--local', '--port', str(port), '--var',
        f'POISON_SOURCE:http://127.0.0.1:{upstream.server_port}',
    ], cwd=root / 'crates/cloudflare-worker', env=env, stdout=log, stderr=log, start_new_session=True)
    try:
        deadline = time.monotonic() + 120
        request = Request(f'http://127.0.0.1:{port}/', headers={'Accept-Encoding': 'gzip'})
        while True:
            try:
                with urlopen(request, timeout=20) as response:
                    assert response.status == 200
                    assert response.headers.get_content_type() == 'text/html'
                    assert response.headers['Content-Encoding'] == 'gzip'
                    body = gzip.decompress(response.read()).decode()
                break
            except URLError as error:
                if isinstance(error, HTTPError) or worker.poll() is not None or time.monotonic() >= deadline:
                    raise
                time.sleep(0.25)
        assert '&lt;wasm-probe&gt;worker poison &amp; marker&lt;/wasm-probe&gt;' in body
        assert body.count('&lt;wasm-probe&gt;') == 2048
        assert re.search(r'href="/[0-9a-f-]{36}"', body)
        assert requests and all(ua.startswith('miasma/') for ua in requests)
        print('Worker smoke passed')
    except Exception:
        log.seek(0)
        print(log.read())
        raise
    finally:
        try:
            with suppress(ProcessLookupError):
                os.killpg(worker.pid, signal.SIGTERM)
            worker.wait(timeout=15)
        finally:
            upstream.shutdown()
            upstream.server_close()

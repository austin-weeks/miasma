use async_stream::try_stream;
use futures::StreamExt;
use std::pin::pin;

use crate::MiasmaStream;

/// Escape HTML sequences in the given stream.
pub fn escape_html_stream(html_stream: impl MiasmaStream) -> impl MiasmaStream {
    try_stream! {
        let mut html_stream = pin!(html_stream);
        while let Some(chunk_res) = html_stream.next().await {
            let Ok(mut chunk) = chunk_res else {
                yield chunk_res?;
                continue;
            };
            loop {
                let Some((esc_at_index, escape_seq)) = chunk
                    .iter()
                    .enumerate()
                    .find_map(|(i, b)| match *b {
                        b'<' => Some((i, &b"&lt;"[..])),
                        b'>' => Some((i, b"&gt;")),
                        b'&' => Some((i, b"&amp;")),
                        _ => None,
                    })
                else {
                    yield chunk;
                    break;
                };

                let remaining = chunk.split_off(esc_at_index + 1);
                chunk.truncate(esc_at_index);
                yield chunk;
                yield escape_seq.into();
                chunk = remaining;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils;

    use super::*;
    use bytes::Bytes;

    fn as_stream(s: &'static str) -> impl MiasmaStream {
        try_stream! { yield Bytes::from_static(s.as_bytes()) }
    }

    #[tokio::test]
    async fn targeted_chars_escaped() {
        let test_cases = [
            ("<", "&lt;"),
            (">", "&gt;"),
            ("&", "&amp;"),
            ("<&>", "&lt;&amp;&gt;"),
        ];

        for (input, expected) in test_cases {
            let sanitized =
                test_utils::drain_byte_stream(escape_html_stream(as_stream(input))).await;
            assert_eq!(sanitized, expected);
        }
    }

    #[tokio::test]
    async fn script_tag_is_escaped() {
        let input = "<script>console.log('foo');</script>";
        let sanitized = test_utils::drain_byte_stream(escape_html_stream(as_stream(input))).await;
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("</script>"));
    }

    #[tokio::test]
    async fn content_is_preserved() {
        let input = "<p>The quick brown fox jumps over the lazy dog. & foo.</p>";
        let expected = "&lt;p&gt;The quick brown fox jumps over the lazy dog. &amp; foo.&lt;/p&gt;";
        let sanitized = test_utils::drain_byte_stream(escape_html_stream(as_stream(input))).await;
        assert_eq!(sanitized, expected);
    }
}

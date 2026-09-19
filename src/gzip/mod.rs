mod gzip_stream;

pub use gzip_stream::LowMemGzipStream;
use zlib_rs::DeflateConfig;

const GZIP_OFFSET: i32 = 16;
const DEFLATE_CONFIG: DeflateConfig = DeflateConfig {
    // max cpu effort - best but slowest (miasma is io bound though so this is fine)
    level: 9,

    // Uses about 10kb per stream
    // This is WAY less that the async-compression compression default (15)
    // I found this to be a bit better than 10 ironically enough
    window_bits: 11 + GZIP_OFFSET, // must add 16 to force deflate to encode as gzipped

    // Memory allocated for gzip's internal bookeeping.
    // async-compression defaults to 8
    mem_level: 3,

    // Defaults for these
    method: zlib_rs::Method::Deflated,
    strategy: zlib_rs::Strategy::Default,
};

mod gzip_stream;
mod settings;

pub use gzip_stream::LowMemGzipStream;
#[expect(unused_imports)]
pub use settings::{Effort, GzipSettings, MemLevel, WindowSize};

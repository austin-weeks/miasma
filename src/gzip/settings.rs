use zlib_rs::{DeflateConfig, Method, Strategy};

/// Controls compression settings for [`crate::gzip::LowMemGzipStream`].
pub struct GzipSettings {
    pub effort: Effort,
    pub mem_level: MemLevel,
    pub window_size: WindowSize,
}

const GZIP_OFFSET: i32 = 16;

#[expect(unused)]
/// Level for gzip's algorithm. Higher produces more compressed output but requires more
/// computations.
pub enum Effort {
    /// `level` = 9 - best, slowest.
    Best,
    /// `level` = 7.
    Moderate,
    /// `level` = 1 - worst, fastest.
    Minimal,
}

#[expect(unused)]
/// Memory allocated for gzip's internal bookeeping.
pub enum MemLevel {
    /// `mem_level` = 8 (default).
    High,
    /// `mem_level` = 3.
    Low,
    /// `mem_level` = 2.
    UltraLow,
}

#[expect(unused)]
/// Size of gzip's sliding window.
pub enum WindowSize {
    /// `window_bits` = 15.
    Large,
    /// `window_bits` = 11.
    Small,
    /// `window_bits` = 10.
    Tiny,
}

impl GzipSettings {
    /// Convert to [`zlib_rs::DeflateConfig`].
    pub const fn into_deflate_config(self) -> DeflateConfig {
        DeflateConfig {
            level: match self.effort {
                Effort::Best => 9,     // deflate's max - best but slowest
                Effort::Moderate => 7, // happy medium ?
                Effort::Minimal => 1,  // fastest but worst
            },
            window_bits: match self.window_size {
                WindowSize::Large => 15, // default for flate2/async-compression
                WindowSize::Small => 11, // ~10KB per stream
                WindowSize::Tiny => 10,  // ~6KB per stream
            } + GZIP_OFFSET, // must add 16 to force deflate to encode as gzipped

            mem_level: match self.mem_level {
                MemLevel::High => 8,     // default for flate2/async-compression
                MemLevel::Low => 3,      // happy medium ?
                MemLevel::UltraLow => 2, // nice and tiny
            },

            method: Method::Deflated, // literally the only option...
            strategy: Strategy::Default,
        }
    }
}

impl Default for GzipSettings {
    // We may change this later, but for now we'll just leave at these settings (9/11/3)
    fn default() -> Self {
        Self {
            effort: Effort::Best,
            window_size: WindowSize::Small,
            mem_level: MemLevel::Low,
        }
    }
}

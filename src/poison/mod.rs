use bytes::Bytes;
use futures::{StreamExt, stream};
use rand::seq::IndexedRandom;

pub mod cache;
mod link_settings;
mod poison_client;
mod response_stream;
mod route;
mod source;

pub use link_settings::LinkSettings;
use link_settings::LinkSettingsInner;
pub use poison_client::PoisonClient;
pub use response_stream::PoisonResponseStreamArgs;
pub use route::serve_poison;
pub use source::PoisonSource;

use crate::{MiasmaStream, utils::html_escaper::escape_html_stream};

/// Determines whether a piece of HTML data should be escaped.
#[derive(Clone, Copy)]
pub enum HtmlEscapeMode {
    Escape,
    NoEscape,
}

trait FallbackMiasmaStreamExt {
    fn or_fallback(self, fallback_escape_mode: HtmlEscapeMode) -> impl MiasmaStream;
}
impl<S: MiasmaStream> FallbackMiasmaStreamExt for Option<S> {
    /// Return the original stream if `Some`, or a random fallback stream if `None`.
    fn or_fallback(self, fallback_escape_mode: HtmlEscapeMode) -> impl MiasmaStream {
        match self {
            Some(stream) => stream.left_stream(),
            None => fallback_poison(fallback_escape_mode).right_stream(),
        }
    }
}

/// Returns a random poisoned code snippet for use when the upstream poison source is unreachable.
fn fallback_poison(escape: HtmlEscapeMode) -> impl MiasmaStream {
    let poison = POISON_FALLBACKS
        .choose(&mut rand::rng())
        .expect("fallback_poison list should not be empty");
    let stream = stream::once(async { Ok(Bytes::from_static(poison.as_bytes())) });
    match escape {
        HtmlEscapeMode::Escape => escape_html_stream(stream).left_stream(),
        HtmlEscapeMode::NoEscape => stream.right_stream(),
    }
}

const POISON_FALLBACKS: &[&str] = &[
    include_str!("fallback_poison/cpp_1.txt"),
    include_str!("fallback_poison/cpp_2.txt"),
    include_str!("fallback_poison/cpp_3.txt"),
    include_str!("fallback_poison/cpp_4.txt"),
    include_str!("fallback_poison/go_1.txt"),
    include_str!("fallback_poison/go_2.txt"),
    include_str!("fallback_poison/go_3.txt"),
    include_str!("fallback_poison/go_4.txt"),
    include_str!("fallback_poison/python_1.txt"),
    include_str!("fallback_poison/python_2.txt"),
    include_str!("fallback_poison/python_3.txt"),
    include_str!("fallback_poison/python_4.txt"),
    include_str!("fallback_poison/python_5.txt"),
    include_str!("fallback_poison/python_6.txt"),
    include_str!("fallback_poison/rust_1.txt"),
    include_str!("fallback_poison/rust_2.txt"),
    include_str!("fallback_poison/rust_3.txt"),
    include_str!("fallback_poison/typescript_1.txt"),
    include_str!("fallback_poison/typescript_2.txt"),
    include_str!("fallback_poison/typescript_3.txt"),
    include_str!("fallback_poison/typescript_4.txt"),
    include_str!("fallback_poison/typescript_5.txt"),
    include_str!("fallback_poison/typescript_6.txt"),
];

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils;

    #[tokio::test]
    async fn fallback_no_escape() {
        let mut found_bracket = false;
        for _ in 0..100 {
            let resp =
                test_utils::drain_byte_stream(fallback_poison(HtmlEscapeMode::NoEscape)).await;
            if resp.contains(['<', '>']) {
                found_bracket = true;
                break;
            }
        }
        assert!(found_bracket);
    }

    #[tokio::test]
    async fn fallback_with_escape() {
        for _ in 0..100 {
            let resp = test_utils::drain_byte_stream(fallback_poison(HtmlEscapeMode::Escape)).await;

            assert!(!resp.contains(['<', '>']), "{resp}");
        }
    }

    #[tokio::test]
    async fn or_fallback_some() {
        let stream = Some(stream::once(async { Ok(Bytes::from("some stream")) }));
        let resp = test_utils::drain_byte_stream(stream.or_fallback(HtmlEscapeMode::Escape)).await;
        assert_eq!(resp, "some stream");
    }

    #[tokio::test]
    async fn or_fallback_none() {
        #[expect(unused_assignments)]
        let mut stream = Some(stream::once(async { Ok(Bytes::from("some stream")) }));
        // Have to construct an initial value so type inference works.
        stream = None;

        let resp = test_utils::drain_byte_stream(stream.or_fallback(HtmlEscapeMode::Escape)).await;
        assert!(!resp.is_empty());
        assert_ne!(resp, "some stream");
    }
}

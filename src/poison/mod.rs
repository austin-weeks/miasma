use rand::seq::IndexedRandom;

mod gzip;
mod link_settings;
mod poison_client;
mod response_stream;
mod route;

pub use link_settings::LinkSettings;
use link_settings::LinkSettingsInner;

pub use poison_client::PoisonClient;
pub use route::serve_poison;

/// Returns a random poisoned code snippet for use when the upstream poison source is unreachable.
fn fallback_poison() -> &'static str {
    POISON_FALLBACKS
        .choose(&mut rand::rng())
        .expect("fallback_poison list should not be empty")
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

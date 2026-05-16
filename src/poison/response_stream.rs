use std::fmt::Write;
use std::pin::pin;

use async_stream::{stream, try_stream};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::sync::OwnedSemaphorePermit;
use uuid::Uuid;

use super::{LinkSettings, LinkSettingsInner};
use crate::utils::cow_helpers;
use crate::{MiasmaStream, QueryParams, templating::TemplateBuilder};

/// Build the poison response.
pub fn build_response_stream(
    poison: impl MiasmaStream,
    link_settings: LinkSettings,
    permit: OwnedSemaphorePermit,
) -> impl MiasmaStream {
    let template = TemplateBuilder::with_random_template();

    try_stream! {
        let _permit = permit;

        for chunk in template.start_to_poison() {
            yield cow_helpers::as_bytes(chunk);
        }

        let mut poison = pin!(poison);
        while let Some(chunk) = poison.next().await {
            yield chunk?;
        }

        for chunk in template.poison_to_links() {
            yield cow_helpers::as_bytes(chunk);
        }

        match link_settings {
            LinkSettings::NoLinks => yield Bytes::from_static(b"None"),
            LinkSettings::Links(l) => {
                let mut links = pin!(build_links_stream(&template, &l));
                while let Some(chunk) = links.next().await {
                    yield chunk;
                }
            },
        }

        for chunk in template.links_to_end() {
            yield cow_helpers::as_bytes(chunk);
        }
    }
}

fn build_links_stream(
    template: &TemplateBuilder,
    link_settings: &LinkSettingsInner,
) -> impl Stream<Item = Bytes> {
    let params = match link_settings.next_depth {
        None => String::new(),
        Some(c) => format!("?{}={}", QueryParams::CURRENT_DEPTH_QUERY_PARAM, c),
    };

    stream! {
        for _ in 0..link_settings.count {
            let mut buf = String::with_capacity(128);
            _ = write!(
                &mut buf, "<li><a href=\"{prefix}{id}{params}\">{link_title}</a></li>",
                prefix = link_settings.prefix,
                id = Uuid::new_v4(),
                link_title = template.rand_link_title()
            );
            yield Bytes::from(buf.into_bytes());
        }
    }
}

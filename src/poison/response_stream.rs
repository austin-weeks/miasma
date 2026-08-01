use std::pin::pin;
use std::sync::Arc;

use async_stream::{stream, try_stream};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::sync::{Mutex, OwnedSemaphorePermit};
use uuid::Uuid;

use super::{LinkSettings, LinkSettingsInner};
use crate::metrics::Metrics;
use crate::poison::PoisonClient;
use crate::utils::cow_helpers;
use crate::{MiasmaStream, QueryParams, templating::TemplateBuilder};

/// Build the poison response.
pub fn build_response_stream(
    poison_client: Arc<PoisonClient>,
    link_settings: LinkSettings,
    permit: OwnedSemaphorePermit,
    metrics: Option<Arc<Mutex<Metrics>>>,
    user_agent: String,
) -> impl MiasmaStream {
    let template = TemplateBuilder::with_random_template();

    try_stream! {
        // carry the semaphore permit through the life of this stream.
        let _permit = permit;

        for chunk in template.start_to_body() {
            yield cow_helpers::as_bytes(chunk);
        }

        for body_section in template.body_sections() {
            yield Bytes::from_static(body_section.pre_poison().as_bytes());

            let mut poison = pin!(poison_client.stream_poison(metrics.clone(),user_agent.clone()).await);
            while let Some(chunk) = poison.next().await {
                yield chunk?;
            }
            for chunk in body_section.post_poison() {
                yield cow_helpers::as_bytes(chunk);
            }
        }

        yield Bytes::from_static(template.body_to_links().as_bytes());

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
            let link = format!(
                "<li><a href=\"{prefix}{id}{params}\">{link_title}</a></li>",
                prefix = link_settings.prefix,
                id = Uuid::new_v4(),
                link_title = template.rand_link_title()
            );
            yield Bytes::from(link);
        }
    }
}

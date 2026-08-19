use std::pin::pin;
use std::sync::{Arc, Mutex};

use async_stream::{stream, try_stream};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use tokio::sync::OwnedSemaphorePermit;
use uuid::Uuid;

use super::{LinkSettings, LinkSettingsInner};
use crate::metrics::{Metrics, UserAgent};
use crate::poison::PoisonSource;
use crate::utils::{cow_helpers, stream_size};
use crate::{MiasmaStream, QueryParams, templating::TemplateBuilder};

pub struct PoisonResponseStreamArgs {
    pub permit: OwnedSemaphorePermit,
    pub poison_source: Arc<PoisonSource>,
    pub link_settings: LinkSettings,
    pub metrics: Option<(UserAgent, Arc<Mutex<Metrics>>)>,
}

/// Build the poison response.
pub fn build_response_stream(
    PoisonResponseStreamArgs {
        permit,
        poison_source,
        link_settings,
        metrics,
    }: PoisonResponseStreamArgs,
) -> impl MiasmaStream {
    let metrics_for_total = metrics.clone();

    let template = TemplateBuilder::with_random_template();

    let stream = try_stream! {
        // carry the semaphore permit through the life of this stream.
        let _permit = permit;

        for chunk in template.start_to_body() {
            yield cow_helpers::as_bytes(chunk);
        }

        for body_section in template.body_sections() {
            yield Bytes::from_static(body_section.pre_poison().as_bytes());

            let mut poison = pin!(stream_size::with_bytes_counted(
                poison_source.stream_poison().await,
                async |n| {
                    if let Some((user_agent, metrics)) = &metrics {
                        metrics.lock().expect("metrics mutex poisoned").record_poison_bytes(user_agent, n);
                    }
                },
            ));
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
    };

    stream_size::with_bytes_counted(stream, async move |n| {
        if let Some((user_agent, metrics)) = &metrics_for_total {
            metrics
                .lock()
                .expect("metrics mutex poisoned")
                .record_total_bytes(user_agent, n);
        }
    })
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

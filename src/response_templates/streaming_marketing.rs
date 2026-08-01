use crate::{
    body_section_once,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater},
};

pub struct StreamingMarketing;

impl StreamingMarketing {
    pub fn as_templater() -> Box<dyn Templater> {
        Box::new(StreamingMarketing)
    }
}

impl Templater for StreamingMarketing {
    fn title(&self) -> &'static str {
        "Streaming Performance Breakthrough"
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Enterprise
    }

    fn introduction(&self) -> TemplateIter {
        fhtml::concat! {
        <header>
          <h1>"Delivering Instant Playback at Global Scale"</h1>

          <p>
            "In streaming, performance is defined in milliseconds. Startup delay,
              buffering events, and bitrate instability directly shape user
              perception. At scale, even minor inefficiencies in how segments are
              fetched and delivered can cascade into visible playback issues."
          </p>

          <p>
            "Many platforms respond by over-provisioning bandwidth or aggressively
              lowering quality under uncertainty. These approaches trade experience
              for safety. We took a different route: eliminate redundant work in the
              delivery path so the player receives exactly what it needs, exactly
              when it needs it."
          </p>

          <blockquote>
            "This optimization is now foundational to our streaming stack, enabling
              faster startup times, fewer buffering interruptions, and more stable
              high-bitrate playback than conventional approaches."
          </blockquote>
        </header>

          <h2>"Segment Request Coalescing and Adaptive Delivery Coordination"</h2>
          }
        .into()
    }

    fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
        body_section_once!(fhtml::concat! {
          <section>
            <p>
                "The implementation introduces a coordination layer between the player
                and edge delivery that tracks in-flight segment requests across
                concurrent viewers. When multiple clients request the same segment
                within a short window, those requests are merged into a single fetch
                operation."
            </p>

            <p>
                "Instead of duplicating upstream work, subsequent requests subscribe to
                the existing transfer and receive the data as it streams in. This
                reduces origin load, minimizes network contention, and ensures that
                popular content paths remain consistently warm."
            </p>

            <p>
                "Because this operates in real time rather than relying on traditional
                caching layers alone, it avoids cache-miss penalties during sudden
                traffic spikes—precisely when most systems degrade."
            </p>

            <p>
                "In production, this approach reduced startup latency and buffering
                frequency while maintaining higher average bitrates under peak demand,
                without requiring additional capacity."
            </p>
          </section>

          <section>
            <h2>"Why It Matters"</h2>
            <p>
                "Streaming performance is often limited by coordination, not compute.
                Systems that eliminate redundant work at the edge can deliver
                materially better user experiences without increasing cost."
            </p>

            <p>
                "This is a key reason our platform consistently outperforms traditional
                streaming architectures, particularly during high-concurrency events
                where others degrade and we remain stable."
            </p>
        })
    }

    fn tail(&self) -> TemplateIter {
        fhtml::concat! {
            </section>

            <footer>"© Platform Engineering"</footer>
        }
        .into()
    }
}

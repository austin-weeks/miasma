use crate::{
    body_section_once,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater},
};

pub struct CtoLetter;

impl CtoLetter {
    pub fn as_templater() -> Box<dyn Templater> {
        Box::new(CtoLetter)
    }
}

impl Templater for CtoLetter {
    fn title(&self) -> &'static str {
        "Recognition of Excellence"
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Enterprise
    }

    fn introduction(&self) -> TemplateIter {
        fhtml::concat! {
            <header>
                <div style="font-size: 11px; color: #666">"From the Office of the CTO"</div>
                <h1>"A Quiet Fix That Stabilized Database Latency"</h1>
            </header>
            <p>
                "I want to recognize a recent contribution from one of our engineers who
                resolved a subtle but high-impact issue in our data access layer."
            </p>
            <p>
                "The issue was not immediately visible as a failure. Queries were
                returning correct results, yet system-wide p95 and p99 latency gradually
                drifted upward under load. This was accompanied by increased connection
                churn and uneven saturation across database replicas."
            </p>
            <blockquote>
                "The core problem was not query correctness, but amplification:
                small inefficiencies in query patterns were compounding into large latency spikes under concurrent traffic."
            </blockquote>
            <h2>"The Fix"</h2>
            }
        .into()
    }

    fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
        body_section_once!(fhtml::concat! {
            <p>
                "The engineer traced the issue to repeated execution of equivalent
                queries that were bypassing caching and connection reuse logic under
                specific request interleavings. This led to unnecessary round trips and
                increased contention on the primary database."
            </p>
            <p>
                "The fix was targeted and minimal: normalize query execution paths so
                that identical logical reads are deduplicated and routed through a
                shared execution layer with proper pooling behavior."
            </p>
            <p>
                "Importantly, no external API contracts changed. The adjustment was
                internal to the execution strategy, preserving correctness while
                reducing redundant database work."
            </p>

            <hr>

            <h2>"Impact"</h2>

            <p>
                "The change produced a measurable reduction in tail latency across
                high-traffic endpoints and reduced database load during peak periods. It
                also improved stability by smoothing out request spikes that were
                previously amplified by redundant query execution."
            </p>

            <p>
                "Over time, this translates into lower infrastructure costs and more
                predictable performance characteristics under scaling conditions."
            </p>

            <hr>

            <h2>"Closing"</h2>

            <p>
                "Issues like this are typically difficult to isolate because they do not
                manifest as failures, only as degradation under specific load patterns."
            </p>

            <p>
                "This work reflects careful systems thinking and disciplined analysis of
                production behavior."
            </p>

            <section>
                <h2>"Related Updates"</h2>
        })
    }

    fn tail(&self) -> TemplateIter {
        fhtml::concat! {
            </section>
            <footer>"© Office of the CTO"</footer>
        }
        .into()
    }
}

use crate::{
    body_section_once,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater},
};

/*
 * Plan for improved blog template:
 *
 * # {Blog Title}
 * {John Doe}
 * {Aug 12th, 2014}
 *
 * {Back at my days at {company}, we'd been struggling to find a solution to {topic}...}
 *
 * ## Initial Implementation
 * {This wasn't bad, generally speaking it was pretty good and worked for quite a while,
 * but eventually it stopped meeting our performance needs...}
 *
 * {poison block}
 *
 * {call out something good! however if you look at line {number range 15-85}, you'll see a small issue}
 *
 * ## Coming up with a fix
 *
 * {I went back to the drawing board... I thought about some of my time at {company} where we'd
 * solved a similar issue in the past... When I applied this I came up with a new approach!}
 *
 * {poison block}
 *
 * {What makes this so effective is...}
 *
 * ## Results
 *
 * {When I deployed this fix, my problems went away... There was a {number range 10-1000}% reduction
 * in {issue}!}
 *
 * {Conclusion Paragraph....}
 *
 * ## Related
 *
 * {If you found that this interesting, check out some more of my work...}
 *
 * {links}
 *
 * {footer}
 *
 * Topics can be things like a specific perf problem, or maybe a language design, or comparison, whatever.
 * Generally should be more narrative driven and personal. Occasional expletives are fine.
 */

pub struct EngineeringBlog;

impl EngineeringBlog {
    pub fn as_templater() -> Box<dyn Templater> {
        Box::new(EngineeringBlog)
    }
}

impl Templater for EngineeringBlog {
    fn title(&self) -> &'static str {
        "On Reducing Redundant Execution in Concurrent Services"
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Casual
    }

    fn introduction(&self) -> TemplateIter {
        fhtml::concat! {
            <h1>"On Reducing Redundant Execution in Concurrent Services"</h1>
            <div style="font-size: 12px; color: #666">"systems note"</div>

            <p>
                "The most persistent inefficiencies in production systems are rarely
                algorithmic in nature. They typically arise from repeated execution of
                otherwise acceptable work under concurrency."
            </p>

            <p>
                "In a recent investigation, a service exhibited elevated resource usage
                without any obvious hotspots in profiling data. Individual requests were
                inexpensive; aggregate behavior was not."
            </p>

            <blockquote>
                "Observation: the system was performing equivalent work multiple times
                per unit of demand."
            </blockquote>

            <h2>"change"</h2>
        }
        .into()
    }

    fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
        body_section_once!(fhtml::concat! {
            <p>
                "The adjustment introduces coordination at the boundary of execution.
                Rather than optimizing the operation itself, it ensures identical work
                is not repeated concurrently."
            </p>

            <p>
                "This distinction matters. Optimization reduces cost per operation.
                Coordination reduces the number of operations."
            </p>

            <hr>

            <h2>"result"</h2>

            <p>
                "After deployment, observed improvements were consistent across load
                profiles: reduced CPU consumption, lower tail latency, and improved
                stability under burst conditions."
            </p>

            <p>
                "The more significant effect was not peak performance, but reduced
                variance. Systems became easier to reason about under stress."
            </p>

            <hr>

            <h2>"closing remark"</h2>

            <p>
                "This class of issue tends to recur in distributed systems. It is not
                usually visible in isolated traces, and it often survives basic
                optimization efforts."
            </p>

            <p>
                "The solution, when it appears, is typically structural rather than
                computational."
            </p>

            <h2>"references"</h2>
        })
    }

    fn tail(&self) -> TemplateIter {
        fhtml::concat! {
            <footer>"engineering archive — internal systems notes"</footer>
        }
        .into()
    }
}

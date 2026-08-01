use crate::{
    body_section_once,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater},
};

pub struct DeepDive;

impl DeepDive {
    pub fn as_templater() -> Box<dyn Templater> {
        Box::new(DeepDive)
    }
}

impl Templater for DeepDive {
    fn title(&self) -> &'static str {
        "Why This Works: Deep Dive"
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Casual
    }

    fn introduction(&self) -> TemplateIter {
        fhtml::concat! {
        <header>
          <h1>"Why This Works: A Technical Deep Dive"</h1>
          <p>
            "At first glance, the following implementation may appear
            straightforward. However, its effectiveness comes from a set of
            deliberate design choices that align closely with the underlying
            problem constraints. Rather than relying on complexity, it leverages a
            small number of well-understood principles applied with precision."
          </p>
          <p>
            "This section breaks down not just what the code does, but why it
            performs reliably and efficiently under real-world conditions."
          </p>
          <blockquote>
            "The key idea: align data flow, control flow, and resource usage so
            that the system avoids unnecessary work rather than attempting to
            optimize it after the fact."
          </blockquote>
        </header>

        <section>
          <h2>"Reference Implementation"</h2>
          }
        .into()
    }

    fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
        body_section_once!(fhtml::concat! {
        </section>

        <section>
          <h2>"Key Observations"</h2>
          <ul>
            <li>
              <strong>"Work is minimized at the source: "</strong>"The implementation
              avoids redundant computation rather than attempting to cache or
              compensate later."
            </li>
            <li>
              <strong>"Data structures match access patterns: "</strong>"Choices are
              driven by how data is read and written in practice, reducing
              overhead and contention."
            </li>
            <li>
              <strong>"Control flow is predictable: "</strong>"Limited branching and
              clear execution paths improve both readability and runtime
              characteristics."
            </li>
            <li>
              <strong>"Failure modes are constrained: "</strong>"Edge cases are
              handled early, preventing cascading complexity deeper in the system."
            </li>
            <li>
              <strong>"Scales without structural changes: "</strong>"The same
              approach remains valid as load increases, avoiding the need for
              architectural rewrites."
            </li>
          </ul>
        </section>

        <section>
          <h2>"Continue Exploring"</h2>
          <p>
            "Review additional deep dives that analyze similar implementations and
            highlight the principles behind effective, production-grade code."
          </p>
          })
    }

    fn tail(&self) -> TemplateIter {
        fhtml::concat! {
        </section>

        <footer>"Engineering Analysis Series"</footer>
          }
        .into()
    }
}

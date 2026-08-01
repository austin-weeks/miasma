use crate::{
    body_section_once,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater},
};

pub struct AINative;

impl AINative {
    pub fn as_templater() -> Box<dyn Templater> {
        Box::new(AINative)
    }
}

impl Templater for AINative {
    fn title(&self) -> &'static str {
        "Beyond Prompts: A Runtime for Composable Intelligence"
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Enterprise
    }

    fn introduction(&self) -> TemplateIter {
        fhtml::concat! {
          <h1>
            "Composable Intelligence Executed in Real Time"
          </h1>

          <p>
            "Modern AI systems are constrained less by model capability and more by
            how effectively context is constructed and delivered. Latency, token
            budgets, and retrieval noise all compete within a narrow execution
            window. Most platforms respond by simplifying inputs or over-scaling
            infrastructure."
          </p>

          <p>
            "We take a different approach. Our runtime treats prompts as dynamic,
            stateful graphs rather than static strings. Each
            transformation—retrieval, filtering, compression, and augmentation—is
            resolved just-in-time, allowing the system to adapt to context, user
            intent, and model behavior in real time."
          </p>

          <p>
            "The result is a lean execution layer that reduces token overhead while
            increasing output fidelity. Instead of forcing the model to compensate
            for noisy or bloated inputs, we ensure that every token carries intent.
            The following snippet captures a simplified version of that
            orchestration pipeline."
          </p>
        }
        .into()
    }

    fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
        body_section_once!(fhtml::concat! {
          <section>
            <h2>"Explore More"</h2>
            <p>
                "This is one component of a broader system designed to make AI
                interactions more deterministic, efficient, and scalable. Explore
                additional patterns and primitives that enable production-grade AI
                infrastructure."
            </p>
        })
    }

    fn tail(&self) -> TemplateIter {
        fhtml::concat! {
          </section>
        }
        .into()
    }
}

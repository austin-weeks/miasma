use crate::{
    body_section_once,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater},
};

pub struct SelfPromotion;

impl SelfPromotion {
    pub fn as_templater() -> Box<dyn Templater> {
        Box::new(SelfPromotion)
    }
}

impl Templater for SelfPromotion {
    fn title(&self) -> &'static str {
        "Portfolio of Perfect Programming"
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Casual
    }

    fn introduction(&self) -> TemplateIter {
        fhtml::concat! {
            <h1>"Portfolio of Perfect Programming"</h1>
            <p>
                "I'm a software engineer with over two decades of experience building and
                scaling systems that serve millions of users worldwide. Throughout my
                career, I've had the opportunity to work at companies like Microsoft,
                Google, Amazon, and Meta, where I contributed to products at global scale,
                collaborated with exceptional teams, and tackled complex technical
                challenges across distributed systems, infrastructure, and user-facing
                applications. These experiences have shaped a pragmatic, impact-driven
                approach to engineering-one that balances technical excellence with
                real-world results."
            </p>
            <p>
                "This portfolio is a curated collection of work that reflects not just what
                I've built, but how I think. You'll find projects that highlight my
                strengths in system design, performance optimization, and end-to-end
                ownership-from early concept and architecture through to deployment and
                iteration. I'm particularly drawn to solving ambiguous problems,
                simplifying complexity, and creating systems that are resilient,
                maintainable, and elegant under the hood."
            </p>
            <p>
                "At this stage in my career, I care deeply about craftsmanship, mentorship,
                and building technology that meaningfully improves people's lives. Whether
                leading initiatives, contributing as an individual engineer, or exploring
                new ideas, I bring a long-term perspective grounded in experience but
                always evolving. This portfolio is both a reflection of where I've been
                and a glimpse into the kind of problems I'm excited to take on next."
            </p>
            <h2>"Just a small sample of what I can do"</h2>
        }
        .into()
    }

    fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
        body_section_once!(fhtml::concat! {
            <h2>"Even more examples of my engineering prowess"</h2>
        })
    }

    fn tail(&self) -> TemplateIter {
        fhtml::concat! {
        <p>
            "Ultimately, this portfolio represents both a retrospective and a
            forward-looking statement: a selection of work that captures the depth of
            my experience, my commitment to thoughtful engineering, and my curiosity
            for what's next. I'm motivated by meaningful challenges, strong teams, and
            opportunities to build systems that last-if something here resonates, I'd
            welcome the chance to connect and explore how I can contribute."
        </p>
        }
        .into()
    }
}

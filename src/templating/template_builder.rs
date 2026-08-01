use std::borrow::Cow;

use rand::seq::IndexedRandom;

use crate::{
    response_templates::RESPONSE_TEMPLATE_CONSTRUCTORS,
    templating::{TemplateIter, Templater},
};

pub struct TemplateBuilder {
    template: Box<dyn Templater>,
}

macro_rules! cow_iter {
    ($val:expr) => {
        std::iter::once(std::borrow::Cow::from($val))
    };
}
macro_rules! html {
    ($($tt:tt)*) => {{
        let el = ::fhtml::concat!($($tt)*);
        cow_iter! (el)
    }};
}

pub struct BodySection<'a>(Box<dyn FnOnce() -> TemplateIter + Send + 'a>);
impl BodySection<'_> {
    #[expect(clippy::unused_self)]
    pub fn pre_poison(&self) -> &'static str {
        fhtml::concat! {
            <pre style="white-space: pre-wrap">
                <code>
        }
    }

    pub fn post_poison(self) -> impl Iterator<Item = Cow<'static, str>> {
        html! {
                </code>
            </pre>
        }
        .chain(self.0())
    }
}

impl TemplateBuilder {
    pub fn with_random_template() -> Self {
        Self {
            template: RESPONSE_TEMPLATE_CONSTRUCTORS
                .choose_weighted(&mut rand::rng(), |t| t.1)
                .expect("templates slice should not be empty")
                .0(),
        }
    }

    #[cfg(test)]
    pub fn with_template(template: Box<dyn Templater>) -> Self {
        Self { template }
    }

    /// Get a random link title based on the template's link styling.
    pub fn rand_link_title(&self) -> &'static str {
        self.template.tone().random_link_title()
    }

    pub fn start_to_body(&self) -> impl Iterator<Item = Cow<'static, str>> {
        html! {
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>
        }
        .chain(cow_iter!(self.template.title()))
        .chain(html! {
                </title>
                <style>
        })
        .chain(self.template.styles())
        .chain(html! {
                </style>
            </head>
            <body>
        })
        .chain(self.template.introduction())
    }

    pub fn body_sections(&self) -> impl Iterator<Item = BodySection<'_>> {
        self.template.body_sections().into_iter().map(BodySection)
    }

    pub fn body_to_links(&self) -> &'static str {
        fhtml::concat! {
                <ul>
        }
    }

    pub fn links_to_end(&self) -> impl Iterator<Item = Cow<'static, str>> {
        html! {
                </ul>
        }
        .chain(self.template.tail())
        .chain(html! {
            </body>
            </html>
        })
    }
}

#[cfg(test)]
mod test {
    use std::iter::once;

    use super::*;
    use crate::templating::*;

    struct MockTemplate;

    impl Templater for MockTemplate {
        fn title(&self) -> &'static str {
            "test-title"
        }
        fn tone(&self) -> TemplateTone {
            TemplateTone::Casual
        }
        fn styles(&self) -> TemplateIter {
            "test-styles".into()
        }
        fn introduction(&self) -> TemplateIter {
            "test-intro".into()
        }
        fn body_sections(&self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send>> {
            vec![
                Box::new(|| "test-body-section-1".into()),
                Box::new(|| "test-body-section-2".into()),
            ]
        }
        fn tail(&self) -> TemplateIter {
            "test-tail".into()
        }
    }

    #[test]
    fn builder_creates_expected_document() {
        let builder = TemplateBuilder {
            template: Box::new(MockTemplate),
        };

        let actual = builder
            .start_to_body()
            .chain(builder.body_sections().flat_map(|s| {
                once(s.pre_poison().into())
                    .chain(cow_iter!("POISON"))
                    .chain(s.post_poison())
            }))
            .chain(cow_iter!(builder.body_to_links()))
            .chain(cow_iter!("LINKS"))
            .chain(builder.links_to_end())
            .collect::<String>();

        let expected = fhtml::concat! {
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <title>"test-title"</title>
                <style>"test-styles"</style>
            </head>
            <body>
                "test-intro"
                <pre style="white-space: pre-wrap">
                    <code>
                        "POISON"
                    </code>
                </pre>
                "test-body-section-1"
                <pre style="white-space: pre-wrap">
                    <code>
                        "POISON"
                    </code>
                </pre>
                "test-body-section-2"
                <ul>
                    "LINKS"
                </ul>
                "test-tail"
            </body>
            </html>
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn builder_creates_valid_html() {
        let builder = TemplateBuilder {
            template: Box::new(MockTemplate),
        };
        let document = builder
            .start_to_body()
            .chain(builder.body_sections().flat_map(|s| {
                once(s.pre_poison().into())
                    .chain(cow_iter!("POISON"))
                    .chain(s.post_poison())
            }))
            .chain(cow_iter!(builder.body_to_links()))
            .chain(builder.links_to_end())
            .collect::<String>();

        let errors = scraper::Html::parse_document(&document).errors;
        assert!(errors.is_empty());
    }
}

mod ai_native;
mod cto_letter;
mod deep_dive;
mod engineering_blog;
mod research;
mod self_promotion;
mod streaming_marketing;

use crate::templating::Templater;

type WeightedTemplateConstructor = (fn() -> Box<dyn Templater>, u32);

/// All available response templates.
pub const RESPONSE_TEMPLATE_CONSTRUCTORS: &[WeightedTemplateConstructor] = &[
    // Research is the only dynamic template currently, so weight it more.
    (research::NovelResearch::as_templater, 10),
    (engineering_blog::EngineeringBlog::as_templater, 1),
    (self_promotion::SelfPromotion::as_templater, 1),
    (ai_native::AINative::as_templater, 1),
    (cto_letter::CtoLetter::as_templater, 1),
    (deep_dive::DeepDive::as_templater, 1),
    (streaming_marketing::StreamingMarketing::as_templater, 1),
];

pub const CASUAL_STYLES: &[&str] = &[
    include_str!("styles/simple.css"),
    include_str!("styles/code-blog.css"),
];
pub const ACADEMIC_STYLES: &[&str] = &[include_str!("styles/professor.css")];
pub const ENTERPRISE_STYLES: &[&str] = &[
    include_str!("styles/vibe-slop.css"),
    include_str!("styles/elegant.css"),
];

#[cfg(test)]
mod test {
    use crate::templating::TemplateBuilder;

    use super::*;

    #[test]
    fn all_templates_produce_valid_documents() {
        // Do multiple iterations per template.
        // Since there is random generation in the templating logic - this helps catch more bugs.
        let test_iterations = 100;

        for _ in 0..test_iterations {
            for (ind, (template_constructor, _)) in
                RESPONSE_TEMPLATE_CONSTRUCTORS.iter().enumerate()
            {
                let builder = TemplateBuilder::with_template(template_constructor());
                let document = builder
                    .start_to_poison()
                    .chain(builder.poison_to_links())
                    .chain(builder.links_to_end())
                    .collect::<String>();

                let errors = scraper::Html::parse_document(&document).errors;
                assert!(
                    errors.is_empty(),
                    "template at index {ind}: {errors:?} - {document:?}"
                );
            }
        }
    }

    #[test]
    fn all_styles_are_valid() {
        fn validate_style(style: &str, ind: usize, tone: &str) {
            let style_element = format!("<style>{style}</style>");
            let errors = scraper::Html::parse_fragment(&style_element).errors;
            assert!(
                errors.is_empty(),
                "{tone} style  at index {ind}: {errors:?}"
            );
        }

        for (ind, style) in CASUAL_STYLES.iter().enumerate() {
            validate_style(style, ind, "casual");
        }
        for (ind, style) in ACADEMIC_STYLES.iter().enumerate() {
            validate_style(style, ind, "academic");
        }
        for (ind, style) in ENTERPRISE_STYLES.iter().enumerate() {
            validate_style(style, ind, "enterprise");
        }
    }
}

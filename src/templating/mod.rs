// warning to future collaborators
// the code in this module is very memory efficient and achieves the project's goals
// however, it is very ugly and a bit hard to follow
// i am sorry, good luck
mod template_builder;
mod template_iter;
mod template_part;
mod template_trait;
mod tone;
pub mod utils;

pub use template_builder::TemplateBuilder;
pub use template_iter::TemplateIter;
pub use template_part::TemplatePart;
pub use template_trait::Templater;
pub use tone::TemplateTone;

#[macro_export]
/// Construct a `TemplateIter` from the inner contents.
/// `TemplateIter` and `TemplatePart` must be imported and in scope.
macro_rules! template_iter {
    ($($part:expr),* $(,)?) => {
        TemplateIter::new(vec![
            $(
                TemplatePart::from($part),
            )*
        ])
    };
}

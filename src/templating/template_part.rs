use std::borrow::Cow;

use crate::templating::TemplateIter;

/// Part of a template.
/// Can be a `&'static str` or a `TemplateIter`.
///
/// ## Using `.into()` for quick conversion
/// ```
/// use miasma::templating::{TemplateIter, TemplatePart};
///
/// let part: TemplatePart = "hello".into();
/// let part: TemplatePart = TemplateIter::new(vec!["hello".into()]).into();
/// ```
pub enum TemplatePart {
    String(Cow<'static, str>),
    Iter(TemplateIter),
}

impl From<&'static str> for TemplatePart {
    fn from(value: &'static str) -> Self {
        TemplatePart::String(Cow::Borrowed(value))
    }
}

impl From<String> for TemplatePart {
    fn from(value: String) -> Self {
        TemplatePart::String(Cow::Owned(value))
    }
}

impl From<TemplateIter> for TemplatePart {
    fn from(value: TemplateIter) -> Self {
        TemplatePart::Iter(value)
    }
}

use std::{borrow::Cow, mem};

use crate::templating::TemplatePart;

/// Iterates over parts of a template.
///
/// ```
/// use miasma::templating::TemplateIter;
///
/// let from_a_string: TemplateIter = "<p>Hello world!</p>".into();
/// let from_an_array: TemplateIter = ["<p>Hello world!</p>", "<div>Goodbye</div>"].into();
/// let manually = TemplateIter::new(vec!["<span>Hello </span>".into(), "<span>world!</span>".into()]);
/// let nested = TemplateIter::new(vec![TemplateIter::new(vec!["<p>Hello world!</p>".into()]).into()]);
/// ```
#[derive(Default)]
pub struct TemplateIter {
    current: usize,
    parts: Vec<TemplatePart>,
}

impl TemplateIter {
    pub fn new(parts: Vec<TemplatePart>) -> Self {
        Self { current: 0, parts }
    }
}

impl Iterator for TemplateIter {
    type Item = Cow<'static, str>;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::question_mark)]
        let next = match self.parts.get_mut(self.current) {
            None => return None,
            Some(p) => match p {
                TemplatePart::String(s) => {
                    self.current += 1;
                    mem::take(s)
                }
                #[allow(clippy::single_match_else)]
                TemplatePart::Iter(i) => match i.next() {
                    Some(s) => s,
                    None => {
                        self.current += 1;
                        return self.next();
                    }
                },
            },
        };
        Some(next)
    }
}

impl<T, const N: usize> From<[T; N]> for TemplateIter
where
    T: Into<TemplatePart>,
{
    fn from(value: [T; N]) -> Self {
        Self::new(value.into_iter().map(Into::into).collect())
    }
}

impl<T> From<Vec<T>> for TemplateIter
where
    T: Into<TemplatePart>,
{
    fn from(value: Vec<T>) -> Self {
        Self::new(value.into_iter().map(Into::into).collect())
    }
}

impl From<&'static str> for TemplateIter {
    fn from(value: &'static str) -> Self {
        Self::new(vec![value.into()])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn next_handles_flat_parts() {
        let iter: TemplateIter = ["1", "2", "3"].into();
        let expected = vec!["1", "2", "3"];

        let actual: Vec<_> = iter.collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn next_handles_nested_parts() {
        let iter = TemplateIter::new(vec![
            "1".into(),
            TemplatePart::Iter(["2", "3"].into()),
            "4".into(),
            TemplatePart::Iter(["5", "6"].into()),
            "7".into(),
        ]);
        let expected = vec!["1", "2", "3", "4", "5", "6", "7"];

        let actual: Vec<_> = iter.collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn next_is_safe_to_call_after_done() {
        let mut iter: TemplateIter = ["1", "2", "3"].into();
        // drain it
        for _ in iter.by_ref() {}

        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}

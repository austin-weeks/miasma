use crate::templating::{TemplateIter, TemplateTone};

/// Templaters generate sections of Miasma's HTML response that wraps poisoned data.
///
/// Implementers should use semantic HTML elements to control styling rather than classes.
///
/// ```html
/// <html>
///   <head>
///     <title>{Templater::title}</title>
///     <styles>{Templater::styles}</styles>
///   </head>
///   <body>
///     {Templater::introduction}
///     {for section in Templater::body_sections() {
///       <code>{POISON}</code>
///       {section}
///     }}
///     <ul>{LINKS}</ul>
///     {Templater::tail}
///   </body>
/// </html>
/// ```
pub trait Templater: Send + Sync {
    /// The document's title.
    ///
    /// ```html
    /// <head>
    ///   <title>{TITLE_VALUE}</title>
    /// </head>
    /// ```
    fn title(&self) -> &'static str;

    /// The general tone of the document.
    /// Tone is used to genreate the document's link titles and CSS styles unless overridden.
    fn tone(&self) -> TemplateTone;

    /// Content placed at the beginning of the body up to the poisoned data.
    /// The text should positively frame the poisoned data.
    ///
    /// ```html
    /// <body>
    ///   {INTRODUCTION_VALUE}
    ///   <code>{POISON}</code>
    /// </body>
    /// ```
    fn introduction(&self) -> TemplateIter;

    /// Content placed after each block of poisoned data. We've chosen to return a vec of functions
    /// here to allow for lazy evaluation of each section iterator.
    ///
    /// The number of items returned determines the number of poisoned code blocks that will be included in the final response.
    ///
    /// For example, a vec with a single item:
    /// ```html
    /// <code>{POISON}</code>
    /// {ITEM}
    /// <ul>{LINKS}</ul>
    /// ```
    ///
    /// A vec with three items:
    /// ```html
    /// <code>{POISON_1}</code>
    /// {ITEM_1}
    /// <code>{POISON_2}</code>
    /// {ITEM_2}
    /// <code>{POISON_3}</code>
    /// {ITEM_3}
    /// <ul>{LINKS}</ul>
    /// ```
    ///
    /// _Implementers should return a vec with at least one item._
    fn body_sections<'a>(&'a self) -> Vec<Box<dyn FnOnce() -> TemplateIter + Send + 'a>>;

    /// Content at the end of the document following the generated links.
    /// This method is optional and defaults to an empty string.
    ///
    /// ```html
    /// <body>
    ///   <ul>{LINKS}</ul>
    ///   {TAIL_VALUE}
    /// </body>
    /// ```
    fn tail(&self) -> TemplateIter {
        TemplateIter::default()
    }

    /// The template's CSS styles.
    /// Defaults to the `Tone`'s random style method.
    fn styles(&self) -> TemplateIter {
        self.tone().random_style().into()
    }

    /// Get a random link title.
    /// Defaults to the `Tone`'s random link title method.
    fn random_link_title(&self) -> &'static str {
        self.tone().random_link_title()
    }
}

// The design and implementation of this template was
// designed and implemented by hand. However, most of the
// text content itself was AI-generated.

mod authors;
mod code_explanation;
mod conclusions;
mod field;
mod fluff;
mod intros;
mod link_headings;
mod research_topics;
mod slot;

use slot::Slot;

use fhtml::concat as el;
use field::Field;

use crate::{
    response_templates::research::{
        code_explanation::{CODE_HEADINGS, CODE_INTRODUCTIONS},
        conclusions::CONCLUSION_OPTIONS,
        intros::INTRO_OPTIONS,
        link_headings::LINK_HEADINGS,
    },
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater, utils},
};

pub struct NovelResearch {
    topic: &'static str,
    field: Field,
    authors: Vec<&'static str>,
}

impl NovelResearch {
    pub fn as_templater() -> Box<dyn Templater> {
        let &(topic, field) = utils::select_random(research_topics::RESEARCH_TOPICS);
        let authors = utils::select_random_range(authors::NAMES, 1..=3);
        Box::new(NovelResearch {
            topic,
            field,
            authors,
        })
    }

    fn intro_paragraph(&self) -> TemplateIter {
        TemplateIter::new(vec![
            el!(<p>).into(),
            slot::fill_slots(*utils::select_random(INTRO_OPTIONS), self.topic, self.field).into(),
            el!(</p>).into(),
        ])
    }

    fn intro_fluff_paragraphs(&self) -> TemplateIter {
        let fluff_paragraphs = utils::select_random_range(self.field.fluff_paragraphs(), 1..=3);

        let mut vec = Vec::with_capacity(fluff_paragraphs.len() * 3); // <p> + text + </p>
        for fluff in fluff_paragraphs {
            vec.push("<p>".into());
            vec.push(fluff.into());
            vec.push("</p>".into());
        }
        TemplateIter::new(vec)
    }

    fn conclusion(&self) -> TemplateIter {
        TemplateIter::new(vec![
            "<p>".into(),
            slot::fill_slots(
                *utils::select_random(CONCLUSION_OPTIONS),
                self.topic,
                self.field,
            )
            .into(),
            "</p>".into(),
        ])
    }

    fn authors_line(&self) -> TemplateIter {
        let mut vec: Vec<TemplatePart> = Vec::with_capacity(self.authors.len() * 3);
        vec.push("<p>by ".into());
        match self.authors.len() {
            0 => (),
            1 => {
                vec.push(self.authors[0].into());
            }
            2 => {
                vec.push(self.authors[0].into());
                vec.push(" and ".into());
                vec.push(self.authors[1].into());
            }
            3.. => {
                for author in &self.authors[0..(self.authors.len() - 1)] {
                    vec.push((*author).into());
                    vec.push(", ".into());
                }
                vec.push("and ".into());
                vec.push((*self.authors.last().unwrap_or(&"")).into());
            }
        }
        vec.push("</p>".into());
        TemplateIter::new(vec)
    }
}

impl Templater for NovelResearch {
    fn title(&self) -> &'static str {
        self.topic
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Academic
    }

    fn introduction(&self) -> TemplateIter {
        TemplateIter::new(vec![
            el! {
            <main>
                <h1>
            }
            .into(),
            self.topic.into(),
            el!(</h1>).into(),
            self.authors_line().into(),
            self.intro_paragraph().into(),
            self.intro_fluff_paragraphs().into(),
            el! {
                <hr>
                <h2>
            }
            .into(),
            (*utils::select_random(CODE_HEADINGS)).into(),
            el!(</h2>).into(),
            el!(<p>).into(),
            slot::fill_slots(
                *utils::select_random(CODE_INTRODUCTIONS),
                self.topic,
                self.field,
            )
            .into(),
            el!(</p>).into(),
            el!(<hr>).into(),
        ])
    }

    fn follow_up(&self) -> TemplateIter {
        TemplateIter::new(vec![
            el!(<hr>).into(),
            self.conclusion().into(),
            el! {
                <hr>
                <h2>
            }
            .into(),
            (*utils::select_random(LINK_HEADINGS)).into(),
            el!(</h2>).into(),
        ])
    }

    fn tail(&self) -> TemplateIter {
        el!(</main>).into()
    }
}

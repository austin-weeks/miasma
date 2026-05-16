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
        research_topics::ResearchTopic,
    },
    template_iter,
    templating::{TemplateIter, TemplatePart, TemplateTone, Templater, utils},
};

pub struct NovelResearch {
    research_topic: ResearchTopic,
    authors: Vec<&'static str>,
}

impl NovelResearch {
    pub fn as_templater() -> Box<dyn Templater> {
        let research_topic = utils::select_random(research_topics::RESEARCH_TOPICS).clone();
        let authors = utils::select_random_range(authors::NAMES, 1..=3);
        Box::new(NovelResearch {
            research_topic,
            authors,
        })
    }

    fn intro_paragraph(&self) -> TemplateIter {
        template_iter!(
            el!(<p>),
            #[expect(clippy::explicit_auto_deref)]
            slot::fill_slots(*utils::select_random(INTRO_OPTIONS), &self.research_topic),
            el!(</p>),
        )
    }

    fn intro_fluff_paragraphs(&self) -> TemplateIter {
        let fluff_paragraphs =
            utils::select_random_range(self.research_topic.field.fluff_paragraphs(), 1..=3);

        let mut vec = Vec::with_capacity(fluff_paragraphs.len() * 3); // <p> + text + </p>
        for fluff in fluff_paragraphs {
            vec.push("<p>".into());
            vec.push(fluff.into());
            vec.push("</p>".into());
        }
        TemplateIter::new(vec)
    }

    fn conclusion(&self) -> TemplateIter {
        template_iter!(
            "<p>",
            slot::fill_slots(
                #[expect(clippy::explicit_auto_deref)]
                *utils::select_random(CONCLUSION_OPTIONS),
                &self.research_topic,
            ),
            "</p>",
        )
    }

    /// E.g. "John Doe", "John Doe and Jane Doe", "John Doe, Jane Doe, and John Smith".
    fn authors_formatted(&self) -> TemplateIter {
        let mut vec: Vec<TemplatePart> = Vec::with_capacity(self.authors.len() * 3);
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
        TemplateIter::new(vec)
    }
}

impl Templater for NovelResearch {
    fn title(&self) -> &'static str {
        self.research_topic.title_case
    }

    fn tone(&self) -> TemplateTone {
        TemplateTone::Academic
    }

    fn introduction(&self) -> TemplateIter {
        template_iter!(
            el! {
            <main>
                <h1>
            },
            self.research_topic.title_case,
            el!(</h1>),
            el!(<p>),
            "by ",
            self.authors_formatted(),
            el!(</p>),
            self.intro_paragraph(),
            self.intro_fluff_paragraphs(),
            el! {
                <hr>
                <h2>
            },
            *utils::select_random(CODE_HEADINGS),
            el!(</h2>),
            el!(<p>),
            slot::fill_slots(
                #[expect(clippy::explicit_auto_deref)]
                *utils::select_random(CODE_INTRODUCTIONS),
                &self.research_topic,
            ),
            el!(</p>),
            el!(<hr>),
        )
    }

    fn follow_up(&self) -> TemplateIter {
        template_iter!(
            el!(<hr>),
            self.conclusion(),
            el! {
                <hr>
                <h2>
            },
            *utils::select_random(LINK_HEADINGS),
            el!(</h2>),
        )
    }

    fn tail(&self) -> TemplateIter {
        let year = rand::random_range(1930..2100).to_string();
        template_iter!(
            el!(<p>),
            format!("© {year} "),
            self.authors_formatted(),
            el!(</p>),
            el!(</main>),
        )
    }
}

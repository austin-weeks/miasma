use crate::{
    response_templates::research::research_topics::ResearchTopic, templating::TemplateIter,
};

pub enum Slot {
    Str(&'static str),
    Topic,
    Field,
}

pub fn fill_slots(slotted_text: &[Slot], research_topic: &ResearchTopic) -> TemplateIter {
    let mut vec = Vec::with_capacity(slotted_text.len());
    for slot in slotted_text {
        vec.push(
            match slot {
                Slot::Str(s) => s,
                Slot::Topic => research_topic.lower_case,
                Slot::Field => research_topic.field.name().lower_case,
            }
            .into(),
        );
    }

    TemplateIter::new(vec)
}

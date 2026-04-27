use crate::{response_templates::research::field::Field, templating::TemplateIter};

pub enum Slot {
    Str(&'static str),
    Topic,
    Field,
}

pub fn fill_slots(slotted_text: &[Slot], topic: &'static str, field: Field) -> TemplateIter {
    let mut vec = Vec::with_capacity(slotted_text.len());
    for slot in slotted_text {
        vec.push(
            match slot {
                Slot::Str(s) => s,
                Slot::Topic => topic,
                Slot::Field => field.str().lower_case,
            }
            .into(),
        );
    }

    TemplateIter::new(vec)
}

use stencila_codec::stencila_schema::{
    CreativeWorkType, Inline, IntegerOrString, Person, PersonOrOrganization, Reference,
    ReferenceOptions, StringOrNumber,
};

/// Create an `is_part_of` value representing a journal
pub fn in_journal(
    title: Vec<Inline>,
    volume_number: Option<IntegerOrString>,
    issue_number: Option<IntegerOrString>,
) -> Option<Box<Reference>> {
    Some(Box::new(Reference {
        work_type: Some(CreativeWorkType::Periodical),
        title: Some(title),
        options: Box::new(ReferenceOptions {
            volume_number,
            issue_number,
            ..Default::default()
        }),
        ..Default::default()
    }))
}

/// Create an `is_part_of` value representing a book
pub fn in_book(
    title: Vec<Inline>,
    editors: Option<Vec<Person>>,
    publisher: Option<PersonOrOrganization>,
    version: Option<StringOrNumber>,
) -> Option<Box<Reference>> {
    Some(Box::new(Reference {
        work_type: Some(CreativeWorkType::Book),
        title: Some(title),
        options: Box::new(ReferenceOptions {
            editors,
            publisher,
            version,
            ..Default::default()
        }),
        ..Default::default()
    }))
}

/// Create an `is_part_of` value representing conference proceedings
pub fn in_proceedings(
    title: Vec<Inline>,
    editors: Option<Vec<Person>>,
    publisher: Option<PersonOrOrganization>,
) -> Option<Box<Reference>> {
    Some(Box::new(Reference {
        work_type: Some(CreativeWorkType::Collection),
        title: Some(title),
        options: Box::new(ReferenceOptions {
            editors,
            publisher,
            ..Default::default()
        }),
        ..Default::default()
    }))
}

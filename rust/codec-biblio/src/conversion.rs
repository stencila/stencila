use hayagriva::{
    Entry,
    types::{
        Date as HDate, EntryType, FormatString, MaybeTyped, Numeric, PageRanges, PageRangesPart,
        Person as HPerson, QualifiedUrl, SerialNumber,
    },
};
use reqwest::Url;

use stencila_codec::{
    eyre::Result,
    stencila_schema::{
        Author, Cord, CreativeWorkType, Date, Inline, IntegerOrString, Organization, Person,
        PersonOrOrganization, PostalAddressOrString, Primitive, PropertyValue,
        PropertyValueOrString, Reference, ReferenceOptions, StringOrNumber, Text,
    },
};
use stencila_codec_text_trait::to_text;

// Constants for repeated string literals
const DOI_KEY: &str = "doi";
const ISBN_KEY: &str = "isbn";
const IDENTIFIER_KEY: &str = "serial";
const PAGES_SUFFIX: &str = " pages";

/// Convert a Hayagriva Entry to a Stencila Reference
pub fn entry_to_reference(entry: &Entry) -> Result<Reference> {
    // Extract work type
    let work_type = entry_type_to_work_type(entry.entry_type());

    // Extract ID
    let id = Some(entry.key().to_string());

    // Extract title
    let title = entry.title().map(|title| {
        vec![Inline::Text(Text {
            value: Cord::from(title.to_string()),
            ..Default::default()
        })]
    });

    // Extract contributors (authors and editors)
    let (authors, editors) = extract_contributors(entry);

    // Extract date
    let date = entry.date().map(|date| {
        let date_string = match (date.month, date.day) {
            (Some(month), Some(day)) => format!("{:04}-{:02}-{:02}", date.year, month, day),
            (Some(month), None) => format!("{:04}-{:02}", date.year, month),
            _ => format!("{:04}", date.year),
        };

        Date {
            value: date_string,
            ..Default::default()
        }
    });

    // Extract DOI - prioritize direct DOI, fallback to serial numbers
    let doi = entry.doi().map(|doi| doi.to_string()).or_else(|| {
        entry
            .serial_number()
            .and_then(|serial_numbers| serial_numbers.0.get(DOI_KEY).cloned())
    });

    // Extract URL
    let url = entry.url().map(|url| url.to_string());

    // Extract container/parent work as a Reference
    let parents = entry.parents();
    let is_part_of = parents.first().and_then(|first_parent| {
        first_parent.title().map(|parent_title| {
            // Create a parent Reference based on the parent entry
            let parent_work_type = entry_type_to_work_type(first_parent.entry_type());

            // Extract contributors (authors and editors) from parent
            let (parent_authors, parent_editors) = extract_contributors(first_parent);

            // Extract date from parent
            let parent_date = first_parent.date().map(|date| {
                let date_string = match (date.month, date.day) {
                    (Some(month), Some(day)) => {
                        format!("{:04}-{:02}-{:02}", date.year, month, day)
                    }
                    (Some(month), None) => format!("{:04}-{:02}", date.year, month),
                    _ => format!("{:04}", date.year),
                };
                Date {
                    value: date_string,
                    ..Default::default()
                }
            });

            // Extract publisher from parent
            let parent_publisher = first_parent.publisher().and_then(|publisher| {
                publisher.name().map(|pub_name| {
                    let mut org = Organization {
                        name: Some(pub_name.to_string()),
                        ..Default::default()
                    };

                    // Add location if available
                    if let Some(location) = first_parent.location() {
                        let location_str = location.to_string();
                        org.options.address = Some(PostalAddressOrString::String(location_str));
                    }

                    PersonOrOrganization::Organization(org)
                })
            });

            // Extract version/edition from parent
            let parent_version = first_parent.edition().map(|edition| {
                let edition_str = maybe_typed_to_string(edition);
                StringOrNumber::String(edition_str)
            });

            // Extract identifiers from parent
            let parent_identifiers = first_parent.serial_number().and_then(|serial_numbers| {
                let mut ids = Vec::new();
                for (key, value) in &serial_numbers.0 {
                    ids.push(create_property_value(key, value));
                }
                (!ids.is_empty()).then_some(ids)
            });

            // Extract URL from parent
            let parent_url = first_parent.url().map(|url| url.to_string());

            // Extract page-total from parent and convert to pagination
            let parent_pagination = first_parent.page_total().map(|total| {
                let total_str = total.to_string();
                format!("{total_str}{PAGES_SUFFIX}")
            });

            // Extract volume and issue numbers for parent
            let parent_volume_number = extract_numeric_field(first_parent.volume());
            let parent_issue_number = extract_numeric_field(first_parent.issue());

            Box::new(Reference {
                work_type: parent_work_type,
                id: Some(format!("{}_parent", entry.key())), // Generate a unique ID for the parent
                title: Some(vec![Inline::Text(Text {
                    value: Cord::from(parent_title.to_string()),
                    ..Default::default()
                })]),
                authors: parent_authors,
                date: parent_date,
                url: parent_url,
                options: Box::new(ReferenceOptions {
                    editors: parent_editors,
                    publisher: parent_publisher,
                    version: parent_version,
                    identifiers: parent_identifiers,
                    pagination: parent_pagination,
                    volume_number: parent_volume_number,
                    issue_number: parent_issue_number,
                    ..Default::default()
                }),
                ..Default::default()
            })
        })
    });

    // Extract page information
    let (page_start, page_end, pagination) =
        entry.page_range().map_or((None, None, None), |page_range| {
            let page_str = maybe_typed_to_string(page_range);

            // Try to parse page range
            if page_str.contains('-') {
                let parts: Vec<&str> = page_str.split('-').collect();
                if parts.len() == 2 {
                    (
                        Some(string_to_integer_or_string(parts[0].trim())),
                        Some(string_to_integer_or_string(parts[1].trim())),
                        None,
                    )
                } else {
                    (None, None, Some(page_str))
                }
            } else {
                (
                    Some(string_to_integer_or_string(page_str.trim())),
                    None,
                    None,
                )
            }
        });

    // Extract publisher (including location)
    let publisher = entry.publisher().and_then(|publisher| {
        publisher.name().map(|pub_name| {
            let mut org = Organization {
                name: Some(pub_name.to_string()),
                ..Default::default()
            };

            // Extract location if available
            if let Some(location) = entry.location() {
                let location_str = location.to_string();
                // Store location in address for now - this is the closest field we have
                org.options.address = Some(PostalAddressOrString::String(location_str));
            }

            PersonOrOrganization::Organization(org)
        })
    });

    // Extract version/edition - always treat as string to preserve formatting
    let version = entry.edition().map(|edition| {
        let edition_str = maybe_typed_to_string(edition);
        StringOrNumber::String(edition_str)
    });

    // Extract volume number (for books, different from publication volume)
    let volume_number = extract_numeric_field(entry.volume());

    // Extract issue number (for serials, different from publication issue)
    let issue_number = extract_numeric_field(entry.issue());

    // Extract identifiers from serial numbers
    let identifiers = entry.serial_number().and_then(|serial_numbers| {
        let mut ids = Vec::new();

        // Add ISBN if present
        if let Some(isbn) = serial_numbers.0.get(ISBN_KEY) {
            ids.push(create_property_value(ISBN_KEY, isbn));
        }

        // Add other serial numbers
        for (key, value) in &serial_numbers.0 {
            if key != ISBN_KEY && key != DOI_KEY {
                // DOI is handled separately
                ids.push(create_property_value(key, value));
            }
        }

        (!ids.is_empty()).then_some(ids)
    });

    // Extract page-total and convert to pagination if no other pagination exists
    let pagination_from_total = entry.page_total().map(|total| {
        let total_str = total.to_string();
        format!("{total_str}{PAGES_SUFFIX}")
    });

    // Use page-total pagination if no other pagination exists
    let final_pagination = pagination.or(pagination_from_total);

    // Construct the Reference
    Ok(Reference {
        work_type,
        id,
        title,
        authors,
        date,
        doi,
        url,
        is_part_of,
        options: Box::new(ReferenceOptions {
            editors,
            page_start,
            page_end,
            pagination: final_pagination,
            publisher,
            version,
            volume_number,
            issue_number,
            identifiers,
            ..Default::default()
        }),
        ..Default::default()
    })
}

/// Get the key for a reference
///
/// Use the reference's id, if available, falling back to it's DOI.
pub fn reference_key(reference: &Reference) -> Option<&str> {
    reference.id.as_deref().or(reference.doi.as_deref())
}

/// Convert a Stencila Reference to a Hayagriva Entry
pub fn reference_to_entry(reference: &Reference) -> Result<Entry> {
    let key = reference_key(reference).unwrap_or("ref");

    // Determine entry type based on kind field first, then is_part_of
    let entry_type = if let Some(work_type) = &reference.work_type {
        work_type_to_entry_type(work_type)
    } else if let Some(container) = &reference.is_part_of {
        // Fallback to determining from container type
        match container.work_type {
            Some(CreativeWorkType::Periodical) => EntryType::Article,
            _ => EntryType::Misc,
        }
    } else {
        EntryType::Misc
    };

    let mut entry = Entry::new(key, entry_type);

    // Set title
    if let Some(title) = &reference.title {
        let title_str = title.iter().map(to_text).collect::<Vec<_>>().join("");
        entry.set_title(FormatString::from(title_str));
    }

    // Set contributors (authors and editors)
    set_contributors(&mut entry, reference);

    // Set date
    if let Some(date) = &reference.date
        && let Some(parsed_date) = parse_iso_date(&date.value)
    {
        entry.set_date(parsed_date);
    }

    // Set DOI
    if let Some(doi) = &reference.doi {
        entry.set_doi(doi.to_string());
    }

    // Set URL
    if let Some(url) = reference.url.as_ref().and_then(|url| Url::parse(url).ok()) {
        entry.set_url(QualifiedUrl::new(url, None));
    }

    // Handle is_part_of (container/parent work) - simplified since it's now a Reference
    if let Some(container) = &reference.is_part_of {
        handle_reference_container(&mut entry, container.as_ref());
    }

    // Set publisher and location
    if let Some(publisher) = &reference.options.publisher {
        match publisher {
            PersonOrOrganization::Organization(org) => {
                if let Some(pub_name) = &org.name {
                    let publisher = hayagriva::types::Publisher::new(
                        Some(FormatString::from(pub_name.clone())),
                        None,
                    );
                    entry.set_publisher(publisher);

                    // Set location if available in address field
                    if let Some(address) = &org.options.address {
                        let location_str = match address {
                            PostalAddressOrString::String(s) => s.clone(),
                            PostalAddressOrString::PostalAddress(addr) => {
                                // Extract street address or locality as location
                                addr.street_address
                                    .clone()
                                    .or_else(|| addr.address_locality.clone())
                                    .unwrap_or_default()
                            }
                        };
                        if !location_str.is_empty() {
                            entry.set_location(FormatString::from(location_str));
                        }
                    }
                }
            }
            PersonOrOrganization::Person(person) => {
                // Handle person publisher - combine names
                let name_str = format_person_name(&person.family_names, &person.given_names);

                if !name_str.is_empty() {
                    let publisher =
                        hayagriva::types::Publisher::new(Some(FormatString::from(name_str)), None);
                    entry.set_publisher(publisher);
                }
            }
        }
    }

    // Set version/edition
    if let Some(version) = &reference.options.version {
        let edition_str = match version {
            StringOrNumber::String(s) => s.clone(),
            StringOrNumber::Number(n) => n.to_string(),
        };
        entry.set_edition(MaybeTyped::String(edition_str));
    }

    // Set volume number (for books)
    if let Some(volume_number) = &reference.options.volume_number {
        entry.set_volume(integer_or_string_to_maybe_numeric(volume_number));
    }

    // Set issue number (for serials)
    if let Some(issue_number) = &reference.options.issue_number {
        entry.set_issue(integer_or_string_to_maybe_numeric(issue_number));
    }

    // Set page numbers - prioritize page-total over pagination for total pages
    if let Some(pagination) = &reference.options.pagination {
        // Check if pagination is in "X pages" format for page-total
        if pagination.ends_with(PAGES_SUFFIX) {
            let page_count = pagination.trim_end_matches(PAGES_SUFFIX);
            if let Ok(count) = page_count.parse::<i32>() {
                entry.set_page_total(Numeric::new(count));
            } else {
                entry.set_page_range(MaybeTyped::String(pagination.clone()));
            }
        } else {
            entry.set_page_range(MaybeTyped::String(pagination.clone()));
        }
    } else {
        match (&reference.options.page_start, &reference.options.page_end) {
            (Some(start), Some(end)) => {
                let page_str = format!(
                    "{}-{}",
                    integer_or_string_to_string(start),
                    integer_or_string_to_string(end)
                );
                entry.set_page_range(MaybeTyped::String(page_str));
            }
            (Some(start), None) => {
                entry.set_page_range(integer_or_string_to_maybe_page_ranges(start));
            }
            _ => {}
        }
    }

    // Convert identifiers and DOI to serial numbers
    let mut serial_map = std::collections::BTreeMap::new();

    // Add DOI if present
    if let Some(doi) = &reference.doi {
        serial_map.insert(DOI_KEY.to_string(), doi.clone());
    }

    // Add identifiers from the identifiers field
    if let Some(identifiers) = &reference.options.identifiers {
        for identifier in identifiers {
            match identifier {
                PropertyValueOrString::PropertyValue(prop_val) => {
                    if let Some(property_id) = &prop_val.property_id
                        && let Primitive::String(value) = &prop_val.value
                    {
                        serial_map.insert(property_id.clone(), value.clone());
                    }
                }
                PropertyValueOrString::String(s) => {
                    // For string identifiers without property ID, use generic key
                    serial_map.insert(IDENTIFIER_KEY.to_string(), s.clone());
                }
            }
        }
    }

    // Set serial numbers if we have any
    if !serial_map.is_empty() {
        let serial_numbers = SerialNumber(serial_map);
        entry.set_serial_number(serial_numbers);
    }

    Ok(entry)
}

// Helper functions for common patterns

/// Create a PropertyValue for identifiers
fn create_property_value(property_id: &str, value: &str) -> PropertyValueOrString {
    PropertyValueOrString::PropertyValue(PropertyValue {
        property_id: Some(property_id.to_string()),
        value: Primitive::String(value.to_string()),
        ..Default::default()
    })
}

/// Extract a numeric field from a Hayagriva entry
fn extract_numeric_field<T: ToString>(value: Option<&MaybeTyped<T>>) -> Option<IntegerOrString> {
    value.map(|v| {
        let str_value = maybe_typed_to_string(v);
        string_to_integer_or_string(&str_value)
    })
}

/// Format person names for display
fn format_person_name(
    family_names: &Option<Vec<String>>,
    given_names: &Option<Vec<String>>,
) -> String {
    let family = family_names
        .as_ref()
        .map(|names| names.join(" "))
        .unwrap_or_default();
    let given = given_names
        .as_ref()
        .map(|names| names.join(" "))
        .unwrap_or_default();

    if family.is_empty() {
        String::new()
    } else if given.is_empty() {
        family
    } else {
        format!("{given} {family}")
    }
}

// Person conversion helpers

/// Convert a Hayagriva person to a Stencila Person
fn hayagriva_person_to_stencila_person(person: &HPerson) -> Person {
    let family = person.name.as_str();
    let given = person.given_name.as_deref().unwrap_or("");

    let given_names = if given.is_empty() {
        None
    } else {
        Some(
            given
                .split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        )
    };

    Person {
        family_names: Some(vec![family.to_string()]),
        given_names,
        ..Default::default()
    }
}

/// Convert a Hayagriva person to a Stencila Author
fn hayagriva_person_to_stencila_author(person: &HPerson) -> Author {
    Author::Person(hayagriva_person_to_stencila_person(person))
}

/// Convert a Stencila Person to a Hayagriva person
fn stencila_person_to_hayagriva_person(person: &Person) -> Option<HPerson> {
    let family = person
        .family_names
        .as_ref()
        .map(|names| names.join(" "))
        .unwrap_or_default();

    if family.is_empty() {
        return None;
    }

    let given_name = person
        .given_names
        .as_ref()
        .filter(|names| !names.is_empty())
        .map(|names| names.join(" "));

    Some(HPerson {
        name: family,
        given_name,
        prefix: None,
        suffix: None,
        alias: None,
    })
}

/// Convert a Stencila Author to a Hayagriva person
fn stencila_author_to_hayagriva_person(author: &Author) -> Option<HPerson> {
    match author {
        Author::Person(person) => stencila_person_to_hayagriva_person(person),
        Author::Organization(org) => org.name.as_ref().map(|name| HPerson {
            name: name.clone(),
            given_name: None,
            prefix: None,
            suffix: None,
            alias: None,
        }),
        _ => None,
    }
}

/// Extract contributors (authors and editors) from a Hayagriva entry
fn extract_contributors(entry: &Entry) -> (Option<Vec<Author>>, Option<Vec<Person>>) {
    let authors = entry.authors().and_then(|authors| {
        let ref_authors: Vec<Author> = authors
            .iter()
            .filter(|person| !person.name.is_empty())
            .map(hayagriva_person_to_stencila_author)
            .collect();
        (!ref_authors.is_empty()).then_some(ref_authors)
    });

    let editors = entry.editors().and_then(|editors| {
        let ref_editors: Vec<Person> = editors
            .iter()
            .filter(|person| !person.name.is_empty())
            .map(hayagriva_person_to_stencila_person)
            .collect();
        (!ref_editors.is_empty()).then_some(ref_editors)
    });

    (authors, editors)
}

/// Set contributors (authors and editors) on a Hayagriva entry
fn set_contributors(entry: &mut Entry, reference: &Reference) {
    if let Some(authors) = &reference.authors {
        let persons: Vec<HPerson> = authors
            .iter()
            .filter_map(stencila_author_to_hayagriva_person)
            .collect();

        if !persons.is_empty() {
            entry.set_authors(persons);
        }
    }

    if let Some(editors) = &reference.options.editors {
        let persons: Vec<HPerson> = editors
            .iter()
            .filter_map(stencila_person_to_hayagriva_person)
            .collect();

        if !persons.is_empty() {
            entry.set_editors(persons);
        }
    }
}

/// Convert Hayagriva [`EntryType`] to Stencila [`CreativeWorkType`]
fn entry_type_to_work_type(entry_type: &EntryType) -> Option<CreativeWorkType> {
    Some(match entry_type {
        EntryType::Anthology => CreativeWorkType::Collection,
        EntryType::Anthos => CreativeWorkType::Article,
        EntryType::Article => CreativeWorkType::Article,
        EntryType::Artwork => CreativeWorkType::ImageObject,
        EntryType::Audio => CreativeWorkType::AudioObject,
        EntryType::Blog => CreativeWorkType::Blog,
        EntryType::Book => CreativeWorkType::Book,
        EntryType::Case => CreativeWorkType::Article,
        EntryType::Chapter => CreativeWorkType::Chapter,
        EntryType::Conference => CreativeWorkType::Presentation,
        EntryType::Exhibition => CreativeWorkType::Collection,
        EntryType::Legislation => CreativeWorkType::Legislation,
        EntryType::Manuscript => CreativeWorkType::Manuscript,
        EntryType::Newspaper => CreativeWorkType::Periodical,
        EntryType::Patent => CreativeWorkType::Article,
        EntryType::Performance => CreativeWorkType::Presentation,
        EntryType::Periodical => CreativeWorkType::Periodical,
        EntryType::Post => CreativeWorkType::Comment,
        EntryType::Proceedings => CreativeWorkType::Collection,
        EntryType::Reference => CreativeWorkType::Book,
        EntryType::Report => CreativeWorkType::Report,
        EntryType::Repository => CreativeWorkType::SoftwareRepository,
        EntryType::Scene => CreativeWorkType::VideoObject,
        EntryType::Thesis => CreativeWorkType::Thesis,
        EntryType::Thread => CreativeWorkType::Comment,
        EntryType::Video => CreativeWorkType::VideoObject,
        EntryType::Web => CreativeWorkType::WebPage,
        _ => return None,
    })
}

/// Convert Stencila [`CreativeWorkType`] to Hayagriva [`EntryType`]
fn work_type_to_entry_type(work_type: &CreativeWorkType) -> EntryType {
    match work_type {
        CreativeWorkType::Article => EntryType::Article,
        CreativeWorkType::AudioObject => EntryType::Audio,
        CreativeWorkType::Blog => EntryType::Blog,
        CreativeWorkType::Book => EntryType::Book,
        CreativeWorkType::Chapter => EntryType::Chapter,
        CreativeWorkType::Collection => EntryType::Proceedings,
        CreativeWorkType::Comment => EntryType::Post,
        CreativeWorkType::Drawing => EntryType::Artwork,
        CreativeWorkType::ImageObject => EntryType::Artwork,
        CreativeWorkType::Legislation => EntryType::Legislation,
        CreativeWorkType::Manuscript => EntryType::Manuscript,
        CreativeWorkType::Map => EntryType::Artwork,
        CreativeWorkType::Periodical => EntryType::Periodical,
        CreativeWorkType::Photograph => EntryType::Artwork,
        CreativeWorkType::Poster => EntryType::Artwork,
        CreativeWorkType::Presentation => EntryType::Conference,
        CreativeWorkType::Report => EntryType::Report,
        CreativeWorkType::Review => EntryType::Article,
        CreativeWorkType::SoftwareApplication => EntryType::Repository,
        CreativeWorkType::SoftwareRepository => EntryType::Repository,
        CreativeWorkType::SoftwareSourceCode => EntryType::Repository,
        CreativeWorkType::Thesis => EntryType::Thesis,
        CreativeWorkType::VideoObject => EntryType::Video,
        CreativeWorkType::WebPage => EntryType::Web,
        // No direct mapping for these types
        CreativeWorkType::Chat
        | CreativeWorkType::Claim
        | CreativeWorkType::Dataset
        | CreativeWorkType::Datatable
        | CreativeWorkType::Figure
        | CreativeWorkType::Agent
        | CreativeWorkType::File
        | CreativeWorkType::MediaObject
        | CreativeWorkType::Prompt
        | CreativeWorkType::PublicationIssue
        | CreativeWorkType::PublicationVolume
        | CreativeWorkType::Skill
        | CreativeWorkType::Table
        | CreativeWorkType::Workflow => EntryType::Misc,
    }
}

/// Convert IntegerOrString to MaybeTyped<Numeric> for Hayagriva
fn integer_or_string_to_maybe_numeric(value: &IntegerOrString) -> MaybeTyped<Numeric> {
    match value {
        IntegerOrString::Integer(i) => MaybeTyped::Typed(Numeric::new(*i as i32)),
        IntegerOrString::String(s) => MaybeTyped::String(s.clone()),
    }
}

/// Convert IntegerOrString to MaybeTyped<PageRanges> for Hayagriva page ranges
fn integer_or_string_to_maybe_page_ranges(value: &IntegerOrString) -> MaybeTyped<PageRanges> {
    match value {
        IntegerOrString::Integer(i) => {
            MaybeTyped::Typed(PageRanges::new(vec![PageRangesPart::SinglePage(
                Numeric::new(*i as i32),
            )]))
        }
        IntegerOrString::String(s) => MaybeTyped::String(s.clone()),
    }
}

/// Convert IntegerOrString to String
fn integer_or_string_to_string(value: &IntegerOrString) -> String {
    match value {
        IntegerOrString::Integer(i) => i.to_string(),
        IntegerOrString::String(s) => s.clone(),
    }
}

/// Convert string to IntegerOrString, attempting numeric parsing first
fn string_to_integer_or_string(value: &str) -> IntegerOrString {
    if let Ok(i) = value.parse::<i64>() {
        IntegerOrString::Integer(i)
    } else {
        IntegerOrString::String(value.to_string())
    }
}

/// Convert MaybeTyped<T> to String where T: ToString
fn maybe_typed_to_string<T: ToString>(value: &MaybeTyped<T>) -> String {
    match value {
        MaybeTyped::Typed(t) => t.to_string(),
        MaybeTyped::String(s) => s.clone(),
    }
}

/// Parse ISO date string into Hayagriva Date
fn parse_iso_date(date_str: &str) -> Option<HDate> {
    // Try to parse the string as a year first
    if let Ok(year) = date_str.parse::<i32>() {
        return Some(HDate::from_year(year));
    }

    // Try to parse as ISO date (YYYY-MM-DD or YYYY-MM)
    let parts: Vec<&str> = date_str.split('-').collect();
    match parts.len() {
        1 => {
            if let Ok(year) = parts[0].parse::<i32>() {
                Some(HDate::from_year(year))
            } else {
                None
            }
        }
        2 | 3 => {
            if let Ok(year) = parts[0].parse::<i32>() {
                let mut date = HDate::from_year(year);
                if parts.len() >= 2
                    && let Ok(month) = parts[1].parse::<u8>()
                {
                    date.month = Some(month);
                }
                if parts.len() >= 3
                    && let Ok(day) = parts[2].parse::<u8>()
                {
                    date.day = Some(day);
                }
                Some(date)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Handle reference container and set parent information on entry
fn handle_reference_container(entry: &mut Entry, container: &Reference) {
    // Create a parent entry from the Reference
    if let Some(container_title) = &container.title {
        let title_str = container_title
            .iter()
            .map(to_text)
            .collect::<Vec<_>>()
            .join("");

        let parent_type = container
            .work_type
            .as_ref()
            .map(work_type_to_entry_type)
            .unwrap_or(EntryType::Misc);

        let mut parent = Entry::new("parent", parent_type);
        parent.set_title(FormatString::from(title_str));

        // Set contributors
        set_contributors(&mut parent, container);

        // Set date
        if let Some(date) = &container.date
            && let Some(parsed_date) = parse_iso_date(&date.value)
        {
            parent.set_date(parsed_date);
        }

        // Set publisher
        if let Some(publisher) = &container.options.publisher {
            match publisher {
                PersonOrOrganization::Organization(org) => {
                    if let Some(pub_name) = &org.name {
                        let pub_obj = hayagriva::types::Publisher::new(
                            Some(FormatString::from(pub_name.clone())),
                            None,
                        );
                        parent.set_publisher(pub_obj);

                        // Set location if available
                        if let Some(address) = &org.options.address {
                            let location_str = match address {
                                PostalAddressOrString::String(s) => s.clone(),
                                PostalAddressOrString::PostalAddress(addr) => addr
                                    .street_address
                                    .clone()
                                    .or_else(|| addr.address_locality.clone())
                                    .unwrap_or_default(),
                            };
                            if !location_str.is_empty() {
                                parent.set_location(FormatString::from(location_str));
                            }
                        }
                    }
                }
                PersonOrOrganization::Person(_) => {
                    // Handle person publisher if needed
                }
            }
        }

        // Set edition/version if available
        if let Some(version) = &container.options.version {
            let edition = match version {
                StringOrNumber::String(s) => MaybeTyped::String(s.clone()),
                StringOrNumber::Number(n) => MaybeTyped::Typed(Numeric::new(*n as i32)),
            };
            parent.set_edition(edition);
        }

        // Set volume and issue if available - ensure they're stored as integers when possible
        if let Some(volume_number) = &container.options.volume_number {
            match volume_number {
                IntegerOrString::Integer(i) => {
                    parent.set_volume(MaybeTyped::Typed(Numeric::new(*i as i32)))
                }
                IntegerOrString::String(s) => {
                    if let Ok(i) = s.parse::<i32>() {
                        parent.set_volume(MaybeTyped::Typed(Numeric::new(i)));
                    } else {
                        parent.set_volume(MaybeTyped::String(s.clone()));
                    }
                }
            }
        }

        if let Some(issue_number) = &container.options.issue_number {
            match issue_number {
                IntegerOrString::Integer(i) => {
                    parent.set_issue(MaybeTyped::Typed(Numeric::new(*i as i32)))
                }
                IntegerOrString::String(s) => {
                    if let Ok(i) = s.parse::<i32>() {
                        parent.set_issue(MaybeTyped::Typed(Numeric::new(i)));
                    } else {
                        parent.set_issue(MaybeTyped::String(s.clone()));
                    }
                }
            }
        }

        // Set page-total if pagination is available
        if let Some(pagination) = &container.options.pagination
            && pagination.ends_with(PAGES_SUFFIX)
        {
            let page_count = pagination.trim_end_matches(PAGES_SUFFIX);
            if let Ok(count) = page_count.parse::<i32>() {
                parent.set_page_total(Numeric::new(count));
            }
        }

        // Set identifiers (ISBN, etc.)
        if let Some(identifiers) = &container.options.identifiers {
            let mut serial_numbers = std::collections::BTreeMap::new();
            for identifier in identifiers {
                if let PropertyValueOrString::PropertyValue(prop_val) = identifier
                    && let Some(prop_id) = &prop_val.property_id
                    && let Primitive::String(value) = &prop_val.value
                {
                    if prop_id.to_lowercase() == "isbn" {
                        serial_numbers.insert(ISBN_KEY.to_string(), value.clone());
                    } else {
                        serial_numbers.insert(prop_id.clone(), value.clone());
                    }
                }
            }
            if !serial_numbers.is_empty() {
                use hayagriva::types::SerialNumber;
                parent.set_serial_number(SerialNumber(serial_numbers));
            }
        }

        // Set URL if available
        if let Some(url_str) = &container.url
            && let Ok(url) = Url::parse(url_str)
        {
            parent.set_url(QualifiedUrl::new(url, None));
        }

        entry.set_parents(vec![parent]);
    }
}

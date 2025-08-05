use codec::{
    common::{eyre::Result, reqwest::Url},
    schema::{
        Author, Cord, CreativeWorkType, CreativeWorkVariant, Date, Inline, IntegerOrString,
        Organization, Periodical, PeriodicalOptions, Person, PersonOrOrganization,
        PostalAddressOrString, PropertyValue, PropertyValueOrString, PublicationIssue,
        PublicationVolume, Reference, StringOrNumber, Text,
    },
};
use hayagriva::{
    Entry,
    types::{
        Date as HDate, EntryType, FormatString, MaybeTyped, Numeric, PageRanges, PageRangesPart,
        Person as HPerson, QualifiedUrl,
    },
};

// Constants for repeated string literals
const DOI_KEY: &str = "doi";
const ISBN_KEY: &str = "isbn";
const IDENTIFIER_KEY: &str = "identifier";
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

    // Extract container/journal from parents and volume/issue
    // Note: Hayagriva stores volume/issue at the parent level for periodicals
    let parents = entry.parents();
    let is_part_of = parents.first().and_then(|first_parent| {
        let parent_type = first_parent.entry_type();

        // Extract volume/issue from parent (for periodicals) or from entry (for other types)
        let (volume, issue) = if matches!(parent_type, EntryType::Periodical | EntryType::Newspaper)
        {
            // For periodicals, volume/issue are typically stored in the parent
            (first_parent.volume(), first_parent.issue())
        } else {
            // For other types, check the entry itself
            (entry.volume(), entry.issue())
        };

        first_parent.title().map(|journal_name| {
            let publisher = first_parent.publisher().and_then(|publisher| {
                publisher.name().map(|pub_name| {
                    PersonOrOrganization::Organization(Organization {
                        name: Some(pub_name.to_string()),
                        ..Default::default()
                    })
                })
            });

            let periodical = Periodical {
                name: Some(journal_name.to_string()),
                options: Box::new(PeriodicalOptions {
                    publisher,
                    ..Default::default()
                }),
                ..Default::default()
            };

            // Build the publication hierarchy
            let mut base: CreativeWorkVariant = CreativeWorkVariant::Periodical(periodical);

            // Add volume layer if present
            if let Some(vol) = volume {
                let volume_str = maybe_typed_to_string(vol);

                let pub_vol = PublicationVolume {
                    is_part_of: Some(Box::new(base)),
                    volume_number: Some(IntegerOrString::String(volume_str)),
                    ..Default::default()
                };
                base = CreativeWorkVariant::PublicationVolume(pub_vol);
            }

            // Add issue layer if present
            if let Some(issue) = issue {
                let issue_str = maybe_typed_to_string(issue);

                let pub_issue = PublicationIssue {
                    is_part_of: Some(Box::new(base)),
                    issue_number: Some(IntegerOrString::String(issue_str)),
                    ..Default::default()
                };
                base = CreativeWorkVariant::PublicationIssue(pub_issue);
            }

            Box::new(base)
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
        editors,
        date,
        doi,
        url,
        is_part_of,
        page_start,
        page_end,
        pagination: final_pagination,
        publisher,
        version,
        volume_number,
        issue_number,
        identifiers,
        ..Default::default()
    })
}

/// Convert a Stencila Reference to a Hayagriva Entry
pub fn reference_to_entry(reference: &Reference) -> Result<Entry> {
    let key = reference.id.clone().unwrap_or_else(|| "ref".to_string());

    // Determine entry type based on kind field first, then is_part_of
    let entry_type = if let Some(work_type) = &reference.work_type {
        work_type_to_entry_type(work_type)
    } else if let Some(container) = &reference.is_part_of {
        // Fallback to determining from container type
        match container.as_ref() {
            CreativeWorkVariant::Periodical(_) => EntryType::Article,
            _ => EntryType::Misc,
        }
    } else {
        EntryType::Misc
    };

    let mut entry = Entry::new(&key, entry_type);

    // Set title
    if let Some(title) = &reference.title {
        let title_str = title
            .iter()
            .map(inline_to_string)
            .collect::<Vec<_>>()
            .join("");
        entry.set_title(FormatString::from(title_str));
    }

    // Set contributors (authors and editors)
    set_contributors(&mut entry, reference);

    // Set date
    if let Some(date) = &reference.date {
        if let Some(parsed_date) = parse_iso_date(&date.value) {
            entry.set_date(parsed_date);
        }
    }

    // Set DOI
    if let Some(doi) = &reference.doi {
        entry.set_doi(doi.to_string());
    }

    // Set URL
    if let Some(url) = reference.url.as_ref().and_then(|url| Url::parse(url).ok()) {
        entry.set_url(QualifiedUrl::new(url, None));
    }

    // Handle is_part_of (container/parent work) and extract volume/issue
    if let Some(container) = &reference.is_part_of {
        handle_periodical_container(&mut entry, container.as_ref());
    }

    // Set publisher and location
    if let Some(publisher) = &reference.publisher {
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
    if let Some(version) = &reference.version {
        let edition_str = match version {
            StringOrNumber::String(s) => s.clone(),
            StringOrNumber::Number(n) => n.to_string(),
        };
        entry.set_edition(MaybeTyped::String(edition_str));
    }

    // Set volume number (for books)
    if let Some(volume_number) = &reference.volume_number {
        entry.set_volume(integer_or_string_to_maybe_numeric(volume_number));
    }

    // Set issue number (for serials)
    if let Some(issue_number) = &reference.issue_number {
        entry.set_issue(integer_or_string_to_maybe_numeric(issue_number));
    }

    // Set page numbers - prioritize page-total over pagination for total pages
    if let Some(pagination) = &reference.pagination {
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
        match (&reference.page_start, &reference.page_end) {
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
    if let Some(identifiers) = &reference.identifiers {
        for identifier in identifiers {
            match identifier {
                PropertyValueOrString::PropertyValue(prop_val) => {
                    if let Some(property_id) = &prop_val.property_id {
                        if let codec::schema::Primitive::String(value) = &prop_val.value {
                            serial_map.insert(property_id.clone(), value.clone());
                        }
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
        let serial_numbers = hayagriva::types::SerialNumber(serial_map);
        entry.set_serial_number(serial_numbers);
    }

    Ok(entry)
}

// Helper functions for common patterns

/// Create a PropertyValue for identifiers
/// 
/// Creates a PropertyValue with the given property ID and string value.
fn create_property_value(property_id: &str, value: &str) -> PropertyValueOrString {
    PropertyValueOrString::PropertyValue(PropertyValue {
        property_id: Some(property_id.to_string()),
        value: codec::schema::Primitive::String(value.to_string()),
        ..Default::default()
    })
}

/// Extract a numeric field from a Hayagriva entry
/// 
/// Converts a MaybeTyped value to IntegerOrString, handling both typed and string variants.
fn extract_numeric_field<T: ToString>(value: Option<&MaybeTyped<T>>) -> Option<IntegerOrString> {
    value.map(|v| {
        let str_value = maybe_typed_to_string(v);
        string_to_integer_or_string(&str_value)
    })
}

/// Format person names for display
/// 
/// Combines family and given names with appropriate formatting.
/// Returns formatted name or empty string if no family name exists.
fn format_person_name(family_names: &Option<Vec<String>>, given_names: &Option<Vec<String>>) -> String {
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
/// 
/// Extracts family and given names from Hayagriva's person format and converts
/// them to Stencila's Person structure with separate family_names and given_names fields.
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
/// 
/// Wraps the person conversion in an Author::Person variant.
fn hayagriva_person_to_stencila_author(person: &HPerson) -> Author {
    Author::Person(hayagriva_person_to_stencila_person(person))
}

/// Convert a Stencila Person to a Hayagriva person
/// 
/// Combines family and given names into the format expected by Hayagriva.
/// Returns None if the person has no family name.
fn stencila_person_to_hayagriva_person(person: &Person) -> Option<HPerson> {
    let family = person
        .family_names
        .as_ref()
        .map(|names| names.join(" "))
        .unwrap_or_default();
    let given = person
        .given_names
        .as_ref()
        .map(|names| names.join(" "))
        .unwrap_or_default();

    if !family.is_empty() {
        let name_str = if given.is_empty() {
            family
        } else {
            format!("{family}, {given}")
        };
        HPerson::from_strings(vec![&name_str]).ok()
    } else {
        None
    }
}

/// Convert a Stencila Author to a Hayagriva person
/// 
/// Handles both Person and Organization authors, converting organizations
/// to person entries using the organization name.
fn stencila_author_to_hayagriva_person(author: &Author) -> Option<HPerson> {
    match author {
        Author::Person(person) => stencila_person_to_hayagriva_person(person),
        Author::Organization(org) => org
            .name
            .as_ref()
            .and_then(|name| HPerson::from_strings(vec![name]).ok()),
        _ => None,
    }
}

/// Extract contributors (authors and editors) from a Hayagriva entry
/// 
/// Returns a tuple of (authors, editors) converted to Stencila format.
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
/// 
/// Converts Stencila authors and editors to Hayagriva format and sets them on the entry.
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

    if let Some(editors) = &reference.editors {
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
        CreativeWorkType::Collection => EntryType::Anthology,
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
        | CreativeWorkType::MediaObject
        | CreativeWorkType::Prompt
        | CreativeWorkType::PublicationIssue
        | CreativeWorkType::PublicationVolume
        | CreativeWorkType::Table
        | CreativeWorkType::Workflow => EntryType::Misc,
    }
}

/// Convert IntegerOrString to MaybeTyped<Numeric> for Hayagriva
///
/// Hayagriva uses MaybeTyped<Numeric> for fields like volume and issue numbers.
/// This helper converts our IntegerOrString type to the appropriate MaybeTyped variant.
fn integer_or_string_to_maybe_numeric(value: &IntegerOrString) -> MaybeTyped<Numeric> {
    match value {
        IntegerOrString::Integer(i) => MaybeTyped::Typed(Numeric::new(*i as i32)),
        IntegerOrString::String(s) => MaybeTyped::String(s.clone()),
    }
}

/// Convert IntegerOrString to MaybeTyped<PageRanges> for Hayagriva page ranges
///
/// When setting page ranges in Hayagriva, we need to convert our IntegerOrString
/// to either a structured PageRanges object (for numeric pages) or keep as string.
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
///
/// Simple utility to convert our IntegerOrString type to a plain String,
/// used when we need string representations for page ranges or other fields.
fn integer_or_string_to_string(value: &IntegerOrString) -> String {
    match value {
        IntegerOrString::Integer(i) => i.to_string(),
        IntegerOrString::String(s) => s.clone(),
    }
}

/// Convert string to IntegerOrString, attempting numeric parsing first
///
/// Tries to parse the string as an integer first, and if that fails,
/// stores it as a string. This is useful for page numbers and other fields
/// that might be either numeric or contain non-numeric characters.
fn string_to_integer_or_string(value: &str) -> IntegerOrString {
    if let Ok(i) = value.parse::<i64>() {
        IntegerOrString::Integer(i)
    } else {
        IntegerOrString::String(value.to_string())
    }
}

/// Convert MaybeTyped<T> to String where T: ToString
///
/// Hayagriva uses MaybeTyped<T> for fields that can be either structured data
/// or raw strings. This helper extracts a string representation from either variant.
/// Used for volume numbers, issue numbers, and page ranges.
fn maybe_typed_to_string<T: ToString>(value: &MaybeTyped<T>) -> String {
    match value {
        MaybeTyped::Typed(t) => t.to_string(),
        MaybeTyped::String(s) => s.clone(),
    }
}

/// Determine Hayagriva EntryType from journal/periodical name patterns
///
/// Analyzes the journal name to determine the most appropriate Hayagriva EntryType.
/// This helps with proper citation formatting by distinguishing between:
/// - Proceedings (conference proceedings)
/// - Repository (preprint servers like arXiv)
/// - Periodical (regular journals, default case)
///
/// # Arguments
/// * `journal_name` - The name of the journal/periodical to analyze
///
/// # Returns
/// The most appropriate EntryType based on name patterns
fn determine_journal_entry_type(journal_name: &str) -> EntryType {
    let journal_lower = journal_name.to_lowercase();
    if journal_lower.starts_with("proceedings") {
        EntryType::Proceedings
    } else if matches!(
        journal_lower.as_str(),
        "arxiv" | "biorxiv" | "medrxiv" | "chemrxiv" | "peerj preprints"
    ) {
        EntryType::Repository
    } else {
        EntryType::Periodical
    }
}

/// Extract publisher from periodical options and convert to Hayagriva Publisher
///
/// Extracts publisher information from a Stencila Periodical and converts it
/// to the format expected by Hayagriva. Only handles Organization publishers
/// (not Person publishers) as this is the most common case for academic publishing.
///
/// # Arguments
/// * `periodical` - The Stencila Periodical to extract publisher from
///
/// # Returns
/// Some(Publisher) if an organization publisher is found, None otherwise
fn extract_publisher_from_periodical(
    periodical: &Periodical,
) -> Option<hayagriva::types::Publisher> {
    if let Some(PersonOrOrganization::Organization(org)) = &periodical.options.publisher {
        if let Some(pub_name) = &org.name {
            return Some(hayagriva::types::Publisher::new(
                Some(FormatString::from(pub_name.clone())),
                None,
            ));
        }
    }
    None
}

/// Create a Hayagriva parent entry for a journal/periodical
///
/// Creates a complete Hayagriva Entry representing the parent publication
/// (journal, conference proceedings, etc.) with appropriate type, title, and
/// publisher information. This is used as the parent entry for articles.
///
/// # Arguments
/// * `journal_name` - The name of the journal/periodical
/// * `periodical` - The Stencila Periodical containing additional metadata
///
/// # Returns
/// A fully configured Hayagriva Entry representing the parent publication
fn create_hayagriva_parent_entry(journal_name: &str, periodical: &Periodical) -> Entry {
    let parent_type = determine_journal_entry_type(journal_name);
    let mut parent = Entry::new(journal_name, parent_type);
    parent.set_title(FormatString::from(journal_name.to_string()));

    if let Some(publisher) = extract_publisher_from_periodical(periodical) {
        parent.set_publisher(publisher);
    }

    parent
}

/// Convert inline content to string representation
///
/// Converts Stencila Inline content to a plain string. Text content is extracted
/// directly, while other inline types (like emphasis, links, etc.) are converted
/// to their debug representation as a fallback. This is primarily used for
/// extracting title text for bibliography entries.
///
/// # Arguments
/// * `inline` - The Inline content to convert
///
/// # Returns
/// String representation of the inline content
fn inline_to_string(inline: &Inline) -> String {
    match inline {
        Inline::Text(Text { value, .. }) => value.to_string(),
        _ => format!("{inline:?}"),
    }
}

/// Parse ISO date string into Hayagriva Date
///
/// Parses various date string formats into Hayagriva's Date structure.
/// Supports:
/// - Year only: "2023"
/// - Year-month: "2023-03"
/// - Full ISO date: "2023-03-15"
///
/// Returns None for invalid date strings or formats that can't be parsed.
///
/// # Arguments
/// * `date_str` - The date string to parse
///
/// # Returns
/// Some(HDate) if parsing succeeds, None otherwise
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
                if parts.len() >= 2 {
                    if let Ok(month) = parts[1].parse::<u8>() {
                        date.month = Some(month);
                    }
                }
                if parts.len() >= 3 {
                    if let Ok(day) = parts[2].parse::<u8>() {
                        date.day = Some(day);
                    }
                }
                Some(date)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Handle periodical container and set volume/issue information on entry
///
/// Processes different types of publication containers (Periodical, PublicationVolume,
/// PublicationIssue) and sets up the appropriate parent relationships and metadata
/// in the Hayagriva entry. This consolidates the logic that was previously duplicated
/// across three large match arms.
///
/// The function handles the hierarchical nature of academic publishing:
/// - Periodical: Simple journal with no volume/issue
/// - PublicationVolume: Journal with volume information
/// - PublicationIssue: Journal with both volume and issue information
///
/// Volume and issue numbers are set either on the parent entry (for periodicals)
/// or on the main entry (for other publication types like proceedings).
///
/// # Arguments
/// * `entry` - The Hayagriva entry to modify (mutable reference)
/// * `container` - The publication container to process
fn handle_periodical_container(entry: &mut Entry, container: &CreativeWorkVariant) {
    match container {
        CreativeWorkVariant::Periodical(periodical) => {
            if let Some(journal_name) = &periodical.name {
                let parent = create_hayagriva_parent_entry(journal_name, periodical);
                entry.set_parents(vec![parent]);
            }
        }
        CreativeWorkVariant::PublicationVolume(pub_vol) => {
            let vol_num = pub_vol.volume_number.as_ref();

            if let Some(CreativeWorkVariant::Periodical(periodical)) =
                pub_vol.is_part_of.as_ref().map(|b| b.as_ref())
            {
                if let Some(journal_name) = &periodical.name {
                    let mut parent = create_hayagriva_parent_entry(journal_name, periodical);
                    let parent_type = determine_journal_entry_type(journal_name);

                    if parent_type == EntryType::Periodical {
                        // For periodicals, set volume on the parent
                        if let Some(vol) = vol_num {
                            parent.set_volume(integer_or_string_to_maybe_numeric(vol));
                        }
                    } else {
                        // For non-periodicals, set on the entry itself
                        if let Some(vol) = vol_num {
                            entry.set_volume(integer_or_string_to_maybe_numeric(vol));
                        }
                    }

                    entry.set_parents(vec![parent]);
                }
            }
        }
        CreativeWorkVariant::PublicationIssue(pub_issue) => {
            let issue_num = pub_issue.issue_number.as_ref();

            if let Some(CreativeWorkVariant::PublicationVolume(pub_vol)) =
                pub_issue.is_part_of.as_ref().map(|b| b.as_ref())
            {
                let vol_num = pub_vol.volume_number.as_ref();

                if let Some(CreativeWorkVariant::Periodical(periodical)) =
                    pub_vol.is_part_of.as_ref().map(|b| b.as_ref())
                {
                    if let Some(journal_name) = &periodical.name {
                        let mut parent = create_hayagriva_parent_entry(journal_name, periodical);
                        let parent_type = determine_journal_entry_type(journal_name);

                        let vol = vol_num.map(integer_or_string_to_maybe_numeric);
                        let issue = issue_num.map(integer_or_string_to_maybe_numeric);

                        if parent_type == EntryType::Periodical {
                            // For periodicals, set volume/issue on the parent
                            if let Some(vol) = vol {
                                parent.set_volume(vol);
                            }
                            if let Some(issue) = issue {
                                parent.set_issue(issue);
                            }
                        } else {
                            // For non-periodicals, set on the entry itself
                            if let Some(vol) = vol {
                                entry.set_volume(vol);
                            }
                            if let Some(issue) = issue {
                                entry.set_issue(issue);
                            }
                        }

                        entry.set_parents(vec![parent]);
                    }
                }
            }
        }
        _ => {}
    }
}

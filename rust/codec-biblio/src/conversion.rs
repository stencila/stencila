use codec::{
    common::{eyre::Result, reqwest::Url},
    schema::{
        Author, Cord, CreativeWorkType, CreativeWorkVariant, Date, Inline, IntegerOrString,
        Organization, Periodical, Person, PersonOrOrganization, PublicationIssue,
        PublicationVolume, Reference, Text,
    },
};
use hayagriva::{
    Entry,
    types::{
        Date as HDate, EntryType, FormatString, MaybeTyped, Numeric, PageRanges, PageRangesPart,
        Person as HPerson, QualifiedUrl,
    },
};

/// Convert a Hayagriva Entry to a Stencila Reference
pub fn entry_to_reference(entry: &Entry) -> Result<Reference> {
    let mut reference = Reference::new();

    // Set work type
    reference.work_type = entry_type_to_work_type(entry.entry_type());

    // Set ID
    reference.id = Some(entry.key().to_string());

    // Set title
    if let Some(title) = entry.title() {
        reference.title = Some(vec![Inline::Text(Text {
            value: Cord::from(title.to_string()),
            ..Default::default()
        })]);
    }

    // Set authors
    if let Some(authors) = entry.authors() {
        let mut ref_authors = vec![];
        for person in authors {
            // Hayagriva stores the family name in `name` and given names in `given_name`
            let family = person.name.as_str();
            let given = person.given_name.as_deref().unwrap_or("");

            if !family.is_empty() {
                // Split given names by spaces to handle multiple given names
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

                ref_authors.push(Author::Person(Person {
                    family_names: Some(vec![family.to_string()]),
                    given_names,
                    ..Default::default()
                }));
            }
        }
        if !ref_authors.is_empty() {
            reference.authors = Some(ref_authors);
        }
    }

    // Set date
    if let Some(date) = entry.date() {
        let date_string = match (date.month, date.day) {
            (Some(month), Some(day)) => format!("{:04}-{:02}-{:02}", date.year, month, day),
            (Some(month), None) => format!("{:04}-{:02}", date.year, month),
            _ => format!("{:04}", date.year),
        };

        reference.date = Some(Date {
            value: date_string,
            ..Default::default()
        });
    }

    // Set DOI
    if let Some(doi) = entry.doi() {
        reference.doi = Some(doi.to_string());
    }

    // Set URL
    if let Some(url) = entry.url() {
        reference.url = Some(url.to_string());
    }

    // Set container/journal from parents and volume/issue
    // Note: Hayagriva stores volume/issue at the parent level for periodicals
    let parents = entry.parents();

    if let Some(first_parent) = parents.first() {
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

        if let Some(journal_name) = first_parent.title() {
            let mut periodical = Periodical {
                name: Some(journal_name.to_string()),
                ..Default::default()
            };

            // Extract publisher from parent
            if let Some(publisher) = first_parent.publisher() {
                if let Some(pub_name) = publisher.name() {
                    periodical.options.publisher =
                        Some(PersonOrOrganization::Organization(Organization {
                            name: Some(pub_name.to_string()),
                            ..Default::default()
                        }));
                }
            }

            // Build the publication hierarchy
            let mut base: CreativeWorkVariant = CreativeWorkVariant::Periodical(periodical);

            // Add volume layer if present
            if let Some(vol) = volume {
                let volume_str = match vol {
                    MaybeTyped::Typed(n) => n.to_string(),
                    MaybeTyped::String(s) => s.clone(),
                };

                let pub_vol = PublicationVolume {
                    is_part_of: Some(Box::new(base)),
                    volume_number: Some(IntegerOrString::String(volume_str)),
                    ..Default::default()
                };
                base = CreativeWorkVariant::PublicationVolume(pub_vol);
            }

            // Add issue layer if present
            if let Some(issue) = issue {
                let issue_str = match issue {
                    MaybeTyped::Typed(n) => n.to_string(),
                    MaybeTyped::String(s) => s.clone(),
                };

                let pub_issue = PublicationIssue {
                    is_part_of: Some(Box::new(base)),
                    issue_number: Some(IntegerOrString::String(issue_str)),
                    ..Default::default()
                };
                base = CreativeWorkVariant::PublicationIssue(pub_issue);
            }

            reference.is_part_of = Some(Box::new(base));
        }
    }

    // Set page range
    if let Some(page_range) = entry.page_range() {
        let page_str = match page_range {
            MaybeTyped::Typed(n) => n.to_string(),
            MaybeTyped::String(s) => s.clone(),
        };

        // Try to parse page range
        if page_str.contains('-') {
            let parts: Vec<&str> = page_str.split('-').collect();
            if parts.len() == 2 {
                reference.page_start = Some(string_to_integer_or_string(parts[0].trim()));
                reference.page_end = Some(string_to_integer_or_string(parts[1].trim()));
            } else {
                reference.pagination = Some(page_str);
            }
        } else {
            reference.page_start = Some(string_to_integer_or_string(page_str.trim()));
        }
    }

    // Extract serial numbers - for now we only handle DOI from serial numbers
    // since Reference doesn't have an identifiers field
    if let Some(serial_numbers) = entry.serial_number() {
        // SerialNumber is a BTreeMap, so we access fields through the map
        if let Some(doi) = serial_numbers.0.get("doi") {
            // Only set DOI if not already set
            if reference.doi.is_none() {
                reference.doi = Some(doi.clone());
            }
        }

        // TODO: Handle other serial numbers (ISBN, ISSN, etc.) when Reference schema supports them
        // For now, these are lost in the conversion
    }

    Ok(reference)
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
            .map(|inline| match inline {
                Inline::Text(Text { value, .. }) => value.to_string(),
                _ => format!("{inline:?}"),
            })
            .collect::<Vec<_>>()
            .join("");
        entry.set_title(FormatString::from(title_str));
    }

    // Set authors
    if let Some(authors) = &reference.authors {
        let mut persons = vec![];
        for author in authors {
            match author {
                Author::Person(person) => {
                    // Hayagriva expects family name, then given names
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
                        // Create a string in the format "Family, Given"
                        let name_str = if given.is_empty() {
                            family.clone()
                        } else {
                            format!("{family}, {given}")
                        };

                        if let Ok(p) = HPerson::from_strings(vec![&name_str]) {
                            persons.push(p);
                        }
                    }
                }
                Author::Organization(org) => {
                    if let Some(name) = &org.name {
                        if let Ok(p) = HPerson::from_strings(vec![&name]) {
                            persons.push(p);
                        }
                    }
                }
                _ => {}
            }
        }
        if !persons.is_empty() {
            entry.set_authors(persons);
        }
    }

    // Set date
    if let Some(date) = &reference.date {
        let date_str = &date.value;
        // Try to parse the string as a year
        if let Ok(year) = date_str.parse::<i32>() {
            entry.set_date(HDate::from_year(year));
        } else {
            // Try to parse as ISO date
            let parts: Vec<&str> = date_str.split('-').collect();
            match parts.len() {
                1 => {
                    if let Ok(year) = parts[0].parse::<i32>() {
                        entry.set_date(HDate::from_year(year));
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
                        entry.set_date(date);
                    }
                }
                _ => {}
            }
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
        match container.as_ref() {
            CreativeWorkVariant::Periodical(periodical) => {
                // Simple journal, no volume/issue
                if let Some(journal_name) = &periodical.name {
                    // Determine parent type based on title patterns
                    let journal_lower = journal_name.to_lowercase();
                    let parent_type = if journal_lower.starts_with("proceedings") {
                        EntryType::Proceedings
                    } else if matches!(
                        journal_lower.as_str(),
                        "arxiv" | "biorxiv" | "medrxiv" | "chemrxiv" | "peerj preprints"
                    ) {
                        EntryType::Repository
                    } else {
                        EntryType::Periodical
                    };

                    let mut parent = Entry::new(journal_name, parent_type);
                    parent.set_title(FormatString::from(journal_name.clone()));

                    // Add publisher if present
                    if let Some(PersonOrOrganization::Organization(org)) =
                        &periodical.options.publisher
                    {
                        if let Some(pub_name) = &org.name {
                            let publisher = hayagriva::types::Publisher::new(
                                Some(FormatString::from(pub_name.clone())),
                                None,
                            );
                            parent.set_publisher(publisher);
                        }
                    }

                    entry.set_parents(vec![parent]);
                }
            }
            CreativeWorkVariant::PublicationVolume(pub_vol) => {
                // Extract volume number
                let vol_num = pub_vol.volume_number.as_ref();

                // Extract journal from parent
                if let Some(CreativeWorkVariant::Periodical(periodical)) =
                    pub_vol.is_part_of.as_ref().map(|b| b.as_ref())
                {
                    if let Some(journal_name) = &periodical.name {
                        // Determine parent type based on title patterns
                        let journal_lower = journal_name.to_lowercase();
                        let parent_type = if journal_lower.starts_with("proceedings") {
                            EntryType::Proceedings
                        } else if matches!(
                            journal_lower.as_str(),
                            "arxiv" | "biorxiv" | "medrxiv" | "chemrxiv" | "peerj preprints"
                        ) {
                            EntryType::Repository
                        } else {
                            EntryType::Periodical
                        };

                        let mut parent = Entry::new(journal_name, parent_type);
                        parent.set_title(FormatString::from(journal_name.clone()));

                        // Add publisher if present
                        if let Some(PersonOrOrganization::Organization(org)) =
                            &periodical.options.publisher
                        {
                            if let Some(pub_name) = &org.name {
                                let publisher = hayagriva::types::Publisher::new(
                                    Some(FormatString::from(pub_name.clone())),
                                    None,
                                );
                                parent.set_publisher(publisher);
                            }
                        }

                        // For periodicals, set volume on the parent
                        if parent_type == EntryType::Periodical {
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
                // Extract issue number
                let issue_num = pub_issue.issue_number.as_ref();

                // Extract volume and journal from parent hierarchy
                if let Some(CreativeWorkVariant::PublicationVolume(pub_vol)) =
                    pub_issue.is_part_of.as_ref().map(|b| b.as_ref())
                {
                    let vol_num = pub_vol.volume_number.as_ref();

                    // Extract journal from parent of volume
                    if let Some(CreativeWorkVariant::Periodical(periodical)) =
                        pub_vol.is_part_of.as_ref().map(|b| b.as_ref())
                    {
                        if let Some(journal_name) = &periodical.name {
                            // Determine parent type based on title patterns
                            let journal_lower = journal_name.to_lowercase();
                            let parent_type = if journal_lower.starts_with("proceedings") {
                                EntryType::Proceedings
                            } else if matches!(
                                journal_lower.as_str(),
                                "arxiv" | "biorxiv" | "medrxiv" | "chemrxiv" | "peerj preprints"
                            ) {
                                EntryType::Repository
                            } else {
                                EntryType::Periodical
                            };

                            let mut parent = Entry::new(journal_name, parent_type);
                            parent.set_title(FormatString::from(journal_name.clone()));

                            // Add publisher if present
                            if let Some(PersonOrOrganization::Organization(org)) =
                                &periodical.options.publisher
                            {
                                if let Some(pub_name) = &org.name {
                                    let publisher = hayagriva::types::Publisher::new(
                                        Some(FormatString::from(pub_name.clone())),
                                        None,
                                    );
                                    parent.set_publisher(publisher);
                                }
                            }

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

    // Set page numbers
    if let Some(pagination) = &reference.pagination {
        entry.set_page_range(MaybeTyped::String(pagination.clone()));
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

    // Convert DOI back to serial numbers if needed
    // Note: Other serial numbers (ISBN, ISSN, etc.) are lost in the round-trip
    // since Reference doesn't have fields for them
    if let Some(doi) = &reference.doi {
        let mut serial_map = std::collections::BTreeMap::new();
        serial_map.insert("doi".to_string(), doi.clone());
        let serial_numbers = hayagriva::types::SerialNumber(serial_map);
        entry.set_serial_number(serial_numbers);
    }

    Ok(entry)
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

fn integer_or_string_to_maybe_numeric(value: &IntegerOrString) -> MaybeTyped<Numeric> {
    match value {
        IntegerOrString::Integer(i) => MaybeTyped::Typed(Numeric::new(*i as i32)),
        IntegerOrString::String(s) => MaybeTyped::String(s.clone()),
    }
}

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

fn integer_or_string_to_string(value: &IntegerOrString) -> String {
    match value {
        IntegerOrString::Integer(i) => i.to_string(),
        IntegerOrString::String(s) => s.clone(),
    }
}

fn string_to_integer_or_string(value: &str) -> IntegerOrString {
    if let Ok(i) = value.parse::<i64>() {
        IntegerOrString::Integer(i)
    } else {
        IntegerOrString::String(value.to_string())
    }
}

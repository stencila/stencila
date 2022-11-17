use super::prelude::*;
use stencila_schema::*;

patchable_variants!(
    CreativeWorkTypes,
    CreativeWorkTypes::Article,
    CreativeWorkTypes::CreativeWork,
    CreativeWorkTypes::Periodical,
    CreativeWorkTypes::PublicationIssue,
    CreativeWorkTypes::PublicationVolume
);

// The follow structs has all properties (as at 2021-11-18) except `id`
// Commented out properties have types that do not yet have a `impl Patchable`.
// In the future is is likely that this code will be generated directly from
// schema definitions

patchable_struct!(
    CreativeWork,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    keywords,
    licenses,
    maintainers,
    name,
    parts,
    publisher,
    references,
    text,
    title,
    url,
    version
);

patchable_struct!(
    Article,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    keywords,
    licenses,
    maintainers,
    name,
    page_end,
    page_start,
    pagination,
    parts,
    publisher,
    references,
    text,
    title,
    url,
    version
);

patchable_struct!(
    Directory,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    keywords,
    licenses,
    maintainers,
    name,
    parts,
    path,
    publisher,
    references,
    text,
    title,
    url,
    version
);

patchable_struct!(
    File,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    keywords,
    licenses,
    maintainers,
    name,
    parts,
    path,
    publisher,
    references,
    text,
    title,
    url,
    version
);

patchable_struct!(
    Periodical,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    issns,
    keywords,
    licenses,
    maintainers,
    name,
    parts,
    publisher,
    references,
    text,
    title,
    url,
    version
);

patchable_struct!(
    PublicationIssue,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    issue_number,
    keywords,
    licenses,
    maintainers,
    name,
    page_end,
    page_start,
    pagination,
    parts,
    publisher,
    references,
    text,
    title,
    url,
    version
);

patchable_struct!(
    PublicationVolume,
    //about,
    alternate_names,
    authors,
    //comments,
    content,
    date_accepted,
    date_created,
    date_modified,
    date_published,
    date_received,
    description,
    editors,
    //funded_by,
    funders,
    genre,
    //identifiers,
    //images,
    is_part_of,
    keywords,
    licenses,
    maintainers,
    name,
    page_end,
    page_start,
    pagination,
    parts,
    publisher,
    references,
    text,
    title,
    url,
    version,
    volume_number
);

// To avoid bloat it is likely that a lot of these enums
// will be generalized e.g. `OrganizationOrPerson`. `CreativeWorkTypesOrString`

patchable_variants!(
    CreativeWorkAuthors,
    CreativeWorkAuthors::Organization,
    CreativeWorkAuthors::Person
);

patchable_variants!(
    CreativeWorkFunders,
    CreativeWorkFunders::Organization,
    CreativeWorkFunders::Person
);

patchable_variants!(
    CreativeWorkMaintainers,
    CreativeWorkMaintainers::Organization,
    CreativeWorkMaintainers::Person
);

patchable_variants!(
    CreativeWorkPublisher,
    CreativeWorkPublisher::Organization,
    CreativeWorkPublisher::Person
);

patchable_variants!(
    CreativeWorkContent,
    CreativeWorkContent::VecNode,
    CreativeWorkContent::String
);

patchable_variants!(
    CreativeWorkLicenses,
    CreativeWorkLicenses::CreativeWorkTypes,
    CreativeWorkLicenses::String
);

patchable_variants!(
    CreativeWorkReferences,
    CreativeWorkReferences::CreativeWorkTypes,
    CreativeWorkReferences::String
);

patchable_variants!(
    CreativeWorkVersion,
    CreativeWorkVersion::String,
    CreativeWorkVersion::Number
);

patchable_variants!(
    ArticlePageStart,
    ArticlePageStart::String,
    ArticlePageStart::Integer
);

patchable_variants!(
    ArticlePageEnd,
    ArticlePageEnd::String,
    ArticlePageEnd::Integer
);

patchable_variants!(
    DirectoryParts,
    DirectoryParts::File,
    DirectoryParts::Directory
);

patchable_variants!(
    PublicationIssuePageStart,
    PublicationIssuePageStart::String,
    PublicationIssuePageStart::Integer
);

patchable_variants!(
    PublicationIssuePageEnd,
    PublicationIssuePageEnd::String,
    PublicationIssuePageEnd::Integer
);

patchable_variants!(
    PublicationVolumePageStart,
    PublicationVolumePageStart::String,
    PublicationVolumePageStart::Integer
);

patchable_variants!(
    PublicationVolumePageEnd,
    PublicationVolumePageEnd::String,
    PublicationVolumePageEnd::Integer
);

patchable_variants!(
    PublicationIssueIssueNumber,
    PublicationIssueIssueNumber::String,
    PublicationIssueIssueNumber::Integer
);

patchable_variants!(
    PublicationVolumeVolumeNumber,
    PublicationVolumeVolumeNumber::String,
    PublicationVolumeVolumeNumber::Integer
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff;
    use stencila_schema::{BlockContent, Paragraph};

    #[test]
    fn test_article() {
        let article1 = Article {
            content: Some(vec![]),
            ..Default::default()
        };
        let article2 = Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph::default())]),
            ..Default::default()
        };

        diff(&article1, &article2);
    }
}

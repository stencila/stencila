use super::prelude::*;
use stencila_schema::*;

replaceable_struct!(Date, value);

patchable_struct!(
    Organization,
    // All properties except `id` (as at 2021-11-18)
    // Commented out properties have types that do not yet have a `impl Patchable`.

    //address,
    alternate_names,
    //brands,
    //contact_points,
    departments,
    //description,
    //funders,
    id,
    //identifiers,
    //images,
    legal_name,
    //logo,
    //members,
    name,
    parent_organization,
    url
);

patchable_struct!(
    Person,
    // All properties except `id` (as at 2021-11-18)
    // Commented out properties have types that do not yet have a `impl Patchable`.

    //address,
    affiliations,
    alternate_names,
    //description,
    emails,
    family_names,
    //funders,
    given_names,
    honorific_prefix,
    honorific_suffix,
    id,
    //identifiers,
    //images,
    job_title,
    member_of,
    name,
    telephone_numbers,
    url
);

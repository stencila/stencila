// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type or nature of a citation, both factually and rhetorically.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum CitationIntent {
    /// The citing entity agrees with statements, ideas or conclusions presented in the cited entity
    #[default]
    AgreesWith,

    /// The citing entity cites the cited entity as one that provides an authoritative description or definition of the subject under discussion
    CitesAsAuthority,

    /// The citing entity cites the cited entity as source of data
    CitesAsDataSource,

    /// The citing entity cites the cited entity as source of factual evidence for statements it contains
    CitesAsEvidence,

    /// The citing entity cites the cited entity as being the container of metadata describing the citing entity
    CitesAsMetadataDocument,

    /// The citing entity cites the cited entity as providing or containing a possible solution to the issues being discussed
    CitesAsPotentialSolution,

    /// The citing entity cites the cited entity as an item of recommended reading
    CitesAsRecommendedReading,

    /// The citing entity cites the cited entity as one that is related
    CitesAsRelated,

    /// The citing entity cites the cited entity as being the entity from which the citing entity is derived, or about which the citing entity contains metadata
    CitesAsSourceDocument,

    /// The citing entity cites the cited entity as a source of information on the subject under discussion
    CitesForInformation,

    /// The citing entity is used to create or compile the cited entity
    Compiles,

    /// The citing entity confirms facts, ideas or statements presented in the cited entity
    Confirms,

    /// The citing entity contains a statement of fact or a logical assertion (or a collection of such facts and/or assertions) originally present in the cited entity
    ContainsAssertionFrom,

    /// The citing entity corrects statements, ideas or conclusions presented in the cited entity
    Corrects,

    /// The citing entity acknowledges contributions made by the cited entity
    Credits,

    /// The citing entity critiques statements, ideas or conclusions presented in the cited entity
    Critiques,

    /// The citing entity express derision for the cited entity, or for ideas or conclusions contained within it
    Derides,

    /// The citing entity describes the cited entity
    Describes,

    /// The citing entity disagrees with statements, ideas or conclusions presented in the cited entity
    DisagreesWith,

    /// The citing entity discusses statements, ideas or conclusions presented in the cited entity
    Discusses,

    /// The citing entity disputes statements, ideas or conclusions presented in the cited entity
    Disputes,

    /// The citing entity documents information about the cited entity
    Documents,

    /// The citing entity extends facts, ideas or understandings presented in the cited entity
    Extends,

    /// The cited entity provides background information for the citing entity
    GivesBackgroundTo,

    /// The cited entity provides intellectual or factual support for the citing entity
    GivesSupportTo,

    /// The cited entity evokes a reply from the citing entity
    HasReplyFrom,

    /// The citing entity includes one or more excerpts from the cited entity
    IncludesExcerptFrom,

    /// The citing entity includes one or more quotations from the cited entity
    IncludesQuotationFrom,

    /// The cited entity contains statements, ideas or conclusions with which the citing entity agrees
    IsAgreedWithBy,

    /// The cited entity is cited as providing an authoritative description or definition of the subject under discussion in the citing entity
    IsCitedAsAuthorityBy,

    /// The cited entity is cited as a data source by the citing entity
    IsCitedAsDataSourceBy,

    /// The cited entity is cited for providing factual evidence to the citing entity
    IsCitedAsEvidenceBy,

    /// The cited entity is cited as being the container of metadata relating to the citing entity
    IsCitedAsMetadataDocumentBy,

    /// The cited entity is cited as providing or containing a possible solution to the issues being discussed in the citing entity
    IsCitedAsPotentialSolutionBy,

    /// The cited entity is cited by the citing entity as an item of recommended reading
    IsCitedAsRecommendedReadingBy,

    /// The cited entity is cited as being related to the citing entity
    IsCitedAsRelatedBy,

    /// The cited entity is cited as being the entity from which the citing entity is derived, or about which the citing entity contains metadata
    IsCitedAsSourceDocumentBy,

    /// The cited entity (the subject of the RDF triple) is cited by the citing entity (the object of the triple)
    IsCitedBy,

    /// The cited entity is cited as a source of information on the subject under discussion in the citing entity
    IsCitedForInformationBy,

    /// The cited entity is the result of a compile or creation event using the citing entity
    IsCompiledBy,

    /// The cited entity presents facts, ideas or statements that are confirmed by the citing entity
    IsConfirmedBy,

    /// The cited entity presents statements, ideas or conclusions that are corrected by the citing entity
    IsCorrectedBy,

    /// The cited entity makes contributions that are acknowledged by the citing entity
    IsCreditedBy,

    /// The cited entity presents statements, ideas or conclusions that are critiqued by the citing entity
    IsCritiquedBy,

    /// The cited entity contains ideas or conclusions for which the citing entity express derision
    IsDeridedBy,

    /// The cited entity is described by the citing entity
    IsDescribedBy,

    /// The cited entity presents statements, ideas or conclusions that are disagreed with by the citing entity
    IsDisagreedWithBy,

    /// The cited entity presents statements, ideas or conclusions that are discussed by the citing entity
    IsDiscussedBy,

    /// The cited entity presents statements, ideas or conclusions that are disputed by the citing entity
    IsDisputedBy,

    /// Information about the cited entity is documented by the citing entity
    IsDocumentedBy,

    /// The cited entity presents facts, ideas or understandings that are extended by the citing entity
    IsExtendedBy,

    /// The cited entity is the target for an HTTP Uniform Resource Locator (URL) link within the citing entity
    IsLinkedToBy,

    /// The characteristic style or content of the cited entity is imitated by the citing entity for comic effect, usually without explicit citation
    IsParodiedBy,

    /// The cited entity is plagiarized by the author of the citing entity, who includes within the citing entity textual or other elements from the cited entity without formal acknowledgement of their source
    IsPlagiarizedBy,

    /// The cited entity presents statements, ideas or conclusions that are qualified or have conditions placed upon them by the citing entity
    IsQualifiedBy,

    /// The cited entity presents statements, ideas or conclusions that are refuted by the citing entity
    IsRefutedBy,

    /// The cited entity is formally retracted by the citing entity
    IsRetractedBy,

    /// The cited entity presents statements, ideas or conclusions that are reviewed by the citing entity
    IsReviewedBy,

    /// The cited entity or aspects of its contents are ridiculed by the citing entity
    IsRidiculedBy,

    /// The cited entity is cited because the citing article contains speculations on its content or ideas
    IsSpeculatedOnBy,

    /// The cited entity receives intellectual or factual support from the citing entity
    IsSupportedBy,

    /// The cited entity presents statements, ideas, hypotheses or understanding that are updated by the cited entity
    IsUpdatedBy,

    /// A property that permits you to express appreciation of or interest in something that is the object of the RDF triple, or to express that it is worth thinking about even if you do not agree with its content, enabling social media 'likes' statements to be encoded in RDF
    Likes,

    /// The citing entity provides a link, in the form of an HTTP Uniform Resource Locator (URL), to the cited entity
    LinksTo,

    /// The citing entity obtains background information from the cited entity
    ObtainsBackgroundFrom,

    /// The citing entity obtains intellectual or factual support from the cited entity
    ObtainsSupportFrom,

    /// The citing entity imitates the characteristic style or content of the cited entity for comic effect, usually without explicit citation
    Parodies,

    /// A property indicating that the author of the citing entity plagiarizes the cited entity, by including textual or other elements from the cited entity without formal acknowledgement of their source
    Plagiarizes,

    /// The cited entity contains and is the original source of a statement of fact or a logical assertion (or a collection of such facts and/or assertions) that is to be found in the citing entity
    ProvidesAssertionFor,

    /// The cited entity presents conclusions that are used in work described in the citing entity
    ProvidesConclusionsFor,

    /// The cited entity presents data that are used in work described in the citing entity
    ProvidesDataFor,

    /// The cited entity contains information, usually of a textual nature, that is excerpted by (used as an excerpt within) the citing entity
    ProvidesExcerptFor,

    /// The cited entity details a method that is used in work described by the citing entity
    ProvidesMethodFor,

    /// The cited entity contains information, usually of a textual nature, that is quoted by (used as a quotation within) the citing entity
    ProvidesQuotationFor,

    /// The citing entity qualifies or places conditions or restrictions upon statements, ideas or conclusions presented in the cited entity
    Qualifies,

    /// The citing entity refutes statements, ideas or conclusions presented in the cited entity
    Refutes,

    /// The citing entity replies to statements, ideas or criticisms presented in the cited entity
    RepliesTo,

    /// The citing entity constitutes a formal retraction of the cited entity
    Retracts,

    /// The citing entity reviews statements, ideas or conclusions presented in the cited entity
    Reviews,

    /// The citing entity ridicules the cited entity or aspects of its contents
    Ridicules,

    /// Each entity has at least one author that shares a common institutional affiliation with an author of the other entity
    SharesAuthorInstitutionWith,

    /// Each entity has at least one author in common with the other entity
    SharesAuthorWith,

    /// The two entities result from activities that have been funded by the same funding agency
    SharesFundingAgencyWith,

    /// The citing and cited bibliographic resources are published in the same journal
    SharesJournalWith,

    /// The citing and cited bibliographic resources are published in same publication venue
    SharesPublicationVenueWith,

    /// The citing entity speculates on something within or related to the cited entity, without firm evidence
    SpeculatesOn,

    /// The citing entity provides intellectual or factual support for statements, ideas or conclusions presented in the cited entity
    Supports,

    /// The citing entity updates statements, ideas, hypotheses or understanding presented in the cited entity
    Updates,

    /// The citing entity describes work that uses conclusions presented in the cited entity
    UsesConclusionsFrom,

    /// The citing entity describes work that uses data presented in the cited entity
    UsesDataFrom,

    /// The citing entity describes work that uses a method detailed in the cited entity
    UsesMethodIn,
}

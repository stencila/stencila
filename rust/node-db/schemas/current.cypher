// Generated file, do not edit. See the Rust `schema-gen` crate;

CREATE NODE TABLE IF NOT EXISTS `_migrations` (
  `version` STRING PRIMARY KEY,
  `appliedAt` TIMESTAMP,
  `checksum` STRING
);

CREATE NODE TABLE IF NOT EXISTS `Admonition` (
  `admonitionType` STRING,
  `isFolded` BOOLEAN,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Annotation` (
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Article` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `title` STRING,
  `abstract` STRING,
  `embeddings` FLOAT[384],
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `AudioObject` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `contentSize` DOUBLE,
  `contentUrl` STRING,
  `mediaType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `AuthorRole` (
  `roleName` STRING,
  `format` STRING,
  `lastModified` TIMESTAMP,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Citation` (
  `target` STRING,
  `citationMode` STRING,
  `citationIntent` STRING[],
  `text` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `CitationGroup` (
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Claim` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `claimType` STRING,
  `label` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `CodeBlock` (
  `code` STRING,
  `programmingLanguage` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `CodeChunk` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `code` STRING,
  `programmingLanguage` STRING,
  `executionBounds` STRING,
  `executionBounded` STRING,
  `labelType` STRING,
  `label` STRING,
  `isEchoed` BOOLEAN,
  `isHidden` BOOLEAN,
  `executionPure` BOOLEAN,
  `caption` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `CodeExpression` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `code` STRING,
  `programmingLanguage` STRING,
  `executionBounds` STRING,
  `executionBounded` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Datatable` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `DatatableColumn` (
  `name` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Directory` (
  `name` STRING,
  `path` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Figure` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `label` STRING,
  `caption` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `File` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `mediaType` STRING,
  `size` UINT64,
  `content` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `ForBlock` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `code` STRING,
  `programmingLanguage` STRING,
  `executionBounds` STRING,
  `executionBounded` STRING,
  `variable` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Function` (
  `name` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Heading` (
  `labelType` STRING,
  `label` STRING,
  `level` INT64,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `IfBlock` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `IfBlockClause` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `code` STRING,
  `programmingLanguage` STRING,
  `executionBounds` STRING,
  `executionBounded` STRING,
  `isActive` BOOLEAN,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `ImageObject` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `contentSize` DOUBLE,
  `contentUrl` STRING,
  `mediaType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `IncludeBlock` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `source` STRING,
  `mediaType` STRING,
  `select` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Link` (
  `target` STRING,
  `rel` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `List` (
  `order` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `ListItem` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `isChecked` BOOLEAN,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `MathBlock` (
  `code` STRING,
  `mathLanguage` STRING,
  `label` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `MathInline` (
  `code` STRING,
  `mathLanguage` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `MediaObject` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `contentSize` DOUBLE,
  `contentUrl` STRING,
  `mediaType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Note` (
  `noteType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Organization` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `ror` STRING,
  `legalName` STRING,
  PRIMARY KEY (`ror`)
);

CREATE NODE TABLE IF NOT EXISTS `Paragraph` (
  `text` STRING,
  `embeddings` FLOAT[384],
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Parameter` (
  `executionMode` STRING,
  `executionCount` INT64,
  `executionRequired` STRING,
  `executionStatus` STRING,
  `executionEnded` TIMESTAMP,
  `executionDuration` INTERVAL,
  `name` STRING,
  `label` STRING,
  `derivedFrom` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Periodical` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `dateStart` DATE,
  `dateEnd` DATE,
  `issns` STRING[],
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Person` (
  `url` STRING,
  `orcid` STRING,
  `familyNames` STRING[],
  `givenNames` STRING[],
  `honorificPrefix` STRING,
  `honorificSuffix` STRING,
  `name` STRING,
  PRIMARY KEY (`orcid`)
);

CREATE NODE TABLE IF NOT EXISTS `PublicationIssue` (
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `issueNumber` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `PublicationVolume` (
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `volumeNumber` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `QuoteBlock` (
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `QuoteInline` (
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `RawBlock` (
  `format` STRING,
  `content` STRING,
  `css` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Reference` (
  `appearanceIndex` UINT64,
  `workType` STRING,
  `doi` STRING,
  `date` DATE,
  `url` STRING,
  `title` STRING,
  PRIMARY KEY (`doi`)
);

CREATE NODE TABLE IF NOT EXISTS `Section` (
  `sectionType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Sentence` (
  `text` STRING,
  `embeddings` FLOAT[384],
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `SoftwareSourceCode` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `programmingLanguage` STRING,
  `codeSampleType` STRING,
  `runtimePlatform` STRING[],
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `StyledBlock` (
  `code` STRING,
  `styleLanguage` STRING,
  `css` STRING,
  `classList` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `StyledInline` (
  `code` STRING,
  `styleLanguage` STRING,
  `css` STRING,
  `classList` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Supplement` (
  `label` STRING,
  `target` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Table` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `label` STRING,
  `caption` STRING,
  `notes` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `TableCell` (
  `cellType` STRING,
  `text` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `TableRow` (
  `rowType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `ThematicBreak` (
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `Variable` (
  `name` STRING,
  `programmingLanguage` STRING,
  `nativeType` STRING,
  `nodeType` STRING,
  `nativeHint` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE NODE TABLE IF NOT EXISTS `VideoObject` (
  `description` STRING,
  `name` STRING,
  `url` STRING,
  `doi` STRING,
  `dateCreated` DATE,
  `dateReceived` DATE,
  `dateAccepted` DATE,
  `dateModified` DATE,
  `datePublished` DATE,
  `genre` STRING[],
  `keywords` STRING[],
  `repository` STRING,
  `path` STRING,
  `commit` STRING,
  `contentSize` DOUBLE,
  `contentUrl` STRING,
  `mediaType` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

CREATE REL TABLE IF NOT EXISTS `author` (
  FROM `AuthorRole` TO `Person`,
  FROM `AuthorRole` TO `Organization`,
  ONE_ONE
);

CREATE REL TABLE IF NOT EXISTS `annotation` (
  FROM `Annotation` TO `Admonition`,
  FROM `Annotation` TO `AudioObject`,
  FROM `Annotation` TO `Claim`,
  FROM `Annotation` TO `CodeBlock`,
  FROM `Annotation` TO `CodeChunk`,
  FROM `Annotation` TO `Figure`,
  FROM `Annotation` TO `File`,
  FROM `Annotation` TO `ForBlock`,
  FROM `Annotation` TO `Heading`,
  FROM `Annotation` TO `IfBlock`,
  FROM `Annotation` TO `ImageObject`,
  FROM `Annotation` TO `IncludeBlock`,
  FROM `Annotation` TO `List`,
  FROM `Annotation` TO `MathBlock`,
  FROM `Annotation` TO `Paragraph`,
  FROM `Annotation` TO `QuoteBlock`,
  FROM `Annotation` TO `RawBlock`,
  FROM `Annotation` TO `Section`,
  FROM `Annotation` TO `StyledBlock`,
  FROM `Annotation` TO `Supplement`,
  FROM `Annotation` TO `Table`,
  FROM `Annotation` TO `ThematicBreak`,
  FROM `Annotation` TO `VideoObject`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `cells` (
  FROM `TableRow` TO `TableCell`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `clauses` (
  FROM `IfBlock` TO `IfBlockClause`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `columns` (
  FROM `Datatable` TO `DatatableColumn`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `content` (
  FROM `Admonition` TO `Admonition`,
  FROM `Admonition` TO `AudioObject`,
  FROM `Admonition` TO `Claim`,
  FROM `Admonition` TO `CodeBlock`,
  FROM `Admonition` TO `CodeChunk`,
  FROM `Admonition` TO `Figure`,
  FROM `Admonition` TO `File`,
  FROM `Admonition` TO `ForBlock`,
  FROM `Admonition` TO `Heading`,
  FROM `Admonition` TO `IfBlock`,
  FROM `Admonition` TO `ImageObject`,
  FROM `Admonition` TO `IncludeBlock`,
  FROM `Admonition` TO `List`,
  FROM `Admonition` TO `MathBlock`,
  FROM `Admonition` TO `Paragraph`,
  FROM `Admonition` TO `QuoteBlock`,
  FROM `Admonition` TO `RawBlock`,
  FROM `Admonition` TO `Section`,
  FROM `Admonition` TO `StyledBlock`,
  FROM `Admonition` TO `Supplement`,
  FROM `Admonition` TO `Table`,
  FROM `Admonition` TO `ThematicBreak`,
  FROM `Admonition` TO `VideoObject`,
  FROM `Annotation` TO `Annotation`,
  FROM `Annotation` TO `AudioObject`,
  FROM `Annotation` TO `Citation`,
  FROM `Annotation` TO `CitationGroup`,
  FROM `Annotation` TO `CodeExpression`,
  FROM `Annotation` TO `ImageObject`,
  FROM `Annotation` TO `Link`,
  FROM `Annotation` TO `MathInline`,
  FROM `Annotation` TO `MediaObject`,
  FROM `Annotation` TO `Note`,
  FROM `Annotation` TO `Parameter`,
  FROM `Annotation` TO `QuoteInline`,
  FROM `Annotation` TO `Sentence`,
  FROM `Annotation` TO `StyledInline`,
  FROM `Annotation` TO `VideoObject`,
  FROM `Article` TO `Admonition`,
  FROM `Article` TO `AudioObject`,
  FROM `Article` TO `Claim`,
  FROM `Article` TO `CodeBlock`,
  FROM `Article` TO `CodeChunk`,
  FROM `Article` TO `Figure`,
  FROM `Article` TO `File`,
  FROM `Article` TO `ForBlock`,
  FROM `Article` TO `Heading`,
  FROM `Article` TO `IfBlock`,
  FROM `Article` TO `ImageObject`,
  FROM `Article` TO `IncludeBlock`,
  FROM `Article` TO `List`,
  FROM `Article` TO `MathBlock`,
  FROM `Article` TO `Paragraph`,
  FROM `Article` TO `QuoteBlock`,
  FROM `Article` TO `RawBlock`,
  FROM `Article` TO `Section`,
  FROM `Article` TO `StyledBlock`,
  FROM `Article` TO `Supplement`,
  FROM `Article` TO `Table`,
  FROM `Article` TO `ThematicBreak`,
  FROM `Article` TO `VideoObject`,
  FROM `Claim` TO `Admonition`,
  FROM `Claim` TO `AudioObject`,
  FROM `Claim` TO `Claim`,
  FROM `Claim` TO `CodeBlock`,
  FROM `Claim` TO `CodeChunk`,
  FROM `Claim` TO `Figure`,
  FROM `Claim` TO `File`,
  FROM `Claim` TO `ForBlock`,
  FROM `Claim` TO `Heading`,
  FROM `Claim` TO `IfBlock`,
  FROM `Claim` TO `ImageObject`,
  FROM `Claim` TO `IncludeBlock`,
  FROM `Claim` TO `List`,
  FROM `Claim` TO `MathBlock`,
  FROM `Claim` TO `Paragraph`,
  FROM `Claim` TO `QuoteBlock`,
  FROM `Claim` TO `RawBlock`,
  FROM `Claim` TO `Section`,
  FROM `Claim` TO `StyledBlock`,
  FROM `Claim` TO `Supplement`,
  FROM `Claim` TO `Table`,
  FROM `Claim` TO `ThematicBreak`,
  FROM `Claim` TO `VideoObject`,
  FROM `Figure` TO `Admonition`,
  FROM `Figure` TO `AudioObject`,
  FROM `Figure` TO `Claim`,
  FROM `Figure` TO `CodeBlock`,
  FROM `Figure` TO `CodeChunk`,
  FROM `Figure` TO `Figure`,
  FROM `Figure` TO `File`,
  FROM `Figure` TO `ForBlock`,
  FROM `Figure` TO `Heading`,
  FROM `Figure` TO `IfBlock`,
  FROM `Figure` TO `ImageObject`,
  FROM `Figure` TO `IncludeBlock`,
  FROM `Figure` TO `List`,
  FROM `Figure` TO `MathBlock`,
  FROM `Figure` TO `Paragraph`,
  FROM `Figure` TO `QuoteBlock`,
  FROM `Figure` TO `RawBlock`,
  FROM `Figure` TO `Section`,
  FROM `Figure` TO `StyledBlock`,
  FROM `Figure` TO `Supplement`,
  FROM `Figure` TO `Table`,
  FROM `Figure` TO `ThematicBreak`,
  FROM `Figure` TO `VideoObject`,
  FROM `ForBlock` TO `Admonition`,
  FROM `ForBlock` TO `AudioObject`,
  FROM `ForBlock` TO `Claim`,
  FROM `ForBlock` TO `CodeBlock`,
  FROM `ForBlock` TO `CodeChunk`,
  FROM `ForBlock` TO `Figure`,
  FROM `ForBlock` TO `File`,
  FROM `ForBlock` TO `ForBlock`,
  FROM `ForBlock` TO `Heading`,
  FROM `ForBlock` TO `IfBlock`,
  FROM `ForBlock` TO `ImageObject`,
  FROM `ForBlock` TO `IncludeBlock`,
  FROM `ForBlock` TO `List`,
  FROM `ForBlock` TO `MathBlock`,
  FROM `ForBlock` TO `Paragraph`,
  FROM `ForBlock` TO `QuoteBlock`,
  FROM `ForBlock` TO `RawBlock`,
  FROM `ForBlock` TO `Section`,
  FROM `ForBlock` TO `StyledBlock`,
  FROM `ForBlock` TO `Supplement`,
  FROM `ForBlock` TO `Table`,
  FROM `ForBlock` TO `ThematicBreak`,
  FROM `ForBlock` TO `VideoObject`,
  FROM `Heading` TO `Annotation`,
  FROM `Heading` TO `AudioObject`,
  FROM `Heading` TO `Citation`,
  FROM `Heading` TO `CitationGroup`,
  FROM `Heading` TO `CodeExpression`,
  FROM `Heading` TO `ImageObject`,
  FROM `Heading` TO `Link`,
  FROM `Heading` TO `MathInline`,
  FROM `Heading` TO `MediaObject`,
  FROM `Heading` TO `Note`,
  FROM `Heading` TO `Parameter`,
  FROM `Heading` TO `QuoteInline`,
  FROM `Heading` TO `Sentence`,
  FROM `Heading` TO `StyledInline`,
  FROM `Heading` TO `VideoObject`,
  FROM `IfBlockClause` TO `Admonition`,
  FROM `IfBlockClause` TO `AudioObject`,
  FROM `IfBlockClause` TO `Claim`,
  FROM `IfBlockClause` TO `CodeBlock`,
  FROM `IfBlockClause` TO `CodeChunk`,
  FROM `IfBlockClause` TO `Figure`,
  FROM `IfBlockClause` TO `File`,
  FROM `IfBlockClause` TO `ForBlock`,
  FROM `IfBlockClause` TO `Heading`,
  FROM `IfBlockClause` TO `IfBlock`,
  FROM `IfBlockClause` TO `ImageObject`,
  FROM `IfBlockClause` TO `IncludeBlock`,
  FROM `IfBlockClause` TO `List`,
  FROM `IfBlockClause` TO `MathBlock`,
  FROM `IfBlockClause` TO `Paragraph`,
  FROM `IfBlockClause` TO `QuoteBlock`,
  FROM `IfBlockClause` TO `RawBlock`,
  FROM `IfBlockClause` TO `Section`,
  FROM `IfBlockClause` TO `StyledBlock`,
  FROM `IfBlockClause` TO `Supplement`,
  FROM `IfBlockClause` TO `Table`,
  FROM `IfBlockClause` TO `ThematicBreak`,
  FROM `IfBlockClause` TO `VideoObject`,
  FROM `IncludeBlock` TO `Admonition`,
  FROM `IncludeBlock` TO `AudioObject`,
  FROM `IncludeBlock` TO `Claim`,
  FROM `IncludeBlock` TO `CodeBlock`,
  FROM `IncludeBlock` TO `CodeChunk`,
  FROM `IncludeBlock` TO `Figure`,
  FROM `IncludeBlock` TO `File`,
  FROM `IncludeBlock` TO `ForBlock`,
  FROM `IncludeBlock` TO `Heading`,
  FROM `IncludeBlock` TO `IfBlock`,
  FROM `IncludeBlock` TO `ImageObject`,
  FROM `IncludeBlock` TO `IncludeBlock`,
  FROM `IncludeBlock` TO `List`,
  FROM `IncludeBlock` TO `MathBlock`,
  FROM `IncludeBlock` TO `Paragraph`,
  FROM `IncludeBlock` TO `QuoteBlock`,
  FROM `IncludeBlock` TO `RawBlock`,
  FROM `IncludeBlock` TO `Section`,
  FROM `IncludeBlock` TO `StyledBlock`,
  FROM `IncludeBlock` TO `Supplement`,
  FROM `IncludeBlock` TO `Table`,
  FROM `IncludeBlock` TO `ThematicBreak`,
  FROM `IncludeBlock` TO `VideoObject`,
  FROM `Link` TO `Annotation`,
  FROM `Link` TO `AudioObject`,
  FROM `Link` TO `Citation`,
  FROM `Link` TO `CitationGroup`,
  FROM `Link` TO `CodeExpression`,
  FROM `Link` TO `ImageObject`,
  FROM `Link` TO `Link`,
  FROM `Link` TO `MathInline`,
  FROM `Link` TO `MediaObject`,
  FROM `Link` TO `Note`,
  FROM `Link` TO `Parameter`,
  FROM `Link` TO `QuoteInline`,
  FROM `Link` TO `Sentence`,
  FROM `Link` TO `StyledInline`,
  FROM `Link` TO `VideoObject`,
  FROM `ListItem` TO `Admonition`,
  FROM `ListItem` TO `AudioObject`,
  FROM `ListItem` TO `Claim`,
  FROM `ListItem` TO `CodeBlock`,
  FROM `ListItem` TO `CodeChunk`,
  FROM `ListItem` TO `Figure`,
  FROM `ListItem` TO `File`,
  FROM `ListItem` TO `ForBlock`,
  FROM `ListItem` TO `Heading`,
  FROM `ListItem` TO `IfBlock`,
  FROM `ListItem` TO `ImageObject`,
  FROM `ListItem` TO `IncludeBlock`,
  FROM `ListItem` TO `List`,
  FROM `ListItem` TO `MathBlock`,
  FROM `ListItem` TO `Paragraph`,
  FROM `ListItem` TO `QuoteBlock`,
  FROM `ListItem` TO `RawBlock`,
  FROM `ListItem` TO `Section`,
  FROM `ListItem` TO `StyledBlock`,
  FROM `ListItem` TO `Supplement`,
  FROM `ListItem` TO `Table`,
  FROM `ListItem` TO `ThematicBreak`,
  FROM `ListItem` TO `VideoObject`,
  FROM `Note` TO `Admonition`,
  FROM `Note` TO `AudioObject`,
  FROM `Note` TO `Claim`,
  FROM `Note` TO `CodeBlock`,
  FROM `Note` TO `CodeChunk`,
  FROM `Note` TO `Figure`,
  FROM `Note` TO `File`,
  FROM `Note` TO `ForBlock`,
  FROM `Note` TO `Heading`,
  FROM `Note` TO `IfBlock`,
  FROM `Note` TO `ImageObject`,
  FROM `Note` TO `IncludeBlock`,
  FROM `Note` TO `List`,
  FROM `Note` TO `MathBlock`,
  FROM `Note` TO `Paragraph`,
  FROM `Note` TO `QuoteBlock`,
  FROM `Note` TO `RawBlock`,
  FROM `Note` TO `Section`,
  FROM `Note` TO `StyledBlock`,
  FROM `Note` TO `Supplement`,
  FROM `Note` TO `Table`,
  FROM `Note` TO `ThematicBreak`,
  FROM `Note` TO `VideoObject`,
  FROM `Paragraph` TO `Annotation`,
  FROM `Paragraph` TO `AudioObject`,
  FROM `Paragraph` TO `Citation`,
  FROM `Paragraph` TO `CitationGroup`,
  FROM `Paragraph` TO `CodeExpression`,
  FROM `Paragraph` TO `ImageObject`,
  FROM `Paragraph` TO `Link`,
  FROM `Paragraph` TO `MathInline`,
  FROM `Paragraph` TO `MediaObject`,
  FROM `Paragraph` TO `Note`,
  FROM `Paragraph` TO `Parameter`,
  FROM `Paragraph` TO `QuoteInline`,
  FROM `Paragraph` TO `Sentence`,
  FROM `Paragraph` TO `StyledInline`,
  FROM `Paragraph` TO `VideoObject`,
  FROM `QuoteBlock` TO `Admonition`,
  FROM `QuoteBlock` TO `AudioObject`,
  FROM `QuoteBlock` TO `Claim`,
  FROM `QuoteBlock` TO `CodeBlock`,
  FROM `QuoteBlock` TO `CodeChunk`,
  FROM `QuoteBlock` TO `Figure`,
  FROM `QuoteBlock` TO `File`,
  FROM `QuoteBlock` TO `ForBlock`,
  FROM `QuoteBlock` TO `Heading`,
  FROM `QuoteBlock` TO `IfBlock`,
  FROM `QuoteBlock` TO `ImageObject`,
  FROM `QuoteBlock` TO `IncludeBlock`,
  FROM `QuoteBlock` TO `List`,
  FROM `QuoteBlock` TO `MathBlock`,
  FROM `QuoteBlock` TO `Paragraph`,
  FROM `QuoteBlock` TO `QuoteBlock`,
  FROM `QuoteBlock` TO `RawBlock`,
  FROM `QuoteBlock` TO `Section`,
  FROM `QuoteBlock` TO `StyledBlock`,
  FROM `QuoteBlock` TO `Supplement`,
  FROM `QuoteBlock` TO `Table`,
  FROM `QuoteBlock` TO `ThematicBreak`,
  FROM `QuoteBlock` TO `VideoObject`,
  FROM `QuoteInline` TO `Annotation`,
  FROM `QuoteInline` TO `AudioObject`,
  FROM `QuoteInline` TO `Citation`,
  FROM `QuoteInline` TO `CitationGroup`,
  FROM `QuoteInline` TO `CodeExpression`,
  FROM `QuoteInline` TO `ImageObject`,
  FROM `QuoteInline` TO `Link`,
  FROM `QuoteInline` TO `MathInline`,
  FROM `QuoteInline` TO `MediaObject`,
  FROM `QuoteInline` TO `Note`,
  FROM `QuoteInline` TO `Parameter`,
  FROM `QuoteInline` TO `QuoteInline`,
  FROM `QuoteInline` TO `Sentence`,
  FROM `QuoteInline` TO `StyledInline`,
  FROM `QuoteInline` TO `VideoObject`,
  FROM `Section` TO `Admonition`,
  FROM `Section` TO `AudioObject`,
  FROM `Section` TO `Claim`,
  FROM `Section` TO `CodeBlock`,
  FROM `Section` TO `CodeChunk`,
  FROM `Section` TO `Figure`,
  FROM `Section` TO `File`,
  FROM `Section` TO `ForBlock`,
  FROM `Section` TO `Heading`,
  FROM `Section` TO `IfBlock`,
  FROM `Section` TO `ImageObject`,
  FROM `Section` TO `IncludeBlock`,
  FROM `Section` TO `List`,
  FROM `Section` TO `MathBlock`,
  FROM `Section` TO `Paragraph`,
  FROM `Section` TO `QuoteBlock`,
  FROM `Section` TO `RawBlock`,
  FROM `Section` TO `Section`,
  FROM `Section` TO `StyledBlock`,
  FROM `Section` TO `Supplement`,
  FROM `Section` TO `Table`,
  FROM `Section` TO `ThematicBreak`,
  FROM `Section` TO `VideoObject`,
  FROM `Sentence` TO `Annotation`,
  FROM `Sentence` TO `AudioObject`,
  FROM `Sentence` TO `Citation`,
  FROM `Sentence` TO `CitationGroup`,
  FROM `Sentence` TO `CodeExpression`,
  FROM `Sentence` TO `ImageObject`,
  FROM `Sentence` TO `Link`,
  FROM `Sentence` TO `MathInline`,
  FROM `Sentence` TO `MediaObject`,
  FROM `Sentence` TO `Note`,
  FROM `Sentence` TO `Parameter`,
  FROM `Sentence` TO `QuoteInline`,
  FROM `Sentence` TO `Sentence`,
  FROM `Sentence` TO `StyledInline`,
  FROM `Sentence` TO `VideoObject`,
  FROM `StyledBlock` TO `Admonition`,
  FROM `StyledBlock` TO `AudioObject`,
  FROM `StyledBlock` TO `Claim`,
  FROM `StyledBlock` TO `CodeBlock`,
  FROM `StyledBlock` TO `CodeChunk`,
  FROM `StyledBlock` TO `Figure`,
  FROM `StyledBlock` TO `File`,
  FROM `StyledBlock` TO `ForBlock`,
  FROM `StyledBlock` TO `Heading`,
  FROM `StyledBlock` TO `IfBlock`,
  FROM `StyledBlock` TO `ImageObject`,
  FROM `StyledBlock` TO `IncludeBlock`,
  FROM `StyledBlock` TO `List`,
  FROM `StyledBlock` TO `MathBlock`,
  FROM `StyledBlock` TO `Paragraph`,
  FROM `StyledBlock` TO `QuoteBlock`,
  FROM `StyledBlock` TO `RawBlock`,
  FROM `StyledBlock` TO `Section`,
  FROM `StyledBlock` TO `StyledBlock`,
  FROM `StyledBlock` TO `Supplement`,
  FROM `StyledBlock` TO `Table`,
  FROM `StyledBlock` TO `ThematicBreak`,
  FROM `StyledBlock` TO `VideoObject`,
  FROM `StyledInline` TO `Annotation`,
  FROM `StyledInline` TO `AudioObject`,
  FROM `StyledInline` TO `Citation`,
  FROM `StyledInline` TO `CitationGroup`,
  FROM `StyledInline` TO `CodeExpression`,
  FROM `StyledInline` TO `ImageObject`,
  FROM `StyledInline` TO `Link`,
  FROM `StyledInline` TO `MathInline`,
  FROM `StyledInline` TO `MediaObject`,
  FROM `StyledInline` TO `Note`,
  FROM `StyledInline` TO `Parameter`,
  FROM `StyledInline` TO `QuoteInline`,
  FROM `StyledInline` TO `Sentence`,
  FROM `StyledInline` TO `StyledInline`,
  FROM `StyledInline` TO `VideoObject`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `items` (
  FROM `CitationGroup` TO `Citation`,
  FROM `List` TO `ListItem`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `parameters` (
  FROM `Function` TO `Parameter`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `rows` (
  FROM `Table` TO `TableRow`,
  ONE_MANY
);

CREATE REL TABLE IF NOT EXISTS `affiliations` (
  FROM `Person` TO `Organization`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `authors` (
  FROM `Admonition` TO `Person`,
  FROM `Admonition` TO `Organization`,
  FROM `Admonition` TO `AuthorRole`,
  FROM `Article` TO `Person`,
  FROM `Article` TO `Organization`,
  FROM `Article` TO `AuthorRole`,
  FROM `AudioObject` TO `Person`,
  FROM `AudioObject` TO `Organization`,
  FROM `AudioObject` TO `AuthorRole`,
  FROM `Claim` TO `Person`,
  FROM `Claim` TO `Organization`,
  FROM `Claim` TO `AuthorRole`,
  FROM `CodeBlock` TO `Person`,
  FROM `CodeBlock` TO `Organization`,
  FROM `CodeBlock` TO `AuthorRole`,
  FROM `CodeChunk` TO `Person`,
  FROM `CodeChunk` TO `Organization`,
  FROM `CodeChunk` TO `AuthorRole`,
  FROM `CodeExpression` TO `Person`,
  FROM `CodeExpression` TO `Organization`,
  FROM `CodeExpression` TO `AuthorRole`,
  FROM `Datatable` TO `Person`,
  FROM `Datatable` TO `Organization`,
  FROM `Datatable` TO `AuthorRole`,
  FROM `Figure` TO `Person`,
  FROM `Figure` TO `Organization`,
  FROM `Figure` TO `AuthorRole`,
  FROM `File` TO `Person`,
  FROM `File` TO `Organization`,
  FROM `File` TO `AuthorRole`,
  FROM `ForBlock` TO `Person`,
  FROM `ForBlock` TO `Organization`,
  FROM `ForBlock` TO `AuthorRole`,
  FROM `Heading` TO `Person`,
  FROM `Heading` TO `Organization`,
  FROM `Heading` TO `AuthorRole`,
  FROM `IfBlockClause` TO `Person`,
  FROM `IfBlockClause` TO `Organization`,
  FROM `IfBlockClause` TO `AuthorRole`,
  FROM `ImageObject` TO `Person`,
  FROM `ImageObject` TO `Organization`,
  FROM `ImageObject` TO `AuthorRole`,
  FROM `List` TO `Person`,
  FROM `List` TO `Organization`,
  FROM `List` TO `AuthorRole`,
  FROM `MathBlock` TO `Person`,
  FROM `MathBlock` TO `Organization`,
  FROM `MathBlock` TO `AuthorRole`,
  FROM `MathInline` TO `Person`,
  FROM `MathInline` TO `Organization`,
  FROM `MathInline` TO `AuthorRole`,
  FROM `MediaObject` TO `Person`,
  FROM `MediaObject` TO `Organization`,
  FROM `MediaObject` TO `AuthorRole`,
  FROM `Paragraph` TO `Person`,
  FROM `Paragraph` TO `Organization`,
  FROM `Paragraph` TO `AuthorRole`,
  FROM `QuoteBlock` TO `Person`,
  FROM `QuoteBlock` TO `Organization`,
  FROM `QuoteBlock` TO `AuthorRole`,
  FROM `RawBlock` TO `Person`,
  FROM `RawBlock` TO `Organization`,
  FROM `RawBlock` TO `AuthorRole`,
  FROM `Reference` TO `Person`,
  FROM `Reference` TO `Organization`,
  FROM `Reference` TO `AuthorRole`,
  FROM `Section` TO `Person`,
  FROM `Section` TO `Organization`,
  FROM `Section` TO `AuthorRole`,
  FROM `SoftwareSourceCode` TO `Person`,
  FROM `SoftwareSourceCode` TO `Organization`,
  FROM `SoftwareSourceCode` TO `AuthorRole`,
  FROM `StyledBlock` TO `Person`,
  FROM `StyledBlock` TO `Organization`,
  FROM `StyledBlock` TO `AuthorRole`,
  FROM `StyledInline` TO `Person`,
  FROM `StyledInline` TO `Organization`,
  FROM `StyledInline` TO `AuthorRole`,
  FROM `Table` TO `Person`,
  FROM `Table` TO `Organization`,
  FROM `Table` TO `AuthorRole`,
  FROM `VideoObject` TO `Person`,
  FROM `VideoObject` TO `Organization`,
  FROM `VideoObject` TO `AuthorRole`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `cites` (
  FROM `Citation` TO `Reference`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `contributors` (
  FROM `Article` TO `Person`,
  FROM `Article` TO `Organization`,
  FROM `Article` TO `AuthorRole`,
  FROM `AudioObject` TO `Person`,
  FROM `AudioObject` TO `Organization`,
  FROM `AudioObject` TO `AuthorRole`,
  FROM `Claim` TO `Person`,
  FROM `Claim` TO `Organization`,
  FROM `Claim` TO `AuthorRole`,
  FROM `Datatable` TO `Person`,
  FROM `Datatable` TO `Organization`,
  FROM `Datatable` TO `AuthorRole`,
  FROM `Figure` TO `Person`,
  FROM `Figure` TO `Organization`,
  FROM `Figure` TO `AuthorRole`,
  FROM `File` TO `Person`,
  FROM `File` TO `Organization`,
  FROM `File` TO `AuthorRole`,
  FROM `ImageObject` TO `Person`,
  FROM `ImageObject` TO `Organization`,
  FROM `ImageObject` TO `AuthorRole`,
  FROM `MediaObject` TO `Person`,
  FROM `MediaObject` TO `Organization`,
  FROM `MediaObject` TO `AuthorRole`,
  FROM `SoftwareSourceCode` TO `Person`,
  FROM `SoftwareSourceCode` TO `Organization`,
  FROM `SoftwareSourceCode` TO `AuthorRole`,
  FROM `Table` TO `Person`,
  FROM `Table` TO `Organization`,
  FROM `Table` TO `AuthorRole`,
  FROM `VideoObject` TO `Person`,
  FROM `VideoObject` TO `Organization`,
  FROM `VideoObject` TO `AuthorRole`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `editors` (
  FROM `Article` TO `Person`,
  FROM `AudioObject` TO `Person`,
  FROM `Claim` TO `Person`,
  FROM `Datatable` TO `Person`,
  FROM `Figure` TO `Person`,
  FROM `File` TO `Person`,
  FROM `ImageObject` TO `Person`,
  FROM `MediaObject` TO `Person`,
  FROM `Reference` TO `Person`,
  FROM `SoftwareSourceCode` TO `Person`,
  FROM `Table` TO `Person`,
  FROM `VideoObject` TO `Person`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `isPartOf` (
  FROM `Reference` TO `Reference`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `parentOrganization` (
  FROM `Organization` TO `Organization`,
  MANY_MANY
);

CREATE REL TABLE IF NOT EXISTS `references` (
  FROM `Article` TO `Reference`,
  FROM `AudioObject` TO `Reference`,
  FROM `Claim` TO `Reference`,
  FROM `Datatable` TO `Reference`,
  FROM `Figure` TO `Reference`,
  FROM `File` TO `Reference`,
  FROM `ImageObject` TO `Reference`,
  FROM `MediaObject` TO `Reference`,
  FROM `SoftwareSourceCode` TO `Reference`,
  FROM `Table` TO `Reference`,
  FROM `VideoObject` TO `Reference`,
  MANY_MANY
);

INSTALL FTS;
LOAD EXTENSION FTS;
CALL CREATE_FTS_INDEX('Article', 'fts', ['title','abstract','description']);
CALL CREATE_FTS_INDEX('CodeChunk', 'fts', ['caption','code']);
CALL CREATE_FTS_INDEX('Figure', 'fts', ['caption']);
CALL CREATE_FTS_INDEX('Paragraph', 'fts', ['text']);
CALL CREATE_FTS_INDEX('Sentence', 'fts', ['text']);
CALL CREATE_FTS_INDEX('Table', 'fts', ['caption']);

INSTALL VECTOR;
LOAD EXTENSION VECTOR;
CALL CREATE_VECTOR_INDEX('Article', 'vector', 'embeddings');
CALL CREATE_VECTOR_INDEX('Paragraph', 'vector', 'embeddings');
CALL CREATE_VECTOR_INDEX('Sentence', 'vector', 'embeddings');

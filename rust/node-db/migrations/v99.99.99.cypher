CREATE NODE TABLE IF NOT EXISTS `Supplement` (
  `label` STRING,
  `target` STRING,
  `docId` STRING,
  `nodeId` STRING PRIMARY KEY,
  `nodePath` STRING,
  `nodeAncestors` STRING,
  `position` UINT32
);

ALTER TABLE `CodeChunk` DROP `labelAutomatically`;

ALTER TABLE `File` ADD `dateModified` DATE DEFAULT NULL;

ALTER TABLE `File` ADD `dateCreated` DATE DEFAULT NULL;

ALTER TABLE `File` ADD `repository` STRING DEFAULT NULL;

ALTER TABLE `File` ADD `url` STRING DEFAULT NULL;

ALTER TABLE `File` ADD `doi` STRING DEFAULT NULL;

ALTER TABLE `File` ADD `datePublished` DATE DEFAULT NULL;

ALTER TABLE `File` ADD `description` STRING DEFAULT NULL;

ALTER TABLE `File` ADD `genre` STRING[] DEFAULT NULL;

ALTER TABLE `File` ADD `keywords` STRING[] DEFAULT NULL;

ALTER TABLE `File` ADD `dateReceived` DATE DEFAULT NULL;

ALTER TABLE `File` ADD `commit` STRING DEFAULT NULL;

ALTER TABLE `File` ADD `dateAccepted` DATE DEFAULT NULL;

ALTER TABLE `Table` DROP `labelAutomatically`;

ALTER TABLE `MathBlock` DROP `labelAutomatically`;

ALTER TABLE `Figure` DROP `labelAutomatically`;
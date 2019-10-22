## [0.30.2](https://github.com/stencila/schema/compare/v0.30.1...v0.30.2) (2019-10-22)


### Bug Fixes

* Added listen arg to JS manifest ([e9b5716](https://github.com/stencila/schema/commit/e9b5716))
* Added minimist and logga as dependencies ([8c28196](https://github.com/stencila/schema/commit/8c28196))

## [0.30.1](https://github.com/stencila/schema/compare/v0.30.0...v0.30.1) (2019-10-21)


### Bug Fixes

* Package get-stdin added to dependencies ([044fb3e](https://github.com/stencila/schema/commit/044fb3e))

# [0.30.0](https://github.com/stencila/schema/compare/v0.29.0...v0.30.0) (2019-10-17)


### Bug Fixes

* Fix version getting in setup.py ([ee6ef34](https://github.com/stencila/schema/commit/ee6ef34))
* Fixed floating promise in main() call ([ade2abe](https://github.com/stencila/schema/commit/ade2abe))


### Features

* Added deregister method ([f4c3bd8](https://github.com/stencila/schema/commit/f4c3bd8))
* Added listen command for Interpreter ([32d70c9](https://github.com/stencila/schema/commit/32d70c9))
* Added Node execution engine/delegator ([7ab2c91](https://github.com/stencila/schema/commit/7ab2c91))
* Added TS/Py interpreter loops ([1898d99](https://github.com/stencila/schema/commit/1898d99))

# [0.29.0](https://github.com/stencila/schema/compare/v0.28.0...v0.29.0) (2019-09-10)


### Bug Fixes

* Added conversion of ndarray to list for JSON encoding ([f433e3d](https://github.com/stencila/schema/commit/f433e3d))
* Renamed to_dict to object_encode to be more accurate ([6931651](https://github.com/stencila/schema/commit/6931651))
* **Thing, CreativeWork:** Allow Thing.description and CreativeWork.title to be content (ie. Node[]) ([ad6a002](https://github.com/stencila/schema/commit/ad6a002))


### Features

* **JS:** Interpreter now requires command ([d9d275f](https://github.com/stencila/schema/commit/d9d275f))
* **Py:** 'compile' arg and MPL figure fixes ([5b791d5](https://github.com/stencila/schema/commit/5b791d5))

# [0.28.0](https://github.com/stencila/schema/compare/v0.27.0...v0.28.0) (2019-09-02)


### Bug Fixes

* **Code:** Refactor code related classes ([deb1c51](https://github.com/stencila/schema/commit/deb1c51)), closes [#92](https://github.com/stencila/schema/issues/92)
* Fixed behaviour of ConstantSchema and EnumSchema in python executor ([c50d5ac](https://github.com/stencila/schema/commit/c50d5ac))
* **Py:** Fixed Execution timing to include entire CodeChunk ([44338e5](https://github.com/stencila/schema/commit/44338e5))
* **R:** Add include tag so collation order is correct ([3cee6d8](https://github.com/stencila/schema/commit/3cee6d8))
* **R:** Correct Datatable functions  for new schema ([c50903a](https://github.com/stencila/schema/commit/c50903a))
* **R:** Fix and improve generated bindings ([cffc5fe](https://github.com/stencila/schema/commit/cffc5fe))
* Refactor after rebasing ([f21ad6c](https://github.com/stencila/schema/commit/f21ad6c))
* Treating typed variables as declarations and other as assigns ([dbefd62](https://github.com/stencila/schema/commit/dbefd62))
* TS generation of function function and type usage in CodeError ([2f43bfa](https://github.com/stencila/schema/commit/2f43bfa))


### Features

* **Js/WIP:** Parsing of CodeChunk properties ([1fdbd1d](https://github.com/stencila/schema/commit/1fdbd1d))
* Add Parameter schema ([cf6e358](https://github.com/stencila/schema/commit/cf6e358))
* **CodeChunk:** Add more properties to CodeChunk ([49c3543](https://github.com/stencila/schema/commit/49c3543))
* **Js:** Added Handling of for statements ([e6799f6](https://github.com/stencila/schema/commit/e6799f6))
* **Js:** Adding timing of CodeChunk execution ([b1aa9cc](https://github.com/stencila/schema/commit/b1aa9cc))
* Added parsing of alters and error capturing ([3e43901](https://github.com/stencila/schema/commit/3e43901))
* **Js:** Capturing files read by readFile/readFileSync and open ([aaf3fa4](https://github.com/stencila/schema/commit/aaf3fa4))
* **Js:** Catching exceptions during parsing/execution ([e499eb4](https://github.com/stencila/schema/commit/e499eb4))
* **JS:** Added checking for empty string semaphore in imports ([d2e2d48](https://github.com/stencila/schema/commit/d2e2d48))
* **JS:** Added parsing of try/except ([81942ec](https://github.com/stencila/schema/commit/81942ec))
* **Parameter:** Add schema schemas ([d5b67b0](https://github.com/stencila/schema/commit/d5b67b0))
* **Py:** Added checking for empty string semaphore in imports ([648ac8e](https://github.com/stencila/schema/commit/648ac8e))
* **Py:** Added Exception parsing ([5e55bcb](https://github.com/stencila/schema/commit/5e55bcb))
* **Py:** Added Python args/kwargs parsing ([2f4b927](https://github.com/stencila/schema/commit/2f4b927))
* **R:** Add compilation of CodeChunks ([68a183e](https://github.com/stencila/schema/commit/68a183e))
* Add Python command line executor ([e4dbe3d](https://github.com/stencila/schema/commit/e4dbe3d))
* Added 'repeats' and 'extends' properties for Parameter ([398e658](https://github.com/stencila/schema/commit/398e658))
* Added first draft of JavaScript executor ([0bdc46e](https://github.com/stencila/schema/commit/0bdc46e))
* Added parsing of If, While etc to JS interpreter ([7c062d1](https://github.com/stencila/schema/commit/7c062d1))
* Converting matplotlib figures to ImageObjects during Py execution ([e080f6b](https://github.com/stencila/schema/commit/e080f6b))
* Converting Pandas DataFrames to Datatables in Python JSON output ([39406e5](https://github.com/stencila/schema/commit/39406e5))
* Extracting features from CodeChunks ([790f9bf](https://github.com/stencila/schema/commit/790f9bf))

# [0.27.0](https://github.com/stencila/schema/compare/v0.26.0...v0.27.0) (2019-08-23)


### Bug Fixes

* **Code:** Revert to `programmingLanguage` for consistency with id ([426bcb5](https://github.com/stencila/schema/commit/426bcb5))
* **ContactPoint:** Make telephone number prop conistent with Person ([d5e0f87](https://github.com/stencila/schema/commit/d5e0f87))
* **CreativeWork:** Add csi codec to CreativeWork.authors ([60cc14f](https://github.com/stencila/schema/commit/60cc14f))
* **Environment:** Remove unused and conflicting source prop ([c41e520](https://github.com/stencila/schema/commit/c41e520))
* **Items prop:** Use `schema:itemListElement` for all `items` properties ([4df5443](https://github.com/stencila/schema/commit/4df5443))
* **JSON Schema:** Check for conflicting names and `[@id](https://github.com/id)`s ([645f736](https://github.com/stencila/schema/commit/645f736))
* **JSON-LD:** Fix build of JSON-LD context ([94c2a5f](https://github.com/stencila/schema/commit/94c2a5f))
* **JSON-LD:** Generate files for custom types and properties ([46d7cd5](https://github.com/stencila/schema/commit/46d7cd5))
* **JSON-LD:** Improve generation of JSON-LD context ([0f6fea9](https://github.com/stencila/schema/commit/0f6fea9))
* **JSON-LD:** Improve JSON-LD context generation ([af2b8e9](https://github.com/stencila/schema/commit/af2b8e9))
* **Link:** Use consistent `[@id](https://github.com/id)` for title property ([4ab903d](https://github.com/stencila/schema/commit/4ab903d))
* **MediaObject:** Remove uri format constraint ([92c0871](https://github.com/stencila/schema/commit/92c0871))
* **Person:** Rename ssv to ssi codec ([d9a6291](https://github.com/stencila/schema/commit/d9a6291))
* **Product:** Make `brand` prop consistent with Organization ([f4d2a9f](https://github.com/stencila/schema/commit/f4d2a9f))
* **Quote, QuoteBlock:** Use `cite` instead of `citation` ([cef76af](https://github.com/stencila/schema/commit/cef76af))
* **TableCell, TableRow:** Rename props to `cellType` and `rowType` ([2f9321d](https://github.com/stencila/schema/commit/2f9321d))


### Features

* **CreativeWork:** Add `keywords` property and alias for `references` ([b44a34e](https://github.com/stencila/schema/commit/b44a34e))
* **Date:** Add Date schema ([008247f](https://github.com/stencila/schema/commit/008247f))

# [0.26.0](https://github.com/stencila/schema/compare/v0.25.0...v0.26.0) (2019-08-15)


### Features

* **Cite:** Add content field to Cite schema ([e7826cb](https://github.com/stencila/schema/commit/e7826cb))

# [0.25.0](https://github.com/stencila/schema/compare/v0.24.1...v0.25.0) (2019-08-08)


### Features

* Add Figure schema ([b031afb](https://github.com/stencila/schema/commit/b031afb))

## [0.24.1](https://github.com/stencila/schema/compare/v0.24.0...v0.24.1) (2019-08-06)


### Bug Fixes

* **Package:** Rename `schema-interface.ts` so it is packaged ([ebd69d0](https://github.com/stencila/schema/commit/ebd69d0))

# [0.24.0](https://github.com/stencila/schema/compare/v0.23.0...v0.24.0) (2019-08-05)


### Bug Fixes

* **Schema:** Inherit propertyAliases ([a29f215](https://github.com/stencila/schema/commit/a29f215)), closes [#126](https://github.com/stencila/schema/issues/126)


### Features

* **Type Guards:** Allow typeGuards to work on undefined nodes ([35a9ba7](https://github.com/stencila/schema/commit/35a9ba7))

# [0.23.0](https://github.com/stencila/schema/compare/v0.22.1...v0.23.0) (2019-08-01)

### Features

- Add Cite and CiteGroup types ([e222035](https://github.com/stencila/schema/commit/e222035))
- Added categories for each schema [#102](https://github.com/stencila/schema/issues/102) ([deffe0d](https://github.com/stencila/schema/commit/deffe0d))

## [0.22.1](https://github.com/stencila/schema/compare/v0.22.0...v0.22.1) (2019-08-01)

### Bug Fixes

- **Schema:** Add CreativeWork to CreativeWorkTypes ([34aa44a](https://github.com/stencila/schema/commit/34aa44a))

# [0.22.0](https://github.com/stencila/schema/compare/v0.21.0...v0.22.0) (2019-08-01)

### Features

- **Factory Functions:** Filter properties if their value is undefined ([64872fa](https://github.com/stencila/schema/commit/64872fa))

# [0.21.0](https://github.com/stencila/schema/compare/v0.20.2...v0.21.0) (2019-07-31)

### Features

- Add Periodical, PublicationIssue and PublicationVolume schema ([4c2e574](https://github.com/stencila/schema/commit/4c2e574))

## [0.20.2](https://github.com/stencila/schema/compare/v0.20.1...v0.20.2) (2019-07-31)

### Bug Fixes

- **CI:** Avoid package.json regressions when installing on CI ([3560fc6](https://github.com/stencila/schema/commit/3560fc6))

## [0.20.1](https://github.com/stencila/schema/compare/v0.20.0...v0.20.1) (2019-07-31)

### Bug Fixes

- **CI:** Avoid package.json regressions when installing on CI ([fcb0614](https://github.com/stencila/schema/commit/fcb0614))

# [0.20.0](https://github.com/stencila/schema/compare/v0.19.0...v0.20.0) (2019-07-31)

### Features

- **Typescript:** Add a more convienient single-type type guard ([0e59220](https://github.com/stencila/schema/commit/0e59220))
- **Typescript:** Add isType type guard ([ed8fb4a](https://github.com/stencila/schema/commit/ed8fb4a))

# [0.19.0](https://github.com/stencila/schema/compare/v0.18.0...v0.19.0) (2019-07-30)

### Bug Fixes

- **R:** Improve code generation ([9a438f3](https://github.com/stencila/schema/commit/9a438f3))
- **Schema:** Fix missing id and description properties ([5904015](https://github.com/stencila/schema/commit/5904015))
- **TableCell:** Fix long description ([ffd7ec5](https://github.com/stencila/schema/commit/ffd7ec5))

### Features

- **Docs:** Improve property table generation ([8bfdc5d](https://github.com/stencila/schema/commit/8bfdc5d))
- **R:** Add JSON and data.frame conversion functions ([8d1176b](https://github.com/stencila/schema/commit/8d1176b))
- **R:** Conversion between Datatable and data.frame ([e34786d](https://github.com/stencila/schema/commit/e34786d))
- **Table:** Add properties to indicate header cells ([129f722](https://github.com/stencila/schema/commit/129f722))

# [0.18.0](https://github.com/stencila/schema/compare/v0.17.0...v0.18.0) (2019-07-25)

### Bug Fixes

- **Package:** Remove unnecessary files from module ([1fe7dbd](https://github.com/stencila/schema/commit/1fe7dbd))

### Features

- **Docs:** Sort properties table by required fields then alphabetically ([d41cadd](https://github.com/stencila/schema/commit/d41cadd))

# [0.17.0](https://github.com/stencila/schema/compare/v0.16.3...v0.17.0) (2019-07-25)

### Bug Fixes

- **DatatableColumn:** Extend Thing to have name property ([d97c997](https://github.com/stencila/schema/commit/d97c997))
- **DatatableColumn:** Remove duplicated meta property ([8638638](https://github.com/stencila/schema/commit/8638638))
- **Entity:** Move meta property from Thing to Entity ([c03f3f8](https://github.com/stencila/schema/commit/c03f3f8)), closes [/github.com/stencila/encoda/pull/177#issuecomment-514822427](https://github.com//github.com/stencila/encoda/pull/177/issues/issuecomment-514822427)
- **Language bindings:** Flag a property if it is an override ([6bb1ec5](https://github.com/stencila/schema/commit/6bb1ec5)), closes [/github.com/stencila/schema/pull/97#issuecomment-514400261](https://github.com//github.com/stencila/schema/pull/97/issues/issuecomment-514400261)
- **Link:** Add title property to Link ([8d43755](https://github.com/stencila/schema/commit/8d43755)), closes [/github.com/stencila/encoda/pull/177#issuecomment-514822427](https://github.com//github.com/stencila/encoda/pull/177/issues/issuecomment-514822427)
- **Python bindings:** Fix spacing and regnerate test snapshots ([7050b5c](https://github.com/stencila/schema/commit/7050b5c))
- **Python bindings:** Use is not None ([2f41f2a](https://github.com/stencila/schema/commit/2f41f2a))
- **Schema generation:** Sort children and descendants for more deterministic output ([d04a423](https://github.com/stencila/schema/commit/d04a423))
- **Typescript bindings:** Create a dist/index.js file ([f03c2e1](https://github.com/stencila/schema/commit/f03c2e1))
- Replace \$extends and remove unecessary use of object ([b24d8e3](https://github.com/stencila/schema/commit/b24d8e3))
- Updated Python types generation to be more PEP8 compliant ([1e7a6c0](https://github.com/stencila/schema/commit/1e7a6c0))

### Features

- **Python and R bindings:** Initial versions of bindings for Python and R ([8266cf7](https://github.com/stencila/schema/commit/8266cf7))
- **Python bindings:** Add utilty functions for converting to/from JSON ([b4c8aa4](https://github.com/stencila/schema/commit/b4c8aa4))
- **Typescript:** Adds factory functions for Typescript ([39d0fc6](https://github.com/stencila/schema/commit/39d0fc6)), closes [#84](https://github.com/stencila/schema/issues/84)
- **Util:** Add markTypes TypeMap ([1552d06](https://github.com/stencila/schema/commit/1552d06))

## [0.16.3](https://github.com/stencila/schema/compare/v0.16.2...v0.16.3) (2019-07-24)

### Bug Fixes

- **Build:** Add missing TypeScript types to fix TypeGuard usage ([f57d055](https://github.com/stencila/schema/commit/f57d055))

## [0.16.2](https://github.com/stencila/schema/compare/v0.16.1...v0.16.2) (2019-07-24)

### Bug Fixes

- **Build:** Expose TypeScript files in module distribution ([a985686](https://github.com/stencila/schema/commit/a985686))

## [0.16.1](https://github.com/stencila/schema/compare/v0.16.0...v0.16.1) (2019-07-24)

### Bug Fixes

- **Build:** Add outDir option to fix module distribution ([05b1bac](https://github.com/stencila/schema/commit/05b1bac))

# [0.16.0](https://github.com/stencila/schema/compare/v0.15.0...v0.16.0) (2019-07-24)

### Bug Fixes

- **Type Guards:** Expose TypeMaps in packaged module ([cdb61e4](https://github.com/stencila/schema/commit/cdb61e4))

### Features

- **TypeGuards:** Port type guards from Encoda to Schema project ([cb0c050](https://github.com/stencila/schema/commit/cb0c050))
- Added Entity and made it the base of everything (including Thing) ([a0d89b8](https://github.com/stencila/schema/commit/a0d89b8))

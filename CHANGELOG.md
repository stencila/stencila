# [0.23.0](https://github.com/stencila/schema/compare/v0.22.1...v0.23.0) (2019-08-01)


### Features

* Add Cite and CiteGroup types ([e222035](https://github.com/stencila/schema/commit/e222035))
* Added categories for each schema [#102](https://github.com/stencila/schema/issues/102) ([deffe0d](https://github.com/stencila/schema/commit/deffe0d))

## [0.22.1](https://github.com/stencila/schema/compare/v0.22.0...v0.22.1) (2019-08-01)


### Bug Fixes

* **Schema:** Add CreativeWork to CreativeWorkTypes ([34aa44a](https://github.com/stencila/schema/commit/34aa44a))

# [0.22.0](https://github.com/stencila/schema/compare/v0.21.0...v0.22.0) (2019-08-01)


### Features

* **Factory Functions:** Filter properties if their value is undefined ([64872fa](https://github.com/stencila/schema/commit/64872fa))

# [0.21.0](https://github.com/stencila/schema/compare/v0.20.2...v0.21.0) (2019-07-31)


### Features

* Add Periodical, PublicationIssue and PublicationVolume schema ([4c2e574](https://github.com/stencila/schema/commit/4c2e574))

## [0.20.2](https://github.com/stencila/schema/compare/v0.20.1...v0.20.2) (2019-07-31)


### Bug Fixes

* **CI:** Avoid package.json regressions when installing on CI ([3560fc6](https://github.com/stencila/schema/commit/3560fc6))

## [0.20.1](https://github.com/stencila/schema/compare/v0.20.0...v0.20.1) (2019-07-31)


### Bug Fixes

* **CI:** Avoid package.json regressions when installing on CI ([fcb0614](https://github.com/stencila/schema/commit/fcb0614))

# [0.20.0](https://github.com/stencila/schema/compare/v0.19.0...v0.20.0) (2019-07-31)


### Features

* **Typescript:** Add a more convienient single-type type guard ([0e59220](https://github.com/stencila/schema/commit/0e59220))
* **Typescript:** Add isType type guard ([ed8fb4a](https://github.com/stencila/schema/commit/ed8fb4a))

# [0.19.0](https://github.com/stencila/schema/compare/v0.18.0...v0.19.0) (2019-07-30)


### Bug Fixes

* **R:** Improve code generation ([9a438f3](https://github.com/stencila/schema/commit/9a438f3))
* **Schema:** Fix missing id and description properties ([5904015](https://github.com/stencila/schema/commit/5904015))
* **TableCell:** Fix long description ([ffd7ec5](https://github.com/stencila/schema/commit/ffd7ec5))


### Features

* **Docs:** Improve property table generation ([8bfdc5d](https://github.com/stencila/schema/commit/8bfdc5d))
* **R:** Add JSON and data.frame conversion functions ([8d1176b](https://github.com/stencila/schema/commit/8d1176b))
* **R:** Conversion between Datatable and data.frame ([e34786d](https://github.com/stencila/schema/commit/e34786d))
* **Table:** Add properties to indicate header cells ([129f722](https://github.com/stencila/schema/commit/129f722))

# [0.18.0](https://github.com/stencila/schema/compare/v0.17.0...v0.18.0) (2019-07-25)


### Bug Fixes

* **Package:** Remove unnecessary files from module ([1fe7dbd](https://github.com/stencila/schema/commit/1fe7dbd))


### Features

* **Docs:** Sort properties table by required fields then alphabetically ([d41cadd](https://github.com/stencila/schema/commit/d41cadd))

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

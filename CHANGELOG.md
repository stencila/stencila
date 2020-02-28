# [1.8.0](https://github.com/stencila/thema/compare/v1.7.2...v1.8.0) (2020-02-28)


### Bug Fixes

* **Elife:** Don't duplicate font blocks ([f7ba6ec](https://github.com/stencila/thema/commit/f7ba6eccb4a96ac1f43359aa78a17d9f95564f3b))


### Features

* **Fonts:** Use Noto Sans SemiBold for eLife headings ([bd38d09](https://github.com/stencila/thema/commit/bd38d096f2466ee17d26c651d12229675a2548dd))

## [1.7.2](https://github.com/stencila/thema/compare/v1.7.1...v1.7.2) (2020-02-26)


### Bug Fixes

* **CSS:** Malformed custom property usage ([e69dcd5](https://github.com/stencila/thema/commit/e69dcd5cec2c798b9192403e063e58774fa95a0f))

## [1.7.1](https://github.com/stencila/thema/compare/v1.7.0...v1.7.1) (2020-02-25)


### Bug Fixes

* **Semantic Selector:** Fix CSS syntax when generating selectors ([4963cc6](https://github.com/stencila/thema/commit/4963cc6739fa40352dc7d93f626d226b9b60fad6))

# [1.7.0](https://github.com/stencila/thema/compare/v1.6.4...v1.7.0) (2020-02-25)


### Bug Fixes

* **Utility functions:** Require spaces between attributes; better support for custom selectors ([f45abb9](https://github.com/stencila/thema/commit/f45abb901c0e9f271be3fcfd70669dde4094a2c1))


### Features

* **Utility functions:** Add more, test and document ([8b8abb8](https://github.com/stencila/thema/commit/8b8abb81dcc93b4063e637415c1d14ddefff4405))

## [1.6.4](https://github.com/stencila/thema/compare/v1.6.3...v1.6.4) (2020-02-24)


### Bug Fixes

* **Selectors:** Upgrade Schema version and regenerate selectors ([9eb59da](https://github.com/stencila/thema/commit/9eb59dab7b5232f02b2c320b933cf0e9746cc77f))
* **Themes:** Fix invalid custom selectors ([c972fcd](https://github.com/stencila/thema/commit/c972fcd89c01342c7fd50e11c351628eed173aba))

## [1.6.3](https://github.com/stencila/thema/compare/v1.6.2...v1.6.3) (2020-02-23)


### Bug Fixes

* **Docs:** Re-evaluate theme JavaScript when switching themes ([befaa76](https://github.com/stencila/thema/commit/befaa7635bed00953ee182d8a2b1c7675e6417e5))
* **Package:** Setup for sepearate browser and lib distributions ([2ba3594](https://github.com/stencila/thema/commit/2ba35944b40f48188d8c646ddbca44307a32136a))

## [1.6.2](https://github.com/stencila/thema/compare/v1.6.1...v1.6.2) (2020-02-22)


### Bug Fixes

* **Build:** Fix production build failure due to missing plugin ([5aed49f](https://github.com/stencila/thema/commit/5aed49f1b8e8e0af4e7fb5a37db83d1bbae67572))

## [1.6.1](https://github.com/stencila/thema/compare/v1.6.0...v1.6.1) (2020-02-21)


### Bug Fixes

* **Package:** Use prepare instead of postinstall ([1be7091](https://github.com/stencila/thema/commit/1be70914be65c9e3ace8be16b3e3e17f778f9d23))

# [1.6.0](https://github.com/stencila/thema/compare/v1.5.6...v1.6.0) (2020-02-21)


### Bug Fixes

* **Bootstrap theme:** Do not use mixins from shared; docs ([8347907](https://github.com/stencila/thema/commit/83479076db951895593bbb9de3baf1f8eaff16a6))
* **Build:** Fix build issues due to circular dependencies ([4d7c13e](https://github.com/stencila/thema/commit/4d7c13e773c839348016b95b470f28e73dd3ca64))
* **Code extension:** Fix itemtype; add default language; do not style executable nodes ([1315c67](https://github.com/stencila/thema/commit/1315c67d352d52da5bec39959ea974957b73e725))
* **Components:** Load components in index.html ([a32262a](https://github.com/stencila/thema/commit/a32262a3ebcafbb06152b70c6a826e4e1f0fb0dd))
* **Demo:** Generate examples standalone ([a2eb9f1](https://github.com/stencila/thema/commit/a2eb9f1a788e03168a08810c45a6e242a18ca15c))
* **Demo:** Initialize theme each time it, and  example, set ([801a7e9](https://github.com/stencila/thema/commit/801a7e99c22de83ae4dfbea971274bbcc2ddac8b))
* **Demo:** Make demo paths relative for use on GH Pages ([07d980a](https://github.com/stencila/thema/commit/07d980a57b49ad93eb871d6c7d34d2f615a7aadc))
* **Docs:** Fix Publication step on TravisCI ([35cfad0](https://github.com/stencila/thema/commit/35cfad0097e5f0488dadc8c87d7aad7a01419a23))
* **eLife theme:** Changes to dir names and selectors ([6ef7ac2](https://github.com/stencila/thema/commit/6ef7ac22ae732d4632d015be5cf719b8730dcd4f))
* **Examples:** Actually run the functions ([0907a34](https://github.com/stencila/thema/commit/0907a34e71d271122879d28cc6c746f18690b0ea))
* **Extensions:** Only quote name when need to ([f189065](https://github.com/stencila/thema/commit/f1890657f2073b4b0e8b0a53e85db7862c5fd6ca))
* **Generate:** Use Promise<unknown> when generating themes ([87736d1](https://github.com/stencila/thema/commit/87736d15422f1824cf351ef828230d495b278475))
* **Hooks:** Update pre-commit script name to match renamed script ([cc69db8](https://github.com/stencila/thema/commit/cc69db80016f6a7dd3c70b4d7d155c374dec9f57))
* **Hooks:** Update pre-commit script name to match renamed script ([4814aea](https://github.com/stencila/thema/commit/4814aea723ae1f400ac3ac7f45e891a3f866e0c6))
* **Javascript:** Allow themes to be loaded in Node.js ([4f307e5](https://github.com/stencila/thema/commit/4f307e55cabf483df75521e9580ba96dcbe0cb54))
* **Linting:** Add Stylelint to enforce semantic selector usage ([a6a4b89](https://github.com/stencila/thema/commit/a6a4b8961236e4e35f5d299821acf63483159350))
* **References:** Fix Reference formatting selectors and type castings ([b3e23e1](https://github.com/stencila/thema/commit/b3e23e1b9e53688a2a8ada78928dbf729a8727ad))
* **Selectors:** Fix selectors import path in configuration ([c37e331](https://github.com/stencila/thema/commit/c37e3319757b9b9847eed99fc3b3d4c8682798b6))
* **Selectors:** Match Headings & elements with multiple itemtypes ([d35e8f1](https://github.com/stencila/thema/commit/d35e8f1139ef91df63de242facb09bf33a701692))
* **Selectors:** Prevent Prettier from mangling Custom Selectors ([45ae358](https://github.com/stencila/thema/commit/45ae3589c82d67f7e9fb9f1fa9c7a20b6b66f157))
* **Selectors:** Reove trailing newline to avoid stylelint error ([08728e1](https://github.com/stencila/thema/commit/08728e16d89443c7739c6627bbce47c22ad61bf2))
* **Selectors:** Update selectors after upgrade of Schema version ([b37c2fb](https://github.com/stencila/thema/commit/b37c2fb5a8451eebc3947c35659b23160d0865cf))
* **Selectors:** Update selectors to also target data- prefixed versions ([4306d68](https://github.com/stencila/thema/commit/4306d687172d1eeb31ac83d81d8e811c9a6406e6))
* **Skeleton:** Do not rely on anything in shared; add README; linting ([7b238c3](https://github.com/stencila/thema/commit/7b238c358fec0ad135c1199e0a7995fac99cfe63))
* **Skeleton:** Ensure index.ts is a module ([d1d941e](https://github.com/stencila/thema/commit/d1d941ec70b9bbd068cea5e34d801f8a9362bf0b))
* **Themes:** Remove zombie theme ([15d336e](https://github.com/stencila/thema/commit/15d336e854f2cdca6411f2fe27512d39d504fa84))
* **Types:** Type root ([f4e8160](https://github.com/stencila/thema/commit/f4e816056f45a0933c53df7be23d64094cde6434))


### Features

* **Bootstrap:** Add bootsrap theme ([68d42ec](https://github.com/stencila/thema/commit/68d42ec94b271ecf6adafad1041c997ee31068f7))
* **Build:** Combine & sort media queries ([16ed2b6](https://github.com/stencila/thema/commit/16ed2b6941a2fac4d866efdcdcd442570f8ffcb6))
* **Mathjax CSS:** Add generation of MathJax CSS ([d1fda5d](https://github.com/stencila/thema/commit/d1fda5d519c558fa1da27ed6df1b6012c1c1789d))
* **Nature:** Refactor and update Nature theme to emulate new branding ([d552f57](https://github.com/stencila/thema/commit/d552f57be5c96745a36e39d04bc819524ac53b1f))
* **Pages extension:** Add the pages extension ([eb64e1a](https://github.com/stencila/thema/commit/eb64e1adb55ff9700b1eceacbd616319bfa799cb)), closes [#23](https://github.com/stencila/thema/issues/23)
* **PLOS:** Update theme with new semantic selectors ([ca36008](https://github.com/stencila/thema/commit/ca36008dcb828014d910df6af39a3928c086a1f0))
* **Prism addon:** Add prisom addon for syntax highlighting ([967fe53](https://github.com/stencila/thema/commit/967fe537ce567b9ea6906b2fa7588e610c38ce3a))
* **RPNG Theme:** Add theme for generating RPNGs. ([c5b18b7](https://github.com/stencila/thema/commit/c5b18b75d3bb1d1bd0d76c079cfdd29d41ea3bac)), closes [#31](https://github.com/stencila/thema/issues/31)
* **Selector functions:** Add module for handling custom selectors ([59be9e9](https://github.com/stencila/thema/commit/59be9e9ff450c1b141aac762a31dbd0a10494576))
* **Selectors:** Autogenerate semantic selectors from Stencila Schema ([8c5862c](https://github.com/stencila/thema/commit/8c5862c4bb8ec26586f01db13a82a34e6b1378cc))
* **Selectors:** Draft: Auto-Generate semantic selectors ([1caacfd](https://github.com/stencila/thema/commit/1caacfd5fc6fe55dfe06f533e2d5be5f6a715296))
* **Shared JS:** Add functions for DOM manipulation ([01d6d71](https://github.com/stencila/thema/commit/01d6d71271a48ded4f4b7c6b652c9495d117a88c))
* **Shared scripts:** Add DOM manipulation fixes ([c049b5f](https://github.com/stencila/thema/commit/c049b5fc5f996a506e7e4b88cb2198a96b089742))
* **Skeleton:** Add Skeleton starter theme ([c70dbb4](https://github.com/stencila/thema/commit/c70dbb41f025a2a9a3fa3aca73113e05ae62fccc))
* **Skeleton theme:** Add all existing addons ([e73d853](https://github.com/stencila/thema/commit/e73d8530d8dddb2523444d35d04c1be367bbdaeb))
* **Stencila components addon:** Addon for adding Stencila Web Components for document nodes ([65d84a9](https://github.com/stencila/thema/commit/65d84a9070fa0658673b57623acdb93041dc93a4))
* **Themes:** Add a script to create a new theme ([5081903](https://github.com/stencila/thema/commit/5081903e28777c26786d1dbed554096a29b08e47))
* **Themes:** Horizontally centre eLife article ([e7fc6a7](https://github.com/stencila/thema/commit/e7fc6a78e22c174077894a8f0bd4b65f7d561cc2))
* **Variables:** Add a shared layer of variables for all themes ([15bb7da](https://github.com/stencila/thema/commit/15bb7da6dcd9893f3b6ebf38cd5b2681e5a0989b))


### Performance Improvements

* **CSS:** Reduce file sizes by using CSSNano during builds ([87f7b2b](https://github.com/stencila/thema/commit/87f7b2b1092a9ea0b3166665f63acc76894c3527))

## [1.5.6](https://github.com/stencila/thema/compare/v1.5.5...v1.5.6) (2019-10-11)


### Bug Fixes

* **Stencila:** Fix vertical spacing for nested lists after paragraphs ([2ef401a](https://github.com/stencila/thema/commit/2ef401a))

## [1.5.5](https://github.com/stencila/thema/compare/v1.5.4...v1.5.5) (2019-10-09)


### Bug Fixes

* **Examples:** Update Simple example with new component tag names ([e1b738e](https://github.com/stencila/thema/commit/e1b738e))
* **Stencila:** Fix short paragraphs being centre aligned ([236f1e7](https://github.com/stencila/thema/commit/236f1e7))
* **Stencila:** Reduce max-width of CodeChunk components ([5ee20b0](https://github.com/stencila/thema/commit/5ee20b0))

## [1.5.4](https://github.com/stencila/thema/compare/v1.5.3...v1.5.4) (2019-10-07)


### Bug Fixes

* **Pre:** Fix Safari bug with rendering Pre elements inside iFrames ([6b5c8b0](https://github.com/stencila/thema/commit/6b5c8b0))

## [1.5.3](https://github.com/stencila/thema/compare/v1.5.2...v1.5.3) (2019-10-07)


### Bug Fixes

* **Citations:** Handle race condition with formatReferences ([38c6386](https://github.com/stencila/thema/commit/38c6386))
* **CodeChunk:** Show x-scrollbars for overflowing codechunks ([5039d15](https://github.com/stencila/thema/commit/5039d15))
* **Common Styles:** Move description/abstract from themes to common.css ([b2eab4b](https://github.com/stencila/thema/commit/b2eab4b))
* **CreativeWork:** Don't add extraneous commas in reference authors ([3967e65](https://github.com/stencila/thema/commit/3967e65))
* **eLife:** Handle YAML frontmatter for authors, organizations, abstract ([375f54c](https://github.com/stencila/thema/commit/375f54c))
* **Nature:** Adjust theme to handle YAML frontmatter + CodeChunks ([802cfcf](https://github.com/stencila/thema/commit/802cfcf))
* **Themes:** Refinements for eLife, Nature, PLOS themes ([80dac6d](https://github.com/stencila/thema/commit/80dac6d))
* **Themes:** Use semantic selectors, comment setTimeout usage ([9b2da0c](https://github.com/stencila/thema/commit/9b2da0c))

## [1.5.2](https://github.com/stencila/thema/compare/v1.5.1...v1.5.2) (2019-09-29)


### Bug Fixes

* **Selectors:** Fix custom selectors ([7bd1398](https://github.com/stencila/thema/commit/7bd1398))
* **Selectors:** Rename code-chunk to CodeChunk ([70e669f](https://github.com/stencila/thema/commit/70e669f))

## [1.5.1](https://github.com/stencila/thema/compare/v1.5.0...v1.5.1) (2019-09-12)


### Bug Fixes

* **Demo:** Reflect active theme/article in dropdown when reloading ([8e050b9](https://github.com/stencila/thema/commit/8e050b9))
* **JS:** Initalize JS even if script is loaded after DOMContentLoaded ([7639b4b](https://github.com/stencila/thema/commit/7639b4b))
* **Stencila:** Add common theme dependencies ([4c856bd](https://github.com/stencila/thema/commit/4c856bd))

# [1.5.0](https://github.com/stencila/thema/compare/v1.4.3...v1.5.0) (2019-09-12)


### Features

* Add & refine styles for CodeChunk component ([14b2f19](https://github.com/stencila/thema/commit/14b2f19))

## [1.4.3](https://github.com/stencila/thema/compare/v1.4.2...v1.4.3) (2019-09-10)


### Bug Fixes

* **Build:** Refactor TS modules and fix build task ([4f0815b](https://github.com/stencila/thema/commit/4f0815b))

## [1.4.2](https://github.com/stencila/thema/compare/v1.4.1...v1.4.2) (2019-09-10)


### Bug Fixes

* Fix selectors used in references for chnages in Encoda ([854f427](https://github.com/stencila/thema/commit/854f427))
* Use headline custom selector ([9864393](https://github.com/stencila/thema/commit/9864393))
* **Selectors:** Update selectors ([50c14a9](https://github.com/stencila/thema/commit/50c14a9))
* **Stencila theme:** Ensure init function ([602550a](https://github.com/stencila/thema/commit/602550a))

## [1.4.1](https://github.com/stencila/thema/compare/v1.4.0...v1.4.1) (2019-08-30)


### Bug Fixes

* **TypeScript:** Fix getTheme function logic ([39652d6](https://github.com/stencila/thema/commit/39652d6))

# [1.4.0](https://github.com/stencila/thema/compare/v1.3.0...v1.4.0) (2019-08-30)


### Features

* **TypeScript:** Generate TS declarations & export theme names ([be71bd0](https://github.com/stencila/thema/commit/be71bd0))

# [1.3.0](https://github.com/stencila/thema/compare/v1.2.0...v1.3.0) (2019-08-30)


### Bug Fixes

* **Demos:** Clean up demos, move script from .html to .ts ([446794e](https://github.com/stencila/thema/commit/446794e))
* **ELife:** Update markup based on microdata discussion ([dd2e11e](https://github.com/stencila/thema/commit/dd2e11e))
* **ELife Theme:** Add theme switcher ([51b5c49](https://github.com/stencila/thema/commit/51b5c49))
* **ELife Theme:** Fix regressions with `converted-article.html` ([ea0ca02](https://github.com/stencila/thema/commit/ea0ca02))
* **ELife Theme:** Render citations in eLife theme by default ([fed4c61](https://github.com/stencila/thema/commit/fed4c61))
* **Nature Theme:** Tweaks to Nature theme markup and styles ([cea4860](https://github.com/stencila/thema/commit/cea4860))
* **PLoS theme:** Tweaks to table, references, and markup ([184ede6](https://github.com/stencila/thema/commit/184ede6))
* **PLoS theme:** Update references style + table markup ([136b4f3](https://github.com/stencila/thema/commit/136b4f3))
* **Reference Styles:** Fix regressions in Nature theme ([2cc7861](https://github.com/stencila/thema/commit/2cc7861))
* **References, eLife:** Add MLA, APA citations to references.html ([806a1f3](https://github.com/stencila/thema/commit/806a1f3))
* **Styles:** Clean up eLife and Nature styles ([903d86a](https://github.com/stencila/thema/commit/903d86a))
* **Themes:** Clean up common styles, eLife, Nature, Plos, Stencila themes ([3ac0897](https://github.com/stencila/thema/commit/3ac0897))
* **Update Demos:** Update HTML encoded example (converted-article.html) ([3c56e0a](https://github.com/stencila/thema/commit/3c56e0a))


### Features

* **Microdata:** Use microdata-based CSS selectors for references. ([a7b690d](https://github.com/stencila/thema/commit/a7b690d))
* **PLoS Theme:** Add PLoS WIP theme ([c4819f9](https://github.com/stencila/thema/commit/c4819f9))

# [1.2.0](https://github.com/stencila/thema/compare/v1.1.0...v1.2.0) (2019-07-24)


### Bug Fixes

* **Build:** Simplify build command and fix output directory structure ([c0793bf](https://github.com/stencila/thema/commit/c0793bf))
* **Stencila Theme:** Refine some spacing and layouts ([97c12b9](https://github.com/stencila/thema/commit/97c12b9))
* **Syntax Highlighting:** Fix syntax highlighting for JSON code blocks ([57cd42a](https://github.com/stencila/thema/commit/57cd42a))


### Features

* **Stencila Theme:** Refactor styles to be mobile first ([bf9336d](https://github.com/stencila/thema/commit/bf9336d))

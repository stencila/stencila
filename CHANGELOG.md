# [0.127.0](https://github.com/stencila/stencila/compare/v0.126.3...v0.127.0) (2021-11-12)


### Bug Fixes

* **HTML:** Do not strip cell type from cells in header rows ([c96c2e2](https://github.com/stencila/stencila/commit/c96c2e2b3cc9ddd07217576c6ba08b44978edc02))
* **HTML encoding:** Do not add non-existent slot names to error parts ([8d3cd0a](https://github.com/stencila/stencila/commit/8d3cd0a07da07fcd52d6e78723d331f64a7ed1a9))
* **HTML encoding:** Do not force media objects to blocks ([5476a45](https://github.com/stencila/stencila/commit/5476a450e2fc41eff1869b5dd50ee156685e2bec))
* **HTML encoding:** Encode `CodeError`s as custom `<stencila-code-error>` component ([890fbfc](https://github.com/stencila/stencila/commit/890fbfc33a850853668c893325d97f2c6c6e2ebd))
* **HTML encoding:** Encode `colspan` and `rowspan` properties of `TableCell` ([fadbd8c](https://github.com/stencila/stencila/commit/fadbd8cb4a5f4c6b30e0a13817a7ef30026ea8c0))
* **HTML encoding:** Implement `to_html` for `Table` components and create placeholder elements for optional properties ([92b1305](https://github.com/stencila/stencila/commit/92b1305a14789b1c06e788831fdc1471f6d7f92a))
* **HTML encoding:** Remove extraneous spaces between attributes ([213a348](https://github.com/stencila/stencila/commit/213a348f670303e7764987a558317e1087d6ee6c))
* **HTML encoding:** Translate `CreativeWork` nodes to blocks before encoding ([2ed5374](https://github.com/stencila/stencila/commit/2ed53746b9ce6283f84c7cd8c0dc4f42fea902d1))
* **HTML encoding:** Use placeholder elements for `Figure` and `CodeChunk` ([6a3465c](https://github.com/stencila/stencila/commit/6a3465c0d4802fa2e5f28d731a8011c0adffd792))
* **HTML encoding:** Various fixes to encoding of code chunks ([c2b0203](https://github.com/stencila/stencila/commit/c2b020323166372f5de020bea65bd1bd70e0a7ba))
* **HTML encoding:** Wrap strings in spans for consistency when applying patches ([b446d26](https://github.com/stencila/stencila/commit/b446d26aae80924fdc28b7cd0a69d4ee7605c929))
* **HTML Encoding & Patches:** Use lower kebab case for attributes; resolve `pre` text ([393bc1f](https://github.com/stencila/stencila/commit/393bc1feb2e620437d9ce904d677f84bb63ee407))
* **Jupyter kernels:** Get paths from `jupyter --paths` if possible ([ce8b13a](https://github.com/stencila/stencila/commit/ce8b13aacefd1dd00d0a65c384cd4b5bfce01fb4))
* **Markdown decoding:** Pass down header row type to cells ([e2be079](https://github.com/stencila/stencila/commit/e2be079b5aa9dd45581d4f40cc568d1842b11a1e))
* **Node compilation:** Handle data URIs ([fce92c6](https://github.com/stencila/stencila/commit/fce92c62033f42c641e13d199ad8d1bcc39887ed))
* **Pandoc decoding:** Pass down header row type to cells ([db1ed60](https://github.com/stencila/stencila/commit/db1ed60f123b39d1c0c04ea77aa948fa91626aec))
* **Patches:** Add `CodeChunkCaption` ([c364373](https://github.com/stencila/stencila/commit/c3643737ac2fbacc86ef27b85dd4783c2cc07674))
* **Patches:** Detect text DOM nodes during adddress resolution ([772a364](https://github.com/stencila/stencila/commit/772a364144807e19937cb3af6b563dcb751dfb71))
* **Patches:** Handle `label` and `caption` properties of `CodeChunk` nodes ([fe34b7c](https://github.com/stencila/stencila/commit/fe34b7c055ab304ed5d8f8dc26dc3b40b6f88ecb))
* **Patches:** Handle more types when serializing patches ([0745925](https://github.com/stencila/stencila/commit/074592530135e8445ca78810a95cf12348e8f207))
* **Patches:** Implement `Patchable` for more types and variants ([4d386e1](https://github.com/stencila/stencila/commit/4d386e17cda626e01d16fe4bf2489ab67dfea373))
* **Patches:** Reinstate diffing after execution; serialize JSON values ([4f672bc](https://github.com/stencila/stencila/commit/4f672bcba9b7bcf4b47ab9285efbda0ea39ff04f))
* **Web:** Allow for attribute aliases for properties ([2965924](https://github.com/stencila/stencila/commit/2965924574725f23c116d8c6810edfc47c75ba6f))
* **Web:** Convert operations where tag name needs to be changed ([6eab7db](https://github.com/stencila/stencila/commit/6eab7dbecbce288f922189a9635a7b9faf829ccb))
* **Web:** Use patch `value` and escape / unescape as needed for text DOM nodes ([c385be6](https://github.com/stencila/stencila/commit/c385be6a803520528e1aa04d008c27b60e2590ae))
* **Web client:** Add into placeholder elements for options ([772f5f4](https://github.com/stencila/stencila/commit/772f5f42c41ad9fdb8c412a5ae7a008553d51b10))
* **Web client:** Fix resolving of node type ([1d1a5f1](https://github.com/stencila/stencila/commit/1d1a5f18b730b35c34461d598459b9da7b3f1673))
* **Web client:** Handle editor events and consoldate in `documents` module ([44336bc](https://github.com/stencila/stencila/commit/44336bc2cb649fed4f4fb27585899441ca81973a))
* **Web patches:** Handle placeholder elements when patching optional properties ([251664e](https://github.com/stencila/stencila/commit/251664e9ac8c0a25c99b8f875714da0f7a835c74))
* **Web patches:** Improve `createFragment` ([9d54be5](https://github.com/stencila/stencila/commit/9d54be51e0d7b6680026ac0a7cd4b0b9ac81961b))
* **Web patches:** Insert a text node if one does not exist ([d50c073](https://github.com/stencila/stencila/commit/d50c073210a92fee8429fb031ac437f82253cb64))
* **Web patches:** Limit resolving single text node to spans ([49860a0](https://github.com/stencila/stencila/commit/49860a02c25de3803b4dcf62ff1817bd2739ca8c))
* **Web patches:** Use correct regex and document ([4827e54](https://github.com/stencila/stencila/commit/4827e54f5c91caede189bf186ec28f7ad7e08074))


### Features

* **Jupyter kernels:** List searched directories ([6d5a37a](https://github.com/stencila/stencila/commit/6d5a37acf12807b3c427c8493f8bb35aeb75300b))
* **Server:** Add option to not load WebComponents ([6a61f61](https://github.com/stencila/stencila/commit/6a61f61e9cd114020608f5223ed7776308da56aa))
* **Server:** Optionally serve components from local `static` folder ([a5148bb](https://github.com/stencila/stencila/commit/a5148bbef375dd6cbb944754e2308c928e914f89))


### Performance Improvements

* **Patches:** Do not set `html` when it is the same as `value` ([893e9e6](https://github.com/stencila/stencila/commit/893e9e6cac07f283dce0e985dd668a818a8e7e12))
* **Web patches:** Transform CodeMirror changes into `Operation`s ([81f140b](https://github.com/stencila/stencila/commit/81f140b0687f843307c5b70f8c4c646d0ada5314))

## [0.126.3](https://github.com/stencila/stencila/compare/v0.126.2...v0.126.3) (2021-11-12)


### Bug Fixes

* **dependencies:** update dependency @stencila/brand to v0.7.14 ([c3db756](https://github.com/stencila/stencila/commit/c3db7569ca90f8e4a68329a3f9c0565b06732334))
* **dependencies:** update dependency i18next to v21.4.2 ([af20c28](https://github.com/stencila/stencila/commit/af20c282f2f9b8fa1567f2c556341528c27719d7))
* **prosemirror:** pin dependencies ([3f36053](https://github.com/stencila/stencila/commit/3f360530aba15635c4cdb2750f5184def521f6b0))

## [0.126.2](https://github.com/stencila/stencila/compare/v0.126.1...v0.126.2) (2021-11-09)


### Bug Fixes

* **Documents:** Warn on errors rather than halting load; get the document before dumping it ([1a4e641](https://github.com/stencila/stencila/commit/1a4e641a859504b52859b53d5aa7aa85ca5d8aea))
* **Downloads:** Add user agent header to avoid 403s ([e41a666](https://github.com/stencila/stencila/commit/e41a666708fc40010de026909d78ac9c95779648))
* **Serving:** Fix symlink path ([349ade1](https://github.com/stencila/stencila/commit/349ade188441506597622ec9cc425314ed7bc180))

## [0.126.1](https://github.com/stencila/stencila/compare/v0.126.0...v0.126.1) (2021-11-08)


### Bug Fixes

* **dependencies:** update rust crate nom to 7.1.0 ([c064600](https://github.com/stencila/stencila/commit/c0646007c7f52be379f2fea07d3b3c63810b7297))
* **dependencies:** update rust crate pyo3 to v0.15.0 ([c4cfa17](https://github.com/stencila/stencila/commit/c4cfa1716f8a3b1e13b16226f8c1a17674a40f23))
* **Rust:** Turn on used features ([f81da7e](https://github.com/stencila/stencila/commit/f81da7e63ad58f9c7e8da720aa96ce2c3518dd76))
* **tracing:** update tracing dependencies ([dcd4ce9](https://github.com/stencila/stencila/commit/dcd4ce95df3002251796dd2bf3bbf29d2707ee47))
* **Tree-sitter:** Pin to 0.19.x ([c153852](https://github.com/stencila/stencila/commit/c1538526959b548c7f822b95117bcc9b83398dd6))

# [0.126.0](https://github.com/stencila/stencila/compare/v0.125.0...v0.126.0) (2021-11-08)


### Bug Fixes

* **CLI:** Enable `ansi_term` for `cli-pretty` feature ([72ad1df](https://github.com/stencila/stencila/commit/72ad1df46f6671f2c0325f80e99938b18e3e40e7))
* **CLI:** Enable interactive mode with correct feature name ([6d18181](https://github.com/stencila/stencila/commit/6d1818135292b81c3a0d1824147caacb03abab7e))
* **Codecs:** Get codec by label ([943c906](https://github.com/stencila/stencila/commit/943c9062e99a3056733f541503390037242429e7))
* **Codecs:** Use `codec-format` crate ([d4f94e5](https://github.com/stencila/stencila/commit/d4f94e55d328c114891eec376218f419c7f6c171))
* **dependencies:** update rust crate handlebars to v4.1.4 ([0ba5b0d](https://github.com/stencila/stencila/commit/0ba5b0dcb53badaeee0cc864fe6d9c78a1cd47b9))
* **dependencies:** update rust crate serde_json to v1.0.69 ([3e4990c](https://github.com/stencila/stencila/commit/3e4990cd9ff46154b352a302194e84d9412de7a7))
* **dependencies:** update rust crate termimad to 0.17.1 ([8095e12](https://github.com/stencila/stencila/commit/8095e12ea5ff90eae581b8c0636469f3843af9ba))
* **HTML codec:** Fix decoding of table row types ([067df8b](https://github.com/stencila/stencila/commit/067df8b1048b3a644b48220ca4d849cd40f16fce))
* **JSON Codec:** Use `coerce` function ([3a38403](https://github.com/stencila/stencila/commit/3a38403a0408d0c22bc63162e0371516c0acef19))
* **Jupyter kernels:** Use lowercase name when generating list of available kernels ([d466f59](https://github.com/stencila/stencila/commit/d466f593394c7d6a2026875b29821c8ed90dcc7a))
* **Kernels:** Start kernel when created; use id based on language ([404beef](https://github.com/stencila/stencila/commit/404beefce404712623f6d759faaae5577b47f5b1))
* **Markdown codec:** Pass text decoding function to `HtmlCodec` ([7bb4759](https://github.com/stencila/stencila/commit/7bb4759a2c6d1a0ac76cc9a0833f57f7f5519253))
* **Pandoc codec:** Ignore autogenerated heading ids ([1403200](https://github.com/stencila/stencila/commit/14032006b15cf03691dd662f13971fe3dcc8ed40))
* **Parsers:** Get parser by label ([0a1340f](https://github.com/stencila/stencila/commit/0a1340ff2112478e4e62aefc05fb05784daa456c))
* **Serve:** Allow to be run with minimal features ([6be5898](https://github.com/stencila/stencila/commit/6be5898629d0555a1747e38770aca24232b6ad13))
* **UUIDs:** Add `Display` and `PartialEq` implementations ([d303253](https://github.com/stencila/stencila/commit/d30325325b41a82c4015426eae758cc13d30bc1c))


### Features

* **Parsers:** Add `parsers` crate and CLI command ([d1e321c](https://github.com/stencila/stencila/commit/d1e321cf82902a616b18710ffadf13aa999bcb82))
* **UUIDs:** Add traits for custom types ([44c792b](https://github.com/stencila/stencila/commit/44c792ba9c619668e3891870d63d4895d3a70314))

# [0.125.0](https://github.com/stencila/stencila/compare/v0.124.0...v0.125.0) (2021-11-04)


### Bug Fixes

* **CLI:** Improve error reporting in interactive mode ([c237ee3](https://github.com/stencila/stencila/commit/c237ee339039bb5d377474ad6d5e919dee930881))
* **CLI:** Reduce verbosity of backtrace output ([49da3a0](https://github.com/stencila/stencila/commit/49da3a06903057396911ae3422cb5d2887421d41))
* **dependencies:** update dependency rpc-websockets to ^7.4.16 ([a252ce4](https://github.com/stencila/stencila/commit/a252ce4f08e8c4f31c8895f61978875e3cc3c016))
* **dependencies:** update rust crate jsonschema to v0.13.2 ([d60f4ad](https://github.com/stencila/stencila/commit/d60f4ad175252f91be9a1a97b6fddb905dc09ca7))
* **dependencies:** update rust crate termimad to v0.17.0 ([832e749](https://github.com/stencila/stencila/commit/832e749ec3601eccb3b64df15288070c99fb7284))
* **dependencies:** update rust crate tokio to v1.13.0 ([6edc95a](https://github.com/stencila/stencila/commit/6edc95a27f88c097551648bbca1f8184314d76b2))
* **dependencies:** update rust crate tokio-stream to v0.1.8 ([5a1b0a0](https://github.com/stencila/stencila/commit/5a1b0a0cacb6461bd61ff6116fab5db62a5d07d7))
* **Jupyter:** Improve startup; waiting for kernel to be ready ([d79c9ad](https://github.com/stencila/stencila/commit/d79c9ad9887a8a8dedf1edf6590e4e4c54d01f70))
* **Jupyter kernels:** Collect outputs and errors for each exec request ([5c7be71](https://github.com/stencila/stencila/commit/5c7be7124f74a7f0d74ab403d433c783848010d1))
* **Jupyter kernels:** Handle outputs and errors ([24c53e0](https://github.com/stencila/stencila/commit/24c53e050b7062fec54e2d54889e4111b443f67b))
* **Jupyter kernels:** Set correct permissions on files ([eb7c4e1](https://github.com/stencila/stencila/commit/eb7c4e1e7c5326ba08f784a00ee140ed39319025))


### Features

* **Calc kernel:** Improve error messages ([140523e](https://github.com/stencila/stencila/commit/140523eeb247c442c83fb0585ee9d575c943e96b))
* **Jupyter kernels:** Add error handling and message content types ([3d1092d](https://github.com/stencila/stencila/commit/3d1092d288a25f5100c27ed5538d5b8fb6d11249))
* **Jupyter kernels:** Allow connection to already running kernels ([3e4e744](https://github.com/stencila/stencila/commit/3e4e744e91e0f08a1847378aa713bb0c1f47cc6d))
* **Jupyter kernels:** Create connection file and start process ([0a5d707](https://github.com/stencila/stencila/commit/0a5d70705b0f44437fa3d6195473b86cc42831a6))
* **Kernels:** Add `start`, `stop`, `status` and `show` CLI commands ([5a5d30b](https://github.com/stencila/stencila/commit/5a5d30b3b05a39064af95309a4392da242e9cf4f))
* **Kernels:** Add ability to get a list of available language kernels ([5abd846](https://github.com/stencila/stencila/commit/5abd84605255c4d7c2cbd810e9b0965261c71d39))

# [0.123.0](https://github.com/stencila/stencila/compare/v0.122.0...v0.123.0) (2021-10-27)


### Features

* **Editor:** Add basic WYSIWYG editor menu ([39dd2f9](https://github.com/stencila/stencila/commit/39dd2f91a44c75be3ffeb07da8e07e2d8e111483)), closes [#1200](https://github.com/stencila/stencila/issues/1200)

# [0.122.0](https://github.com/stencila/stencila/compare/v0.121.0...v0.122.0) (2021-10-26)


### Bug Fixes

* **dependencies:** update dependency fp-ts to v2.11.5 ([009eed7](https://github.com/stencila/stencila/commit/009eed74575ab9e33eec427e9b356f1d0a4c015f))
* **dependencies:** update dependency i18next to v21.3.3 ([30c4559](https://github.com/stencila/stencila/commit/30c455947443553ba22af3ae0ebf9b4400d3d232))
* **dependencies:** update dependency lit to ^2.0.2 ([96988be](https://github.com/stencila/stencila/commit/96988bea040ff28e5592558e808c427adcdf12c9))
* **dependencies:** update dependency prosemirror-commands to ^1.1.11 ([e7ca347](https://github.com/stencila/stencila/commit/e7ca3471256744820ad5fe9711924f3399e4e281))
* **dependencies:** update dependency prosemirror-view to ^1.20.3 ([d7feff8](https://github.com/stencila/stencila/commit/d7feff8d10bd766c6269b353741688b4e39b4ca0))
* **dependencies:** update docusaurus monorepo to v2.0.0-beta.8 ([4fdd859](https://github.com/stencila/stencila/commit/4fdd8593b161cde1835b0756d58e96de043e2289))
* **dependencies:** update rust crate jsonschema to v0.12.2 ([571a5ca](https://github.com/stencila/stencila/commit/571a5ca4b1b42bd033e3edae57414054ff8d4e9b))
* **dependencies:** update rust crate reqwest to v0.11.6 ([3a99729](https://github.com/stencila/stencila/commit/3a99729ca2d83ff5518d3d1c9e1e3b44bb2a663a))
* **dependencies:** update rust crate serde_with to v1.11.0 ([4f81182](https://github.com/stencila/stencila/commit/4f811828e397ad67f27c0559cf5aa078a44083e3))
* **dependencies:** update rust crate structopt to v0.3.25 ([7c738cf](https://github.com/stencila/stencila/commit/7c738cf8f3a552c58b2a323d86186f9633f0acc7))
* **dependencies:** update rust crate strum to v0.22.0 ([987858e](https://github.com/stencila/stencila/commit/987858e181e4dba1c6786e6b06c59cb72b13d50f))
* **dependencies:** update rust crate strum_macros to v0.22.0 ([6e27fae](https://github.com/stencila/stencila/commit/6e27fae19d84ec4934074cf22623e71a7a73aa3c))
* **dependencies:** update rust crate termimad to v0.16.4 ([14e1cff](https://github.com/stencila/stencila/commit/14e1cffefef18993940aa7863c5212ce8ba1d03b))
* **Dependencies:** Use latest version of `png` crate ([a9763d7](https://github.com/stencila/stencila/commit/a9763d702822be5dc1da09b857f142e2edd1c74f))
* **Desktop:** Remove duplicated Window Close menu item ([b467ca1](https://github.com/stencila/stencila/commit/b467ca185ab0505098e80aa6a7657995a33ba2b6)), closes [#1176](https://github.com/stencila/stencila/issues/1176)
* **Editor:** Fix parsing of `CodeFragment` nodes ([0296876](https://github.com/stencila/stencila/commit/0296876b8627dc7baff8e31aac67a92fd4f4e6ed))
* **Editor:** Fix programming language update event ([cbf30f0](https://github.com/stencila/stencila/commit/cbf30f026bf598df4dfc836c3cfe615c65384ca3))
* **Editor:** Preserve node type when copy/pasting CodeBlock elements ([d3958b2](https://github.com/stencila/stencila/commit/d3958b22f428df38391252313da0046ac5428316))
* **Patches:** Ensure case conversion of property names in addresses ([227202b](https://github.com/stencila/stencila/commit/227202bba0002de8c05fd75dd545ad6ee96ca593)), closes [#1213](https://github.com/stencila/stencila/issues/1213)
* **Patches:** Serialization of value to JSON ([61523cb](https://github.com/stencila/stencila/commit/61523cb02ef83563b297dbf3f0bc92b31cf39091))


### Features

* **Editor:** Render CodeBlocks in a stencila-editor component ([e0aaf93](https://github.com/stencila/stencila/commit/e0aaf939a7822aa1283e225aad6fae6b4324d54f))
* **Editor:** Support setting CodeBlock language with Markdown syntax ([152b680](https://github.com/stencila/stencila/commit/152b680db21f2fd37c59dc95115985c71d44cf29)), closes [#1211](https://github.com/stencila/stencila/issues/1211)

# [0.121.0](https://github.com/stencila/stencila/compare/v0.120.1...v0.121.0) (2021-10-13)


### Bug Fixes

* **Deps:** Update dependencies ([b41ae00](https://github.com/stencila/stencila/commit/b41ae0034d72511c4827a08dab0c60f908b3e074))
* **Desktop:** Fix capturing of unhandled promise rejection errors ([e95b5e2](https://github.com/stencila/stencila/commit/e95b5e22ac512e990fb67edee9a54445e763c529))
* **Launcher:** Fix project directory name being clipped vertically ([561d197](https://github.com/stencila/stencila/commit/561d1977a6f5f709d7fc38de1e7c77b521260320))


### Features

* **Logs:** Capture all logs, & add filter UI to Desktop ([3a58f73](https://github.com/stencila/stencila/commit/3a58f73ca3848b2139fee46f635ccc64b4eaed7e))

## [0.120.1](https://github.com/stencila/stencila/compare/v0.120.0...v0.120.1) (2021-10-11)


### Bug Fixes

* **Article editor:** Add extra test for valid items ([c188226](https://github.com/stencila/stencila/commit/c18822667f2d40fb2f39894a856985278bbed73b))
* **Article editor:** Include ProseMirror editor styles ([8db95bc](https://github.com/stencila/stencila/commit/8db95bc93a2f05a24fce51527740e0e8713e83f4))
* **dependencies:** update dependency rpc-websockets to ^7.4.15 ([f2d6416](https://github.com/stencila/stencila/commit/f2d64163052da0ff813a28987784ed73a66fad9f))
* **dependencies:** update rust crate reqwest to v0.11.5 ([950ec62](https://github.com/stencila/stencila/commit/950ec62d5742ebb70f97351918d8a3cead7317e9))
* **dependencies:** update rust crate thiserror to v1.0.30 ([f8db6d2](https://github.com/stencila/stencila/commit/f8db6d23fd9f07cde2a54f6643fdf66997663cc9))
* **dependencies:** update rust crate tracing to v0.1.29 ([b7e871f](https://github.com/stencila/stencila/commit/b7e871fd65a42bb32b804ed61ed4d71aa558ea6f))
* **dependencies:** update rust crate tracing-subscriber to v0.2.25 ([c1c3342](https://github.com/stencila/stencila/commit/c1c334215c2e0a1670ff973b0bc451c4acbcfffb))

# [0.120.0](https://github.com/stencila/stencila/compare/v0.119.1...v0.120.0) (2021-10-11)


### Bug Fixes

* **Article editor:** Add tranformations for node types ([44ec0ae](https://github.com/stencila/stencila/commit/44ec0aebe84a17f1fe7cd3076de49e701dff5514))
* **Article editor:** Fix order attribute for ordered lists ([adf52cb](https://github.com/stencila/stencila/commit/adf52cb507ee091bf07fe42f1650ed5fed87413e))
* **HTML:** Fixes to list and table encoding for compatability with address resolving algorithm ([a57d4b0](https://github.com/stencila/stencila/commit/a57d4b0f2fa67f616e8b6a1e92485fb14dbd1985))
* **HTML Encoding:** Use attributes that are consistent with themes ([a0cbc9f](https://github.com/stencila/stencila/commit/a0cbc9f8c5f5c1c2cf62b0d9d48ff518be8fcfc2))
* **Node bindings:** Use oneOf instead of anyOf ([a2e0d81](https://github.com/stencila/stencila/commit/a2e0d8156edfd267a9f58af3fac2f728f90af368))
* **Patches:** Add `DeserializeOwned` bound to avoid manual`from_value` overrides; fix tests ([5c0cd10](https://github.com/stencila/stencila/commit/5c0cd1074f7a47d4df10fa3330abc11256972a29))
* **Patches:** Allow single item values for vectors ([a2a5558](https://github.com/stencila/stencila/commit/a2a5558d2561cff226ab6e7e529e94e5392a3c9f))
* **Patches:** Handle single item values ([b8ffeda](https://github.com/stencila/stencila/commit/b8ffedad4478a3a981f353c3d25184c35d5e58f2))
* **Rust:** Republish patches to other subscribers ([a02b40e](https://github.com/stencila/stencila/commit/a02b40e04e67a2de313ab00232fb587c6874959d))
* **Serve:** Return formats like Markdown as plain text ([d5355a6](https://github.com/stencila/stencila/commit/d5355a63e745805a63ac8d19a7a9deb9beb87fc2))
* **Web:** Exclude map files from embedding ([fd2ea89](https://github.com/stencila/stencila/commit/fd2ea89adfb2a220ce90b2a42d0caa5d024fa5ef))
* **Web:** Improve robustness of JSON patches ([921e881](https://github.com/stencila/stencila/commit/921e881f95a1df849458f39b51bd4338b7bd62d9))
* **Web:** Optimize browser bundle ([6938653](https://github.com/stencila/stencila/commit/6938653a58e3f0fcf7eeab01393164f2aa363cc1))
* **Web:** Use diffing to generate patches ([7f25135](https://github.com/stencila/stencila/commit/7f2513597842966ba5ee857904542f98a436bc53))
* **Web patches:** Add JSON diffing ([41ab4b0](https://github.com/stencila/stencila/commit/41ab4b0bc8bbc66b2f7331debfc7d58b9b27256f))
* **Web patches:** Allow for empty paragraphs ([b118e36](https://github.com/stencila/stencila/commit/b118e3664b5b4d37064f593e952d1ee59374dd7e))
* **Web patches:** Check for deep equality before returning replace ([3d62ee4](https://github.com/stencila/stencila/commit/3d62ee4e5eaea0fafbebcc0ed945ad7d0e573fac))


### Features

* **Article editor:** Add support for code fragments and blocks ([3501e69](https://github.com/stencila/stencila/commit/3501e69075c419f419088a5dfc82168b36a4d012))
* **Web:** Add article editor ([68601d0](https://github.com/stencila/stencila/commit/68601d096fd3b3d7b22ed514538332a374cec1f5))
* **Web patches:** Add JSON patching module ([2002a96](https://github.com/stencila/stencila/commit/2002a96f9e706663e8c0916dc72eda1e040185f5))
* **Web patches:** Diffing of strings ([ebf8e61](https://github.com/stencila/stencila/commit/ebf8e6101367a78464af8c547a968b195c02bf3a))

## [0.119.1](https://github.com/stencila/stencila/compare/v0.119.0...v0.119.1) (2021-10-06)


### Bug Fixes

* **CLI:** Ensure web module is built and embedded ([f04938d](https://github.com/stencila/stencila/commit/f04938d919f89dfdf85cbd5335f3e96a33084afe))
* **dependencies:** update rust crate similar to v2.1.0 ([e606746](https://github.com/stencila/stencila/commit/e60674625ebe955d27b28222d353b400ae2035ec))
* **ELife Source:** Add a user agent header to avoid 403 ([f15125b](https://github.com/stencila/stencila/commit/f15125b8d4846c7e9273884d8d6e571b47f9fd2f))

# [0.119.0](https://github.com/stencila/stencila/compare/v0.118.5...v0.119.0) (2021-10-03)


### Bug Fixes

* **CLI:** Add interactive as an alias ([5420cc7](https://github.com/stencila/stencila/commit/5420cc70c2902579ccdf7ca8dc9ece111f05db4c))
* **CLI:** Always enter interactive mode when option is supplied ([5b0bf7e](https://github.com/stencila/stencila/commit/5b0bf7e04fdf308299b180408789093958c81a4f))
* **CLI:** Change arrow directions for interactive mode command prefix ([da93388](https://github.com/stencila/stencila/commit/da9338840ed1ad1c02212c9f097619e68dd0cb35))
* **dependencies:** update rust crate pathdiff to v0.2.1 ([0d42a92](https://github.com/stencila/stencila/commit/0d42a921c19fe129f0ba674667612440fc7a79d3))
* **dependencies:** update rust crate schemars to v0.8.6 ([1d87126](https://github.com/stencila/stencila/commit/1d871262190303beb36e4063c6e190953fe6808a))
* **dependencies:** update rust crate termimad to v0.16.2 ([9e005ae](https://github.com/stencila/stencila/commit/9e005aef88e0c845f8da5b05526b13a69f28a744))
* **dependencies:** update rust crate tokio to v1.12.0 ([8ae8653](https://github.com/stencila/stencila/commit/8ae86530e50f81e36b93b9534bffa189f9a834f0))
* **DOM Patches:** Add JSON to add and replace operations for use on WebComponents ([45bf1cf](https://github.com/stencila/stencila/commit/45bf1cf74874a96608b63da01fba4d00c68bdb59))
* **Execute:** Correct function signature ([8fd8347](https://github.com/stencila/stencila/commit/8fd83477a5ca9b89011acc403cb8ae3f2d9d254c))
* **HTML encoding:** Use both class attribute and `meta` element for code language ([bba2864](https://github.com/stencila/stencila/commit/bba28649b86f4a9134d08536282b5766fca3e313))
* **HTML Encoding:** Fix `CodeExpression` output ([c839560](https://github.com/stencila/stencila/commit/c8395604b81ba029d326af3c0583fd49d07f3960))
* **Kernels:** Error if no matching language is found ([8e1959a](https://github.com/stencila/stencila/commit/8e1959a62572b6fcba609f05caafbca465386386))
* **Patches:** Add `cast_value` method ([2dc13ed](https://github.com/stencila/stencila/commit/2dc13edeb5fe7f1250d0fece0f0a7642c9d7975c))
* **Patches:** Fix handling of already boxed values ([6820881](https://github.com/stencila/stencila/commit/6820881a4a869b94f7e97529523e49907ca609c5))
* **Rust:** Update Schema : new `Null` type and no `type_name` method ([fa78233](https://github.com/stencila/stencila/commit/fa782338e08acbbb5e472dbcf72eea6de7f2278b))
* **Server:** Log errors on the server ([fe970d9](https://github.com/stencila/stencila/commit/fe970d9fdb55a7a947265630382eee436f2180a5))
* **Server:** Startup web client on load ([410741a](https://github.com/stencila/stencila/commit/410741ae8691c040272f1a8c91e8e99c167f5d65))
* **Web:** Implement move operation ([a58329b](https://github.com/stencila/stencila/commit/a58329bd23597c062f3c8bc42b52a4e039dc5782))
* **Web:** Make able to handle empty addresses for add and replace operations ([69b5f57](https://github.com/stencila/stencila/commit/69b5f5787692979f471c7a7ebaba0e7c8e71da87))
* **Web:** Quote slot in error messages for improve readability ([2fb45d5](https://github.com/stencila/stencila/commit/2fb45d59b09beeb32132700cf4746553fc08ca83))
* **Web:** Use graphemes when applying string patch operations ([1d6f725](https://github.com/stencila/stencila/commit/1d6f725fbc0ed1a66ff4e122bfee079ac4bf94d5))


### Features

* **Cacl kernel:** Execute `calc` expressions and patch outputs ([022ab96](https://github.com/stencila/stencila/commit/022ab96969df768e65736fb5c8d2e388c58c53dd))
* **Calc:** Add `calc` language ([46e8745](https://github.com/stencila/stencila/commit/46e874541c27194cf27454fdf305f8d402e5c953))
* **CLI:** Add `with` command ([f1e1beb](https://github.com/stencila/stencila/commit/f1e1beb4e3d96aee92005c70aa9f1423b640537b))
* **CLI:** Add shortcuts to allow appending and truncating the command prefix ([22f3f26](https://github.com/stencila/stencila/commit/22f3f2667b3c63cdf30882dac5f85b8bea90aedb))
* **CLI:** Allow `documents show` to show `content` and `root` ([2f592bf](https://github.com/stencila/stencila/commit/2f592bf83a7954359cbba6d29937e1fcb7e174a9))
* **Documents:** Add execute command ([8d213dd](https://github.com/stencila/stencila/commit/8d213ddfc10effefee285fcfc4c2873bf26e32f1))
* **Documents:** Add resolve and find functions ([26bb387](https://github.com/stencila/stencila/commit/26bb387435629b2b32af5cf15690d8f7a781d58d))
* **Documents:** Published `patched` events when documented is updated ([77f5778](https://github.com/stencila/stencila/commit/77f577885d5d9a4dd499c903b865199cf9477eaa))
* **HTML:** Initial encoding and decoding of parameters ([62d3cce](https://github.com/stencila/stencila/commit/62d3cce2247d44fd2c57618a77c70f3175e590f8))
* **HTML Encoding:** Add encoding for vectors of primitives ([a00ca9f](https://github.com/stencila/stencila/commit/a00ca9f0685eef32ce13113de21cb2797a889d4b))
* **Kernels:** Add initial iteration of kernels ([0838353](https://github.com/stencila/stencila/commit/0838353ebee779a123ee461b6b79bc5ff9a779fa))
* **Kernels:** Add mirroring of variables across kernels ([119fe88](https://github.com/stencila/stencila/commit/119fe884d9ad365cc071d4c58ab47b786baad442))
* **Kernels:** Instantiate and resolve using language compatability ([e683cd6](https://github.com/stencila/stencila/commit/e683cd6ce0c2a3ddfd74cf9a47531490e55c6625))
* **Kernels:** Only mirror variables if necessary ([dd1321f](https://github.com/stencila/stencila/commit/dd1321f743b5adae957a928db45600ea7e6211a0))
* **Patches:** Add address resolution ([c95ad21](https://github.com/stencila/stencila/commit/c95ad21603b2fc816b65269cd59a882596c68e3d))
* **Patches:** Add initial implementation of `DomOperation` ([a692e2b](https://github.com/stencila/stencila/commit/a692e2b648061fc350bab1bc4ec8b07de023e3c4))
* **Patches:** Allow patches to be applied at a specific id in a node tree ([02a1056](https://github.com/stencila/stencila/commit/02a1056d203d37ff7a8cc13b0e360339129b7968))
* **Server:** Add flag to run in background ([967e2a9](https://github.com/stencila/stencila/commit/967e2a9e3beadd3514292ce478530828e4051915))
* **Web:** Add patches module for applying `DomPatch`s sent by server ([9931425](https://github.com/stencila/stencila/commit/99314257a4ccafd33ec42cfcd5801890dd746504))

## [0.118.5](https://github.com/stencila/stencila/compare/v0.118.4...v0.118.5) (2021-09-28)


### Bug Fixes

* **dependencies:** update dependency @stencila/brand to v0.7.7 ([5a4b32a](https://github.com/stencila/stencila/commit/5a4b32a1393ff820de92b8cbcaed6c2bf96ca1c8))
* **dependencies:** update dependency i18next to v21 ([e189292](https://github.com/stencila/stencila/commit/e189292a2adb2477fe288a31e4b3814ff01fd641))

## [0.118.4](https://github.com/stencila/stencila/compare/v0.118.3...v0.118.4) (2021-09-26)


### Bug Fixes

* **dependencies:** update rust crate json5 to v0.4.1 ([dd7e08b](https://github.com/stencila/stencila/commit/dd7e08bd41f9476a2a56a95b2476cb81eafb043f))
* **dependencies:** update rust crate schemars to v0.8.5 ([82741f6](https://github.com/stencila/stencila/commit/82741f6671ddfedbb685bfb71d67b2611777593c))
* **dependencies:** update rust crate tracing-subscriber to v0.2.24 ([8fc8c66](https://github.com/stencila/stencila/commit/8fc8c66d5d8e45c461c8a514d863dd9081a53546))

## [0.118.3](https://github.com/stencila/stencila/compare/v0.118.2...v0.118.3) (2021-09-23)


### Bug Fixes

* **dependencies:** update dependency @sentry/electron to v2.5.4 ([96b009d](https://github.com/stencila/stencila/commit/96b009dcd0bdd11324f336fac74452ccced63b6a))
* **dependencies:** update dependency fp-ts to v2.11.3 ([3f3451d](https://github.com/stencila/stencila/commit/3f3451dd4597b2bac58073caf51c495628337429))

## [0.118.2](https://github.com/stencila/stencila/compare/v0.118.1...v0.118.2) (2021-09-19)


### Bug Fixes

* **Config:** Remove unecessary validate attributes ([61108b9](https://github.com/stencila/stencila/commit/61108b95d0c123b46dc05c9459b96b96ba2b21fe))
* **dependencies:** update rust crate flate2 to v1.0.22 ([3cb0237](https://github.com/stencila/stencila/commit/3cb02375445c4c2dffcd279fe50b346de7951cfa))
* **dependencies:** update rust crate json5 to v0.4.0 ([d5e5350](https://github.com/stencila/stencila/commit/d5e5350a0b9ec8bac3be885adc080a2a968d9604))
* **dependencies:** update rust crate schemars to v0.8.4 ([14b6d9f](https://github.com/stencila/stencila/commit/14b6d9f49a7fbfc7cd74ebd7a31b2d818791efa7))
* **dependencies:** update rust crate serde_json to v1.0.68 ([65d844e](https://github.com/stencila/stencila/commit/65d844ef6dc563ecdec66598a73bcadfc51faeb5))
* **dependencies:** update rust crate tracing to v0.1.28 ([29b9479](https://github.com/stencila/stencila/commit/29b947920b20cdb2d8c93ce04ee44f2c9b876b12))
* **dependencies:** update rust crate tracing-subscriber to v0.2.23 ([ba94992](https://github.com/stencila/stencila/commit/ba94992ba06493416782490e638444dd49d84f36))

## [0.118.1](https://github.com/stencila/stencila/compare/v0.118.0...v0.118.1) (2021-09-15)


### Bug Fixes

* **dependencies:** update dependency @sentry/electron to v2.5.3 ([1494bf7](https://github.com/stencila/stencila/commit/1494bf79ba0f7622421cf4b83b9daafdc6fdc514))
* **dependencies:** update dependency @stencila/brand to v0.7.4 ([8a03e6f](https://github.com/stencila/stencila/commit/8a03e6fa50a7e670462305ae056c35dc1ca916bf))
* **dependencies:** update dependency i18next to v20.6.1 ([dcd61c8](https://github.com/stencila/stencila/commit/dcd61c8fdca16cc6cfdf61e0d10ff60a00cd8e8c))

# [0.118.0](https://github.com/stencila/stencila/compare/v0.117.0...v0.118.0) (2021-09-14)


### Bug Fixes

* **Documents:** Make serialization of relations more robust and less lossy ([018ad39](https://github.com/stencila/stencila/commit/018ad3992cd3b8de3119f7eb81cf6ee0cbf0a564)), closes [#1118](https://github.com/stencila/stencila/issues/1118)


### Features

* **Plugins:** Remove mention of Plugins from desktop client ([871f7a6](https://github.com/stencila/stencila/commit/871f7a6c289258c2784e5718b8776340debad177)), closes [#1137](https://github.com/stencila/stencila/issues/1137)


### Performance Improvements

* **Graph:** Remove Graph update listeners when closing Graph view ([e96d02d](https://github.com/stencila/stencila/commit/e96d02d4ab325cfea39771bd5cfe0bc868053729))

# [0.117.0](https://github.com/stencila/stencila/compare/v0.116.1...v0.117.0) (2021-09-13)


### Bug Fixes

* **IPYNB Encoding:** Encode article metadata ([0712026](https://github.com/stencila/stencila/commit/071202659d1645e284c5fbb2ff98f2823f4b9b6b))
* **IPYNB Encoding:** Encode image outputs ([00e3da2](https://github.com/stencila/stencila/commit/00e3da2ebdb73eae57a46391e1bb3a2bbf977edd))
* **IPYNB Encoding:** Indent using one space ([57bd63f](https://github.com/stencila/stencila/commit/57bd63f8b122ea9f73aac9cfe9ade3b2f88133a6))
* **Markdown:** Remove dollar-brace code expression syntax due to clash with math syntax ([a17580b](https://github.com/stencila/stencila/commit/a17580b1bb7a1bfaa5ac64d3710bf7c978ba8dfa))
* **Markdown & IPYNB:** Allow default language to be supplied ([64229e5](https://github.com/stencila/stencila/commit/64229e5b85c42008ba906ab274ecac3026f5310d))


### Features

* **IPYNB:** Add decoding and encoding of Jupyter Notebooks ([4ee5129](https://github.com/stencila/stencila/commit/4ee5129dffb41f8630a0a6b5507c467833ca942c))
* **IPYNB:** Add encoding of Jupyter notebooks ([5fe7ed9](https://github.com/stencila/stencila/commit/5fe7ed9d3dbf9f1ff1fd72f676cf357f757ec26f))
* **IPYNB:** Convert pre-formatted text output to `CodeBlock` ([31d7b63](https://github.com/stencila/stencila/commit/31d7b633414681d80325df0224d8e90a0da8d156))
* **IPYNB:** Convert stderr streams and errors into `CodeError`s ([ca59335](https://github.com/stencila/stencila/commit/ca59335c12fc2f764a9a4ed7d58d10a9dadc0345))
* **IPYNB:** Decode plain text cell outputs into nodes where possible ([231341c](https://github.com/stencila/stencila/commit/231341c1fba8ce9b4d6392b0721b9690054d0d3d))
* **IPYNB:** Handle Plotly and Vega outputs ([5c2d090](https://github.com/stencila/stencila/commit/5c2d090a4de92c22f6388bea116227fa8ad9d858))
* **IPYNB:** Handle top level metadata in notebook ([f4491f6](https://github.com/stencila/stencila/commit/f4491f6cdb7ef35ae0136634876529b5e964b9c4))
* **MD, IPYNB:** Support syntax for inline code nodes ([eccf426](https://github.com/stencila/stencila/commit/eccf4260b739fca18b04eb4e7120003ac379892c))

## [0.116.1](https://github.com/stencila/stencila/compare/v0.116.0...v0.116.1) (2021-09-12)


### Bug Fixes

* **dependencies:** update rust crate handlebars to v4.1.3 ([ee65be4](https://github.com/stencila/stencila/commit/ee65be4287d08e24eafdb00f8f0f07024a17a3df))
* **dependencies:** update rust crate serde_yaml to v0.8.21 ([c45a920](https://github.com/stencila/stencila/commit/c45a92045b1d2f7b6a2738060cfa2efa39388a23))
* **dependencies:** update rust crate sha2 to 0.9.8 ([bc66834](https://github.com/stencila/stencila/commit/bc66834e7012a89f24458ae7361ed44bccc975c4))
* **dependencies:** update rust crate similar to v2 ([a55eeb3](https://github.com/stencila/stencila/commit/a55eeb37007695d25d540b16e58df56d83ed822c))
* **dependencies:** update rust crate termimad to v0.16.1 ([f43f150](https://github.com/stencila/stencila/commit/f43f150fc356601604b5cc08001a59ccb2961842))

# [0.116.0](https://github.com/stencila/stencila/compare/v0.115.0...v0.116.0) (2021-09-09)


### Bug Fixes

* **Block patches:** Add fields for heading ([a5ccc74](https://github.com/stencila/stencila/commit/a5ccc74c86c6710edcbeb70d0dc36ffeb005de3f))
* **Document merge:**  Make CLI command usable as a Git merge driver ([2810b25](https://github.com/stencila/stencila/commit/2810b2529d9e21134d41b28f8cce6edbf9a0b1ea))
* **Documents:** Use `debug` instead of `warn` log level ([bcf6af6](https://github.com/stencila/stencila/commit/bcf6af6c259b687ed9796a0f8c89d862255e5c85)), closes [#1135](https://github.com/stencila/stencila/issues/1135)
* **Patches:** Include type name of structs in hash ([28d410f](https://github.com/stencila/stencila/commit/28d410fd31dcdabb7bb65c96579f08ee5e4be6a4))
* **String patches:** Ignore linter warning which broke tests ([eb50280](https://github.com/stencila/stencila/commit/eb50280c1dc99452281174d74cb038e1f71e1c6e))
* **String patches:** Make unicode character aware ([2f122e4](https://github.com/stencila/stencila/commit/2f122e438ba568ca50b46081df814bd7e12a6224))
* **String patches:** Remove move operations ([6ecaa15](https://github.com/stencila/stencila/commit/6ecaa153502e83fa57272987e351b89eca247bea))
* **String patches:** Various fixes and generative testing ([7212a47](https://github.com/stencila/stencila/commit/7212a474c88e22eef8c68dab9a59c2fb50419e0b))
* **Vector patches:** Apply transform operations ([0b06cbc](https://github.com/stencila/stencila/commit/0b06cbcdff034415312962a1b9357e0363272167))
* **Vector patches:** Fix backwards moves ([0ba5262](https://github.com/stencila/stencila/commit/0ba5262450b6cfbd5c2427151ddfb6b9951ccf2f))
* **Vector patches:** Fix issue when nested replacements ([23dcbf7](https://github.com/stencila/stencila/commit/23dcbf7d5f9032dca7742fbc99688ea48e84dde2))
* **Vector patches:** Fix issue with indexing replacements ([01f1ca1](https://github.com/stencila/stencila/commit/01f1ca11154726692cc17228f191323c2696c4b4))
* **Vector patches:** Only match add operations at top level ([844fe67](https://github.com/stencila/stencila/commit/844fe678ea4c7bd3ff8de131b2a74e26a606b0ab))


### Features

* **Atomics patches:** Implement apply patch ([814e723](https://github.com/stencila/stencila/commit/814e723256bac40b0b792998f87df10219902167))
* **Document diffs:** Generate unified diffs in alternative formats ([f378a06](https://github.com/stencila/stencila/commit/f378a063eb07d60cb5d05d76a46bfa98b801e07a))
* **Inline content:** Implement patching for inline content nodes ([3f116bb](https://github.com/stencila/stencila/commit/3f116bbff9cfab9860b71a1d24909f575025c6c8))
* **Option patches:** Implement apply patch ([0717721](https://github.com/stencila/stencila/commit/071772133ce7050b3bed51a325a2ad08c60a0185))
* **Patches:** Add diff and merge CLI commands ([700cc47](https://github.com/stencila/stencila/commit/700cc47927dea38a01159bd17c3ce3bc39e56d73))
* **Patches:** Introduce functions and traits for diffing and patching document nodes ([1610291](https://github.com/stencila/stencila/commit/1610291295b644b8e5b0615163ef45ba585612e9))
* **String patches:** Add move operation ([4aa3e1f](https://github.com/stencila/stencila/commit/4aa3e1ff63f1dd909784d3736367df6e9173a5f7))
* **Vector patches:** Fine grained operations for replacement items ([731d852](https://github.com/stencila/stencila/commit/731d852b84e0f780246c0fc264fab08a78755044))
* **Vector patches:** Implement forward moves ([9cf4f33](https://github.com/stencila/stencila/commit/9cf4f33f11f27f7c695a3fe214e11bc98ecc34fd))

# [0.115.0](https://github.com/stencila/stencila/compare/v0.114.1...v0.115.0) (2021-09-09)


### Bug Fixes

* **Server:** Check and add option for run-as-root; add docs ([b4376d3](https://github.com/stencila/stencila/commit/b4376d3221b41ca6372d56bced583ea156462e2a))
* **Server:** Handle disconnections ([0d1106d](https://github.com/stencila/stencila/commit/0d1106d3dbceae285cf4c0c85b499a156d4279ec))
* **Server:** Improve error handling,; dedicated WebSocket path ([4a04b2f](https://github.com/stencila/stencila/commit/4a04b2f629cb408bcc49a988c3f80af22bba7ae7))
* **Server:** On;y run sudo check on systems where it is available ([e526cd7](https://github.com/stencila/stencila/commit/e526cd7700ad14d22480e86f6bed56e8ba4af04c))
* **Server:** Warning when key is set on command line ([ad086d7](https://github.com/stencila/stencila/commit/ad086d76098ffcc28352c0ded3c1f6ff11f2c4fc))
* **Session:** Finer grained subscriptions ([b78cbdb](https://github.com/stencila/stencila/commit/b78cbdbd98c960497a7fa0333a31df073a5880b2))
* **Web:** Client takes id ([b8e6fbf](https://github.com/stencila/stencila/commit/b8e6fbffd76863039716b5370ff3c34aa0ff112e))


### Features

* **Compile:** Identify all entity nodes ([950b1a2](https://github.com/stencila/stencila/commit/950b1a2d2a3337a5172b8be4da44724c00656e89))
* **HTML Encoding:** Encode ids on nodes ([5e6ad57](https://github.com/stencila/stencila/commit/5e6ad571fce9ecf50f3a246ee3d784c67fe61a78))
* **Server:** Manage clients and their subscriptions ([2761105](https://github.com/stencila/stencila/commit/276110597a2950d5fdfcfa68acbdfa6c7f6553ee))
* **Sessions:** Initial implementation of sessions ([2b658d7](https://github.com/stencila/stencila/commit/2b658d71cd6c23491bb9fe69db1cd5351fafeeaa))
* **Web:** Add main sessions functions ([78230d2](https://github.com/stencila/stencila/commit/78230d2cf1a3275feb5b39078a0650c24099fb71))
* **Web RPC:** Add handling of `documents` functions ([362dda6](https://github.com/stencila/stencila/commit/362dda6e14cc278e62f520d6742ed4cae89a735a))

## [0.114.1](https://github.com/stencila/stencila/compare/v0.114.0...v0.114.1) (2021-09-05)


### Bug Fixes

* **dependencies:** update docusaurus monorepo to v2.0.0-beta.6 ([1b7c8f6](https://github.com/stencila/stencila/commit/1b7c8f6ad2ee556b4f7f89244c69379c437bd287))
* **dependencies:** update rust crate flate2 to v1.0.21 ([4d696b2](https://github.com/stencila/stencila/commit/4d696b20d4f23f86bce4b882e251ea5d282439c3))
* **dependencies:** update rust crate futures to v0.3.17 ([d466145](https://github.com/stencila/stencila/commit/d46614528544afff3c70439f84c9ac72a7f39109))
* **dependencies:** update rust crate pyo3 to v0.14.5 ([48337ba](https://github.com/stencila/stencila/commit/48337bab052595572247a7692191d5f4541e7cc7))
* **dependencies:** update rust crate rust-embed to v6.2.0 ([1e705f0](https://github.com/stencila/stencila/commit/1e705f0b4fd5a0bfdf75d8bbd50652249bc7c414))
* **dependencies:** update rust crate rustyline to v9 ([92ca10c](https://github.com/stencila/stencila/commit/92ca10c4be941b6c8810314e35f3ea09b83adb1d))
* **dependencies:** update rust crate serde_with to v1.10.0 ([b37260b](https://github.com/stencila/stencila/commit/b37260b968ea57e5cc22e4f6fc16a32bce2ade75))
* **dependencies:** update rust crate structopt to v0.3.23 ([84628eb](https://github.com/stencila/stencila/commit/84628eb20f337791271bd02c43f99415f4c65925))
* **dependencies:** update rust crate termimad to v0.16.0 ([d96156a](https://github.com/stencila/stencila/commit/d96156ade09bb0744b253f53ae6ad7db583731c7))
* **dependencies:** update rust crate thiserror to v1.0.29 ([5ab61cf](https://github.com/stencila/stencila/commit/5ab61cf926e617bca875425ac9c7318dbb760e97))
* **dependencies:** update rust crate tokio to v1.11.0 ([1373a2e](https://github.com/stencila/stencila/commit/1373a2e8b932ab3569f3a66c567f82b5b42c15e5))

# [0.114.0](https://github.com/stencila/stencila/compare/v0.113.0...v0.114.0) (2021-09-01)


### Bug Fixes

* **Deps:** Only require `rust-embed` once; upgrade ([9e16213](https://github.com/stencila/stencila/commit/9e16213211d1e2efa4a26cdbbfdf3e7694096301))
* **Icons:** Fix rendering of icon components ([5e772c4](https://github.com/stencila/stencila/commit/5e772c4c3a32f90929b2cb8853f12226fe513d7c))


### Features

* **Desktop:** Add document state indicator icon to tabs ([3b5a571](https://github.com/stencila/stencila/commit/3b5a5717436652b1576d12a9ca417d4d3f4861e9))
* **Editor:** Add keyboard shortcuts for cycling between document tabs ([dc714c6](https://github.com/stencila/stencila/commit/dc714c689ce67f0aee054f40fbe454d64b588ddb))

# [0.113.0](https://github.com/stencila/stencila/compare/v0.112.0...v0.113.0) (2021-08-31)


### Bug Fixes

* **Binaries:** Do not clear `REQUIRES` map to avoid deadlock ([d317c08](https://github.com/stencila/stencila/commit/d317c08a700cc00afe0632ff3389e9f97a270b5a))
* **Compile:** Move to using strings for hashes ([013c513](https://github.com/stencila/stencila/commit/013c513286027822d4a0442df0f72096b6d416c8))
* **Dependencies:** Upgrade Schema ([29808ab](https://github.com/stencila/stencila/commit/29808abc7b914153052e11e1e4275860fa0bff2b))
* **Documents:** When writing as other format ensure standalone ([07e33a9](https://github.com/stencila/stencila/commit/07e33a9c94fad92fcdbbe68d9d0c87bf2db47486))
* **Formats:** Allow decoding of RPNGs ([b196d42](https://github.com/stencila/stencila/commit/b196d427887ba721b76009befa9da90836a9b210))
* **HTML:** Use consistent attribute name for programming language ([f9dbfb9](https://github.com/stencila/stencila/commit/f9dbfb9888b438b915119d31bdb8db0319a35461))
* **HTML encoding:** Use an ordered map to avoid re-ording of affiliations ([7cc1dea](https://github.com/stencila/stencila/commit/7cc1deaa8aa78293cba3e43bc0b2c0e30bb2b0ac))
* **Makdown decoding:** Use title and caption ([ca46470](https://github.com/stencila/stencila/commit/ca464708bb759866d99efdba359406c4e6d7874a))
* **Markdown decoding:** Handle code chunks and expressions ([c158436](https://github.com/stencila/stencila/commit/c15843687a9ccb943ca67e10368f4a235af937f9))
* **Markdown decoding:** Trim code chunk language ([d5a4763](https://github.com/stencila/stencila/commit/d5a476323316aa2943db42a3bf0e87438690d2eb))
* **Pandoc decode:** Use title and caption ([f9fb2ec](https://github.com/stencila/stencila/commit/f9fb2ec79657fa2e52b388b70feb295cf002aed8))
* **Pandoc decoding:** Also check for node URL when decoding ([8873697](https://github.com/stencila/stencila/commit/887369707334cd1bcdacbbb37f12de6a2d746540))
* **PNG encoding:** Return early if possible ([6cd52a9](https://github.com/stencila/stencila/commit/6cd52a905e916b381df325aa1ca2e91ba06a325c))
* **R markdown:** Handling of specific format for code chunks and expressions ([9975b42](https://github.com/stencila/stencila/commit/9975b42087a997d83752cd6d8041d2994b28dd0b))
* **R Markdown:** Implement encoding ([b7cb681](https://github.com/stencila/stencila/commit/b7cb681cf946b5b8960bbf60c69a06f95daa74cc))


### Features

* **CLI:** Add option for `convert` to stdout ([eae08fc](https://github.com/stencila/stencila/commit/eae08fc1a9ebb4506ee3862244ec41c8035b599d))
* **Encoding JSON:** Add option for compact or indented ([5ad37af](https://github.com/stencila/stencila/commit/5ad37af9dd883d99bf05410480e52918e9c5bace))
* **HTML:** Encoding and decoding of code nodes ([4deafef](https://github.com/stencila/stencila/commit/4deafef336fd0029bf7dde96f545aad7b0fe6c26))
* **Markdown encoding:** Encode code expressions and chunks ([da64b64](https://github.com/stencila/stencila/commit/da64b6429b275231e2e8c758fd5c9081f26702ca))
* **Pandoc:** End-to-end encoding and decoding for code ([7d0d12d](https://github.com/stencila/stencila/commit/7d0d12d2145a2ca3bf0b81f6553c0d07f3b5d78b))
* **Pandoc encoding:** Encode to code chunks to RPNGs ([c2c7148](https://github.com/stencila/stencila/commit/c2c714841905088024c610a3581229db50aaf427))
* **RPNG:** Initial version of encoding and decoding ([ad30b3e](https://github.com/stencila/stencila/commit/ad30b3eebde6e2c707cf591bb09b4e6044afde35))

# [0.112.0](https://github.com/stencila/stencila/compare/v0.111.1...v0.112.0) (2021-08-30)


### Bug Fixes

* **Desktop:** Close launcher window when opening Project using file browser ([391fc1d](https://github.com/stencila/stencila/commit/391fc1db2cfe8a0eed8a41a35d5403d632638ef5))
* **Desktop:** Don't crash when reloading project window ([7f21255](https://github.com/stencila/stencila/commit/7f212554ad7166a15f08bfbeb1f07ca6e075b5da))
* **Desktop:** Fix ordering of Recent Projects list ([01e5081](https://github.com/stencila/stencila/commit/01e5081d4a64b88b1ffb37b4266196c4c461820c))


### Features

* **Desktop:** Add sidebar nav to Project window ([87b8896](https://github.com/stencila/stencila/commit/87b88961877695a6af314fae397988a9a1ac5ff5))
* **Graphs:** Add UI for rendering Project Graph in Desktop client ([48ae974](https://github.com/stencila/stencila/commit/48ae974b2f4bb2f9d00460c23adf23f7bf3eaa52))

## [0.111.1](https://github.com/stencila/stencila/compare/v0.111.0...v0.111.1) (2021-08-29)


### Bug Fixes

* **dependencies:** update docusaurus monorepo to v2.0.0-beta.5 ([f98ed31](https://github.com/stencila/stencila/commit/f98ed310655393a98b5b7d25b4ce17287a4da340))
* **dependencies:** update rust crate neon to v0.9.1 ([1a76f13](https://github.com/stencila/stencila/commit/1a76f1341fd93f96a01c7c03725daf549c79a515))
* **dependencies:** update rust crate pyo3 to v0.14.4 ([dab290d](https://github.com/stencila/stencila/commit/dab290d3cd76e116adc58cf9ae459718f0923441))
* **dependencies:** update rust crate serde to v1.0.130 ([12f2fb7](https://github.com/stencila/stencila/commit/12f2fb7ef863d9bb7fea0ca0826faddded2728d9))
* **dependencies:** update rust crate serde_json to v1.0.67 ([de10000](https://github.com/stencila/stencila/commit/de1000085aa8c18c8586f75b4760c9bb663d4c9a))
* **dependencies:** update rust crate serde_yaml to v0.8.20 ([e4648ce](https://github.com/stencila/stencila/commit/e4648ce6c07f5c725fe36d61756693da80173781))
* **dependencies:** update rust crate sha2 to 0.9.6 ([259d857](https://github.com/stencila/stencila/commit/259d857a833cb7e5f53c7b85cfcdb431175bcaf0))
* **dependencies:** update rust crate termimad to v0.15.0 ([485c1fc](https://github.com/stencila/stencila/commit/485c1fc174627498f93841f947679745499f1ce3))

# [0.111.0](https://github.com/stencila/stencila/compare/v0.110.1...v0.111.0) (2021-08-27)


### Bug Fixes

* **Documents:** Overwrite document relations rather than always extending it ([3d9a4e2](https://github.com/stencila/stencila/commit/3d9a4e254d7ecf0ad32335f851a7515cc2fc1ce6))
* **Documents:** Remove file:// scheme when creating resource ([744fb3f](https://github.com/stencila/stencila/commit/744fb3f8385a6b6e12d74d4e8c816435a8548456))
* **Project graphs:** Always add the main file ([755c12c](https://github.com/stencila/stencila/commit/755c12ca3aaa9061f47399ae7affff4dd5900e33))
* **Project graphs:** Remove the project file path prefix when serializing ([72d26ba](https://github.com/stencila/stencila/commit/72d26ba994d7668b3358bc3bd5dffab250f74179))


### Features

* **Project graphs:** Recompile when there are file changes ([bdef1e3](https://github.com/stencila/stencila/commit/bdef1e383dbabd06a05b9ae857571cec986b74be))

## [0.110.1](https://github.com/stencila/stencila/compare/v0.110.0...v0.110.1) (2021-08-22)


### Bug Fixes

* **dependencies:** update rust crate nom to v7.0.0 ([7fe48a0](https://github.com/stencila/stencila/commit/7fe48a0b1f6db7559acf3418a07cbb01cd9c5432))
* **dependencies:** update rust crate pyo3 to v0.14.3 ([4ea92eb](https://github.com/stencila/stencila/commit/4ea92ebe37ce3068930738f251aa89605bd051f2))
* **dependencies:** update rust crate rust-embed to v6.0.1 ([5c684f8](https://github.com/stencila/stencila/commit/5c684f8a30139e8fcf58bb43bde7bd779964dd13))
* **dependencies:** update rust crate rustyline-derive to v0.5.0 ([d0194d5](https://github.com/stencila/stencila/commit/d0194d5291e996338260196cd4671abaa78b6fb9))
* **dependencies:** update rust crate serde to v1.0.128 ([ad97ef7](https://github.com/stencila/stencila/commit/ad97ef7be3c3df080fb9dd7d9db2b319dd899fd9))
* **dependencies:** update rust crate serde_yaml to v0.8.19 ([34d46b8](https://github.com/stencila/stencila/commit/34d46b8b85bd946c3036db0da97d7502ba707a04))
* **dependencies:** update rust crate tracing-subscriber to v0.2.20 ([73625ee](https://github.com/stencila/stencila/commit/73625eefce913dfbf0211973cf7e5c4c9b14d10c))

# [0.110.0](https://github.com/stencila/stencila/compare/v0.109.2...v0.110.0) (2021-08-18)


### Bug Fixes

* **CLI:** Do not highlight content when using non-TTY devices ([dfc40dd](https://github.com/stencila/stencila/commit/dfc40ddba6a3de0fc23a88395ae4c3ea722c7283))
* **Code analysis:** Allow expressions for first arg when detecting R write files ([add8741](https://github.com/stencila/stencila/commit/add8741913c113c580bd8ffd97b093b32717741d))
* **Compile code:** Use forward slash when merging paths ([cb802c6](https://github.com/stencila/stencila/commit/cb802c64835224f8998dbac82e987f8980b4ac4f))
* **Dependencies:** Upgrade to Schema v1.11.0 ([e288bf6](https://github.com/stencila/stencila/commit/e288bf61f9b820c467d452dec43b9233ca1bfa7e))
* **File paths:** Use `lexiclean` for cross-platform path normalization ([a3e5775](https://github.com/stencila/stencila/commit/a3e5775607854cdf84ac6d4e77e46e38a0281828))
* **Graphs:** Serialize paths as Unix forward slash paths ([c201e15](https://github.com/stencila/stencila/commit/c201e1556d97a724c9f68c3dcee36c4f9eb41eaa))
* **Project graphs:** Avoid cycles when walking over files ([cd7441a](https://github.com/stencila/stencila/commit/cd7441a7becfb521b4d2440e4b7ec09cdad0307a))
* **Project graphs:** Improve DOT visualizations ([dae57f3](https://github.com/stencila/stencila/commit/dae57f3442a6e8f5db3146b6b2d914ef6701ccec))
* **Project graphs:** Use absolute paths for resources ([eca490c](https://github.com/stencila/stencila/commit/eca490ce9e603f17724a4836ccdaf5de84215445))
* **Projects:** Do not walk over subject in triple ([f4b0112](https://github.com/stencila/stencila/commit/f4b011258a82e85ae363adb50fd525349c3e2135))
* **Python and Javascript code analysis:** Enable ranges ([761a1f8](https://github.com/stencila/stencila/commit/761a1f8cf4473e0bae84ccd19e9468961407c2ca))
* **R and Python code analysis:** Capture ranges for reads, writes and imports ([978a2d3](https://github.com/stencila/stencila/commit/978a2d36b538acb34b79cd541f0f64ac40ee21bc))
* **R code analysis:** Ignore attributes ([19870da](https://github.com/stencila/stencila/commit/19870da23e6b2e53b76dfc6e900afcc0603064dd))
* **R code analysis:** Use correct function name ([c71dbcb](https://github.com/stencila/stencila/commit/c71dbcb0aadff7ac87dfa36a161b8eece61a4d25))


### Features

* **Code analysis:** Add ranges to relations ([f5b75b7](https://github.com/stencila/stencila/commit/f5b75b7b13a71f1ce7578b966192b14f8b45d25c))
* **Code analysis:** Allow manual overrides using tags in comments ([8784e05](https://github.com/stencila/stencila/commit/8784e05ac43c4a0003144272f0b0466c8983fc0f))
* **Code analysis:** Decode and compile entire source code files ([9707eef](https://github.com/stencila/stencila/commit/9707eef281fd65eecfdb57e54b9755c8b2460817))
* **Code analysis:** Detect files read for JavaScript ([7a0bf41](https://github.com/stencila/stencila/commit/7a0bf4177415adb598fabbf8b5f5d4a04d014f7f))
* **Code analysis:** Detect files read for Python and R ([60a0450](https://github.com/stencila/stencila/commit/60a04506215625b45a18bf495137c109ed77d344))
* **Code analysis:** Detect files written in JavaScript, Python and R ([e9b25a7](https://github.com/stencila/stencila/commit/e9b25a72e5280b6f07791f3564c3a68a89c11ba9))
* **Compile:** Add code compilation for dependency analysis for CodeChunks etc ([cf55b6b](https://github.com/stencila/stencila/commit/cf55b6b5c681cb4907094b6a760cd0a61bb29591))
* **Dependency graphs:** Include variable and function dependencies for R; visualization for projects ([652101d](https://github.com/stencila/stencila/commit/652101d0c98d055f85126b403b1cbe4f5e4255e9))
* **Documents:** Get the SHA-256 of a document ([d65939c](https://github.com/stencila/stencila/commit/d65939c26f35f0e590613c37bc117a1057e80103))
* **Graphs:** Add alternative visualization formats ([996296a](https://github.com/stencila/stencila/commit/996296acb0b952785d35ca26afd5b83586bca09a))
* **Include:** Allow documents to be included within others ([e9ab21f](https://github.com/stencila/stencila/commit/e9ab21f100e2d533568d3eb20aaa0ced4fb60a62))
* **JavaScript code analysis:** Detect assignments ([32264cc](https://github.com/stencila/stencila/commit/32264ccd96a133f457a17e36568363a3c1ddc41c))
* **JavaScript code analysis:** Detect usage of symbols ([21f127e](https://github.com/stencila/stencila/commit/21f127e71eb96d060ee2eeff510e5286a5dae5fa))
* **Node.js:** Allow project graph to be obtained in `dot` and other formats ([4d37304](https://github.com/stencila/stencila/commit/4d37304120af4cead62665bccf42684dec1bf179))
* **Project conversion:** Allow definition of conversion between files ([c3c574e](https://github.com/stencila/stencila/commit/c3c574ec51c01060de1b60da73438455ab16697f))
* **Project graphs:** Add sources and associated files ([4e9db6d](https://github.com/stencila/stencila/commit/4e9db6dcb89189ed10bd3d3a59798c77d91c1b4f))
* **Project graphs:** Export related JSON schemas ([d4745c3](https://github.com/stencila/stencila/commit/d4745c36d3f5aadb31696f2fcc1a6045b9a80157))
* **Project graphs:** Include document parameters ([c0db190](https://github.com/stencila/stencila/commit/c0db19056f02aa39cdd779f1875d7aae90d776f6))
* **Project sources:** Add flag for whether or not source is active ([ab94670](https://github.com/stencila/stencila/commit/ab9467066a19f1d87f15f44f8e48704a0a6df704))
* **Python code analysis:** Detect assignments ([374667b](https://github.com/stencila/stencila/commit/374667ba6c1f1e37d1c4527becb09d6fd298ec43))
* **Python code analysis:** Detect usage of symbols ([daf107e](https://github.com/stencila/stencila/commit/daf107e2242a7d6d47d3a94bc76a72fb398ff257))
* **R code analysis:** Improve inference of the type of assigned symbols ([db744d7](https://github.com/stencila/stencila/commit/db744d79e0264eacca1a561bced6079d313eadd3))
* **TypeScript code analysis:** Add analysis of TypeScript code ([17d14e2](https://github.com/stencila/stencila/commit/17d14e24cb750e8fcd25d92ec9e72a37fcae595d))

## [0.109.2](https://github.com/stencila/stencila/compare/v0.109.1...v0.109.2) (2021-08-16)


### Bug Fixes

* **dependencies:** update dependency @stencila/schema to v1.10.1 ([44c045c](https://github.com/stencila/stencila/commit/44c045c4d3c4c3dec3a159c86abd43c479ad3282))
* **dependencies:** update rust crate handlebars to v4.1.2 ([3c2f026](https://github.com/stencila/stencila/commit/3c2f02691d250531f8ccebaf94f852fc3b8e0a1c))
* **dependencies:** update rust crate pyo3 to v0.14.2 ([efab7b4](https://github.com/stencila/stencila/commit/efab7b4f0f0f0e75f0d1cfbe65a9fd3889ae0f40))
* **dependencies:** update rust crate tar to v0.4.37 ([4eaa530](https://github.com/stencila/stencila/commit/4eaa530a85a148c7ee5719ee427ae32892b1a92e))
* **dependencies:** update rust crate termimad to v0.14.2 ([307e963](https://github.com/stencila/stencila/commit/307e963783cd1347602e886f6b290f4ff77ec6d7))
* **dependencies:** update rust crate tokio to v1.10.0 ([9ea2d33](https://github.com/stencila/stencila/commit/9ea2d33608f8c0dc76673c38faf5be2b1538b189))

## [0.109.1](https://github.com/stencila/stencila/compare/v0.109.0...v0.109.1) (2021-08-08)


### Bug Fixes

* **dependencies:** update rust crate nom to v7.0.0-alpha2 ([43ffa9c](https://github.com/stencila/stencila/commit/43ffa9cbb2e1647c686e4854b47fc1f0481f3fb1))
* **dependencies:** update rust crate syntect to v4.6.0 ([7e6d7af](https://github.com/stencila/stencila/commit/7e6d7af38359d3c7cd76c301fb974c8a6c3fe5c7))
* **dependencies:** update rust crate termimad to v0.14.1 ([898293e](https://github.com/stencila/stencila/commit/898293e8fe0b7de273864cbe016736635eb51dd8))

# [0.109.0](https://github.com/stencila/stencila/compare/v0.108.0...v0.109.0) (2021-08-06)


### Features

* **Config:** Add config settings for editors ([9106e25](https://github.com/stencila/stencila/commit/9106e25aacb14ac46e0d5f917b15e2ff1cf84774))
* **Config:** Publish events when config is set or reset ([62ab17a](https://github.com/stencila/stencila/commit/62ab17a958b671e55ada33b2b8fcce8c2c709d63))

# [0.108.0](https://github.com/stencila/stencila/compare/v0.107.1...v0.108.0) (2021-08-06)


### Bug Fixes

* **Documents:** Convert content if necessary on load; check that binary file exists on update ([a1b8382](https://github.com/stencila/stencila/commit/a1b83827276518dc9757a8f478387ac614eb85dd))


### Features

* **Documents:** Add "Save as…" functionality to Desktop ([c8c3daa](https://github.com/stencila/stencila/commit/c8c3daa51269d7b8f22597224a1452c0344e8bf6))

## [0.107.1](https://github.com/stencila/stencila/compare/v0.107.0...v0.107.1) (2021-08-04)


### Bug Fixes

* **Tabs:** Fix spacing around tab close icon on Linux & Windows ([cb0b9aa](https://github.com/stencila/stencila/commit/cb0b9aa0eeacbefc4c83c5e8ea014ba697dd28b9))

# [0.107.0](https://github.com/stencila/stencila/compare/v0.106.0...v0.107.0) (2021-08-04)


### Bug Fixes

* **Desktop:** Fix crash on non Mac operating systems ([008adc1](https://github.com/stencila/stencila/commit/008adc178252f23dd38c7fbf330f36a647728957)), closes [#1059](https://github.com/stencila/stencila/issues/1059)


### Features

* **Desktop:** Add ability to view session logs in app ([bea03bb](https://github.com/stencila/stencila/commit/bea03bbcbc081c941715de4d559e76ba0dad1e15))

# [0.106.0](https://github.com/stencila/stencila/compare/v0.105.0...v0.106.0) (2021-08-03)


### Features

* **CLI:** Highlight output in non-interactive mode ([0d39278](https://github.com/stencila/stencila/commit/0d392782326fd0d82ff6a3f1de293ac7cb965d7b))

# [0.105.0](https://github.com/stencila/stencila/compare/v0.104.1...v0.105.0) (2021-08-02)


### Bug Fixes

* **dependencies:** update dependency fp-ts to v2.11.1 ([1def782](https://github.com/stencila/stencila/commit/1def7829262eceaac1ea71f72b668be5c43ae815))
* **Onboarding:** Don't create new Onboarding window if it exists already ([57a2f5b](https://github.com/stencila/stencila/commit/57a2f5bdffb9c8cd4434439e2fefc16111d019b1))
* **Settings:** Standardize spelling of settings items ([7fddc8e](https://github.com/stencila/stencila/commit/7fddc8eb5ead1d68bd785e6b92839ddf9500c8f0))


### Features

* **Desktop:** Enable "New Project" workflow ([431f3c5](https://github.com/stencila/stencila/commit/431f3c56ae2645d264875c77ccc39f896e5946c4))

## [0.104.1](https://github.com/stencila/stencila/compare/v0.104.0...v0.104.1) (2021-08-01)


### Bug Fixes

* **dependencies:** update docusaurus monorepo to v2.0.0-beta.4 ([b9c5680](https://github.com/stencila/stencila/commit/b9c568034f454c64d166ce522d5ca8c298277e73))
* **dependencies:** update rust crate async-trait to v0.1.51 ([d53ed76](https://github.com/stencila/stencila/commit/d53ed76fc2ebcf642f9d7a80c7cc046d4a7e92c4))
* **dependencies:** update rust crate handlebars to v4.1.1 ([aba4f44](https://github.com/stencila/stencila/commit/aba4f44a508ff061fab8d0cc0acf2f1b11b62a48))
* **dependencies:** update rust crate jsonschema to v0.12.1 ([06966c9](https://github.com/stencila/stencila/commit/06966c916a68434bc87491c37d58391e261f5c9a))
* **dependencies:** update rust crate neon to v0.9.0 ([897ee88](https://github.com/stencila/stencila/commit/897ee88cee4ca9a712d68a9b293a0bd3165fb490))
* **dependencies:** update rust crate rust-embed to v6 ([ed6f2e9](https://github.com/stencila/stencila/commit/ed6f2e917ccd73db2b5475bc9efd78e81157b739))
* **dependencies:** update rust crate semver to v1.0.4 ([80a1eb1](https://github.com/stencila/stencila/commit/80a1eb1e4c22bf6c95527ceb3e2cc6f70ed7977b))
* **dependencies:** update rust crate serde to v1.0.127 ([d8502a6](https://github.com/stencila/stencila/commit/d8502a6ae6cc3569587ae4e5de929ab14d51534a))
* **dependencies:** update rust crate serde_json to v1.0.66 ([1970155](https://github.com/stencila/stencila/commit/197015547c64feec0e856a3e4366d1a525a845b3))
* **dependencies:** update rust crate which to v4.2.2 ([ca82078](https://github.com/stencila/stencila/commit/ca8207835df29b9895e158a3b90ec534defb6222))
* **Serve:** Update for API change to `rust-embed` ([d108850](https://github.com/stencila/stencila/commit/d108850e2d7797839d0dd589830c7e4fc55ca1be))

# [0.104.0](https://github.com/stencila/stencila/compare/v0.103.0...v0.104.0) (2021-07-30)


### Bug Fixes

* **Documents:** Set previewable flag based on format ([153018a](https://github.com/stencila/stencila/commit/153018aa2da1d11a1c62e7d4f1e5ba165f844a54))
* **Documents:** Set root to null if no content ([472c1e8](https://github.com/stencila/stencila/commit/472c1e8376b5acd47feda7d7540cbc39bba545a7))
* **Menu:** Re-add "Save" menu item to Project windows ([5bf0ffe](https://github.com/stencila/stencila/commit/5bf0ffebf2ce925c2ab35c0557292ec87330f2d6))
* **Previews:** Regenerate Document preview when altering format ([44f3a87](https://github.com/stencila/stencila/commit/44f3a87af4dc8c9820ca13e088de02d5b0f2edc5))
* **Settings:** Fix reloading of chosen syntax from user config store ([4705abf](https://github.com/stencila/stencila/commit/4705abf6234d8dc03e6d420f8573c8670fa919f5))
* **Settings:** Path user config file with any missing default values ([ca8c43a](https://github.com/stencila/stencila/commit/ca8c43a402f5285a4d10d8930f2326f9105f5c8c))


### Features

* **Desktop:** Add ability to create new empty documents ([7deb8b1](https://github.com/stencila/stencila/commit/7deb8b101bb8a1d4eb70e14439b0f31d99b2e93a))
* **Documents:** Add additional Document call IPC handlers to Desktop ([48154e2](https://github.com/stencila/stencila/commit/48154e2a67566d2e53bca17ec8abb53b7cf14080))
* **Editor:** Update internal Document format when changing language ([31168d4](https://github.com/stencila/stencila/commit/31168d42a085bc1921c3a840fa4f73c3128f9ba9))
* **Settings:** Expose editor settings in UI ([50e3d87](https://github.com/stencila/stencila/commit/50e3d8710ab2ccde4ddc798d680ded92c1a2f248))

# [0.103.0](https://github.com/stencila/stencila/compare/v0.102.1...v0.103.0) (2021-07-28)


### Features

* **Documents:** Allow alteration of path and format of a document ([ef420aa](https://github.com/stencila/stencila/commit/ef420aa81c8613e46d22154323c8d382f5f1cf4b))

## [0.102.1](https://github.com/stencila/stencila/compare/v0.102.0...v0.102.1) (2021-07-27)


### Bug Fixes

* **dependencies:** update dependency @reduxjs/toolkit to v1.6.1 ([bfca95c](https://github.com/stencila/stencila/commit/bfca95ce6e19428d3b6cc11ee6bd099d195df15e))
* **dependencies:** update dependency i18next to v20.3.5 ([6dbf329](https://github.com/stencila/stencila/commit/6dbf3299ce1729cc2be98ee13d019038bea7208e))
* **dependencies:** update dependency uuid to v8 ([63cfb77](https://github.com/stencila/stencila/commit/63cfb771b3d5df8f40d1c02a22e117be5d0e188e))

# [0.102.0](https://github.com/stencila/stencila/compare/v0.101.0...v0.102.0) (2021-07-27)


### Features

* **Editor:** Toggle visibility of editor and preview sections ([3ddb7b6](https://github.com/stencila/stencila/commit/3ddb7b6e4bfb544eb972ca798b79db933c98a0f2))
* **Preview:** Add auto-updating document previews ([5ea9e4a](https://github.com/stencila/stencila/commit/5ea9e4a470c12d8a999d8be332a0096b1c656ccf))


### Performance Improvements

* **Preview:** Debounce live document preview generation for better UX ([87dd76d](https://github.com/stencila/stencila/commit/87dd76d432419832735b7bb288355521a8b95076))

# [0.101.0](https://github.com/stencila/stencila/compare/v0.100.0...v0.101.0) (2021-07-27)


### Bug Fixes

* **dependencies:** update rust crate futures to v0.3.16 ([4ff1108](https://github.com/stencila/stencila/commit/4ff1108a29c6582053a9d57f19c344c7fbd8a7e9))
* **dependencies:** update rust crate jsonschema to v0.12.0 ([7033502](https://github.com/stencila/stencila/commit/703350283d9171ff8f69371a0b168b78cdba9729))
* **dependencies:** update rust crate tokio to v1.9.0 ([2333d82](https://github.com/stencila/stencila/commit/2333d82d58f637889cd82469dd8e04051ffb07ca))
* **Plugins:** Necessary changes for upgraded `jsonschema` Rust crate ([80241d4](https://github.com/stencila/stencila/commit/80241d43b59b568e5f1ff15fd03701eeb4877305))


### Features

* **Node.js:** Add functions for managing project sources ([fee45df](https://github.com/stencila/stencila/commit/fee45dff372366431f13796cea87636a903b10b2))
* **Node.js:** Expose `documents.writeAs` function ([07e08e8](https://github.com/stencila/stencila/commit/07e08e8fd25bef9e74562ae35edf28398c00580d))

# [0.100.0](https://github.com/stencila/stencila/compare/v0.99.0...v0.100.0) (2021-07-23)


### Features

* **Desktop:** Print app version in Launcher window ([f3b3d61](https://github.com/stencila/stencila/commit/f3b3d61c5a0492fee3236c7f44d639cf224f58d9))
* **Editor:** Expose settings for toggling line wrapping & line numbers ([a7d2b77](https://github.com/stencila/stencila/commit/a7d2b77bc7ce41cee768b1f97706436ca5006850))

# [0.99.0](https://github.com/stencila/stencila/compare/v0.98.1...v0.99.0) (2021-07-22)


### Bug Fixes

* **Markdown, HTML, Pandoc:** Handle table headers ([adc92a0](https://github.com/stencila/stencila/commit/adc92a0b9bc024c73be203f4569f11d9d64bf891))
* **Pandoc:** Differentiate media type when decoding images ([5013f9a](https://github.com/stencila/stencila/commit/5013f9a7a2e8af1cd5b99ee87182d92d66b4a27c))
* **Pandoc:** Encode code chunk programming language ([fecd60b](https://github.com/stencila/stencila/commit/fecd60bb449569156ce691db707856cd8c96dc8a))
* **Plugins:** Do not overwrite the manifest of the installed version ([4bcaa34](https://github.com/stencila/stencila/commit/4bcaa34dd804d9f91675407e247e285c14def797)), closes [#1050](https://github.com/stencila/stencila/issues/1050)


### Features

* **HTML:** Initial implementation of decoding for tables ([a47a31b](https://github.com/stencila/stencila/commit/a47a31b27305f09c92f5acf6c7409a6307261963))
* **Pandoc:** Add support for encoding to various formats via Pandoc ([75ee4eb](https://github.com/stencila/stencila/commit/75ee4ebe3cb6e13e43c7a17159482a745223bd98))
* **Pandoc:** Encode lists ([18463a4](https://github.com/stencila/stencila/commit/18463a4f05bb2d7a1bda4aa305a63c383b12dba2))
* **Pandoc:** Implement encoding of simple tables ([3dc9e44](https://github.com/stencila/stencila/commit/3dc9e44db80579944bb81056445158fefc90741c))
* **PDF:** Add encoding to PDFs ([b9f83a8](https://github.com/stencila/stencila/commit/b9f83a8fecd423e043b05afd21acb585aae8e6fb))


### Performance Improvements

* **Binaries:** Memoize require calls ([3a1e243](https://github.com/stencila/stencila/commit/3a1e2434a12867fbf0589de8ca3ed65fb0a35ea2))
* **HTML:** Use concat instead of format when encoding lists ([7984768](https://github.com/stencila/stencila/commit/7984768c874a83d32d3bd545c2bc6454a51952a7))

## [0.98.1](https://github.com/stencila/stencila/compare/v0.98.0...v0.98.1) (2021-07-20)


### Bug Fixes

* **Desktop:** Fix periodic failure to open project window ([39e2ccf](https://github.com/stencila/stencila/commit/39e2ccf8e7d0a49e78656650688c247488dab409))
* **Desktop:** Race condition when loading window & calling IPC method ([80f02df](https://github.com/stencila/stencila/commit/80f02dfa5306586610cd2e02eac3ff6f66f44f71))

# [0.98.0](https://github.com/stencila/stencila/compare/v0.97.4...v0.98.0) (2021-07-19)


### Bug Fixes

* **HTML & Markdown:** Support encoding and decoding of quote blocks ([55b3e78](https://github.com/stencila/stencila/commit/55b3e781a12c44c11d14bbf568e5b905ca0b7d88))
* **Markdown:** Decode media object content ([8dc5191](https://github.com/stencila/stencila/commit/8dc51914362e2300bff245e6363cfcfc32443d5f))
* **Markdown:** Differentiate media files; trim code block text; improve subscript parsing ([1be17c4](https://github.com/stencila/stencila/commit/1be17c4aeb0f60bed4e964f6a14a4fee7a90ba73))
* **Markdown:** Fix handling of inline HTML elements ([a385f1d](https://github.com/stencila/stencila/commit/a385f1d18f0bb74b56bb0f215fa420d0efda376b))
* **Markdown:** Improve decoding of YAML frontmatter ([2495384](https://github.com/stencila/stencila/commit/2495384635afb48d5e0135f3b9d8402fa655f3c1))
* **Markdown:** Improve encoding of lists ([4e11257](https://github.com/stencila/stencila/commit/4e112576c435d306454575f02d281c39defb585f))


### Features

* **Elife:** Add eLife article source ([fcd34a5](https://github.com/stencila/stencila/commit/fcd34a5cfbc968c8c34dda4c83d87f21a584fba8))
* **HTML & Markdown:** Support decoding of inline quote nodes ([d87d90b](https://github.com/stencila/stencila/commit/d87d90babb1b5623072490adef8a0ecbd66c231c))
* **Markdown:** Initial implementation of encoding to Markdown ([87bea25](https://github.com/stencila/stencila/commit/87bea251b81c8f603168eac3e5552d8e706e56f0))
* **Sources:** Add project sources and CLI subcommand for managing them ([536aa83](https://github.com/stencila/stencila/commit/536aa83ac011ebb30e730be4374d395d7a32d390))
* **Sources:** Improve matching and naming of project sources ([66f0f48](https://github.com/stencila/stencila/commit/66f0f488bc842ea0d4cbaa23bb28f413e86c2215))
* **Utilities:** Add download function ([7acba07](https://github.com/stencila/stencila/commit/7acba07c5a6fcb53300c07df1cb56886d74ff63d))

## [0.97.4](https://github.com/stencila/stencila/compare/v0.97.3...v0.97.4) (2021-07-16)


### Bug Fixes

* **Types:** Fix CaptureError IPC call type signature ([3bcfff1](https://github.com/stencila/stencila/commit/3bcfff1d5d87762f175b35c48788ca1676b0f9c0))

## [0.97.3](https://github.com/stencila/stencila/compare/v0.97.2...v0.97.3) (2021-07-16)


### Bug Fixes

* **Desktop:** Only remove IPC handlers if no windows require them ([1b39bba](https://github.com/stencila/stencila/commit/1b39bba37cb82f85c314387a55c4d8b5a02439d3))
* **Editor:** Correctly set editor state after previewing media files ([1ed4d8e](https://github.com/stencila/stencila/commit/1ed4d8e198e355fa9a4837a2fc85bb226ef16132))
* **Errors:** Report unhandled errors ([29f9748](https://github.com/stencila/stencila/commit/29f97481b42b2ac4ae138b86cea82d935ac4e8d6))

## [0.97.2](https://github.com/stencila/stencila/compare/v0.97.1...v0.97.2) (2021-07-15)


### Bug Fixes

* **Build:** Statically link lzma-sys crate ([9a54d70](https://github.com/stencila/stencila/commit/9a54d7043078ab2750b97582e32cbd3b871e15cb))
* **Documents:** Watch parent folder; read and update binary files ([72fd25f](https://github.com/stencila/stencila/commit/72fd25f417e9e5741fa20ea0c8fb58bba80f0e0c))
* **Projects:** Canonicalize the main path of a project ([b575478](https://github.com/stencila/stencila/commit/b575478017f8b4bd78c34fb7739c368b95bba306))

## [0.97.1](https://github.com/stencila/stencila/compare/v0.97.0...v0.97.1) (2021-07-15)


### Bug Fixes

* **dependencies:** update dependency @sentry/electron to v2.5.1 ([09204b6](https://github.com/stencila/stencila/commit/09204b6db2c88b2bab0dbd702da61f108496aff1))
* **Documents:** Add `previewable` property to indicate whether preview panel should be opened by default ([9830da3](https://github.com/stencila/stencila/commit/9830da3be8bffc73601d7b3780ee72acd50e80c1))

# [0.97.0](https://github.com/stencila/stencila/compare/v0.96.0...v0.97.0) (2021-07-14)


### Features

* **Desktop:** Highlight project main file in file tree ([26ef357](https://github.com/stencila/stencila/commit/26ef357a388d38ed71e9d92189368fba2b8b5c40))
* **Desktop:** If project has main file, open it on project launch ([705b894](https://github.com/stencila/stencila/commit/705b8944df788f1fdd6b2a364f527e97284f4dd4))
* **Document Tabs:** Adjust position of close icon based on user OS ([45969c0](https://github.com/stencila/stencila/commit/45969c0b240c78f5d2f43d3cefc50ee23b42aae3))
* **Launcher:** Add button for opening Settings window ([84acb95](https://github.com/stencila/stencila/commit/84acb952108d4f161fef0805977e271cbc51c1c0))
* **Launcher:** Refine design of Project Launcher window ([692c4e6](https://github.com/stencila/stencila/commit/692c4e6957a51f51efb01fa8f2c4a03103181cbd))

# [0.96.0](https://github.com/stencila/stencila/compare/v0.95.0...v0.96.0) (2021-07-14)


### Bug Fixes

* **Formats:** Unregistered formats have binary false ([37d13a8](https://github.com/stencila/stencila/commit/37d13a86e07d78028d9ff6305445da645d9a1415))
* **Node.js:** Add message to error types; update all types ([f7f53ff](https://github.com/stencila/stencila/commit/f7f53ffc2321faade200d4bb0daa379a70134cfd))


### Features

* **Desktop:** Surface errors to users as notifications ([04e42df](https://github.com/stencila/stencila/commit/04e42df3489e44e278370181eed728a24eea9049))
* **Formats:** Add a known property to formats ([979c86c](https://github.com/stencila/stencila/commit/979c86c6846b711f207b41fe96967662aa3a35ee))
* **Launcher:** Make project names more prominent ([aeb68fe](https://github.com/stencila/stencila/commit/aeb68fe9c097feb8074f9db97547ad68619a85cf))
* **Node.js:** Add dispatch function to handle RPC calls ([a36d396](https://github.com/stencila/stencila/commit/a36d396c9efd5339fc6454d46492322597dd4dcb))
* **Node.js:** Export dispatch function variants ([06e202b](https://github.com/stencila/stencila/commit/06e202bccbbda10b8b5abcc636c4188c99c6f279))

# [0.95.0](https://github.com/stencila/stencila/compare/v0.94.0...v0.95.0) (2021-07-14)


### Features

* **Coerce:** Implement coerce method ([80bc2c7](https://github.com/stencila/stencila/commit/80bc2c73fef6491d298e3137365b421dc0cafafa))

# [0.94.0](https://github.com/stencila/stencila/compare/v0.93.0...v0.94.0) (2021-07-12)


### Bug Fixes

* **dependencies:** update rust crate handlebars to v4.1.0 ([31e0a11](https://github.com/stencila/stencila/commit/31e0a11c9b98dc2a004ae0317cf14118de8c1ca0))
* **dependencies:** update rust crate termimad to v0.14.0 ([093d15c](https://github.com/stencila/stencila/commit/093d15c20e7a34c6e4780d6482f5535bdb240b28))
* **dependencies:** update rust crate tokio-tungstenite to v0.15.0 ([9d78457](https://github.com/stencila/stencila/commit/9d784578948d27431a1a14c4d84c4b39b1aa3e55))
* **HTML:** Add encoding of authors and their affiliations ([1da7dea](https://github.com/stencila/stencila/commit/1da7dea6e3f666b0de238987e65a7334f30bc792))
* **HTML:** Close name span ([6c28c7a](https://github.com/stencila/stencila/commit/6c28c7a1295e4d2d554a70c9fe1bdb8f6252aa1a))
* **Markdown:** Add parsing of YAML frontmatter ([8716dbd](https://github.com/stencila/stencila/commit/8716dbda07a3c55475d0ffa694984619247566b4))
* **Plain text:** Encode with newlines between blocks ([d57018a](https://github.com/stencila/stencila/commit/d57018a8dc21c3a2e75202fc4e9fee456cbdc02e))
* **Plain text:** Trim whitespace ([7a888ea](https://github.com/stencila/stencila/commit/7a888eaa916da9eec7719205cac58851eff9227c))
* **Reshape:** Add ampersand to separators ([88f561b](https://github.com/stencila/stencila/commit/88f561b77d5a1709294d3eaec8b1e0095e2af8bb))


### Features

* **Reshape:** Add initial implementaion of reshaping ([b52308f](https://github.com/stencila/stencila/commit/b52308f1414f0f969a5025314eb4386733d61734))

# [0.93.0](https://github.com/stencila/stencila/compare/v0.92.1...v0.93.0) (2021-07-08)


### Bug Fixes

* **Dependencies:** Upgrade tokio after cardo audit ([995f6fb](https://github.com/stencila/stencila/commit/995f6fb58bb521297f1030961cf577a63dd58eee))
* **Desktop:** Handle paths with spaces and other percent encoded names ([fb642b6](https://github.com/stencila/stencila/commit/fb642b6b6f218284c21a63145fed3ce02e7b1e12))
* **HTML:** Audio tag can not be self closing ([d785a10](https://github.com/stencila/stencila/commit/d785a1092621c34c0ea77dedd1c1562dba14dff9))
* **HTML:** Do not escape forward slashes in media object src attributes ([6d58513](https://github.com/stencila/stencila/commit/6d58513e89cfe33a54246ef116c89853bef70a9c))
* **HTML:** Only use data-itemscope once during encoding ([be7930b](https://github.com/stencila/stencila/commit/be7930b9c9aa4fe1697c53d67c061213c202b526))
* **HTML:** Reserve h1 for document title ([c7ef80a](https://github.com/stencila/stencila/commit/c7ef80afb4a56f4c145db2bb938f66a877b65b20))
* **Rust:** Upgrade pandoc-types ([5269f7f](https://github.com/stencila/stencila/commit/5269f7fd31153f00972deacd1f584ebc26ae0f57))
* **Serve:** Enclose rewritten path in double quotes ([9e4892e](https://github.com/stencila/stencila/commit/9e4892eaa3766c57945763bdc1289302db451669))


### Features

* **HTML:** Encode article description ([fd35e16](https://github.com/stencila/stencila/commit/fd35e166fb615bf67e4c7e4436d62f900496093b))
* **HTML:** Render list item checkboxes ([f2f93e9](https://github.com/stencila/stencila/commit/f2f93e9fd2884035ba6734009e8908028d28b017))
* **Plain text:** Add encoding to plain text ([74b2442](https://github.com/stencila/stencila/commit/74b2442b51f34c16e4d6caa29491e3adf0979b0f))

## [0.92.1](https://github.com/stencila/stencila/compare/v0.92.0...v0.92.1) (2021-07-07)


### Bug Fixes

* **Desktop:** Account for removal of format type ([2a590be](https://github.com/stencila/stencila/commit/2a590beb728cd5b4974a05ab18db717698208008))

# [0.92.0](https://github.com/stencila/stencila/compare/v0.91.0...v0.92.0) (2021-07-07)


### Bug Fixes

* **DOCX:** Extract media files when decoding ([cbf8c01](https://github.com/stencila/stencila/commit/cbf8c0130b58e6003ef4f77314b3773ed7f1090f))
* **Rust:** Update stencila-schema ([e59d3c5](https://github.com/stencila/stencila/commit/e59d3c5bde75fb051f0aae277c4d4cdd84a1b151))


### Features

* **DOCX & LaTeX:** Add decoding of Microsoft Word and LaTeX via Pandoc ([0b1da91](https://github.com/stencila/stencila/commit/0b1da9116f8fbe2720c8a92a8b048dd1daa31b6a))
* **Formats:** Allow additional extensions for a format ([2c0ae5a](https://github.com/stencila/stencila/commit/2c0ae5aaa67da4276744ace6dc2cf006e66d6528))
* **Markdown:** Decode list item check boxes ([9ff02c5](https://github.com/stencila/stencila/commit/9ff02c540914aaf27544da3ef93352e2fa37acbb))

# [0.91.0](https://github.com/stencila/stencila/compare/v0.90.0...v0.91.0) (2021-07-04)


### Bug Fixes

* **Binaries:** Set permissions in cross OS way ([5b69333](https://github.com/stencila/stencila/commit/5b69333b87541a4d425a1ffab07deb4280e671ca))
* **dependencies:** update docusaurus monorepo to v2.0.0-beta.3 ([8425878](https://github.com/stencila/stencila/commit/8425878177c5de62b21cc8cf35df3317cd08663c))
* **dependencies:** update rust crate structopt to v0.3.22 ([f66b84a](https://github.com/stencila/stencila/commit/f66b84a025b5b7bda131ad008dcda84bc770e04a))
* **dependencies:** update rust crate termimad to v0.13.0 ([caa1be4](https://github.com/stencila/stencila/commit/caa1be42ea132fa78a36feb2101a651163223c12))
* **dependencies:** update rust crate thiserror to v1.0.26 ([c3f36b1](https://github.com/stencila/stencila/commit/c3f36b1ecb0576a290c4a8edd7d618ced1f94c06))
* **dependencies:** update rust crate tokio to v1.8.0 ([bafa3f0](https://github.com/stencila/stencila/commit/bafa3f0e6ff6f233398b6255705716a3c2001b34))
* **dependencies:** update rust crate validator to v0.14.0 ([95567c4](https://github.com/stencila/stencila/commit/95567c44f3f31b8470e8530103d979a2380d3d5f))


### Features

* **Binaries:** Add binaries module for locating, running and installing  third party binaries ([83d390c](https://github.com/stencila/stencila/commit/83d390c950fab75fc63f391e3fcac3a383abd7d9))
* **Binaries:** Add configuration option to enable, or not, automatic installs; add require function ([e2cd4eb](https://github.com/stencila/stencila/commit/e2cd4eb8a33bec8ffa29be662ebd30f0744eddb9))
* **Binaries:** Add installation for Python ([98b5bdf](https://github.com/stencila/stencila/commit/98b5bdf916c72090eea99e244197777e00e79175))
* **Binaries:** Add uninstall command ([4069af3](https://github.com/stencila/stencila/commit/4069af37c332757a5188809d13ac97dd020a89dc))

# [0.90.0](https://github.com/stencila/stencila/compare/v0.89.1...v0.90.0) (2021-07-02)


### Bug Fixes

* **Decode Markdown:** Handle soft breaks and clean code ([128ef58](https://github.com/stencila/stencila/commit/128ef58bbdf44fe5091c3ec529673492e541af0c))
* **Dependencies:** Pin funty ([1705859](https://github.com/stencila/stencila/commit/17058592469267be09c30124b2b75ce995e2930c))
* **Dependencies:** Upgrade to nom to avoid funty issue ([60252e8](https://github.com/stencila/stencila/commit/60252e885bc3b3c22ac1211c944be5dd3d7067bc))
* **Documents:** Do not bubble up errors from from update mthod ([f8e55be](https://github.com/stencila/stencila/commit/f8e55be0d011b0fa4f4b173085cbb6d3880bf29a))
* **HTML:** Always decode list ordering ([7859082](https://github.com/stencila/stencila/commit/78590821f974dc0ad9c43ce6bd745e12eeb4f100))
* **HTML:** Decode to CodeBlock ([ac6b7a2](https://github.com/stencila/stencila/commit/ac6b7a26b3f88171fce6e642a9ba78073a3382da))
* **HTML:** Escape attributes for media objects ([964e63d](https://github.com/stencila/stencila/commit/964e63def26b97a53b37b151724d7b9fada7bca6))
* **HTML:** Escape attributes when encoding ([adc8f5f](https://github.com/stencila/stencila/commit/adc8f5fa0053e3dc336a8fc329651c0650f43e60))
* **HTML:** Handle inline list item content ([d96d4ed](https://github.com/stencila/stencila/commit/d96d4edc703552184258c066859db78ace298093))
* **HTML:** Ignore empty text when decoding ([92dbbd6](https://github.com/stencila/stencila/commit/92dbbd61229f503fb224f6ff83221966496a8dde))
* **Markdown citations:** Allow for varying spaces around separator ([ac948b4](https://github.com/stencila/stencila/commit/ac948b4da2dd95505cdb0c4466ebc3e33f9300d8))
* **Markdown citations:** Do not consume @ character ([c47540c](https://github.com/stencila/stencila/commit/c47540c239c12e472c6c9631ec91907e8d410770))
* **Markdown code blocks:** Fix decoding ([b87af96](https://github.com/stencila/stencila/commit/b87af96faa7d3ab0e54e8cb0fda9118c901d90e5))
* **Markdown links:** Handle no title correctly ([0a5d56b](https://github.com/stencila/stencila/commit/0a5d56bf78cce9055d19e268cd697eb0fc6fbb73))


### Features

* **Arkdown:** Add decoding of tables ([d9d6f17](https://github.com/stencila/stencila/commit/d9d6f172eeacfe3a934085dcfc6fbced1a4ea431))
* **CLI:** Add `convert` command ([4a1006c](https://github.com/stencila/stencila/commit/4a1006c12f5f7ca132362a8bdfbdba6b56a6efc2))
* **Decode Markdown:** Handle inline content extensions e.g. math, citations ([78ba852](https://github.com/stencila/stencila/commit/78ba8521912ea961d9fdcdb41ca721e5f434fbec))
* **HTML:** Add decoding of audio and video elements ([a42e66b](https://github.com/stencila/stencila/commit/a42e66bef26fec2584e57a695732d3d9857c93b5))
* **HTML:** Add decoding of code blocks and fragments ([14ec9f4](https://github.com/stencila/stencila/commit/14ec9f4c90bd17e95c865c9090f40d91ec91df1d))
* **HTML:** Add decoding of headings ([3055c85](https://github.com/stencila/stencila/commit/3055c8592d9c0246cd7b40cff4b65a02777e6352))
* **HTML:** Add decoding of inline nodes ([a3a1869](https://github.com/stencila/stencila/commit/a3a1869a640a0ef21179cc9e22200363c262911d))
* **HTML:** Add decoding of links ([be2f19d](https://github.com/stencila/stencila/commit/be2f19d473a0aef2a397ef57d7d47eefdd4b1f01))
* **HTML:** Add decoding of lists ([ace5a5d](https://github.com/stencila/stencila/commit/ace5a5d1e7ccda4ed95465ba8e0fd5df22dc8a7c))
* **HTML:** Add decoding of thematic breaks ([be054df](https://github.com/stencila/stencila/commit/be054dfb57c204309b556bd8988e4845bd5f03b8))
* **Markdown:** Add decoding of images ([a9cc803](https://github.com/stencila/stencila/commit/a9cc8035875fcaebe4a8d8522611a1debf2acf54))
* **Markdown:** Add decoding of lists ([e750a77](https://github.com/stencila/stencila/commit/e750a77d345749b49645e338a245e6f6a7b8dc6c))
* **Markdown:** Add decoding of quote blocks ([d03105b](https://github.com/stencila/stencila/commit/d03105b9b11ea61071a1f8cb7f9b431b876ef774))
* **Markdown:** Decode HTML within Markdown ([15df781](https://github.com/stencila/stencila/commit/15df781b4220ebe43785a1aacc79f04e73b45016))

## [0.89.1](https://github.com/stencila/stencila/compare/v0.89.0...v0.89.1) (2021-06-30)


### Bug Fixes

* **Desktop:** Inline env variables when building Renderer files ([f06b73d](https://github.com/stencila/stencila/commit/f06b73dc3d2b66314523c7287bd32c8fb7100d22))

# [0.89.0](https://github.com/stencila/stencila/compare/v0.88.2...v0.89.0) (2021-06-30)


### Features

* **Desktop:** Add prompt to auto-updating the Desktop client ([6f30c83](https://github.com/stencila/stencila/commit/6f30c8309dd204adb21431cf1403b1924ebf95f2)), closes [#933](https://github.com/stencila/stencila/issues/933)

## [0.88.2](https://github.com/stencila/stencila/compare/v0.88.1...v0.88.2) (2021-06-30)


### Bug Fixes

* **Release:** Enable notarization for MacOS ([28decee](https://github.com/stencila/stencila/commit/28decee90b974a95cbabf816f266ebeb49aab288))
* **Release:** Set the exe config option under Windows ([99d7ffe](https://github.com/stencila/stencila/commit/99d7ffec1c3360891d291e1e745a8800dd809047))

## [0.88.1](https://github.com/stencila/stencila/compare/v0.88.0...v0.88.1) (2021-06-30)


### Bug Fixes

* **CLI:** Enable ANSII support on Windows ([9b076e9](https://github.com/stencila/stencila/commit/9b076e9e7371793f3f13ab57224ab82992158323))

# [0.88.0](https://github.com/stencila/stencila/compare/v0.87.4...v0.88.0) (2021-06-29)


### Bug Fixes

* **Preview:** Add horizontal padding to preview panel ([65532a5](https://github.com/stencila/stencila/commit/65532a581d6c32cdcd8f9d4e5cbee5f7479665f7))


### Features

* **Desktop:** Harden security based on best practices ([a210883](https://github.com/stencila/stencila/commit/a210883884c92851c2f5c0340756729c93aa00d2))
* **Desktop:** Keep document history for each document ([77fcd7d](https://github.com/stencila/stencila/commit/77fcd7dcaf1210c5849ec895c2d5e030d5665e81))

## [0.87.4](https://github.com/stencila/stencila/compare/v0.87.3...v0.87.4) (2021-06-27)


### Bug Fixes

* **dependencies:** update docusaurus monorepo to v2.0.0-beta.2 ([f108c56](https://github.com/stencila/stencila/commit/f108c5669ad01a482f176ff63977688ce1178c5c))
* **dependencies:** update rust crate reqwest to v0.11.4 ([6b048ef](https://github.com/stencila/stencila/commit/6b048efe564062568cd61987478ac8ef85646ad1))
* **dependencies:** update rust crate termimad to v0.12.1 ([6bba6d6](https://github.com/stencila/stencila/commit/6bba6d6ae628678ca513484ee070cf10ed7ad820))
* **dependencies:** update rust crate tracing-subscriber to v0.2.19 ([94d93f4](https://github.com/stencila/stencila/commit/94d93f4be9e011e095eb800b447edcbe6cbe818b))

## [0.87.3](https://github.com/stencila/stencila/compare/v0.87.2...v0.87.3) (2021-06-24)


### Bug Fixes

* **Desktop:** Fix error reporting integration ([b184490](https://github.com/stencila/stencila/commit/b184490df7d2f9e9523c16ddb6dc55ca824671f1))
* **Desktop:** Fix loading of SVG assets over custom app protocol ([375d8c3](https://github.com/stencila/stencila/commit/375d8c366513219458c0780ed8821ffcbf012e60))

## [0.87.2](https://github.com/stencila/stencila/compare/v0.87.1...v0.87.2) (2021-06-21)


### Bug Fixes

* **dependencies:** update dependency @stencila/encoda to v0.117.1 ([faa1f52](https://github.com/stencila/stencila/commit/faa1f52cc235e5db272ce2620669b05a39c34bf5))
* **Deps:** Pin all Rust dependencies for more reproducible builds ([bcc3f0c](https://github.com/stencila/stencila/commit/bcc3f0cd18d5472e363efc9796e51bb23bcf832e))
* **Rust:** Upgrade dependencies ([1587395](https://github.com/stencila/stencila/commit/1587395a8c229b8339ffb914750683b754bba8c5))

## [0.87.1](https://github.com/stencila/stencila/compare/v0.87.0...v0.87.1) (2021-06-21)


### Bug Fixes

* **dependencies:** update dependency @sentry/electron to v2.5.0 ([230cb52](https://github.com/stencila/stencila/commit/230cb522ad09472ba667ec017ba5a45a69bd8d3d))
* **dependencies:** update dependency i18next to v20.3.2 ([8e3b5d6](https://github.com/stencila/stencila/commit/8e3b5d675d424faebaa360827ccda4d743b411ec))

# [0.87.0](https://github.com/stencila/stencila/compare/v0.86.1...v0.87.0) (2021-06-20)


### Bug Fixes

* **dependencies:** update docusaurus monorepo to v2.0.0-beta.1 ([683a11b](https://github.com/stencila/stencila/commit/683a11b6dd155da2d865808689fd1a9540acfb57))
* **dependencies:** update rust crate handlebars to 4.0.1 ([5c62381](https://github.com/stencila/stencila/commit/5c6238134e0cb53e5aa65f8818878bed7199e916))
* **dependencies:** update rust crate jsonschema to 0.11.0 ([733c133](https://github.com/stencila/stencila/commit/733c1335a90b0d692279c094b87bdc2da26ee80f))
* **dependencies:** update rust crate rand to 0.8.4 ([3e63c06](https://github.com/stencila/stencila/commit/3e63c06ffc6b92bbff826173c7fbac58cd965296))
* **dependencies:** update rust crate serde_with to 1.9.4 ([994a00a](https://github.com/stencila/stencila/commit/994a00a86c4a19b58c6b32096c820e62c5ff2f80))
* **Encode HTML:** Create content for Cite nodes as needed ([d60862e](https://github.com/stencila/stencila/commit/d60862e264f9645eb9d940e870ea538ff11da78d))


### Features

* **CLI:** Add serve command ([5faeb3b](https://github.com/stencila/stencila/commit/5faeb3b98cdd49a8a4a2d726254526b9daeb6c86))

## [0.86.1](https://github.com/stencila/stencila/compare/v0.86.0...v0.86.1) (2021-06-17)


### Bug Fixes

* **Build:** Fix loading of manifest.json in production builds ([69a0bcd](https://github.com/stencila/stencila/commit/69a0bcd22b4776eb168c9b6e3c88a62418d31cc0))
* **Desktop:** Expose tab closing shortcuts on all platforms ([96c6554](https://github.com/stencila/stencila/commit/96c65541160dcd865f4c8282b21eecead5a6681b))
* **File list:** Add keys to items to provide stable identity ([7880374](https://github.com/stencila/stencila/commit/7880374ac38a0c8b9ed684d9e5f877d7bb12c551))

# [0.86.0](https://github.com/stencila/stencila/compare/v0.85.1...v0.86.0) (2021-06-17)


### Bug Fixes

* **Documents:** Add HTML and  `unregistered` formats ([c9c5f9a](https://github.com/stencila/stencila/commit/c9c5f9a6773e6ed5c8c0ef6a1f4a0b1155a65197))
* **Documents:** End watcher thread properly; consistency with `ProjectHandler` ([9eb7ff8](https://github.com/stencila/stencila/commit/9eb7ff8d8a71587df1e0c9f4dabf4b2f9fc91db9))
* **Logging:** Only include log entries from this crate ([e5c0347](https://github.com/stencila/stencila/commit/e5c034733832c2207711dad248413c8b66956d0a))
* **Projects:** Make projects async to avoid try_lock ([fdb464c](https://github.com/stencila/stencila/commit/fdb464c609cfe4b1d011aa769c8025bf79bd3969))


### Features

* **Files:** Give directories their own format ([4cb509e](https://github.com/stencila/stencila/commit/4cb509ecd74ed7d717a4d31c70bf31b35ed496d3))
* **Projects:** Add project events, published when project is updated ([d4be122](https://github.com/stencila/stencila/commit/d4be122bc253987ed6d364ad7547d1ccf70541c6))
* **Projects:** Update properties when project.json changes ([49b2e9c](https://github.com/stencila/stencila/commit/49b2e9c451e17c1afe6740413ef50738ec7270c7))

## [0.85.1](https://github.com/stencila/stencila/compare/v0.85.0...v0.85.1) (2021-06-16)


### Bug Fixes

* **dependencies:** update dependency @stencila/brand to v0.7.1 ([0eab022](https://github.com/stencila/stencila/commit/0eab022296c7fc4e4a5559ad300faaedd4c5b8d8))

# [0.85.0](https://github.com/stencila/stencila/compare/v0.84.0...v0.85.0) (2021-06-15)


### Bug Fixes

* **Desktop:** Allowing loading images using 'local:' protocol ([51cd9e4](https://github.com/stencila/stencila/commit/51cd9e48476b582e33c4ad2ca54a82fdbc58e8a0))


### Features

* **Desktop:** Add shortcut for opening project launcher ([03d8137](https://github.com/stencila/stencila/commit/03d81376a9e4240aa5d93e2c50e1fda8c8e2e79e))

# [0.84.0](https://github.com/stencila/stencila/compare/v0.83.0...v0.84.0) (2021-06-15)


### Features

* **Desktop:** Add basic support styling previews with project themes ([d2e06d8](https://github.com/stencila/stencila/commit/d2e06d81fcf6305fdafe023b763185e6dd414962))
* **Desktop:** Only show editor panel for editable documents ([725bb46](https://github.com/stencila/stencila/commit/725bb46e9a0cc958dcaae561418aec7da7124c09))

# [0.83.0](https://github.com/stencila/stencila/compare/v0.82.0...v0.83.0) (2021-06-15)


### Bug Fixes

* **dependencies:** update docusaurus monorepo to v2.0.0-beta.0 ([1357707](https://github.com/stencila/stencila/commit/1357707e80d6c4e91259b42dbef6bd816df23c4d))
* **HTML:** Add controls attribute to audio and video elements ([2543750](https://github.com/stencila/stencila/commit/25437504085484e19ca439255d5eedf31c9d6379))


### Features

* **Formats:** Add preview property ([b885e8d](https://github.com/stencila/stencila/commit/b885e8da0c12c40323d5c9f65eec9b9bd02db008))

# [0.82.0](https://github.com/stencila/stencila/compare/v0.81.0...v0.82.0) (2021-06-15)


### Features

* **Telemetry:** Add telemetry module ([9c04444](https://github.com/stencila/stencila/commit/9c0444454cffc39a526c1bd7f3798630be0aa45f))

# [0.81.0](https://github.com/stencila/stencila/compare/v0.80.1...v0.81.0) (2021-06-14)


### Bug Fixes

* **dependencies:** update rust crate ignore to 0.4.18 ([7cde18c](https://github.com/stencila/stencila/commit/7cde18c3a3275e54b500049b7e25c26160c2097a))
* **dependencies:** update rust crate linya to 0.2.1 ([a99911f](https://github.com/stencila/stencila/commit/a99911f5687e822e9c34817946d1bf4790bf6ab1))
* **dependencies:** update rust crate once_cell to 1.8.0 ([6ecafdb](https://github.com/stencila/stencila/commit/6ecafdb75f904b48f5c5e9ede92b2be7b4f380b7))
* **dependencies:** update rust crate serde_with to 1.9.2 ([7e805c7](https://github.com/stencila/stencila/commit/7e805c7428650b9e8df179e685d32301a99c447d))
* **HTML:** Escape strings ([e17d0a3](https://github.com/stencila/stencila/commit/e17d0a394a09fe5c4d99d46c9746a7b757c8807c))
* **HTML:** Remove backslash from Cite ([024eaf3](https://github.com/stencila/stencila/commit/024eaf3d1ded21f206e882d86985f0c379f38a47))
* **Open:** Open project or document in browser ([705c8d8](https://github.com/stencila/stencila/commit/705c8d8641f56c5b07cb847ef86b20e0baed6feb))
* **Serve:** Serve local assets ([70a95ff](https://github.com/stencila/stencila/commit/70a95ff3770e2c47d490ab5632a45802baeeefac))


### Features

* **CLI:** Add close command ([1aa4365](https://github.com/stencila/stencila/commit/1aa43653d9d44fe0ae9c8db2c7edc0d6daa77c90))
* **CLI:** Add show command ([7507118](https://github.com/stencila/stencila/commit/75071186220cc8e7238725477dcce7779c7584c4))
* **Rust:** Allow a document to be closed by path ([b6e6819](https://github.com/stencila/stencila/commit/b6e6819a11de26f15dd6b6601c7fc7cd861485be))

## [0.80.1](https://github.com/stencila/stencila/compare/v0.80.0...v0.80.1) (2021-06-11)


### Bug Fixes

* **Onboarding:** Fix bug preventing progressing to next steps ([27c4711](https://github.com/stencila/stencila/commit/27c471150ea51a3416feabfd0d69a46acadde3e7))

# [0.80.0](https://github.com/stencila/stencila/compare/v0.79.0...v0.80.0) (2021-06-11)


### Bug Fixes

* **Desktop:** Add protocol for secure access to media files ([98b5877](https://github.com/stencila/stencila/commit/98b58770bd8562fe10d14c66a27847c9169ad194))
* **Documents:** Ignore midifications after writes ([21f8fd0](https://github.com/stencila/stencila/commit/21f8fd0f33e3a9e52bc19e051d6ca523278f276d))
* **Documents:** Reinstate document watching ([209e051](https://github.com/stencila/stencila/commit/209e051cfdd66762d2bb73d15707de35495106c1))
* **Documents:** Restrict links to the project directory ([4c81020](https://github.com/stencila/stencila/commit/4c81020356ccf9a725537c57cd4aa8507574e0b2))


### Features

* **Documents:** Add ability to query documents using JSON Pointer or JMESPath ([40e7158](https://github.com/stencila/stencila/commit/40e7158f7511e2ef961667eeda2ed0dbeae54ce2))
* **Documents:** Add compile method ([fb06a21](https://github.com/stencila/stencila/commit/fb06a211e0c069bd3b4175d63deaf3b76fdbc877))
* **Documents:** Allow media objects to be opended as documents ([ffadd64](https://github.com/stencila/stencila/commit/ffadd646f31b60c2a8c843952a29ea4c0f87a9a1))
* **Node.js:** Expose `DocumentFormat` type and map ([a4b9ab6](https://github.com/stencila/stencila/commit/a4b9ab6bab6aec173b866414e1e23b51be10cf1b))


### Performance Improvements

* **Rust:** Use crossbeam-channel where possible ([67df93d](https://github.com/stencila/stencila/commit/67df93d5ef9d77d86e0df81dd547d90b3a954dcd))

# [0.79.0](https://github.com/stencila/stencila/compare/v0.78.1...v0.79.0) (2021-06-10)


### Features

* **Desktop:** Add keyboard shortcut for closing active tab ([fa09b54](https://github.com/stencila/stencila/commit/fa09b540f20c936bc47f78694b1392dc6d76ce9f))

## [0.78.1](https://github.com/stencila/stencila/compare/v0.78.0...v0.78.1) (2021-06-10)


### Bug Fixes

* **Desktop:** Fix issues when trying to register duplicate IPC handlers ([1504b37](https://github.com/stencila/stencila/commit/1504b373528846f374fcfae10876451effa981f4))
* **Desktop:** Make first launch experience work across platforms ([af5cca8](https://github.com/stencila/stencila/commit/af5cca8910ee420e436fa74e107f21ffcc9b6e48))

# [0.78.0](https://github.com/stencila/stencila/compare/v0.77.1...v0.78.0) (2021-06-10)


### Features

* **Custom errors:** Define custom error types and publish them under the "errors" topic ([d0c6978](https://github.com/stencila/stencila/commit/d0c69782b1deea79ad3e23f59dc6a552da947df7))

## [0.77.1](https://github.com/stencila/stencila/compare/v0.77.0...v0.77.1) (2021-06-09)


### Bug Fixes

* **Desktop:** Fix mismatched package version failing builds ([30aa48b](https://github.com/stencila/stencila/commit/30aa48b85ec6b23070aa744ff12679def19f9308))

# [0.77.0](https://github.com/stencila/stencila/compare/v0.76.0...v0.77.0) (2021-06-09)


### Bug Fixes

* **Desktop:** Clean up document event listeners to avoid memory leaks ([13bd3c1](https://github.com/stencila/stencila/commit/13bd3c1f2a40e6e7d098f1911940662837e66fe5))
* **Preview:** Fix scrolling of preview pane contents ([7fcf7eb](https://github.com/stencila/stencila/commit/7fcf7eb4e819a77ceef2943cbee373c2b9131076))


### Features

* **Desktop:** Add first-launch onboarding flow ([1afa693](https://github.com/stencila/stencila/commit/1afa6939dfc62fd81f3db1b4bbe4f0bd087d9280))

# [0.76.0](https://github.com/stencila/stencila/compare/v0.75.0...v0.76.0) (2021-06-08)


### Bug Fixes

* **Desktop:** Get and use  preview when changing document ([daeac8d](https://github.com/stencila/stencila/commit/daeac8dd0398bb4d3bd068436466879678a521ba))
* **Desktop:** Unsubscribe from document when component is disconnected ([51d181c](https://github.com/stencila/stencila/commit/51d181c2f70f47564ba6724692950d57ed2b300c))
* **Documents:** Complete implementation of dump method ([b672a95](https://github.com/stencila/stencila/commit/b672a956036f76a5ca6141da7eeabbdac163bea4))


### Features

* **Rust:** Internally encode nodes to HTML ([de11fc9](https://github.com/stencila/stencila/commit/de11fc927a442f806d69c5e0b83592a5d624f6e3))

# [0.75.0](https://github.com/stencila/stencila/compare/v0.74.0...v0.75.0) (2021-06-07)


### Bug Fixes

* **dependencies:** update rust crate neon to 0.8.3 ([c3ee1ab](https://github.com/stencila/stencila/commit/c3ee1abb83bc8384355c35bec14dac612f1602b2))
* **dependencies:** update rust crate semver to 1.0.3 ([74ac76b](https://github.com/stencila/stencila/commit/74ac76b11202c7fcd39356e384aaa508d74db83c))
* **dependencies:** update rust crate strum to 0.21 ([fd1d2ca](https://github.com/stencila/stencila/commit/fd1d2ca5b4023ec334e3a2a6e64168e0d4149810))
* **dependencies:** update rust crate strum_macros to 0.21.1 ([546bb42](https://github.com/stencila/stencila/commit/546bb42b28def3042b211bed317215fd4a22f37a))
* **dependencies:** update rust crate termimad to 0.11.1 ([74b40f6](https://github.com/stencila/stencila/commit/74b40f6c75444ce32253ea4da5a1e3789dcf8935))
* **Dependencies:** Upgrade Stencila Schema ([c58c1b2](https://github.com/stencila/stencila/commit/c58c1b261e62d04473763ebafc62f87a878dfbf7))
* **Desktop:** NPM audit fix ([0405553](https://github.com/stencila/stencila/commit/0405553447156a2cefe150b754dff7674ceb8656))
* **Rust:** Improve error message when no matching document ([40b28cd](https://github.com/stencila/stencila/commit/40b28cdb62a4f444c4b77729bf2941ed466f57fa))
* **Tabs:** Add visual feedback when hovering over tabs ([d8f67ee](https://github.com/stencila/stencila/commit/d8f67eedbaf2e922884e0fac4b91a44b01d1f071))


### Features

* **Desktop:** Allow resizing panes using entire height of right border ([c078a91](https://github.com/stencila/stencila/commit/c078a913c9100a82bd74c3ea2f5abfa34a2e292d))
* **Node.js:** Add separate get function ([d75e684](https://github.com/stencila/stencila/commit/d75e684ab738b8ff8d27c4416be080dae97ce363))

# [0.74.0](https://github.com/stencila/stencila/compare/v0.73.0...v0.74.0) (2021-06-03)


### Bug Fixes

* **dependencies:** update rust crate semver to v1 ([05457ac](https://github.com/stencila/stencila/commit/05457ac653ae2d2dc01fc407e33510936cb15f80))
* **Plugins:** Adjust for upgrade to semver v1 ([15e0112](https://github.com/stencila/stencila/commit/15e011220e224b8ff8938caf7c04cc9881ea9d2b))
* **Plugins:** Ensure that not using an empty string for current version ([ad59c16](https://github.com/stencila/stencila/commit/ad59c168ee6e8cfe00884aa4b761609c4577641b))
* **Rust:** Upgrade Stencila Schema version ([7fa221e](https://github.com/stencila/stencila/commit/7fa221e960b1dd301a7f23058c20778afe82994f))


### Features

* **Documents:** Add function to create a new empty document ([1dbce29](https://github.com/stencila/stencila/commit/1dbce2931c61b2ee5d923fcfb213e598a9d9be50))

# [0.73.0](https://github.com/stencila/stencila/compare/v0.72.0...v0.73.0) (2021-06-03)


### Bug Fixes

* **Desktop:** Send IPC events only to relevant project windows ([ae1e8fe](https://github.com/stencila/stencila/commit/ae1e8fec447ad22d80f65b422e287eb8ca35093f))


### Features

* **Desktop:** Add ability to open project folder from menu item ([51f1730](https://github.com/stencila/stencila/commit/51f1730a510d0403e891ffd068c46c0bb776a2f6))

# [0.72.0](https://github.com/stencila/stencila/compare/v0.71.0...v0.72.0) (2021-06-01)


### Bug Fixes

* **Desktop:** Improve usability and legibility of tab close icon ([f47b582](https://github.com/stencila/stencila/commit/f47b582c62c9bc2188703c4684cc528bdac6989d))
* **Tabs:** Fix width distribution of document tabs ([4dcaedb](https://github.com/stencila/stencila/commit/4dcaedbd92ff5643bb178bce2d453ed79038161a))
* **Tabs:** Improve visibility of active document tab ([f788409](https://github.com/stencila/stencila/commit/f78840991efcc38e47de890278308af448f46822))


### Features

* **Desktop:** Support saving document changes to disk ([ea76aec](https://github.com/stencila/stencila/commit/ea76aecbab3dcac0d30a0363eb0d9689d2d6c85e))

# [0.71.0](https://github.com/stencila/stencila/compare/v0.70.0...v0.71.0) (2021-05-31)


### Bug Fixes

* **CLI:** Don't show projects in traces ([3ab04c6](https://github.com/stencila/stencila/commit/3ab04c689c6263f07679a87ee56660cc258ea13f))
* **dependencies:** update rust crate handlebars to v4 ([a2cb4c2](https://github.com/stencila/stencila/commit/a2cb4c2728fb4747b82f6614e3bfcfc30ad1283a))
* **dependencies:** update rust crate linya to 0.2.0 ([800ae2a](https://github.com/stencila/stencila/commit/800ae2a513b8c0aca4f092fc91fda607f0babf4d))
* **dependencies:** update rust crate stencila-schema to 1.7.2 ([9acd4f4](https://github.com/stencila/stencila/commit/9acd4f4c580a1f908419370d11226deee49e73a3))
* **dependencies:** update rust crate termimad to 0.10.3 ([9d8ea01](https://github.com/stencila/stencila/commit/9d8ea01ef941a2ba946c0197675ed572e3fd25af))
* **dependencies:** update rust crate tokio to 1.6.1 ([a85a67c](https://github.com/stencila/stencila/commit/a85a67c07b6f1669628a0a633ff12cc73818b738))
* **Documents:** Publish encoded event from load; fix Node.js test ([5e0c888](https://github.com/stencila/stencila/commit/5e0c888b9f767f1bfb6d46741e351a376f5a360b))
* **Plugins:** Error on bad status and log as warning ([d69283d](https://github.com/stencila/stencila/commit/d69283d1212740990b31fda84fb30aaca865175c))
* **Project files:** Refresh name and format and remove old path from parent when renaming a file or folder ([69546e3](https://github.com/stencila/stencila/commit/69546e3395df26568477a828a5e1d3fa5ace3d6f))


### Features

* **Plugins:** Delegate method calls to plugins ([2203914](https://github.com/stencila/stencila/commit/22039141e8ffa51bc6764188c1d7ef874c6d406a))
* **Plugins:** Plugin clients for Javascript plugins (installed and linked) ([a4d2a83](https://github.com/stencila/stencila/commit/a4d2a830de1cdc13d144e7c5730039a0882fcecd))

# [0.70.0](https://github.com/stencila/stencila/compare/v0.69.0...v0.70.0) (2021-05-28)


### Bug Fixes

* **Sidebar:** Fix spacing around file tree ([26d76af](https://github.com/stencila/stencila/commit/26d76afeade9056d521d43af78bca4c598694a81))


### Features

* **Desktop:** Open files in separate tabs ([7ff3114](https://github.com/stencila/stencila/commit/7ff31149a6de2b3f6f71177b33ee8dee27d13175))

# [0.69.0](https://github.com/stencila/stencila/compare/v0.68.0...v0.69.0) (2021-05-28)


### Bug Fixes

* **Desktop:** Update editor contents when file changes on disk ([6f1cdd4](https://github.com/stencila/stencila/commit/6f1cdd41e3bc837250d83bcbdd1bfc8b5cf97950))
* **Documents:** Use CreativeWorkTypes for root ([91592ff](https://github.com/stencila/stencila/commit/91592ff4d511d37144c0e5019cbcbdf5fb6e9b3c))
* **Rust:** Preserve order on JSON maps ([7eaff66](https://github.com/stencila/stencila/commit/7eaff6665cb106d89c488e5ca45b5fd0be589c92))
* **Rust:** Upgrade Schema version ([301548c](https://github.com/stencila/stencila/commit/301548c65acf109fa2193adc2486767e783f78d8))
* **Serve:** Check early that path is in current working directory ([9385f44](https://github.com/stencila/stencila/commit/9385f44d38136d938a107fbec2c291284041dc47))
* **Serving:** Serve unknown file types as raw ([7d4e73c](https://github.com/stencila/stencila/commit/7d4e73ceff15c1a1120ed935c723e497c235cf93))
* **Viewer:** Fallback URLs ([331b394](https://github.com/stencila/stencila/commit/331b394fb433ca346e249929601290bbab299fac))
* **Viewer:** Implement more article content components ([ac66ec5](https://github.com/stencila/stencila/commit/ac66ec506c4f295c2a7d62ea9abe3b31f3ae8eb7))
* **Viewer:** Use correct itemtypes ([968bbae](https://github.com/stencila/stencila/commit/968bbae134ea13aabc70515c5fe031d890c6060e))


### Features

* **CLI:** Open viewer for documents ([a9bfa90](https://github.com/stencila/stencila/commit/a9bfa90fc996e58d89e5d95c1e117a6f80bb24b6))
* **Desktop:** Add collapsible folders to the file tree sidebar ([8a16343](https://github.com/stencila/stencila/commit/8a163431d6022359db53ffb72c6578f97dc560ae))
* **Plugins:** Add Encoda to plugin registry ([2ebe178](https://github.com/stencila/stencila/commit/2ebe178b8109c439c38c39dfa0abdfd399b53b0c))
* **Viewer:** Add theme switcher ([49e492b](https://github.com/stencila/stencila/commit/49e492b27d66fcb1832faf552b4bff4c8b4ad0ac))
* **Viewer:** Inital version of document viewer ([cec8d78](https://github.com/stencila/stencila/commit/cec8d7807e22da238c26ba0ac6a3c6ce273befc9))

# [0.68.0](https://github.com/stencila/stencila/compare/v0.67.1...v0.68.0) (2021-05-23)


### Bug Fixes

* **dependencies:** update dependency i18next to v20.3.0 ([f97f78f](https://github.com/stencila/stencila/commit/f97f78f808d48d9886f7c9a028557eb8aca9ba18))
* **dependencies:** update rust crate neon to 0.8.2 ([be11c89](https://github.com/stencila/stencila/commit/be11c89f074c7dfc7f452181f568afc6d94ce1a1))
* **dependencies:** update rust crate rustyline to 8.2.0 ([168d060](https://github.com/stencila/stencila/commit/168d06027554e8f2e0a6afa73d3a74b3c5b18601))
* **dependencies:** update rust crate thiserror to 1.0.25 ([8858729](https://github.com/stencila/stencila/commit/88587293b831c2d177638e45faccea23966cb116))
* **Documents:** Make document path required; create temp path for new docs ([a8e8a4d](https://github.com/stencila/stencila/commit/a8e8a4d503031b9530e9796de1fbc4d81bb0ff61))
* **Documents:** Pluralize topic as for projects ([457d82e](https://github.com/stencila/stencila/commit/457d82e09bc78a40cd43f0da827f47e1c78e4b43))
* **Documents:** Use stencila-schema ([d379615](https://github.com/stencila/stencila/commit/d3796152970646a6e2cd3056132a91089aefd420))
* **Node.js:** Improve document type ([1c0ea25](https://github.com/stencila/stencila/commit/1c0ea250ba03fb2c87092e73ff8dbd375e84c486))
* **Projects:** Generate type for file events ([ee4e69e](https://github.com/stencila/stencila/commit/ee4e69e7309724c77acc05b90281c03652fd27b6))
* **TYpings:** Allow override of property optionality ([ac24ef7](https://github.com/stencila/stencila/commit/ac24ef734e88a0d606a19511ca58525cd23d8b9e))


### Features

* **Documents:** Add documents modules to Rust and Node ([b5bfc93](https://github.com/stencila/stencila/commit/b5bfc93cfe8c171f88a8b52b9f190aa349439829))
* **Documents:** Add sepate subscription topics ([fe3563f](https://github.com/stencila/stencila/commit/fe3563f8192fa659995fd8f9d2eb2cdfd86e7074))
* **Identifiers:** Add uuids module ([34270cf](https://github.com/stencila/stencila/commit/34270cf51e0e8ecb713e6bbc3d26d27a2a400813))

## [0.67.1](https://github.com/stencila/stencila/compare/v0.67.0...v0.67.1) (2021-05-21)


### Bug Fixes

* **Desktop:** Improve configuration of `deb` builds ([de3f939](https://github.com/stencila/stencila/commit/de3f9392a046db9b8720d752a2feaa1bff095bac))
* **Routing:** Avoid resolving client-side routes as filepaths ([41408be](https://github.com/stencila/stencila/commit/41408beebac03e7770d36915c20f30448cc5bcd8))

# [0.67.0](https://github.com/stencila/stencila/compare/v0.66.0...v0.67.0) (2021-05-20)


### Bug Fixes

* **Desktop:** Fix client rendering issues when building on Ubuntu ([a12fa66](https://github.com/stencila/stencila/commit/a12fa669d796a6b36547b2f976b9447c77bf5d75))
* **Desktop:** Use OS specific location when adding Preferences menu item ([81724f4](https://github.com/stencila/stencila/commit/81724f4336d8c30442529ae61e7ed9d0442881de))
* **Plugins:** Fix rendering of available plugins ([fa4bc79](https://github.com/stencila/stencila/commit/fa4bc79cecb2488032c5e581f8b1227d00a712a3))


### Features

* **Desktop:** Add custom scrollbar styles ([9df42dd](https://github.com/stencila/stencila/commit/9df42dd34c91296076e8b404dc77579dc3bf8662))
* **Files:** Add more filetype icons ([3265995](https://github.com/stencila/stencila/commit/3265995f15cd17c1c1abe18ea239fd8d12d62757))
* **Files Sidebar:** Show empty state when project contains no files ([eaac750](https://github.com/stencila/stencila/commit/eaac7508afa10b325756427f9b406fcb1126e557))
* **Launcher:** Add support for "Recent Projects", add initial styles ([5933396](https://github.com/stencila/stencila/commit/59333968849428d420cdcf0924c6c171890f3eb9))

# [0.66.0](https://github.com/stencila/stencila/compare/v0.65.0...v0.66.0) (2021-05-19)


### Bug Fixes

* **Desktop:** Add a preferences menu item ([ed2ed86](https://github.com/stencila/stencila/commit/ed2ed86066f34686b434ad8c65ac6c8d7c5df61d))
* **Editor:** Fix UI jumping around when clicking into code editor ([3eec143](https://github.com/stencila/stencila/commit/3eec143c5f4740d7f2c0b8d57898d45e2ca8f3ad))
* **Files:** Call correct registry function on remove event ([8397e8f](https://github.com/stencila/stencila/commit/8397e8f67ffccfa37b53e24158ed77c880bf4d93))
* **Files:** Handle creation of empty nested dirs; improve robustness of updates ([425045b](https://github.com/stencila/stencila/commit/425045b9a48da06a286e96d6089a8d638200f4d4))


### Features

* **Desktop:** Add empty state message for new Project windows ([f9bc0ea](https://github.com/stencila/stencila/commit/f9bc0eaba2a6ec61d4c6747009330674173c8c52))
* **Files:** Add custom icons for various file formats ([ca133c7](https://github.com/stencila/stencila/commit/ca133c7fdd24cc4078cbe5535be133d4ba8df421))
* **Project:** Listen to project file change events and update sidebar ([dc067aa](https://github.com/stencila/stencila/commit/dc067aabbfecff772a4a8790479bc38f6b7c66d3))

# [0.65.0](https://github.com/stencila/stencila/compare/v0.64.0...v0.65.0) (2021-05-18)


### Bug Fixes

* **CLI:** Match requested value format ([fa43114](https://github.com/stencila/stencila/commit/fa43114ab9ef1ada3124624f8510f9a66934f0de))


### Features

* **Config:** Return display result ([b4999df](https://github.com/stencila/stencila/commit/b4999df5d0071cb4fbbbb5ffadd3faed15ca4497))
* **Plugins:** Return display results ([b40b7ed](https://github.com/stencila/stencila/commit/b40b7ed9f0c079f969520343922a0b5ddd624840))

# [0.64.0](https://github.com/stencila/stencila/compare/v0.63.0...v0.64.0) (2021-05-18)


### Bug Fixes

* **CLI:** Only print once; fix non-pretty rendering ([4e3d32d](https://github.com/stencila/stencila/commit/4e3d32dcc715ab508a9cb3c211bfc0868c5bf69f))
* **CLI:** Print alpha message to stderr, not stdout ([44d661c](https://github.com/stencila/stencila/commit/44d661c5d5dd398ae0b4bc81d6a9341676446ef0))
* **JSON Schemas:** Improves JSON Schema generation: ([81ecd0f](https://github.com/stencila/stencila/commit/81ecd0fa91cea487424432cd8a71759be12e4886))


### Features

* **Node.js:** Generate TypeScript types from JSON Schemas ([3d1358b](https://github.com/stencila/stencila/commit/3d1358b296652403f14a75f52628ac377fe3693d))

# [0.63.0](https://github.com/stencila/stencila/compare/v0.62.2...v0.63.0) (2021-05-17)


### Bug Fixes

* **CLI:** Remove flag gates; implement plain display ([37a9c66](https://github.com/stencila/stencila/commit/37a9c66c65a3872e5eb0fea3d8f1773a4324c44f))


### Features

* **CLI:** Allow alternative user specified formats for displaying results ([277bdf2](https://github.com/stencila/stencila/commit/277bdf2057d2d73c46d598f55f2088f78d19292a))


### Performance Improvements

* **CLI:** Lazily load syntaxes and themes; only highlight in interactive mode ([206da6c](https://github.com/stencila/stencila/commit/206da6cc69a3ae3429113cda724a901062f19b88))

## [0.62.2](https://github.com/stencila/stencila/compare/v0.62.1...v0.62.2) (2021-05-16)


### Bug Fixes

* **CLI:** Update the list of global args ([93f0a61](https://github.com/stencila/stencila/commit/93f0a61827ba928edf8b774d9984fcb471eeb3cd))
* **dependencies:** update rust crate futures to 0.3.15 ([1cd8a38](https://github.com/stencila/stencila/commit/1cd8a38670f572b46cf09a37c39f05ed7f8eba1a))
* **dependencies:** update rust crate notify to 4.0.17 ([0ca93a1](https://github.com/stencila/stencila/commit/0ca93a1c2a52a440eff2791b9923a7c41f2ac917))
* **dependencies:** update rust crate self_update to 0.27.0 ([75a71fc](https://github.com/stencila/stencila/commit/75a71fcebd4a1b8f6e3e508f5faf137680f391ed))
* **dependencies:** update rust crate serde to 1.0.126 ([b2ca64f](https://github.com/stencila/stencila/commit/b2ca64f5b9acb4b407a4ad96aebf5f8be877cd81))
* **dependencies:** update rust crate serde_with to 1.9.1 ([b32479b](https://github.com/stencila/stencila/commit/b32479b282ce0798e23bf0aa334bfef3ec862c74))
* **dependencies:** update rust crate tokio to 1.6.0 ([50b485c](https://github.com/stencila/stencila/commit/50b485c30b8e354e57fce97238ec78b1a911984f))

## [0.62.1](https://github.com/stencila/stencila/compare/v0.62.0...v0.62.1) (2021-05-16)


### Bug Fixes

* **File watching:** Allow glob patterns to be excluded ([37a33c3](https://github.com/stencila/stencila/commit/37a33c337cae4b76d417020c110fb849a0933cfb))
* **Files:** Publish a refresh event when file registry is refreshed ([57d9190](https://github.com/stencila/stencila/commit/57d91906008b1920eb5fb7bc0951507398a34923))


### Performance Improvements

* **Files:** Use cache of files ignored ([f4c0308](https://github.com/stencila/stencila/commit/f4c0308bfae40bd76fa57d477095bc991860ff1c))
* **Files:** Use parallel walk to do initial collection of files for a project ([c953b05](https://github.com/stencila/stencila/commit/c953b0554a65ef6eb48dd7722e1d0d71cca0f986))

# [0.62.0](https://github.com/stencila/stencila/compare/v0.61.0...v0.62.0) (2021-05-14)


### Bug Fixes

* **Files:** Use a BTreeMap and ignore project folder ([a153c9c](https://github.com/stencila/stencila/commit/a153c9c8ba1884a3629037ab98c7dfaf9d8161a7))
* **Pubsub:** Make publish "fire and forget" ([ed10073](https://github.com/stencila/stencila/commit/ed10073c276b2c5bccb7e6b243161fd480d750a7))


### Features

* **Files:** Add file watching with publishing events ([7781645](https://github.com/stencila/stencila/commit/7781645cad72d08abb46e666bb036c4966ad412c))
* **Files:** Add files to project properties ([7e52b57](https://github.com/stencila/stencila/commit/7e52b57be6b1888e3b5749024256831068fa1c91))
* **Files:** Add format property and media type fallbacks ([287a732](https://github.com/stencila/stencila/commit/287a73249598d6bac2071f8f94f7b0b0f3848afc))
* **Files:** Add modified and size properties ([5f78cb4](https://github.com/stencila/stencila/commit/5f78cb442280e83503fbe121e5198341a9473a64))
* **Files:** Mirror file system changes in memory ([6f027c4](https://github.com/stencila/stencila/commit/6f027c4c72b7cac33cafa3223ced9f10d95f6fcc))
* **Files:** Respect gitignore files including during watching ([7ca11bb](https://github.com/stencila/stencila/commit/7ca11bb8321d597237c99661adf4fc7161de94aa))
* **Projects:**  Add CLI commands for initializing and showing projects ([bf8c34c](https://github.com/stencila/stencila/commit/bf8c34c231aeffbee78115949e9766096ee44a1a))
* **Projects:** Add Node bindings for projects ([063d4ea](https://github.com/stencila/stencila/commit/063d4ea1acf3e7d77affabea6f6939d1f3d8acbf))
* **Projects:** Add Rust projects module ([253c00c](https://github.com/stencila/stencila/commit/253c00cb4f10b62149f67f91f62a194a2a45aab7))
* **Projects:** Resolve main file path; add name to file description ([dcfb3f1](https://github.com/stencila/stencila/commit/dcfb3f12b3694e8ec2f3903dd7e57d688cc38e3d))

# [0.61.0](https://github.com/stencila/stencila/compare/v0.60.0...v0.61.0) (2021-05-12)


### Bug Fixes

* **Desktop:** Fix plugins settings view title ([35835b0](https://github.com/stencila/stencila/commit/35835b04773a53137d002f4d024f80c194424679))


### Features

* **Desktop:** Set customized window title for different views ([c1dfdf1](https://github.com/stencila/stencila/commit/c1dfdf124becb507255c2055b7f5eb96faf80ceb))
* **i18n:** Add foundation for internationalization ([8b868cb](https://github.com/stencila/stencila/commit/8b868cbb09288aaa374c17bb40588aa9183f48ac))

# [0.60.0](https://github.com/stencila/stencila/compare/v0.59.0...v0.60.0) (2021-05-09)


### Bug Fixes

* **dependencies:** update dependency @mdx-js/react to v1.6.22 ([48c673f](https://github.com/stencila/stencila/commit/48c673fabb4f8a941d5e4cd4d2ea7c31b9ac24d0))
* **dependencies:** update rust crate jsonschema to 0.9.0 ([b574365](https://github.com/stencila/stencila/commit/b5743654fa18a0e048f09f9fe9cb94608a863dd8))
* **dependencies:** update rust crate neon to 0.8.1 ([664e534](https://github.com/stencila/stencila/commit/664e5345e4f965f144ad046eb8a4e6cbe20c5e68))
* **dependencies:** update rust crate regex to 1.5.4 ([bb12f1a](https://github.com/stencila/stencila/commit/bb12f1a552ea26f9928605eb1ab8b0292c3eeee3))
* **dependencies:** update rust crate url to 2.2.2 ([80e8b8e](https://github.com/stencila/stencila/commit/80e8b8ed850010c4a0dfb05a8941fe1ce8176977))


### Features

* **Desktop:** Add ability to check for plugin updates ([45fd4b0](https://github.com/stencila/stencila/commit/45fd4b00f6259d23bba6804ef548e53480ff4250))
* **Desktop:** Add ability to manage plugin installations ([1969153](https://github.com/stencila/stencila/commit/196915379a3719bfb8d46b54eb2c271cc87c287c))
* **Desktop:** Add Settings window router ([65e9ca0](https://github.com/stencila/stencila/commit/65e9ca0c69194f32d254c253b4dcbfa5f4c268d0))

# [0.59.0](https://github.com/stencila/stencila/compare/v0.58.0...v0.59.0) (2021-05-06)


### Bug Fixes

* **Plugins:** Make image optional ([182bbcd](https://github.com/stencila/stencila/commit/182bbcd00f819012fffd0ef249733bb1db72a506))


### Features

* **Desktop:** Initial foundation for adding settings views ([81d7a09](https://github.com/stencila/stencila/commit/81d7a090b2396db4f4dd3519c4a37386eb5a0e02))
* **Plugins:** Add image property to plugin manifest struct ([9d72e21](https://github.com/stencila/stencila/commit/9d72e21c2c4c4adaae5ca1b8b0521e023420d1e9))

# [0.58.0](https://github.com/stencila/stencila/compare/v0.57.1...v0.58.0) (2021-05-05)


### Bug Fixes

* **Logging:** Improve formatting of plain and filter levels properly ([63e80cb](https://github.com/stencila/stencila/commit/63e80cb10de59267cabfc4f05fa822a80c2a8313))
* **Logging & Config:** Make config properties optional ([b3fa8ae](https://github.com/stencila/stencila/commit/b3fa8aeb4929f0a607253dc2a12166e1edc44426))
* **Plugins:** Remove Docker container when collecting plugin manifest ([ee488ef](https://github.com/stencila/stencila/commit/ee488efa60480fbe85e3290f1ef38fa08a1a0e27))
* **Rust:** Update dependencies ([7e2dcf4](https://github.com/stencila/stencila/commit/7e2dcf4519f63c4bebe8b2791c5212a5dd6108af))


### Features

* **CLI:** Add display of progress events ([f925b5f](https://github.com/stencila/stencila/commit/f925b5fa694f94cd1f30dfca23a4907be6700826))
* **CLI:** Add log level and log format options ([67d87a7](https://github.com/stencila/stencila/commit/67d87a71dce95eb99460a42df02ec2db770564c4))
* **Logging:** Add --trace flag ([7122b19](https://github.com/stencila/stencila/commit/7122b1990a714c522e916da42f9aeba6dbe704a8))

## [0.57.1](https://github.com/stencila/stencila/compare/v0.57.0...v0.57.1) (2021-05-03)


### Bug Fixes

* **CLI:** Make log output more compact ([49b5589](https://github.com/stencila/stencila/commit/49b55892a76c31ba337fe2645b567a03324e7d09)), closes [#892](https://github.com/stencila/stencila/issues/892)
* **Rust:** Update dependencies ([fdb032e](https://github.com/stencila/stencila/commit/fdb032e4c476213b070feff0a5e89faa75d19237))

# [0.57.0](https://github.com/stencila/stencila/compare/v0.56.2...v0.57.0) (2021-05-01)


### Features

* **Docs:** Add Asciicasts player component ([d95db64](https://github.com/stencila/stencila/commit/d95db6434bc5014595dcafa94a3637384c39f9ee))

## [0.56.2](https://github.com/stencila/stencila/compare/v0.56.1...v0.56.2) (2021-04-30)


### Bug Fixes

* **Release:** Publish the correct file for Linux ([ec958c8](https://github.com/stencila/stencila/commit/ec958c8ed92d6bb9cb9897b96a8b0ac19c0cc099))

## [0.56.1](https://github.com/stencila/stencila/compare/v0.56.0...v0.56.1) (2021-04-30)


### Bug Fixes

* **CLI:** Do not should env var section on errors ([a3eb92d](https://github.com/stencila/stencila/commit/a3eb92d41aee10d328420edb3031cdb999cf6ccd))

# [0.56.0](https://github.com/stencila/stencila/compare/v0.55.0...v0.56.0) (2021-04-29)


### Features

* **Desktop:** Add Stencila CLI as a dependency ([ac3e30d](https://github.com/stencila/stencila/commit/ac3e30d9fe16058393d636b5dee4627e2737a81e))
* **Desktop:** Bootstrap client using electron-forge ([12c5099](https://github.com/stencila/stencila/commit/12c5099e90cc61c06dc34837f6bec67c443f94e0))

# [0.55.0](https://github.com/stencila/stencila/compare/v0.54.1...v0.55.0) (2021-04-29)


### Bug Fixes

* **dependencies:** update rust crate jsonschema to 0.8.0 ([675ae18](https://github.com/stencila/stencila/commit/675ae18be29a788c18add78ec2efd843b29e3e0d))
* **dependencies:** update rust crate regex to 1.4.6 ([83357b0](https://github.com/stencila/stencila/commit/83357b093deb2cecc2249845460c5a1832d5a04b))
* **dependencies:** update rust crate termimad to 0.10.2 ([1b9a9ca](https://github.com/stencila/stencila/commit/1b9a9ca837e96fa5400f960bbdf61b5b7ee7c819))
* **Plugins:** Do not attempt to upgrade linked plugins ([f2450e1](https://github.com/stencila/stencila/commit/f2450e1a395f4b07cb211363abb6b4a44bc792c8))
* **Upgrade:** Use config settings only for automatic upgrades ([8c2c405](https://github.com/stencila/stencila/commit/8c2c405967e3af6f188f866735801624c1f6f1bb))


### Features

* **Rust & CLI:** Migrate to eyre for errors ([c704f02](https://github.com/stencila/stencila/commit/c704f024d0d823c1e052bd7caea46c175ca8c84d))

## [0.54.1](https://github.com/stencila/stencila/compare/v0.54.0...v0.54.1) (2021-04-28)


### Bug Fixes

* **CLI:** Add banner warning about alpha state ([c66663a](https://github.com/stencila/stencila/commit/c66663ab046205b0db8b24e292dc2101729ccc62))
* **JSON-RPC:** Use correct code for invalid params error ([e4d6650](https://github.com/stencila/stencila/commit/e4d6650a7ded284dac6b3c4caf0c5e82b8fff763))

# [0.54.0](https://github.com/stencila/stencila/compare/v0.53.0...v0.54.0) (2021-04-23)


### Bug Fixes

* **Config:** Renaming of config options to correctly generate schema; add docs ([2466e20](https://github.com/stencila/stencila/commit/2466e20c5c1ee5aa832e03d41c29c4483db13829))
* **Plugins:** Get latest manifest before attempting install ([af1e624](https://github.com/stencila/stencila/commit/af1e62440e273f1ff6abe65f26860e1bd4014653))


### Features

* **Config:** Generate JSOn Schema and expose on CLI and in Node.js bindings ([7869a92](https://github.com/stencila/stencila/commit/7869a9208ea65225202d6d240b5851ad716a6028))
* **Plugins:** Add schema function for obtaining schema of plugins; fix typings ([360f551](https://github.com/stencila/stencila/commit/360f551b56f2e02dc04e0ac58f8b80f77805352d))

# [0.53.0](https://github.com/stencila/stencila/compare/v0.52.0...v0.53.0) (2021-04-22)


### Bug Fixes

* **CLI:** Improve presentation of interactive help ([e442ead](https://github.com/stencila/stencila/commit/e442ead69e3f44764921bdc0f32d6549fc81445e))
* **dependencies:** update rust crate notify to 4.0.16 ([ca05744](https://github.com/stencila/stencila/commit/ca0574419d94fdc7fcde59b0db99b64472e4a2ce))
* **Plugins:** Avoid warning about missing package.json on NPM install ([c6d4256](https://github.com/stencila/stencila/commit/c6d4256d017b5cd942ceb5f298e5cfd6a0e30e79))
* **Plugins:** Only show next if there is one ([cf41d74](https://github.com/stencila/stencila/commit/cf41d74382ed17b17b88e9eb141e399b48d0fe44))
* **Plugins:** Replace plugin folders when installing as package ([a184f00](https://github.com/stencila/stencila/commit/a184f009fa0f63002706d7d9fe9fef1a7d189ea4))
* **Plugins:** Show blank when not installed ([0c5c60e](https://github.com/stencila/stencila/commit/0c5c60e54408ab05d52f54f5ec09a1ab0dc7fa5b))
* **Plugins:** Show registered and installed in list ([db39e48](https://github.com/stencila/stencila/commit/db39e48ce35928c9b6401f6cb1724d5e77bafdf4))
* **Plugins:** Sort list by alias ([670bd9d](https://github.com/stencila/stencila/commit/670bd9df2ff65c68fa4e31c7dd4ed0e6f752d133))


### Features

* **Node bindings:** Add ability to subscribe to logging events ([5b6421e](https://github.com/stencila/stencila/commit/5b6421e6b91c0ab53935296b6e480084c44f51cc))
* **Node bindings:** Add subscriptions module ([926eab7](https://github.com/stencila/stencila/commit/926eab78fb3ec31f5bf0cec1e8f4f5f77417c689))
* **Plugins:** Install using plugin's installUrl array ([eb4899e](https://github.com/stencila/stencila/commit/eb4899e1db2de5b2794fe8726ddf3fdb9d9a421f))

# [0.52.0](https://github.com/stencila/stencila/compare/v0.51.0...v0.52.0) (2021-04-19)


### Bug Fixes

* **Plugins:** Ensure global aliases are merged with local aliases ([93112b9](https://github.com/stencila/stencila/commit/93112b9e5efe85a708adf8a32dfbf8999e9265b9))
* **Plugins:** Handle alternative plugin states when upgrading ([b109818](https://github.com/stencila/stencila/commit/b10981896f52e09dfda859e4294ae4c661607160))
* **Plugins:** Make updates when plugin is refreshed ([a429629](https://github.com/stencila/stencila/commit/a4296296eb19eb18cc08657e811c05200fac6358))
* **Plugins:** Only upgrade plugins that are currently installed ([f63af68](https://github.com/stencila/stencila/commit/f63af68bb04a413515c0531fb3551e2fdb0f96cc))


### Features

* **Node:** Update Node bindings for plugins ([a0b0c97](https://github.com/stencila/stencila/commit/a0b0c972b621ef4ce2714a16e16502ae0283905c))
* **Plugins:** Implement install from CRAN ([1d561a3](https://github.com/stencila/stencila/commit/1d561a3b79d81cbb22cdf4848983674b2cd238d7))
* **Plugins:** Implement install from NPM ([0e2c0ae](https://github.com/stencila/stencila/commit/0e2c0ae343034082b592d90e2a69a32b19f889ef))
* **Plugins:** Implement install from PyPI ([51f0478](https://github.com/stencila/stencila/commit/51f0478eb20534991bfc33ea366e2e761e79b52a))

# [0.51.0](https://github.com/stencila/stencila/compare/v0.50.0...v0.51.0) (2021-04-17)


### Bug Fixes

* **CLI:** Print errors ([0f1253c](https://github.com/stencila/stencila/commit/0f1253c0ada95d1e8530b49093c0aa8963c5f152))
* **Plugins:** Add list_plugins with aliases; allow no confirm ([af0efac](https://github.com/stencila/stencila/commit/af0efac1a31d6e4b6e50ece8230b6fa763f5ef4d))
* **Plugins:** Display using and with aliases ([1cff1fa](https://github.com/stencila/stencila/commit/1cff1fa6ff7d24e31919391a7ec3e65052cecdeb))


### Features

* **Node bindings:** Add plugins module ([71fe53d](https://github.com/stencila/stencila/commit/71fe53db7b00f2812ca3d191e85a87539cf98ed5))

# [0.50.0](https://github.com/stencila/stencila/compare/v0.49.0...v0.50.0) (2021-04-16)


### Bug Fixes

* **Config:** Store global config in Rust [skip release] ([6fe378f](https://github.com/stencila/stencila/commit/6fe378f21d16b37e9430c59fbeb2509a157f8110))


### Features

* **Plugins:** Upgrade plugins based on current installation type ([828c37a](https://github.com/stencila/stencila/commit/828c37a41646dc2937a165c37d6021c4aa4abe29))

# [0.49.0](https://github.com/stencila/stencila/compare/v0.48.1...v0.49.0) (2021-04-15)


### Features

* **Config:** Expose validate function ([be02573](https://github.com/stencila/stencila/commit/be025735785b7b943fc1d20ff44792d99fa8a6f5))
* **Node package:** Expose config functions ([c8838b9](https://github.com/stencila/stencila/commit/c8838b9dfe2b2d26665dba0a6a9e2ced8bf1f2fe))
* **Rust:** Re-export serde ([8992211](https://github.com/stencila/stencila/commit/89922112f4420e7db8765cffb2a34bafc06cf1c8))

## [0.48.1](https://github.com/stencila/stencila/compare/v0.48.0...v0.48.1) (2021-04-14)


### Bug Fixes

* **dependencies:** update rust crate futures to 0.3.14 ([30d5ac1](https://github.com/stencila/stencila/commit/30d5ac144e5b782428e77a1c41b6fc413fa5776e))
* **dependencies:** update rust crate reqwest to 0.11.3 ([8f229ad](https://github.com/stencila/stencila/commit/8f229ad823501d249a61121b820459a2cd776e43))
* **dependencies:** update rust crate tokio to 1.5.0 ([441440a](https://github.com/stencila/stencila/commit/441440a0bc26ee2eaee7743157d75a8bc3441735))

# [0.48.0](https://github.com/stencila/stencila/compare/v0.47.0...v0.48.0) (2021-04-06)


### Bug Fixes

* **Config:** Add logs directory to dirs command ([28dc7c4](https://github.com/stencila/stencila/commit/28dc7c4299e6c1eb4068b0da160e47e19804af81))
* **Interact:** Add welcome message ([aa6c7ad](https://github.com/stencila/stencila/commit/aa6c7ad6de352f82c6e9002c07d1b81ccbce8c27))
* **Plugins:** Pass store into functions to call add/remove ([684e6b0](https://github.com/stencila/stencila/commit/684e6b0f40129f7fef605390ab671c57d601a960))


### Features

* **Inspect:** Add inspect command ([c57f5ad](https://github.com/stencila/stencila/commit/c57f5addf85531f97595c24f2b70e205adcdd3ca))
* **Interact:** Add completions, hints, validation and highlighting ([8662932](https://github.com/stencila/stencila/commit/8662932d5a25231a971c17169aebf8472d656ff9))

# [0.47.0](https://github.com/stencila/stencila/compare/v0.46.0...v0.47.0) (2021-04-05)


### Bug Fixes

* **CLI:** Consistent ordering and color for help ([71f4cd3](https://github.com/stencila/stencila/commit/71f4cd3e10ea45be6366a3eab30324963e4f126e))
* **Interact:** Do not drop last line of help ([c13824b](https://github.com/stencila/stencila/commit/c13824b53524551573d83d80e71b5b05b5f05c93))
* **Plugins:** Return error rather than logging ([b5278e1](https://github.com/stencila/stencila/commit/b5278e1e3f0ae1945e271d8ce935808698f0954b))


### Features

* **CLI:** Show coloured help ([62f8eb4](https://github.com/stencila/stencila/commit/62f8eb49e2ba8e064e22215a38dd0e5e77845c02))
* **Interact:** Add interactive mode ([8122ba0](https://github.com/stencila/stencila/commit/8122ba0961bad39a9e1d2eea5bc7a29d8e77fbf5))
* **Interact:** Allow setting and resetting of prefix ([a684126](https://github.com/stencila/stencila/commit/a684126f7bff23357aafe078651a338d7abf1cb2))

# [0.46.0](https://github.com/stencila/stencila/compare/v0.45.0...v0.46.0) (2021-04-05)


### Bug Fixes

* **CLI:** Refinements to command descriptions and options ([0c2540f](https://github.com/stencila/stencila/commit/0c2540f56bbfd2a2d33ce45e6dbfa1d2ba73940c))
* **Logging:** Initialize a temporary subscriber as soon as possible ([4c1ff46](https://github.com/stencila/stencila/commit/4c1ff463707c1e971796c2087345c9c1610d180d))
* **Logging:** Remove trace level ([ca5d829](https://github.com/stencila/stencila/commit/ca5d8292f097b2a12c5c4b1b11ebbc402a886e35))
* **RPC:** Use error data arg ([66069e1](https://github.com/stencila/stencila/commit/66069e1df6cb6f12b2db451f092d274c0493ebc5))


### Features

* **Logging:** Add configurable logging ([cd4ac32](https://github.com/stencila/stencila/commit/cd4ac3243789ce702cb488806c73d9294aecedb1))

# [0.45.0](https://github.com/stencila/stencila/compare/v0.44.0...v0.45.0) (2021-03-31)


### Bug Fixes

* **CLI:** Only run upgrade thread if not explicitly upgrading ([4ba156d](https://github.com/stencila/stencila/commit/4ba156d20e49597037661862007a900a46dd5035))
* **dependencies:** update rust crate anyhow to 1.0.40 ([de7567b](https://github.com/stencila/stencila/commit/de7567bbecba42e66323800a3ad77cf0330c51a0))
* **dependencies:** update rust crate handlebars to 3.5.4 ([5b1220c](https://github.com/stencila/stencila/commit/5b1220cdf4011e6550006abc95fed55615357a86))
* **dependencies:** update rust crate jsonschema to 0.6.1 ([ca37acb](https://github.com/stencila/stencila/commit/ca37acb63d2817a9fbf3635e25ed3c77c5130be2))
* **dependencies:** update rust crate warp to 0.3.1 ([5800b97](https://github.com/stencila/stencila/commit/5800b97b1ada4827ae2882908e7abbe1706deb6c))
* **Plugins:** Print message when no plugins are installed ([b7535b2](https://github.com/stencila/stencila/commit/b7535b2675b427d491d0eff2ec86897aee41847d))


### Features

* **CLI:** Add convert command with watch option ([e67cb20](https://github.com/stencila/stencila/commit/e67cb20ea589f40fd4d320e3aee79bc524cf0560))
* **Plugins:** Add command to unlink a local plugin ([66e6475](https://github.com/stencila/stencila/commit/66e6475aac4ec15e2f095582fc35df786fcf4274))
* **Plugins:** Read plugins at startup and display methods ([0ab348e](https://github.com/stencila/stencila/commit/0ab348ebb18fd3b9d5db8f3a4478444a84d83ebe))

# [0.44.0](https://github.com/stencila/stencila/compare/v0.43.4...v0.44.0) (2021-03-27)


### Features

* **Plugins:** Display  details about a plugin ([e93a1a2](https://github.com/stencila/stencila/commit/e93a1a2b6f1835c2cde1799a8b4602c425c3a3fc))
* **Plugins:** Display plugin list as a table ([411f6c0](https://github.com/stencila/stencila/commit/411f6c0cbaaea20b9e054a6592df19ea4195e8c5))

## [0.43.4](https://github.com/stencila/stencila/compare/v0.43.3...v0.43.4) (2021-03-24)


### Bug Fixes

* **dependencies:** update rust crate anyhow to 1.0.39 ([fe38ada](https://github.com/stencila/stencila/commit/fe38ada50044149ef1b6b1869db3d7da7a4db6f2))
* **dependencies:** update rust crate neon to 0.8.0 ([44a9602](https://github.com/stencila/stencila/commit/44a9602e13c9e0fdd808c30c46f104c68978b614))
* **dependencies:** update rust crate serde to 1.0.125 ([1c17e23](https://github.com/stencila/stencila/commit/1c17e2306afde6f883f6d0b492053d43403255d2))
* **dependencies:** update rust crate tokio to 1.4.0 ([7f4c676](https://github.com/stencila/stencila/commit/7f4c676d7c18939b1576618fe23f8b934e8bd9cf))
* **dependencies:** update rust crate validator to 0.13.0 ([72606ac](https://github.com/stencila/stencila/commit/72606ac21a339be81e89a984fba2b723aafb83e7))

## [0.43.3](https://github.com/stencila/stencila/compare/v0.43.2...v0.43.3) (2021-03-18)


### Bug Fixes

* **dependencies:** update rust crate regex to 1.4.5 ([d97670e](https://github.com/stencila/stencila/commit/d97670ef75c4d0d6996ed00ad1ffeaf0b8f41c97))

## [0.43.2](https://github.com/stencila/stencila/compare/v0.43.1...v0.43.2) (2021-03-11)


### Bug Fixes

* **Releases:** Use 7z instead of archive task ([a54d005](https://github.com/stencila/stencila/commit/a54d005a19485f327094fac30a2acc70eef9dca9))

## [0.43.1](https://github.com/stencila/stencila/compare/v0.43.0...v0.43.1) (2021-03-10)


### Bug Fixes

* **Release:** Use UPLOAD_PATH; use set -e on CI ([225d525](https://github.com/stencila/stencila/commit/225d525c2309ece08fe0c0bbc2628b2ca1ba9a38))

# [0.43.0](https://github.com/stencila/stencila/compare/v0.42.1...v0.43.0) (2021-03-10)


### Bug Fixes

* **dependencies:** update rust crate bollard to 0.10.1 ([2a091c8](https://github.com/stencila/stencila/commit/2a091c857f07ffb7a83bc9443a38febe3bed6436))
* **dependencies:** update rust crate reqwest to 0.11.2 ([6d3c08d](https://github.com/stencila/stencila/commit/6d3c08d53b3e8236b5e9cd491e36cc1531a1b8fb))
* **dependencies:** update rust crate self_update to 0.26.0 ([8141bef](https://github.com/stencila/stencila/commit/8141bef700e085e7900030b5d34954464cb41c73))
* **dependencies:** update rust crate serde to 1.0.124 ([093b70f](https://github.com/stencila/stencila/commit/093b70f0c8d8faf62767aca6e6d457e6cd270eb2))
* **dependencies:** update rust crate tokio to 1.3.0 ([3964dcf](https://github.com/stencila/stencila/commit/3964dcf5980097c754c42fa5b987b14e6e1dedae))
* **Install:** Allow for latest and location to be specified; use in Docker image ([d65b52e](https://github.com/stencila/stencila/commit/d65b52e481056b1b40eaaaebe2290c0290a5bc25))


### Features

* **Install:** Add install script and instructions for CLI ([ca2c15a](https://github.com/stencila/stencila/commit/ca2c15accb60aa0c7ac742cab0d3b1ce55b7d6ee))

## [0.42.1](https://github.com/stencila/stencila/compare/v0.42.0...v0.42.1) (2021-03-10)


### Bug Fixes

* **Docker images:** Install binary for faster build ([981cb23](https://github.com/stencila/stencila/commit/981cb23328df5ca518467a91e3a202bc7cb6e168))

# [0.42.0](https://github.com/stencila/stencila/compare/v0.41.0...v0.42.0) (2021-03-10)


### Features

* **Docker images:** Add initial versions ([ff12ab9](https://github.com/stencila/stencila/commit/ff12ab97ec2df6c58ec254efcafc117966c33de7))

# [0.41.0](https://github.com/stencila/stencila/compare/v0.40.1...v0.41.0) (2021-03-09)


### Bug Fixes

* **Load plugin:** Ignore plugin method if invalid schema ([fde35e6](https://github.com/stencila/stencila/commit/fde35e6c60c7c7feebd14d3075021fd986dda8d7))


### Features

* **Plugins:** Add plugin link subcommand; compile features schema ([7bc7e16](https://github.com/stencila/stencila/commit/7bc7e1662453cd7dd9adbb3af1007493734be224))
* **Plugins:** Inital version of plugins subcommand ([6afc011](https://github.com/stencila/stencila/commit/6afc011dada5622b3c1bc127e808d889fb00eaed))
* **Plugins:** Initial implementation for adding binary plugins ([a46315b](https://github.com/stencila/stencila/commit/a46315b2db2f7322f9e84ef7b86025634913ec13))

## [0.40.1](https://github.com/stencila/stencila/compare/v0.40.0...v0.40.1) (2021-03-05)


### Bug Fixes

* **Windows release:** Archive using task because zip not available ([49dabb6](https://github.com/stencila/stencila/commit/49dabb65dba5521ba071f33611dd52c60d91480d))

# [0.40.0](https://github.com/stencila/stencila/compare/v0.39.1...v0.40.0) (2021-03-03)


### Bug Fixes

* **dependencies:** update rust crate once_cell to 1.7.2 ([a1ba977](https://github.com/stencila/stencila/commit/a1ba977f6218f2b288e48c489eb6bc96d07ddce3))
* **dependencies:** update rust crate serde_json to 1.0.64 ([2e44fee](https://github.com/stencila/stencila/commit/2e44fee35dd55e6ecffb19112942c247b8705d72))
* **dependencies:** update rust crate tokio-tungstenite to 0.14.0 ([239d113](https://github.com/stencila/stencila/commit/239d1139ef52cf76bd29e10ffa451f4bb7a3a984))
* **Deps:** Cargo audit fix ([6d8589b](https://github.com/stencila/stencila/commit/6d8589bcfa8ef9515d072e0a312b9a218084d8f8))
* **Deps:** Use dirs-next instead of dirs ([b5fe31d](https://github.com/stencila/stencila/commit/b5fe31d0b1edb2f6ffacb8a8ec052c60cf9a5a01))
* **Docs:** Fix generation of help documentation ([f10aeb4](https://github.com/stencila/stencila/commit/f10aeb46556fd89b4c4997a27c6fcbeae1a0c7ec))


### Features

* **Config:** Add validation of config ([62a5a9b](https://github.com/stencila/stencila/commit/62a5a9b904cbd919a8e226120756d2e274bccc73))
* **Config:** Get, set and reset config using TOML ([af7b626](https://github.com/stencila/stencila/commit/af7b6269f5cd39602439a970515906b463fc97f8))


### Performance Improvements

* **Upgrade:** Release and fetch compressed binaries ([e8cdf44](https://github.com/stencila/stencila/commit/e8cdf4477e7472e3c85fb27dff9c0a1b7359d606))

## [0.39.1](https://github.com/stencila/stencila/compare/v0.39.0...v0.39.1) (2021-03-01)


### Bug Fixes

* **Release:** Drop use of upload label which was causing issues for Windows ([8ac528c](https://github.com/stencila/stencila/commit/8ac528cc9b87808576a564ce100214d82a00d3f3))


### Performance Improvements

* **Directories:** Avoid unecessary create_dir_all call ([95cfd3d](https://github.com/stencila/stencila/commit/95cfd3da7918d43a81c18aec0794c9c1402a92de))

# [0.39.0](https://github.com/stencila/stencila/compare/v0.38.2...v0.39.0) (2021-02-28)


### Features

* **Auto upgrade:** Do automatic upgrades with configurable frequency ([a4ed8bc](https://github.com/stencila/stencila/commit/a4ed8bc489d7980d5fa7bd50a258b9c8f59747d2))

## [0.38.2](https://github.com/stencila/stencila/compare/v0.38.1...v0.38.2) (2021-02-25)


### Bug Fixes

* **Release:** Avoid sed which differs on MacOS ([7377b13](https://github.com/stencila/stencila/commit/7377b13ea287a9e668fdd738ab772f68c9a1657a))
* **Release:** Use asset name when calling upload script ([c813e76](https://github.com/stencila/stencila/commit/c813e768b8d89d5dae66858b92548144ac1b69cb))

## [0.38.1](https://github.com/stencila/stencila/compare/v0.38.0...v0.38.1) (2021-02-25)


### Bug Fixes

* **Release:** Update version in top level package ([4ef4ea8](https://github.com/stencila/stencila/commit/4ef4ea8b512f9beb4ac27506a5df41d60dad0953))

# [0.38.0](https://github.com/stencila/stencila/compare/v0.37.0...v0.38.0) (2021-02-25)


### Bug Fixes

* **Config:** Handle global options ([882d456](https://github.com/stencila/stencila/commit/882d456af904ba998ff72657a4d85f1ff782c39c))


### Features

* **Upgrade:** Add options for desired version and to force upgrade ([ad28f01](https://github.com/stencila/stencila/commit/ad28f019bf1dc8bd628d2ad450efdebe99771c96))

# [0.37.0](https://github.com/stencila/stencila/compare/v0.36.7...v0.37.0) (2021-02-25)


### Features

* **Config:** Add config command ([2ab9fbe](https://github.com/stencila/stencila/commit/2ab9fbe5393cafa2cfaf66d2e618fdf883e24e9d))

## [0.36.7](https://github.com/stencila/stencila/compare/v0.36.6...v0.36.7) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate url to 2.2.1 ([cc19037](https://github.com/stencila/stencila/commit/cc1903772e83a55f6ddb54d67c2452cf1c22bf9b))

## [0.36.6](https://github.com/stencila/stencila/compare/v0.36.5...v0.36.6) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate tracing to 0.1.25 ([a57f933](https://github.com/stencila/stencila/commit/a57f933eb8e91369763ed2681ed1ce8b7717977c))

## [0.36.5](https://github.com/stencila/stencila/compare/v0.36.4...v0.36.5) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate thiserror to 1.0.24 ([0cf99e8](https://github.com/stencila/stencila/commit/0cf99e8447eff64d4d8687619a785187fa8ed5e6))

## [0.36.4](https://github.com/stencila/stencila/compare/v0.36.3...v0.36.4) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate self_update to 0.25.0 ([2b21114](https://github.com/stencila/stencila/commit/2b21114c2c532715b4409450e8f26960d05f9bdd))

## [0.36.3](https://github.com/stencila/stencila/compare/v0.36.2...v0.36.3) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate reqwest to 0.11.1 ([328d50c](https://github.com/stencila/stencila/commit/328d50c71babe07f95473e6c0f6d8848896f9aa1))

## [0.36.2](https://github.com/stencila/stencila/compare/v0.36.1...v0.36.2) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate once_cell to 1.6.0 ([f4e5163](https://github.com/stencila/stencila/commit/f4e51637ff8775eacdc55d61884fda25886dc331))

## [0.36.1](https://github.com/stencila/stencila/compare/v0.36.0...v0.36.1) (2021-02-24)


### Bug Fixes

* **dependencies:** update rust crate futures to 0.3.13 ([1e21537](https://github.com/stencila/stencila/commit/1e2153704742aa984eb6433f5a4230f92957932f))

# [0.36.0](https://github.com/stencila/stencila/compare/v0.35.0...v0.36.0) (2021-02-21)


### Features

* **Upgrade:** Add upgrade command for updating to most recent release ([81a5775](https://github.com/stencila/stencila/commit/81a57751a88a94f6036ae3614654a7a64e293aa1))

# [0.35.0](https://github.com/stencila/stencila/compare/v0.34.3...v0.35.0) (2021-02-20)


### Bug Fixes

* **dependencies:** pin dependencies ([ed7891a](https://github.com/stencila/stencila/commit/ed7891a03d3382e893db665f81c95d9e3b5166ab))
* **dependencies:** update dependency @stencila/encoda to ^0.98.6 ([8c6709c](https://github.com/stencila/stencila/commit/8c6709c0cfd8b9e30651d83cbf0077e82e722a47))
* **dependencies:** update dependency @stencila/encoda to v0.104.5 ([24016c0](https://github.com/stencila/stencila/commit/24016c06246000c35ce86b664dda50084e60743c))
* **dependencies:** update dependency tar to ^6.0.5 ([5b7a39d](https://github.com/stencila/stencila/commit/5b7a39d6fb3160e3785adb1614ae4e281562c679))
* **dependencies:** update dependency tar to v6.1.0 ([9fd0153](https://github.com/stencila/stencila/commit/9fd01534d96ad9d47acad2783ecbda73e79f8d58))
* **dependencies:** update dependency yargs to v16 ([29c7c7e](https://github.com/stencila/stencila/commit/29c7c7e96451c6c83aa635812f1f2b0a25c9d940))
* **dependencies:** update rust crate anyhow to 1.0.38 ([2d9e5fb](https://github.com/stencila/stencila/commit/2d9e5fb44080c7e84223bb6d6f567681e31a54fe))
* **dependencies:** update rust crate env_logger to 0.8.3 ([e019917](https://github.com/stencila/stencila/commit/e019917fdfbe7807a6189f03e3ab3eb9eca0697a))
* **Dependencies:** Update deps to latest versions ([6eff527](https://github.com/stencila/stencila/commit/6eff527c9cc1ee7127eeeee8ccd579e5297504ff))
* **Deps:** Add strum_macros ([505d1e9](https://github.com/stencila/stencila/commit/505d1e9033de94ed4463e68344c54fb573aeeb11))
* **Deps:** Cargo audit fix ([1aca400](https://github.com/stencila/stencila/commit/1aca40075415e41c1a1c21ea12d2ed59de13d3d3))
* **Deps:** Update Encoda and yargs ([980c0bd](https://github.com/stencila/stencila/commit/980c0bda997dc912b4b5b222c7ca4c5c3ea29363))
* **Deps:** Update tokio, warp etc ([0bbc1e4](https://github.com/stencila/stencila/commit/0bbc1e48300d1c3ab14694005d1eee851c90e62a))
* **Docs:** Skip schema coercion as it loses necessary meta data ([67a01c7](https://github.com/stencila/stencila/commit/67a01c7879988669b41e6db1053aac7dfab51b50))


### Features

* **JWT:** Add JSON Web Token authorization ([075f407](https://github.com/stencila/stencila/commit/075f407a70fa97a0f3d68647f3c2ba685c3a9ca7))
* **Node:** Very preliminary version of Stencila for Node.js package ([595308b](https://github.com/stencila/stencila/commit/595308be17a3106543e456233e7133eda154bebf))
* **Open:** Add open command for opening  a stencil in browser ([29afbe1](https://github.com/stencila/stencila/commit/29afbe17d36649b11b09e2032db4fae1de715cb9))
* **R:** Very preliminary version of Stencila for R package ([7aed77e](https://github.com/stencila/stencila/commit/7aed77e91712843f44f232a40b51832253041cbb))
* **Rust:** Add the execute command ([23832b2](https://github.com/stencila/stencila/commit/23832b21f8e2cef57e968e4c30ae733f85663e7d))
* **Serve & request:** Add server and user-agent headers ([00f0335](https://github.com/stencila/stencila/commit/00f0335e19cbb45d74d608e09a5b9ca95647abfd))

## [0.34.3](https://github.com/stencila/stencila/compare/v0.34.2...v0.34.3) (2020-11-06)


### Bug Fixes

* **dependencies:** update dependency @stencila/logga to v3 ([8fe1f98](https://github.com/stencila/stencila/commit/8fe1f98939e01a48d0b7174438d9cd9a2f695608))

## [0.34.2](https://github.com/stencila/stencila/compare/v0.34.1...v0.34.2) (2020-08-27)


### Bug Fixes

* **dependencies:** update dependency @stencila/encoda to ^0.98.5 ([b206f67](https://github.com/stencila/stencila/commit/b206f6768e86a4b068a537cfdb7262d5f8aa756d))

## [0.34.1](https://github.com/stencila/stencila/compare/v0.34.0...v0.34.1) (2020-08-05)


### Bug Fixes

* **dependencies:** update dependency @stencila/encoda to ^0.97.3 ([285e573](https://github.com/stencila/stencila/commit/285e573da2e04a6f67b33e302429eb1fdaf4e43d))

# [0.34.0](https://github.com/stencila/stencila/compare/v0.33.5...v0.34.0) (2020-07-20)


### Bug Fixes

* **Deps:** Remove opn ([6b5bb49](https://github.com/stencila/stencila/commit/6b5bb499cb715c288335f0944a3171917268ead3))
* **Docs:** Fix Intercom doc link generation ([5f8cda6](https://github.com/stencila/stencila/commit/5f8cda61a97198ff9839770bf5c822c3b4838111))
* **Docs:** Fix related articles section generation ([6cad5c3](https://github.com/stencila/stencila/commit/6cad5c326a44daca135d0f724f3d00cf4017c0ab))
* **Docs:** Fix relative links in documentation ([72b3a66](https://github.com/stencila/stencila/commit/72b3a66a2cce9bc27d9700b18f6febe2f1071b38))


### Features

* **Docs:** Add a link to help make documentation better ([6caf76a](https://github.com/stencila/stencila/commit/6caf76a2e677a976be1f32a0cb93a807b68da8d9))
* **Docs:** Terminate documentation build in case of errors ([d09d2cb](https://github.com/stencila/stencila/commit/d09d2cbfed67bee7561a8a72a899983411b91523))
* **Process:** Remove process command ([e0f9562](https://github.com/stencila/stencila/commit/e0f95625e5298a6423e8504a19f696a1130285c1))

## [0.33.5](https://github.com/stencila/stencila/compare/v0.33.4...v0.33.5) (2019-10-11)


### Bug Fixes

* Updated to encoda 0.80.1 ([02518d1](https://github.com/stencila/stencila/commit/02518d1))

## [0.33.4](https://github.com/stencila/stencila/compare/v0.33.3...v0.33.4) (2019-09-30)


### Bug Fixes

* Update Encoda to 0.80.0 ([072f340](https://github.com/stencila/stencila/commit/072f340))
* Update packages using `npm audit fix` ([81a9dcc](https://github.com/stencila/stencila/commit/81a9dcc))

## [0.33.3](https://github.com/stencila/stencila/compare/v0.33.2...v0.33.3) (2019-09-25)


### Bug Fixes

* Update Encoda to 0.78.2 ([4a6c3fa](https://github.com/stencila/stencila/commit/4a6c3fa))

## [0.33.2](https://github.com/stencila/stencila/compare/v0.33.1...v0.33.2) (2019-09-20)


### Bug Fixes

* Updated to encoda 0.78.0 ([1a3b2bc](https://github.com/stencila/stencila/commit/1a3b2bc))

## [0.33.1](https://github.com/stencila/stencila/compare/v0.33.0...v0.33.1) (2019-09-20)


### Bug Fixes

* Updated to encoda 0.77.1 for relative zip dir fixes ([9ab97bd](https://github.com/stencila/stencila/commit/9ab97bd))

# [0.33.0](https://github.com/stencila/stencila/compare/v0.32.1...v0.33.0) (2019-09-17)


### Features

* **Zip archive:** Add option to create a zip archive ([88c9ac7](https://github.com/stencila/stencila/commit/88c9ac7))

## [0.32.1](https://github.com/stencila/stencila/compare/v0.32.0...v0.32.1) (2019-09-15)


### Bug Fixes

* **Packaging:** Fix to library inclusion in macOS build ([4a8627e](https://github.com/stencila/stencila/commit/4a8627e))

# [0.32.0](https://github.com/stencila/stencila/compare/v0.31.1...v0.32.0) (2019-09-13)


### Features

* **Deps:** Upgrade Encoda to 0.75.4 ([1528d09](https://github.com/stencila/stencila/commit/1528d09))

## [0.31.1](https://github.com/stencila/stencila/compare/v0.31.0...v0.31.1) (2019-09-10)


### Bug Fixes

* **CI:** Do no skip CI on tag to trigger new release ([530365f](https://github.com/stencila/stencila/commit/530365f))

# [0.31.0](https://github.com/stencila/stencila/compare/v0.30.5...v0.31.0) (2019-09-10)


### Features

* **Convert:** Upgrade Encoda, allow for multiple convert output files ([d299320](https://github.com/stencila/stencila/commit/d299320))

## [0.30.5](https://github.com/stencila/stencila/compare/v0.30.4...v0.30.5) (2019-09-04)


### Bug Fixes

* **CLI:** Add global error handler ([fef2000](https://github.com/stencila/stencila/commit/fef2000))
* **Encoda:** Update encoda to 0.71.2 ([658e2bc](https://github.com/stencila/stencila/commit/658e2bc))

## [0.30.3](https://github.com/stencila/stencila/compare/v0.30.2...v0.30.3) (2019-09-03)


### Bug Fixes

* **Convert:** Only default to md for stdin ([f4a0332](https://github.com/stencila/stencila/commit/f4a0332))
* **Logging:** Update and configure logga ([4c784cb](https://github.com/stencila/stencila/commit/4c784cb))
* **Package:** Update dependencies and refactor accordingly ([49fffc2](https://github.com/stencila/stencila/commit/49fffc2))
* **Web:** Remove extra argument ([c5ce354](https://github.com/stencila/stencila/commit/c5ce354))

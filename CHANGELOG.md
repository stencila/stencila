# [1.10.0](https://github.com/stencila/schema/compare/v1.9.1...v1.10.0) (2021-07-14)


### Bug Fixes

* **CreativeWork:** Use semicolon separated items for authors, add csi for genre ([56a3bb6](https://github.com/stencila/schema/commit/56a3bb6c66cdb4df0af773af8c218510289b5aba))
* **Primitves:** Refer to internal schema for primitives ([54a8264](https://github.com/stencila/schema/commit/54a8264575abd9e16ae6bcba549b7610518bd932))


### Features

* **Rust:** Include map of schemas in crate ([4b82b26](https://github.com/stencila/schema/commit/4b82b26e406850d73f964ed74acdd04ddba156dd))

## [1.9.1](https://github.com/stencila/schema/compare/v1.9.0...v1.9.1) (2021-07-06)


### Bug Fixes

* **Array and string validators:** Use integer for property types where appropriate ([866423e](https://github.com/stencila/schema/commit/866423eefd32670dd66e73773cd5e3ac675740af))
* **Rust:** Do not box primitive properties; allow overrides ([e817aaa](https://github.com/stencila/schema/commit/e817aaa2f59d90b5c46a6464cedd7d5ccab8717e))

# [1.9.0](https://github.com/stencila/schema/compare/v1.8.1...v1.9.0) (2021-06-21)


### Bug Fixes

* **CreativeWork:** Allow an array of nodes ([a13cbfc](https://github.com/stencila/schema/commit/a13cbfc349b6a1b6d8931d15bbb77357c29c47ac))
* **dependencies:** update rust crate serde to 1.0.126 ([371712e](https://github.com/stencila/schema/commit/371712e27b21e5b9d7a0dadf9578dba9633e6829))
* **dependencies:** update rust crate serde_with to 1.9.4 ([ac06fd3](https://github.com/stencila/schema/commit/ac06fd30aa4ac3a42345acc973be8ca2e64b2b03))
* **InlineContent:** Remove `Array` and `Object` from `InlineContent` ([a844439](https://github.com/stencila/schema/commit/a844439ad0be31d544b863f741d940ef1ba1fba2)), closes [/github.com/stencila/schema/pull/274#pullrequestreview-687650292](https://github.com//github.com/stencila/schema/pull/274/issues/pullrequestreview-687650292)
* **Python:** Avoid generating recursive types ([eceb36a](https://github.com/stencila/schema/commit/eceb36af3e2310711859c62995c7378b19052b56))
* **R bindings:** Avoid type recursion ([efdae5c](https://github.com/stencila/schema/commit/efdae5c434450301f8263b54396489db1a203f94))
* **Rust bindings:** Fix name for Array ([e955f27](https://github.com/stencila/schema/commit/e955f271c4098132b97496d4df3ca42e872ccc28))
* **Types:** Include Entity in Node union type ([e713276](https://github.com/stencila/schema/commit/e7132760a9924257336d483a0aa0ac29778cd019))
* **TypeScript:** Reduce number, and simplify, type guards ([d348d17](https://github.com/stencila/schema/commit/d348d174be660384369d6fe9887bf82745d1b7aa))
* **TypeScript bindings:** Revert to using any ([4d1b761](https://github.com/stencila/schema/commit/4d1b761e03bd258d3670f7caa2033cb0fbf90bc7))


### Features

* **Schema:** Rename property format to mediaType ([bfb10d6](https://github.com/stencila/schema/commit/bfb10d6cc89204fb0b6c3aa413ea867c51d95617))

## [1.8.1](https://github.com/stencila/schema/compare/v1.8.0...v1.8.1) (2021-06-06)


### Bug Fixes

* **Rust:** Box optional properties ([be279c0](https://github.com/stencila/schema/commit/be279c010446a0eacdefb22f2b9c97664075141e))


### Performance Improvements

* **Rust:** Do not include meta property ([893de26](https://github.com/stencila/schema/commit/893de2697a9a505783312d09dd7f4d1a62af470e))
* **Rust:** Reduce memory footprint of type deserialization discriminant ([7c36200](https://github.com/stencila/schema/commit/7c3620054089918ea9ad0b2c2dbdd643a30c7487))
* **Rust:** Use simple structs for creative works when present as inline or block content ([9a650d1](https://github.com/stencila/schema/commit/9a650d13206c0f8d0abba7b0995cebabbc606b0a))

# [1.8.0](https://github.com/stencila/schema/compare/v1.7.3...v1.8.0) (2021-06-04)


### Bug Fixes

* **BlockContent:** Remove ListItem from block content ([b904684](https://github.com/stencila/schema/commit/b9046842a188d44599a6da0eb45a17c86eabb2fc))
* **CreativeWork:** Narrow allowed content to arrays of inline or block content or string ([a85fd06](https://github.com/stencila/schema/commit/a85fd060cee638d6185708bc6c8e1abd75ea9115))
* **Deps:** NPM audit fix ([d6c500d](https://github.com/stencila/schema/commit/d6c500d62ec46304d930684d96e0295e137746fb))
* **Heading, ListItem:** Use integer instead of number for properties ([cfcb195](https://github.com/stencila/schema/commit/cfcb195be857babfb691e4dd88291bfbcf7a27a9))
* **InlineContent:** Add array and object to valid inline content ([7483e80](https://github.com/stencila/schema/commit/7483e80a9b3356603402fd3e68e425eed410e496))
* **InlineContent:** Remove  MediaObject from inline content ([010efad](https://github.com/stencila/schema/commit/010efad536a6d49a8450218d1fc116742cc56dc4))
* **ListItem:** Narrow allowed content to arrays of inline or block content ([5c80b17](https://github.com/stencila/schema/commit/5c80b17177420aa781b1a91130168465c2bd3ecf))
* **Node:** Expand the `Node` union type to include all entity types ([b658222](https://github.com/stencila/schema/commit/b658222f540ae107c458e9b5f9902140c3863941))
* **Rust bindings:** Use a Box rather than Arc since do not need shared ownership ([fa939cd](https://github.com/stencila/schema/commit/fa939cd1650c98ee47deb330bccfeeaa6aa54e4d))
* **Rust bindings:** Use Boolean for consistency with schema.org ([c754d33](https://github.com/stencila/schema/commit/c754d33bb2024a8e508df2730987741c4b0ea570))
* **TableCell:** Narrow allowed content to arrays of inline or block content ([a207782](https://github.com/stencila/schema/commit/a2077829e48749c1f17cc3343366a50f7846b874))


### Features

* **Rust:** Add trait for type name and id ([fd6f0a3](https://github.com/stencila/schema/commit/fd6f0a37dde54d5363b7854aae9be6b4ebcfbfd6))
* **Rust bindings:** Add type name and id traits ([43c0554](https://github.com/stencila/schema/commit/43c05549f8a69f5308708de4ea6189d430d5d984))

## [1.7.3](https://github.com/stencila/schema/compare/v1.7.2...v1.7.3) (2021-06-01)


### Bug Fixes

* **Rust:** Improve primitive types ([98296aa](https://github.com/stencila/schema/commit/98296aad9be1cb5b7e32517aee1d7e034ed90848))

## [1.7.2](https://github.com/stencila/schema/compare/v1.7.1...v1.7.2) (2021-05-27)


### Bug Fixes

* **Schema:** For consistency capitalize all enum variants ([c94405d](https://github.com/stencila/schema/commit/c94405de352f79258e48f44f6fd50e3ce8a450e3))

## [1.7.1](https://github.com/stencila/schema/compare/v1.7.0...v1.7.1) (2021-05-25)


### Bug Fixes

* **CreativeWork:** Make about property refer to ThingTypes ([3d38a3e](https://github.com/stencila/schema/commit/3d38a3ec02e5b98a36cd42e28817f45c994d0260))
* **Rust:** Reuse property types from base types ([7518b31](https://github.com/stencila/schema/commit/7518b31e16a88c2f66617f4bc5db4e48cf0daa2c))
* **Rust:** Use String for date value to avoid errors when deserializing partial date strings ([1b892b9](https://github.com/stencila/schema/commit/1b892b9f26b62e00227950cbef1b1e2cd70c07d1))

# [1.7.0](https://github.com/stencila/schema/compare/v1.6.1...v1.7.0) (2021-05-20)


### Features

* **Rust:** Add Clone derive ([5f1cfb5](https://github.com/stencila/schema/commit/5f1cfb5650902d4ab909191036f27e0e85a334bc))

## [1.6.1](https://github.com/stencila/schema/compare/v1.6.0...v1.6.1) (2021-05-11)


### Bug Fixes

* Update status properties, add note on status to docs and publish experimental schemas ([7fe6693](https://github.com/stencila/schema/commit/7fe6693af95980c925c60b793c43e3c3d124a760))
* **String:** Rename from Text for better consistency with other data types and languages ([a944fea](https://github.com/stencila/schema/commit/a944fea47e7da8ea43a3e57b8611709220864c61))

# [1.6.0](https://github.com/stencila/schema/compare/v1.5.3...v1.6.0) (2021-05-11)


### Bug Fixes

* **Rust:** Expose enum schemas, fix other types ([7c13871](https://github.com/stencila/schema/commit/7c138713df1f9ed999deb0cedf46822ec06aba8f))
* **Rust:** Fix casing and visibility ([47486cf](https://github.com/stencila/schema/commit/47486cf7fdbfed61ee51fcd09abdaea206df25c1))
* **Rust bindings:** Use pointers to avoid recursive types ([40e27ee](https://github.com/stencila/schema/commit/40e27eeade71bf6584bc3d8c129c55e2d28ec5c6))


### Features

* **Rust bindings:** Add  JSON serialization and deserialization ([7bee60f](https://github.com/stencila/schema/commit/7bee60f515a5a2280fc92d9ef93cf98bb6f9f868))
* **Rust bindings:** Add bindings for Rust ([34d789f](https://github.com/stencila/schema/commit/34d789f0b171c9383e6b59b0d956629209c3bb13))

## [1.5.3](https://github.com/stencila/schema/compare/v1.5.2...v1.5.3) (2021-05-06)


### Bug Fixes

* **Works:** Put all types derived from CreativeWork in works category ([1b72009](https://github.com/stencila/schema/commit/1b720096ce7d0b7c9ac617129b2ad911a28d8aa6))

## [1.5.2](https://github.com/stencila/schema/compare/v1.5.1...v1.5.2) (2021-05-06)


### Bug Fixes

* **Docs:** Improve Markdown documents and generate category listing ([a522e14](https://github.com/stencila/schema/commit/a522e14bd595860857811f9ac8a72aa82d21e9cc))

## [1.5.1](https://github.com/stencila/schema/compare/v1.5.0...v1.5.1) (2021-05-01)


### Bug Fixes

* Capitalize folder names for better integration with help site ([cbec688](https://github.com/stencila/schema/commit/cbec688ac01f14829c26c91943fb1d97d52b32f7))

# [1.5.0](https://github.com/stencila/schema/compare/v1.4.3...v1.5.0) (2021-04-28)


### Bug Fixes

* **JSON Schemas:** Fix source URL and apply category to union types ([05aa171](https://github.com/stencila/schema/commit/05aa171a463765d8aac57974122b5a1ffdba9bb2))


### Features

* **Person:** Add example to Person schema ([2e17892](https://github.com/stencila/schema/commit/2e178928c0fa3e760309de4053ad34e505488e0b))

## [1.4.3](https://github.com/stencila/schema/compare/v1.4.2...v1.4.3) (2021-04-21)


### Bug Fixes

* **Docs:** Add redirect path for CSS ([9d7a977](https://github.com/stencila/schema/commit/9d7a97790d913948388b06face17b4b3d450530d))

## [1.4.2](https://github.com/stencila/schema/compare/v1.4.1...v1.4.2) (2021-04-21)


### Bug Fixes

* **Docs:** Fix file names and link paths ([efb8c55](https://github.com/stencila/schema/commit/efb8c55ef557130efa8749044138654a41761da0))

## [1.4.1](https://github.com/stencila/schema/compare/v1.4.0...v1.4.1) (2021-04-20)


### Bug Fixes

* **Prose types:** Specifiy category ([6b72da5](https://github.com/stencila/schema/commit/6b72da529f5b22331ad4af5348cad46f051978a2))
* **Validator:** Use a base Validator type ([ed53b4d](https://github.com/stencila/schema/commit/ed53b4db09d9e8d5269b8445da2daeca767456a3))

# [1.4.0](https://github.com/stencila/schema/compare/v1.3.1...v1.4.0) (2021-04-20)


### Bug Fixes

* **Citation intent:** Rename schema file ([9a77e54](https://github.com/stencila/schema/commit/9a77e5403d61387ab473c19b4ed9e70a4f088837))
* **CitationTypeEnumeration:** Excludes members that are related to citation distance ([dbfcd12](https://github.com/stencila/schema/commit/dbfcd1288807b0772baaac5274179c238b36db28))
* **CitationTypeEnumeration:** Use correct title and file extension ([4d9ca2b](https://github.com/stencila/schema/commit/4d9ca2b4df4f1e24507c023ae751ed5c03b202ce))
* **JSON-LD:** Add CiTO to context ([3bdfe23](https://github.com/stencila/schema/commit/3bdfe2320d8bfe9ae79f07fa5181d65883d53507))
* **Language bindings:** Handle enumeration types ([a524c92](https://github.com/stencila/schema/commit/a524c927ea6a502773affe017ea83e277c0f7424))
* **VSCode bindings:** Preserve case for easier matching during tests ([c272d28](https://github.com/stencila/schema/commit/c272d282b58e39a264443359bfd54e1e8629e251))


### Features

* **CitationTypeEnumeration:** Add an enumeration for CiTO citation types ([dcc3344](https://github.com/stencila/schema/commit/dcc33442b7429835514250ba1c882835e5c2e973))
* **VSCode bindings:** Add maapings between file types and schemas ([2544b6e](https://github.com/stencila/schema/commit/2544b6ea755239cef67cbaaf236a92ebcce1f6c6))

## [1.3.1](https://github.com/stencila/schema/compare/v1.3.0...v1.3.1) (2021-04-14)


### Bug Fixes

* **Claim:** Add Claim to list of valid BlockContent elements ([fd7a01e](https://github.com/stencila/schema/commit/fd7a01e4b5f67e7011193b18e306ef822d30a78e))

# [1.3.0](https://github.com/stencila/schema/compare/v1.2.3...v1.3.0) (2021-04-13)


### Bug Fixes

* **Claim:** Use schema id; order properties alphabetically ([f9d2e67](https://github.com/stencila/schema/commit/f9d2e6727fd0117564d68be79599262b16d20620))


### Features

* **Claim:** Draft Claim specification ([43833b4](https://github.com/stencila/schema/commit/43833b49a9b69f1ead0a13c881aa9f904929222b))

## [1.2.3](https://github.com/stencila/schema/compare/v1.2.2...v1.2.3) (2021-04-13)


### Bug Fixes

* **Node:** Change ordering so array is last ([17cfdfc](https://github.com/stencila/schema/commit/17cfdfc5c79aac7db937174b6dc9db505723f1ad))
* **PropertyValue:** Narrow schema for value property ([0c058fb](https://github.com/stencila/schema/commit/0c058fb8ef905483e3238f567c820039d1a76be9))

## [1.2.2](https://github.com/stencila/schema/compare/v1.2.1...v1.2.2) (2021-04-12)


### Bug Fixes

* Order primitive types consistenctly and always after entities ([810c5da](https://github.com/stencila/schema/commit/810c5da9fce0cbf391ded9b61afc971c4178e880))
* **Inline Content:** Add audio, media and video objects; reorder primitives ([de6cfd5](https://github.com/stencila/schema/commit/de6cfd534f9985218e87648ed525274f1ffaa7e0))
* **Link:** Use uri-reference instead of uri ([fdd6b04](https://github.com/stencila/schema/commit/fdd6b0426d2275d5b336114dd6ea18b952e1f540))
* **Table, Figure, CodeChunk:** Narrow caption to BlockContent falling back to string ([4acc3ba](https://github.com/stencila/schema/commit/4acc3ba691a17844d7a24f1a295c3d352c83aac6))

## [1.2.1](https://github.com/stencila/schema/compare/v1.2.0...v1.2.1) (2021-04-07)


### Bug Fixes

* **Article:** Narrow content to BlockContent types ([ebc4560](https://github.com/stencila/schema/commit/ebc45600cccad68fbabbb83562ac72d4c660f64e))
* **InlineContent:** Add Note and ordering ([aa370b6](https://github.com/stencila/schema/commit/aa370b665f2fd43dd00e32b96838df33bce0d41c))
* **Python bindings:** Ignore type for fields that are overrides ([6f80099](https://github.com/stencila/schema/commit/6f8009937b70efa3ae35d628e4c8769ffa488b06))

# [1.2.0](https://github.com/stencila/schema/compare/v1.1.5...v1.2.0) (2021-03-31)


### Features

* **Note:** Draft Note specification ([b187519](https://github.com/stencila/schema/commit/b187519d72ed8d77fa85bc3c44abc1b81ad5d93a))

## [1.1.5](https://github.com/stencila/schema/compare/v1.1.4...v1.1.5) (2021-03-31)


### Bug Fixes

* Use allOf where needed ([31be54c](https://github.com/stencila/schema/commit/31be54c7f40f18561c2374a0ad2ea26743146101))

## [1.1.4](https://github.com/stencila/schema/compare/v1.1.3...v1.1.4) (2021-03-22)


### Bug Fixes

* Do not use order validation for array properties ([c77f588](https://github.com/stencila/schema/commit/c77f588d16029e15c3662edbf5791ea988d0b821))
* Remove redudant anyOf and allOf in property schemas ([1957053](https://github.com/stencila/schema/commit/19570534ab89fad6e947fead9d46850610ea7641))
* **CreativeWork:** Use anyOf for maintainer ([1d35af9](https://github.com/stencila/schema/commit/1d35af9c6bc74acbe9a800d8648d3b3811989b41)), closes [/json-schema.org/understanding-json-schema/reference/array.html#id6](https://github.com//json-schema.org/understanding-json-schema/reference/array.html/issues/id6)
* **CreativeWork:** Use Date for date* properties ([264dc95](https://github.com/stencila/schema/commit/264dc95abea295ba3d4c902a50c4a7e5e48c8e08))
* **Periodical & SoftwareSession:** Use Date ([94cc6ac](https://github.com/stencila/schema/commit/94cc6acb8184febe05cfcdb962625a8faaa7475c))

## [1.1.3](https://github.com/stencila/schema/compare/v1.1.2...v1.1.3) (2021-03-20)


### Bug Fixes

* **Cite:** Rename to citationPrefix and citationSuffix ([379dffc](https://github.com/stencila/schema/commit/379dffcbfa62e7dff0987bd19a347a816cad1faa))

## [1.1.2](https://github.com/stencila/schema/compare/v1.1.1...v1.1.2) (2021-03-18)


### Bug Fixes

* **Cite:** Use PascalCase for enumeration variants ([fa9a413](https://github.com/stencila/schema/commit/fa9a413bd1168d2315d6f62869cdde719cfa3bdb))

## [1.1.1](https://github.com/stencila/schema/compare/v1.1.0...v1.1.1) (2021-03-18)


### Bug Fixes

* **Cite:** Alter citation modes; add docs ([21acf0c](https://github.com/stencila/schema/commit/21acf0c64caa83588cbf5c2325b717475d23e813))

# [1.1.0](https://github.com/stencila/schema/compare/v1.0.0...v1.1.0) (2021-03-15)


### Features

* **MathBlock:** Add label property ([d1b850d](https://github.com/stencila/schema/commit/d1b850d5ce6ecbf7fa50a3d53da77ea5833bf0cd)), closes [#246](https://github.com/stencila/schema/issues/246)

# [1.0.0](https://github.com/stencila/schema/compare/v0.47.2...v1.0.0) (2021-01-23)


### Bug Fixes

* **Helpers:** Account for change in directory ([4b0e079](https://github.com/stencila/schema/commit/4b0e0797e7291de5bc6b6b07dff8de90a1497bfc))
* **JSON Schemas:** Fix the base URL for "types schemas" ([10e9b35](https://github.com/stencila/schema/commit/10e9b3500f58cc5456bea2b274ddee18d1daf811)), closes [#238](https://github.com/stencila/schema/issues/238)
* **Package:** Export JsonSchema type ([e328278](https://github.com/stencila/schema/commit/e32827832757c7a8b92d09ffcf728bc85d8054e3)), closes [#240](https://github.com/stencila/schema/issues/240)
* **R:** Update NAMESPACE file ([aec2b25](https://github.com/stencila/schema/commit/aec2b254c236e5355a8aebda2bec948b4858e384))


### Code Refactoring

* **Parser keyword:** Rename `codec` keyword to `parser` ([de26e9f](https://github.com/stencila/schema/commit/de26e9f52dc380155e42211c80c029f1369704d2)), closes [#241](https://github.com/stencila/schema/issues/241)


### BREAKING CHANGES

* **Parser keyword:** Renaming of `codec` keyword to `parser` will break existing validation / coercion.

## [0.47.2](https://github.com/stencila/schema/compare/v0.47.1...v0.47.2) (2020-11-19)


### Bug Fixes

* **Language bindings:** Update type bindings ([955fd3b](https://github.com/stencila/schema/commit/955fd3b798ee272360557a77f5c0c3b8a1c61383))

## [0.47.1](https://github.com/stencila/schema/compare/v0.47.0...v0.47.1) (2020-11-16)


### Bug Fixes

* **CI:** Fix config for docs and trigger release ([d52239b](https://github.com/stencila/schema/commit/d52239bb8852139b1f324b9535542bf177f1a3ab))

# [0.47.0](https://github.com/stencila/schema/compare/v0.46.5...v0.47.0) (2020-11-16)


### Bug Fixes

* **Review, Comment:** Move comments to CreativeWork ([b9bad70](https://github.com/stencila/schema/commit/b9bad704385285216393e2878decdfaeec70e302))


### Features

* **Comment:** Add comment aspect ([3c06245](https://github.com/stencila/schema/commit/3c06245eb9ae390650375409de4babf6b833cf6e)), closes [/github.com/stencila/schema/pull/228#discussion_r522498602](https://github.com//github.com/stencila/schema/pull/228/issues/discussion_r522498602)
* **Comment:** Add comment type ([89e93a3](https://github.com/stencila/schema/commit/89e93a356957ab6146d9d879293231abd49da181)), closes [#227](https://github.com/stencila/schema/issues/227)
* **Review:** Add review type ([0779830](https://github.com/stencila/schema/commit/077983073b30f92769a0793063d9a56cb0dd5720)), closes [#227](https://github.com/stencila/schema/issues/227)

## [0.46.5](https://github.com/stencila/schema/compare/v0.46.4...v0.46.5) (2020-10-04)


### Bug Fixes

* **CreativeWork:** Move `maintainer` property from `SoftwareSourceCode` to `CreativeWork`. ([0b10689](https://github.com/stencila/schema/commit/0b10689ba7ab5f40d5f5074e77dd627bd3f0209f))

## [0.46.4](https://github.com/stencila/schema/compare/v0.46.3...v0.46.4) (2020-10-01)


### Bug Fixes

* **R:** Marks property values as scalars where possible ([7b1221e](https://github.com/stencila/schema/commit/7b1221e3536fc3e143d62b39edb99cabbe9e1fa0))

## [0.46.3](https://github.com/stencila/schema/compare/v0.46.2...v0.46.3) (2020-09-29)


### Bug Fixes

* **R typing:** Allow integer values for numeric properties ([d525b06](https://github.com/stencila/schema/commit/d525b069e0c12ee540e7117a46ab697484116cce))

## [0.46.2](https://github.com/stencila/schema/compare/v0.46.1...v0.46.2) (2020-09-25)


### Bug Fixes

* **Build:** Re-run CI to fix missing v0.46.1 NPM release ([4c847f0](https://github.com/stencila/schema/commit/4c847f03bff6d52636b3f8a58da4bb44f0d69cc1)), closes [#220](https://github.com/stencila/schema/issues/220)

## [0.46.1](https://github.com/stencila/schema/compare/v0.46.0...v0.46.1) (2020-09-25)


### Bug Fixes

* **CodeError:** Match required key to property name ([d5fb248](https://github.com/stencila/schema/commit/d5fb24816b62ed999b5920ec7371333bf8e87bf8)), closes [/travis-ci.org/github/stencila/thema/builds/730301506#L1031](https://github.com//travis-ci.org/github/stencila/thema/builds/730301506/issues/L1031)

# [0.46.0](https://github.com/stencila/schema/compare/v0.45.1...v0.46.0) (2020-09-20)


### Bug Fixes

* **Organization:** Singular property name; put in alphabetical order. ([27ff502](https://github.com/stencila/schema/commit/27ff5029e34dea8d877c9577b5aed4e128f0f5c6))


### Features

* **Organization:** Add members field ([f5883dc](https://github.com/stencila/schema/commit/f5883dc19c686a867bf1b7efae621990eddbdb7b))

## [0.45.1](https://github.com/stencila/schema/compare/v0.45.0...v0.45.1) (2020-09-17)


### Bug Fixes

* Reorder property type alternatives for improved coercion ([0b15122](https://github.com/stencila/schema/commit/0b1512248a5a13450e55cb8359e352fce4d2d2d9))

# [0.45.0](https://github.com/stencila/schema/compare/v0.44.2...v0.45.0) (2020-09-08)


### Bug Fixes

* **CreativeWork:** Allow about to be an array of Things ([665842a](https://github.com/stencila/schema/commit/665842a66eeaa61dcbb49955c092832daf534554))


### Features

* **NontextualAnnotation:** Adds node type for text that has a non-textual annotation ([9b593eb](https://github.com/stencila/schema/commit/9b593eb31aeabe397734d08c802591bc13322380)), closes [#211](https://github.com/stencila/schema/issues/211)
* Add extends Thing in DefinedTerm schema ([775d0c4](https://github.com/stencila/schema/commit/775d0c4910c1c92bd206c9d3e952e95b5cd46282))
* Add subjects schema ([de4871e](https://github.com/stencila/schema/commit/de4871e0373254fba44ace2dd5744d618562a533))
* Add type about and genre ([692a9e7](https://github.com/stencila/schema/commit/692a9e7e7c5a94001cfb3acdd7ac8d52eef397d5))

## [0.44.2](https://github.com/stencila/schema/compare/v0.44.1...v0.44.2) (2020-09-01)


### Bug Fixes

* **Build:** Specify directory for type declarations ([9637465](https://github.com/stencila/schema/commit/96374656686e8ca78ac4c1eab0ab4442cd0a365c))

## [0.44.1](https://github.com/stencila/schema/compare/v0.44.0...v0.44.1) (2020-09-01)


### Bug Fixes

* **Package:** Fix path to types file ([af39983](https://github.com/stencila/schema/commit/af39983bf008abd545ab98f5009e9af30af59d1f))

# [0.44.0](https://github.com/stencila/schema/compare/v0.43.3...v0.44.0) (2020-08-31)


### Features

* **CodeChunk:** Add support for caption & label fields ([3d78d9d](https://github.com/stencila/schema/commit/3d78d9d9740ae1a4a346d474a7144eb524c11a29))

## [0.43.3](https://github.com/stencila/schema/compare/v0.43.2...v0.43.3) (2020-07-08)


### Bug Fixes

* **TS:** Fix Typescript definition file path ([07af774](https://github.com/stencila/schema/commit/07af7748244270da72fb4d0d47ed8023d289652e))

## [0.43.2](https://github.com/stencila/schema/compare/v0.43.1...v0.43.2) (2020-07-03)


### Bug Fixes

* **TS:** Fix Schema generation script ([002b320](https://github.com/stencila/schema/commit/002b32096fc8bffe26c0209da75e0e4e677550f6))

## [0.43.1](https://github.com/stencila/schema/compare/v0.43.0...v0.43.1) (2020-05-22)


### Bug Fixes

* **Deps:** npm audit fix ([0af0889](https://github.com/stencila/schema/commit/0af0889cf13faa8d8a6e9290ae138f6e0f2dd67d))

# [0.43.0](https://github.com/stencila/schema/compare/v0.42.1...v0.43.0) (2020-04-21)


### Features

* **Python bindings:** Add node_type utility function ([e4a448a](https://github.com/stencila/schema/commit/e4a448a4ee3d6af848ce8b26a4604550c66bf923))

## [0.42.1](https://github.com/stencila/schema/compare/v0.42.0...v0.42.1) (2020-03-16)


### Bug Fixes

* **BlockContent:** Add Figure and Collection as valid types ([2e0d0bb](https://github.com/stencila/schema/commit/2e0d0bb4f1adcc22c4b8ae83d0e40c9f12baef90))
* **Figure, Table:** Add or update caption and label properties ([34858db](https://github.com/stencila/schema/commit/34858db2e351fa093cbdb49211cd821a21808e4b))

# [0.42.0](https://github.com/stencila/schema/compare/v0.41.2...v0.42.0) (2020-03-12)


### Features

* **Article:** Add pagination, pageStart, pageEnd properties ([276e0b9](https://github.com/stencila/schema/commit/276e0b92fdd5b56376288b4bff8b5289e112aaff))

## [0.41.2](https://github.com/stencila/schema/compare/v0.41.1...v0.41.2) (2020-03-08)


### Bug Fixes

* **Build:** Avoid use of promisify ([7dd52a5](https://github.com/stencila/schema/commit/7dd52a583300ab32db983f669ac637a1c979a7e5)), closes [/travis-ci.org/stencila/executa/jobs/659007810#L684](https://github.com//travis-ci.org/stencila/executa/jobs/659007810/issues/L684)

## [0.41.1](https://github.com/stencila/schema/compare/v0.41.0...v0.41.1) (2020-03-06)


### Bug Fixes

* **Microdata:** Do not use itemscope for primitive nodes ([a598921](https://github.com/stencila/schema/commit/a598921368805e0bea42c6d0af7d13fdc89c9b2a))
* **Organization, Person:** Allow PostalAddress for address property ([9a01142](https://github.com/stencila/schema/commit/9a011422b96f331eda54149c119c96aea93e2c74))
* **PostalAddress:** Add schema: prefix; add checks for this ([0291760](https://github.com/stencila/schema/commit/029176086380fc174c572deb439739b57f5c0ada))

# [0.41.0](https://github.com/stencila/schema/compare/v0.40.0...v0.41.0) (2020-03-06)


### Features

* **ListItem:** Add item and position properties ([2da1545](https://github.com/stencila/schema/commit/2da15458968b5633d37e609d8ce97ecd7f0be24a)), closes [/github.com/stencila/encoda/blob/9190db9fbc77510c73359b4a53fca9b1977e23a0/src/codecs/html/index.ts#L1606](https://github.com//github.com/stencila/encoda/blob/9190db9fbc77510c73359b4a53fca9b1977e23a0/src/codecs/html/index.ts/issues/L1606)
* **PostalAddress:** Add post address schema type ([8a0de66](https://github.com/stencila/schema/commit/8a0de6645fef7d0c60225034cca0a44f13d2f275)), closes [/github.com/stencila/encoda/issues/458#issuecomment-593746231](https://github.com//github.com/stencila/encoda/issues/458/issues/issuecomment-593746231)

# [0.40.0](https://github.com/stencila/schema/compare/v0.39.0...v0.40.0) (2020-02-26)


### Features

* **Microdata:** Add microdataRoot function ([a9b1989](https://github.com/stencila/schema/commit/a9b1989ac07767dc90b091fa6b95ce92d9ac8a3d)), closes [#175](https://github.com/stencila/schema/issues/175)

# [0.39.0](https://github.com/stencila/schema/compare/v0.38.0...v0.39.0) (2020-02-24)


### Bug Fixes

* **ArrayValidator:** Use more specific name to avoid clash with items ([a27039f](https://github.com/stencila/schema/commit/a27039f4fbf105f6ed29a5be7a639f8b6b12b1d9))
* **Figure:** Allow caption to be a string for compatability with caption on other types ([1380fd2](https://github.com/stencila/schema/commit/1380fd2e1ea11b38ba53777ef88574189865ac75))
* **Function:** Property parameters has local id ([314dce8](https://github.com/stencila/schema/commit/314dce81225dff0a414920f406422a89bd7d10d4))
* **Thing:** IMages property should be an array ([83fe1ba](https://github.com/stencila/schema/commit/83fe1ba6c5c58bc228311574b8bdcca81de4c9a5))


### Features

* **Microdata:** Add higher level HTML Microdata functions ([67b850e](https://github.com/stencila/schema/commit/67b850edd0493f60167e170cfabf8df24617f226))
* **Microdata:** Consider `role` when generating itemprop ([65c3772](https://github.com/stencila/schema/commit/65c37722db3043c9d381cd0588bd147c519c004f))
* **Thing:** Add images property ([45eeba0](https://github.com/stencila/schema/commit/45eeba0bb94cb00c14a89e5d2e0f789d744ed35c))
* **Util:** Add Typescript utility functions for inspecting JSON Schemas at runtime ([d5475f9](https://github.com/stencila/schema/commit/d5475f9407b75b5f9b3f8447e3d8a9fee085f492))

# [0.38.0](https://github.com/stencila/schema/compare/v0.37.3...v0.38.0) (2020-02-22)


### Features

* **Primitive types:** Add schemas for primitive types ([e402847](https://github.com/stencila/schema/commit/e4028479abea3fb86243559b709b53f5fe81f378)), closes [/github.com/stencila/encoda/blob/356b8e08f71880f12236bac7b0bcb2c272f4f60b/src/codecs/html/microdata.ts#L148](https://github.com//github.com/stencila/encoda/blob/356b8e08f71880f12236bac7b0bcb2c272f4f60b/src/codecs/html/microdata.ts/issues/L148)
* **Util:** Add version and URL utility functions ([274dd52](https://github.com/stencila/schema/commit/274dd52e0a7361517e28d17f335c1ed5cab7b6a6))

## [0.37.3](https://github.com/stencila/schema/compare/v0.37.2...v0.37.3) (2020-02-13)


### Bug Fixes

* **Math:** mathLanguage is not a schema.org id ([0e2cc61](https://github.com/stencila/schema/commit/0e2cc61858efdb99470f50dd56030e6e363eb236))

## [0.37.2](https://github.com/stencila/schema/compare/v0.37.1...v0.37.2) (2020-02-09)


### Bug Fixes

* **Package:** Add custom release message to trigger Python release ([efea733](https://github.com/stencila/schema/commit/efea733c9bf7e5707356121397796b9dd5399ec5))

## [0.37.1](https://github.com/stencila/schema/compare/v0.37.0...v0.37.1) (2020-02-07)


### Bug Fixes

* **R:** Fix checking of  property types ([0b19165](https://github.com/stencila/schema/commit/0b191655161628bde17525e88ec12a0b709ab15d))

# [0.37.0](https://github.com/stencila/schema/compare/v0.36.0...v0.37.0) (2020-02-04)


### Features

* **SoftwareSession et al:** Promote to status unstable ([74da849](https://github.com/stencila/schema/commit/74da849e7772649642be465f1b9e4e963e3d321d))

# [0.36.0](https://github.com/stencila/schema/compare/v0.35.0...v0.36.0) (2020-01-31)


### Bug Fixes

* **JSON Schema:** Ensure defintions are inherited ([5de74e1](https://github.com/stencila/schema/commit/5de74e14a181caeabd09825c763981e02fb5aad5))
* **JSON Schema:** Only add definitions if necessary ([dfa59cb](https://github.com/stencila/schema/commit/dfa59cb9572df7ebe55908a9ea4202c3fac04bb2))
* **JSON-LD:** Do not alias [@value](https://github.com/value) to avoid conflict with schema.org/value ([a59ca2e](https://github.com/stencila/schema/commit/a59ca2e580dc7c1d1923afa59bc02892bf3a6ebe))
* **JSON-LD:** Do not filter out value ([10249e3](https://github.com/stencila/schema/commit/10249e389a43b3c128839df0ff7616e45f1aafa8))
* **Periodical:** Rename issn to issns for pluralization consistency ([4eba6ea](https://github.com/stencila/schema/commit/4eba6ea99368266a857dc3077ce95fa03e777182))
* **Thing.identifiers:** Apply anyOf to all items in the array ([3e7e81d](https://github.com/stencila/schema/commit/3e7e81d77a636ed4ee863461e034fb7c998d7ed3))


### Features

* **CreativeWork:** Add dateReceived and dateAccepted properties ([788f0bf](https://github.com/stencila/schema/commit/788f0bf460425f828911efb63b86bb1050d4e7ec))
* **Grant & MonetaryGrant:** Add types and properties for representing funding grants ([1c92adf](https://github.com/stencila/schema/commit/1c92adf6f77329633c6f75831f676dcc0c8cd471))
* **JSON Schema:** Allow for inline $refs ([e426380](https://github.com/stencila/schema/commit/e4263805c3725c12eb8a8532fc57dbc90c8bc864))
* **Organization:** Add logo property ([f03d04c](https://github.com/stencila/schema/commit/f03d04c19f1493b23982cefed5199c47521fc31c))
* **PropertyValue, Thing.identifers:** Add ([00ec60f](https://github.com/stencila/schema/commit/00ec60faa9227c537bf01a7d44464ee427299b9d))

# [0.35.0](https://github.com/stencila/schema/compare/v0.34.0...v0.35.0) (2020-01-21)


### Bug Fixes

* **Article:** Do not require authors and title ([17cbe10](https://github.com/stencila/schema/commit/17cbe10f95f7746497d37de461db9f7cca07a492))
* **CodeError:** Message required; rename kind to errorType ([0ab58c0](https://github.com/stencila/schema/commit/0ab58c06e2ba88dfae7fcd68cb94fcf2df1bb013))
* **CodeError:** Modify prop names; errorType comment ([e53d56b](https://github.com/stencila/schema/commit/e53d56bb5164d985abd43aa3d71930fdeeaded44))
* **Function:** Make name optional ([9237114](https://github.com/stencila/schema/commit/9237114c956c751ac44cb614436e852088d38da0))
* **Heading:** Make depth optional, defaulting to 1 ([97c3b7d](https://github.com/stencila/schema/commit/97c3b7d338b7a08366282a98d41d8f738e4a92ac))
* **TableCell:** Relax content to allow any Node ([f048dbb](https://github.com/stencila/schema/commit/f048dbb6236d027202ac617314c6e57c0ee8d55e))
* **Typescript guards:** Allow isA to take a possibly undefined node ([2e5dc24](https://github.com/stencila/schema/commit/2e5dc24d14ce7d7c42885075fc7ee99b15c00621))
* **Variable import and export:** Avoid use of common keywords ([8812e01](https://github.com/stencila/schema/commit/8812e018dfd0ead25319e5253ec2f40a3bd5f7cd))


### Features

* **Compiled nodes:** Refine types used when compiling a doc ([2da8d60](https://github.com/stencila/schema/commit/2da8d606dc7da4e807ac6f5306dcf2db278d5063))
* **TableCell:** Change content to array of BlockContent ([c71681c](https://github.com/stencila/schema/commit/c71681c349553dff927536ceefb55dea1562f13c)), closes [#136](https://github.com/stencila/schema/issues/136)
* **Typescript factory functions:** Only first required prop is unnamed ([02b3483](https://github.com/stencila/schema/commit/02b34831cb2634fab63a91a8ec86fbd11a3efc78))

# [0.34.0](https://github.com/stencila/schema/compare/v0.33.0...v0.34.0) (2020-01-20)


### Bug Fixes

* **Build:** Remove internal links to experimental schemas ([b85f570](https://github.com/stencila/schema/commit/b85f570773e8f92820ea134dd0b4f2d99c2780ff))
* **R package:** Update NAMESPACE file ([94beb2a](https://github.com/stencila/schema/commit/94beb2a24fe1567b8c445385908edee00d7d3310))


### Features

* Promote several types from experimental ([c5941e5](https://github.com/stencila/schema/commit/c5941e55bd15c197b1dd752014e6fff2e35895da))
* **Entity, Thing:** Promote to stable ([234e320](https://github.com/stencila/schema/commit/234e32009b8319e668688f5dc4336b05282918f0))
* **Math:** Add Math, MathFragment and MathBlock nodes ([74f4b55](https://github.com/stencila/schema/commit/74f4b55084042eb63dac0827514b6fffbc5d5e94))

# [0.33.0](https://github.com/stencila/schema/compare/v0.32.4...v0.33.0) (2020-01-19)


### Features

* **JSON Schema:** Generate union type for descendant types ([3376d73](https://github.com/stencila/schema/commit/3376d7351f296f7ed1a4d1bfe0562bec247c6a7d))
* **Type Guards:** Add isInstanceOf guard for matching descendant types ([9985936](https://github.com/stencila/schema/commit/9985936cdd588c30c4ae856f24b019ba6262db56)), closes [#135](https://github.com/stencila/schema/issues/135)

## [0.32.4](https://github.com/stencila/schema/compare/v0.32.3...v0.32.4) (2020-01-17)


### Bug Fixes

* **JSON Schema:** Use versioned URL for $id ([9e9ac85](https://github.com/stencila/schema/commit/9e9ac85ff81e9e148a10e82808a491b3b0742705))

## [0.32.3](https://github.com/stencila/schema/compare/v0.32.2...v0.32.3) (2020-01-17)


### Bug Fixes

* **Docs:** Substantially refactors and fixes docs generation ([b6c1775](https://github.com/stencila/schema/commit/b6c1775005253fbc0ce26aabd6c95ad28cd41a62))

## [0.32.2](https://github.com/stencila/schema/compare/v0.32.1...v0.32.2) (2020-01-17)


### Bug Fixes

* **JSON-LD:** Use versioned URL for context ([8b0e153](https://github.com/stencila/schema/commit/8b0e15317b362629354d26434eb8c10bf5ccfdc4))

## [0.32.1](https://github.com/stencila/schema/compare/v0.32.0...v0.32.1) (2020-01-16)


### Bug Fixes

* **deps:** npm audit fix ([25a6a6a](https://github.com/stencila/schema/commit/25a6a6a859f72c4ee796b97cb2808a9694917653))
* **R bindings:** Improve type specs and checking ([1ef3c27](https://github.com/stencila/schema/commit/1ef3c27dce6b3fa032995a9d6aeb59af7f3826d1))

# [0.32.0](https://github.com/stencila/schema/compare/v0.31.1...v0.32.0) (2019-12-10)


### Features

* Fixes to setup.py ([ee1a6a7](https://github.com/stencila/schema/commit/ee1a6a7))

## [0.31.1](https://github.com/stencila/schema/compare/v0.31.0...v0.31.1) (2019-11-28)


### Bug Fixes

* **TS:** Fix error re. conflicting type definition ([3f227f1](https://github.com/stencila/schema/commit/3f227f1))

# [0.31.0](https://github.com/stencila/schema/compare/v0.30.5...v0.31.0) (2019-11-06)


### Bug Fixes

* **Dependencies:** Move logga to production deps ([c444747](https://github.com/stencila/schema/commit/c444747))


### Features

* **SoftwareSession:** Add properties and rename others ([b7f30de](https://github.com/stencila/schema/commit/b7f30de))

## [0.30.5](https://github.com/stencila/schema/compare/v0.30.4...v0.30.5) (2019-10-28)


### Bug Fixes

* **SoftwareSession:** make environment optional ([85e05af](https://github.com/stencila/schema/commit/85e05af))

## [0.30.4](https://github.com/stencila/schema/compare/v0.30.3...v0.30.4) (2019-10-25)


### Bug Fixes

* **R:** Update NAMESPACE file ([a717d0a](https://github.com/stencila/schema/commit/a717d0a))
* **SoftwareSession:** Refactoring of SoftwareSession and associated types ([eb950f2](https://github.com/stencila/schema/commit/eb950f2))

## [0.30.3](https://github.com/stencila/schema/compare/v0.30.2...v0.30.3) (2019-10-22)


### Bug Fixes

* JS and Py interpreters no longer return arrays in JSON RCP response ([63cfda7](https://github.com/stencila/schema/commit/63cfda7))

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

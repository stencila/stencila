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

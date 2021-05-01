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

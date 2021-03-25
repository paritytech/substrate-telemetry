# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3] - 2021-03-25

### Added

- Cap third party networks to at most 500 connected nodes. Polkadot, Kusama, Westend and Rococo are not subject to this limit. [#314](https://github.com/paritytech/substrate-telemetry/pull/314)
- Fix clippy warnings [#314](https://github.com/paritytech/substrate-telemetry/pull/314)
- Add `--log` switch to configure log levels [#314](https://github.com/paritytech/substrate-telemetry/pull/314)

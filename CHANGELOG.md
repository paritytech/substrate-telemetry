# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3] - 2021-03-25

### Added

- Add `--denylist network1 network2` switch to deny nodes on the given networks to connect [#315](https://github.com/paritytech/substrate-telemetry/pull/314)
- Add `--log` switch to configure log levels [#314](https://github.com/paritytech/substrate-telemetry/pull/314)

### Fixed

- Fix clippy warnings [#314](https://github.com/paritytech/substrate-telemetry/pull/314)

### Changed

- Docker image use alpine (for now) [#326](https://github.com/paritytech/substrate-telemetry/pull/326)
- Mute denied nodes [#322](https://github.com/paritytech/substrate-telemetry/pull/322)
- Build actix-web without compression support [#319](https://github.com/paritytech/substrate-telemetry/pull/319)
- Update to actix v4 beta [#316](https://github.com/paritytech/substrate-telemetry/pull/317)
- Cap third party networks to at most 500 connected nodes. Polkadot, Kusama, Westend and Rococo are not subject to this limit. [#314](https://github.com/paritytech/substrate-telemetry/pull/314)

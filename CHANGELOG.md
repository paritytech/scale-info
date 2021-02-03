# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added

### Changed

### Fixed
- Derive: use known crate name aliases [(#61)](https://github.com/paritytech/scale-info/pull/61)

## [0.5.0] - 2021-01-27
### Added
- Add a new TypeDef variant to handle PhantomData - [(#48)](https://github.com/paritytech/scale-info/pull/48)
- TypeInfo for up to 16 tuples, Clone PortableRegistry - [(#50)](https://github.com/paritytech/scale-info/pull/50)
- Enumerate RegistryReadOnly types, Display Path - [(#27)](https://github.com/paritytech/scale-info/pull/27)
- Add missing 256 bits types which are needed by Solang - [(#25)](https://github.com/paritytech/scale-info/pull/25)

### Changed
- Ensure only static lifetimes appear in derived types - [(#39)](https://github.com/paritytech/scale-info/pull/39)
- Remove unused function `MetaType::of()` - [(#49)](https://github.com/paritytech/scale-info/pull/49)
- Use PortableRegistry for encoding and serializing - [(#40)](https://github.com/paritytech/scale-info/pull/40)
- Rename Compact to Portable - [(#41)](https://github.com/paritytech/scale-info/pull/41)
- Parameterize CompactForm String for optional SCALE impl - [(#35)](https://github.com/paritytech/scale-info/pull/35)
- Derive TypeInfo for fields with associated types without bounds - [(#20)](https://github.com/paritytech/scale-info/pull/20)
- Optional serde feature - [(#34)](https://github.com/paritytech/scale-info/pull/34)
- Consolidate common prelude for std and no_std and usage - [(#33)](https://github.com/paritytech/scale-info/pull/33)
- Add informational field type name - [(#30)](https://github.com/paritytech/scale-info/pull/30)
- Unify transparent wrapper types e.g. references - [(#26)](https://github.com/paritytech/scale-info/pull/26)
- Bump `parity-scale-codec` from 1.0 to 2.0 [(#55)](https://github.com/paritytech/scale-info/pull/55)

## Fixed
- Fix type name scrubbing to handle nested tuples - [(#47)](https://github.com/paritytech/scale-info/pull/47)

## [0.4.1] - 2020-10-11
### Fixed
- Add missing `extern crate proc_macro;` [(#22)](https://github.com/paritytech/scale-info/pull/24)

## [0.4.0] - 2020-10-05
### Added
- Add public getters for fields in meta type hierarchy [(#22)](https://github.com/paritytech/scale-info/pull/22)
- Implement SCALE encoding and serde deserialization [(#19)](https://github.com/paritytech/scale-info/pull/19)

## [0.3.0] - 2020-07-03
### Changed
- Remove string table, inline strings [(#17)](https://github.com/paritytech/scale-info/pull/17)

## [0.2.0] - 2020-06-17
### Changed
- Remove Metadata supertrait [(#15)](https://github.com/paritytech/scale-info/pull/15)
- Unflatten JSON for type def field [(#14)](https://github.com/paritytech/scale-info/pull/14)
- Improve intra doc links

## [0.1.0] - 2020-06-12
### Added
- First release

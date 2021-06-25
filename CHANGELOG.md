
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.0-rc.0] - 2021-06-25 // todo: [AJ] update for day of release
### Added

- Handle more SCALE attributes: skip, index [(#44)](https://github.com/paritytech/scale-info/pull/44)
- Implement `TypeInfo` for `BTreeSet` [(#85)](https://github.com/paritytech/scale-info/pull/85)
- Implement `TypeInfo` for `Cow` [(#84)](https://github.com/paritytech/scale-info/pull/84)
- Implement `TypeInfo` for up to 20 element tuples [(#92)](https://github.com/paritytech/scale-info/pull/92)
- Add `StaticTypeInfo` convenience trait [(#91)](https://github.com/paritytech/scale-info/pull/91)
- Capture doc comments, add variant and field builders [(#87)](https://github.com/paritytech/scale-info/pull/87)
- Handle `#[codec(index = …)]` in regular enums [(#80)](https://github.com/paritytech/scale-info/pull/80)
- Add new top-level attribute `scale_info(bounds(T: SomeTrait + OtherTrait))` [(#88)](https://github.com/paritytech/scale-info/pull/88)
- (aj-vecdeque) Implement TypeInfo for VecDeque [(#99)](https://github.com/paritytech/scale-info/pull/99)
- Add BitVec support [(#98)](https://github.com/paritytech/scale-info/pull/98)
- Add `docs` feature [(#101)](https://github.com/paritytech/scale-info/pull/101)

### Changed

- Upgrade proc-macro-crate to v1 [(#77)](https://github.com/paritytech/scale-info/pull/77)
- Use const generics for array TypeInfo impls [(#54)](https://github.com/paritytech/scale-info/pull/54)
- Replace NonZeroU32 type lookup ids with u32 [(#90)](https://github.com/paritytech/scale-info/pull/90)
- Remove HasCompact::Type bounds [(#83)](https://github.com/paritytech/scale-info/pull/83)
- Unify sequence types [(#100)](https://github.com/paritytech/scale-info/pull/100)
- Bump parity-scale-codec to 2.2.0-rc.2 [(#102)](https://github.com/paritytech/scale-info/pull/102)

### Fixed

- Fix serde and decode features without default features [(#74)](https://github.com/paritytech/scale-info/pull/74)
- Remove type parameter defaults [(#71)](https://github.com/paritytech/scale-info/pull/71)
- Fix trait bounds for associated types [(#76)](https://github.com/paritytech/scale-info/pull/76)

## [0.6.0] - 2021-02-05
### Added
- Add a TypeDef to handle Compact types [(#53)](https://github.com/paritytech/scale-info/pull/53)
- Add feature for enabling decoding [(#59)](https://github.com/paritytech/scale-info/pull/59)

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

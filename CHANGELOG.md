
# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.5.0] - 2022-03-29

### Added
- ty: Make type fields public [(#176)](https://github.com/paritytech/scale-info/pull/176)

## [2.4.0] - 2022-03-23

### Added
- portable: Retain the provided type IDs [(#174)](https://github.com/paritytech/scale-info/pull/174)

## [2.3.1] - 2022-12-09

### Fixed
- Change NonZero* TypeInfo implementation to not be recursive [(#171)](https://github.com/paritytech/scale-info/pull/171)

## [2.3.0] - 2022-10-27

Implement the missing pieces for constructing `PortableRegistry` dynamically at runtime. This allows languages where static rust types are not available to use it.

## [2.2.0] - 2022-09-14

The minimum Rust version is bumped to 1.60.0 in this release owing to using weak dependency crate features. Otherwise there are no breaking changes.

### Changed
- Loosen restriction on `TypeDefBitSequence::new()` so that `bitvec` isn't required, and try to avoid pulling in `bitvec` when the `std` feature is enabled [(#168)](https://github.com/paritytech/scale-info/pull/168)

## [2.1.2] - 2022-05-18

### Fixed
- Strip invisible delimiters from type name [(#156)](https://github.com/paritytech/scale-info/pull/156)

## [2.1.1] - 2022-04-11

### Fixed
- Restore leading `::` for crate access [(#152)](https://github.com/paritytech/scale-info/pull/152)

## [2.1.0] - 2022-04-11

### Added
- Add ability to reexport crate [(#145)](https://github.com/paritytech/scale-info/pull/145)

### Fixed
- Allow raw identifiers e.g. `r#mod` [(#149)](https://github.com/paritytech/scale-info/pull/149)

## [2.0.1] - 2022-02-24

### Changed
- Revert bitvec field order to maintain binary compatiblilty

## [2.0.0] - 2022-02-07

### Changed
- Upgraded to parity-scale-codec 3.0
- Upgraded to bitvec 1.0
- Minimum Rust version is 1.56.1 for edition 2021

## [1.0.0] - 2021-09-01
### Changed
- Replace Range variant with built-in composite definitions [(#130)](https://github.com/paritytech/scale-info/pull/130)

## [0.12.0] - 2021-08-25
### Changed
- Add range getters, combine start and end types [(#126)](https://github.com/paritytech/scale-info/pull/126)

## [0.11.0] - 2021-08-25
### Added
- Add type parameter getters [(#122)](https://github.com/paritytech/scale-info/pull/122)
- Add support for Range and RangeInclusive [(#124)](https://github.com/paritytech/scale-info/pull/124)
- Explicit codec indices for `TypeDef` and `TypeDefPrimitive` enums [(#127)](https://github.com/paritytech/scale-info/pull/127)

## [0.10.0] - 2021-07-29
### Added
- Add capture_docs attribute [(#118)](https://github.com/paritytech/scale-info/pull/118)

### Fixed
- Allow codec attributes, in case missing Encode/Decode derives [(#117)](https://github.com/paritytech/scale-info/pull/117)

### Changed
- Erase PhantomData fields [(#111](https://github.com/paritytech/scale-info/pull/111), [#115)](https://github.com/paritytech/scale-info/pull/115)
- Make variant index explicit, remove discriminant [(#112)](https://github.com/paritytech/scale-info/pull/112)
- Include type id in serialized type registry [(#114)](https://github.com/paritytech/scale-info/pull/114)
- Improve docs feature [(#116)](https://github.com/paritytech/scale-info/pull/116)

## [0.9.2] - 2021-07-09
### Added
- Add index getter to Variant [(#110)](https://github.com/paritytech/scale-info/pull/110)

## [0.9.1] - 2021-07-06
### Fixed
- Option constructor macro hygiene [(#108)](https://github.com/paritytech/scale-info/pull/108)

## [0.9.0] - 2021-06-30
### Changed
- Reverted parity-scale-codec prerelease requirement from [0.8.0-rc.1]
- Reexport parity-scale-codec for derive [(#106)](https://github.com/paritytech/scale-info/pull/106)

### Added
- Add `skip_type_params` attribute [(#96)](https://github.com/paritytech/scale-info/pull/96)

## [0.8.0-rc.1] - 2021-06-29
### Changed
- Bump parity-scale-codec to 2.2.0-rc.2 [(#102)](https://github.com/paritytech/scale-info/pull/102)

## [0.7.0] - 2021-06-29
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

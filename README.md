# type-metadata &middot; ![build](https://github.com/paritytech/scale-info/workflows/Rust/badge.svg)

Compactly serialize meta information about types in your crate.

## Design

[Design draft](https://hackmd.io/0wWm0ueBSF26m2pBG5NaeQ?view)

**Note:** The design draft is outdated partially.

## Overview

Types are describes by their identification (ID) and their structure or definition.
The ID stores information about the name of the type, where the type has been defined and generic types communicate their generic arguments.
The definition communicates the underlying serialization and deserialization structure of the type,
possibly revealing internal fields etc.

The former is important to give a mean to differentiate types easily between each other and also provide a useful hint to users.
The definitions provide third party tools, such as a UI client with information about how they are able to decode types language agnostically.

## Internal Overview

Information about types is provided within the so-called type registry (`Registry`).
Type identifiers and associated definitions are registered there and are associated with unique IDs that the outside
can use to refer to them providing a lightweight way to decrease overhead of using type identifiers instead.

For the purpose of communicating type ID and definition there exists the `HasTypeId` and `HasTypeDef` respectively
that is to be implemented by all supported types.

All concrete `TypeId` and `TypeDef` structures have two forms:
One meta form (`MetaType`) that acts as a bridge to other forms and a compact form that is later to be serialized.
The `IntoCompact` trait is implemented by them in order to compact a type ID or definition using an instance of a type registry.

After compactification all type ID and definitions are stored in the type registry.
Note that during serialization the type registry should be serialized during general serialization procedure.

As a minor additional compaction step non-documentation strings are also compacted by the same mechanics.

## Users

Simply build up any graph of data structures and use `MetaType` instances to communicate type information.
Also provide an `IntoCompact` implementation that converts those `MetaType` instances into their compacted forms.
Upon serialization do not forget to also serialize the type registry used for compaction.

## Test

Generally test the crate with `cargo test`.

If you additionally want to test derive utilities, do `cargo test --features derive`.

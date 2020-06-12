# scale-info &middot; [![build][a1]][a2] [![Latest Version][b1]][b2]

[a1]: https://github.com/paritytech/scale-info/workflows/Rust/badge.svg
[a2]: https://github.com/paritytech/scale-info/actions?query=workflow%3ARust+branch%3Amaster
[b1]: https://img.shields.io/crates/v/scale-info.svg
[b2]: https://crates.io/crates/scale-info

A library to describe Rust types, geared towards providing info about the structure of [SCALE
](https://github.com/paritytech/parity-scale-codec) encodable types.

The definitions provide third party tools (e.g. a UI client) with information about how they
 are able to decode types language agnostically.

At its core is the `TypeInfo` trait:

```rust
pub trait TypeInfo {
    fn type_info() -> Type;
}
```

Types implementing this trait build up and return a `Type` struct:

```rust
pub struct Type<F: Form = MetaForm> {
	/// The unique path to the type. Can be empty for built-in types
	path: Path<F>,
	/// The generic type parameters of the type in use. Empty for non generic types
	type_params: Vec<F::TypeId>,
	/// The actual type definition
	type_def: TypeDef<F>,
}
```
Types are defined as one of the following variants:
```rust
pub enum TypeDef<F: Form = MetaForm> {
	/// A composite type (e.g. a struct or a tuple)
	Composite(TypeDefComposite<F>),
	/// A variant type (e.g. an enum)
	Variant(TypeDefVariant<F>),
	/// A sequence type with runtime known length.
	Sequence(TypeDefSequence<F>),
	/// An array type with compile-time known length.
	Array(TypeDefArray<F>),
	/// A tuple type.
	Tuple(TypeDefTuple<F>),
	/// A Rust primitive type.
	Primitive(TypeDefPrimitive),
}
```

## Built-in Type Definitions

The following "built-in" types have predefined `TypeInfo` definitions:

- **Primitives:** `bool`, `char`, `str`, `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64
`, `i128`.

- **Sequence:** Variable size sequence of elements of `T`, where `T` implements `TypeInfo`. e.g. `[T]`, `&[T]`, `&mut
 [T]`, `Vec<T>`

- **Array:** Fixed size `[T: $n]` for any `T` which implements `TypeInfo`, where `$n` is one of the
 predefined sizes.

- **Tuple:** Tuples consisting of up to 10 fields with types implementing `TypeInfo`.

## User-defined Types

There are two kinds of user-defined types: `Composite` and `Variant`.

Both make use of the `Path` and `Field` types in their definition:

#### Fields

A fundamental building block to represent user defined types is the `Field` struct which defines the `Type` of a
field together with its optional name. Builders for the user defined types enforce the invariant that either all
fields have a name (e.g. structs) or all fields are unnamed (e.g. tuples).

#### Path

The path of a type is a unique sequence of identifiers. Rust types typically construct a path from
the namespace and the identifier e.g. `foo::bar::Baz` is converted to the path `["foo", "bar
", "Baz"]`.

### Composite

[Composite data types](https://en.wikipedia.org/wiki/Composite_data_type) are composed of a set of `Fields`.

**Structs** are represented by a set of *named* fields, enforced during construction:

```rust
use scale_info::{build::Fields, Metadata, MetaType, Path, Type, TypeInfo};

struct Foo<T> {
    bar: T,
    data: u64,
}

impl<T> TypeInfo for Foo<T>
where
    T: Metadata + 'static,
{
    fn type_info() -> Type {
        Type::builder()
            .path(Path::new("Foo", module_path!()))
            .type_params(vec![MetaType::new::<T>()])
            .composite(Fields::named()
                .field_of::<T>("bar")
                .field_of::<u64>("data")
            )
    }
}
```

**Tuples** are represented by a set of *unnamed* fields, enforced during construction:

```rust
// todo
```

### Variant

**todo:** about variant types

https://en.wikipedia.org/wiki/Tagged_union

## The Registry

Information about types is provided within the so-called type registry (`Registry`).
Type definitions are registered there and are associated with unique IDs that the outside
can use to refer to them providing a lightweight way to decrease overhead of using type identifiers instead.

All concrete `TypeInfo` structures have two forms:
One meta form (`MetaType`) that acts as a bridge to other forms and a compact form that is later to be serialized.
The `IntoCompact` trait is implemented by them in order to compact a type definition using an instance of a type registry.

After compactification all type definitions are stored in the type registry.
Note that during serialization the type registry should be serialized during general serialization procedure.

As a minor additional compaction step non-documentation strings are also compacted by the same mechanics.

## Serialization JSON

Currently the only supported serialization format is JSON, an example of which can be found

## Resources

- [Original design draft (*outdated*)](https://hackmd.io/0wWm0ueBSF26m2pBG5NaeQ?view)
- **todo:** link to ink` usage example

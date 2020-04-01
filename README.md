# scale-info &middot; ![build](https://github.com/paritytech/scale-info/workflows/Rust/badge.svg)

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

Types implementing this trait build up and return a `Type` enum:

```rust
pub enum Type<F: Form = MetaForm> {
    /// A composite type (e.g. a struct or a tuple)
    Composite(TypeComposite<F>),
    /// A variant type (e.g. an enum)
    Variant(TypeVariant<F>),
    /// A sequence type with runtime known length.
    Sequence(TypeSequence<F>),
    /// An array type with compile-time known length.
    Array(TypeArray<F>),
    /// A tuple type.
    Tuple(TypeTuple<F>),
    /// A Rust primitive type.
    Primitive(TypePrimitive),
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

** todo: about paths **

### Composite

[Composite data types](https://en.wikipedia.org/wiki/Composite_data_type) are composed of a set of `Fields`. 

**Structs** are represented by a set of *named* fields, enforced during construction:

```rust
struct Foo<T> {
    bar: T,
    data: u64,
}

impl<T> TypeInfo for Foo<T>
where
    T: Metadata + 'static,
{
    fn type_info() -> Type {
        TypeComposite::new("Foo", Namespace::from_module_path(module_path!()).unwrap())
            .type_params(tuple_meta_type!(T))
            .fields(Fields::named()
                .field_of::<T>("bar")
                .field_of::<u64>("data")
            )
            .into()
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

**todo:** update this section 

Information about types is provided within the so-called type registry (`Registry`).
Type definitions are registered there and are associated with unique IDs that the outside
can use to refer to them providing a lightweight way to decrease overhead of using type identifiers instead.

All concrete `TypeInfo` structures have two forms:
One meta form (`MetaType`) that acts as a bridge to other forms and a compact form that is later to be serialized.
The `IntoCompact` trait is implemented by them in order to compact a type definition using an instance of a type registry.

After compactification all type definitions are stored in the type registry.
Note that during serialization the type registry should be serialized during general serialization procedure.

As a minor additional compaction step non-documentation strings are also compacted by the same mechanics.

## Serialized JSON

**todo: example of serialized JSON**

## Resources

- [Original design draft (*outdated*)](https://hackmd.io/0wWm0ueBSF26m2pBG5NaeQ?view)
- **todo:** link to ink` usage example

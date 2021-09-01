// Copyright 2019-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::prelude::{
    vec,
    vec::Vec,
};

use crate::{
    build::TypeBuilder,
    form::{
        Form,
        MetaForm,
        PortableForm,
    },
    IntoPortable,
    MetaType,
    Registry,
    TypeInfo,
};
use derive_more::From;
use scale::Encode;
#[cfg(feature = "serde")]
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

mod composite;
mod fields;
mod path;
mod variant;

pub use self::{
    composite::*,
    fields::*,
    path::*,
    variant::*,
};

/// A [`Type`] definition with optional metadata.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode)]
pub struct Type<T: Form = MetaForm> {
    /// The unique path to the type. Can be empty for built-in types
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Path::is_empty", default)
    )]
    path: Path<T>,
    /// The generic type parameters of the type in use. Empty for non generic types
    #[cfg_attr(
        feature = "serde",
        serde(rename = "params", skip_serializing_if = "Vec::is_empty", default)
    )]
    type_params: Vec<TypeParameter<T>>,
    /// The actual type definition
    #[cfg_attr(feature = "serde", serde(rename = "def"))]
    type_def: TypeDef<T>,
    /// Documentation
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    docs: Vec<T::String>,
}

impl IntoPortable for Type {
    type Output = Type<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        Type {
            path: self.path.into_portable(registry),
            type_params: registry.map_into_portable(self.type_params),
            type_def: self.type_def.into_portable(registry),
            docs: registry.map_into_portable(self.docs),
        }
    }
}

macro_rules! impl_from_type_def_for_type {
    ( $( $t:ty  ), + $(,)?) => { $(
        impl From<$t> for Type {
            fn from(item: $t) -> Self {
                Self::new(Path::voldemort(), Vec::new(), item, Vec::new())
            }
        }
    )* }
}

impl_from_type_def_for_type!(
    TypeDefPrimitive,
    TypeDefArray,
    TypeDefSequence,
    TypeDefTuple,
    TypeDefCompact,
    TypeDefBitSequence,
);

impl Type {
    /// Create a [`TypeBuilder`](`crate::build::TypeBuilder`) the public API for constructing a [`Type`]
    pub fn builder() -> TypeBuilder {
        TypeBuilder::default()
    }

    pub(crate) fn new<I, D>(
        path: Path,
        type_params: I,
        type_def: D,
        docs: Vec<&'static str>,
    ) -> Self
    where
        I: IntoIterator<Item = TypeParameter>,
        D: Into<TypeDef>,
    {
        Self {
            path,
            type_params: type_params.into_iter().collect(),
            type_def: type_def.into(),
            docs,
        }
    }
}

impl<T> Type<T>
where
    T: Form,
{
    /// Returns the path of the type
    pub fn path(&self) -> &Path<T> {
        &self.path
    }

    /// Returns the generic type parameters of the type
    pub fn type_params(&self) -> &[TypeParameter<T>] {
        &self.type_params
    }

    /// Returns the definition of the type
    pub fn type_def(&self) -> &TypeDef<T> {
        &self.type_def
    }

    /// Returns the documentation of the type
    pub fn docs(&self) -> &[T::String] {
        &self.docs
    }
}

/// A generic type parameter.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode)]
pub struct TypeParameter<T: Form = MetaForm> {
    /// The name of the generic type parameter e.g. "T".
    name: T::String,
    /// The concrete type for the type parameter.
    ///
    /// `None` if the type parameter is skipped.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    ty: Option<T::Type>,
}

impl IntoPortable for TypeParameter {
    type Output = TypeParameter<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeParameter {
            name: self.name.into_portable(registry),
            ty: self.ty.map(|ty| registry.register_type(&ty)),
        }
    }
}

impl<T> TypeParameter<T>
where
    T: Form,
{
    /// Create a new [`TypeParameter`].
    pub fn new(name: T::String, ty: Option<T::Type>) -> Self {
        Self { name, ty }
    }

    /// Get the type of the parameter.
    ///
    /// `None` if the parameter is skipped.
    pub fn ty(&self) -> Option<&T::Type> {
        self.ty.as_ref()
    }

    /// Get the name of the parameter.
    pub fn name(&self) -> &T::String {
        &self.name
    }
}

/// The possible types a SCALE encodable Rust value could have.
///
/// # Note
///
/// In order to preserve backwards compatibility, variant indices are explicitly specified instead
/// of depending on the default implicit ordering.
///
/// When adding a new variant, it must be added at the end with an incremented index.
///
/// When removing an existing variant, the rest of variant indices remain the same, and the removed
/// index should not be reused.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode)]
pub enum TypeDef<T: Form = MetaForm> {
    /// A composite type (e.g. a struct or a tuple)
    #[codec(index = 0)]
    Composite(TypeDefComposite<T>),
    /// A variant type (e.g. an enum)
    #[codec(index = 1)]
    Variant(TypeDefVariant<T>),
    /// A sequence type with runtime known length.
    #[codec(index = 2)]
    Sequence(TypeDefSequence<T>),
    /// An array type with compile-time known length.
    #[codec(index = 3)]
    Array(TypeDefArray<T>),
    /// A tuple type.
    #[codec(index = 4)]
    Tuple(TypeDefTuple<T>),
    /// A Rust primitive type.
    #[codec(index = 5)]
    Primitive(TypeDefPrimitive),
    /// A type using the [`Compact`] encoding
    #[codec(index = 6)]
    Compact(TypeDefCompact<T>),
    /// A type representing a sequence of bits.
    #[codec(index = 7)]
    BitSequence(TypeDefBitSequence<T>),
}

impl IntoPortable for TypeDef {
    type Output = TypeDef<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        match self {
            TypeDef::Composite(composite) => composite.into_portable(registry).into(),
            TypeDef::Variant(variant) => variant.into_portable(registry).into(),
            TypeDef::Sequence(sequence) => sequence.into_portable(registry).into(),
            TypeDef::Array(array) => array.into_portable(registry).into(),
            TypeDef::Tuple(tuple) => tuple.into_portable(registry).into(),
            TypeDef::Primitive(primitive) => primitive.into(),
            TypeDef::Compact(compact) => compact.into_portable(registry).into(),
            TypeDef::BitSequence(bitseq) => bitseq.into_portable(registry).into(),
        }
    }
}

/// A primitive Rust type.
///
/// # Note
///
/// Explicit codec indices specified to ensure backwards compatibility. See [`TypeDef`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub enum TypeDefPrimitive {
    /// `bool` type
    #[codec(index = 0)]
    Bool,
    /// `char` type
    #[codec(index = 1)]
    Char,
    /// `str` type
    #[codec(index = 2)]
    Str,
    /// `u8`
    #[codec(index = 3)]
    U8,
    /// `u16`
    #[codec(index = 4)]
    U16,
    /// `u32`
    #[codec(index = 5)]
    U32,
    /// `u64`
    #[codec(index = 6)]
    U64,
    /// `u128`
    #[codec(index = 7)]
    U128,
    /// 256 bits unsigned int (no rust equivalent)
    #[codec(index = 8)]
    U256,
    /// `i8`
    #[codec(index = 9)]
    I8,
    /// `i16`
    #[codec(index = 10)]
    I16,
    /// `i32`
    #[codec(index = 11)]
    I32,
    /// `i64`
    #[codec(index = 12)]
    I64,
    /// `i128`
    #[codec(index = 13)]
    I128,
    /// 256 bits signed int (no rust equivalent)
    #[codec(index = 14)]
    I256,
}

/// An array type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefArray<T: Form = MetaForm> {
    /// The length of the array type.
    len: u32,
    /// The element type of the array type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    type_param: T::Type,
}

impl IntoPortable for TypeDefArray {
    type Output = TypeDefArray<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefArray {
            len: self.len,
            type_param: registry.register_type(&self.type_param),
        }
    }
}

impl TypeDefArray {
    /// Creates a new array type.
    pub fn new(len: u32, type_param: MetaType) -> Self {
        Self { len, type_param }
    }
}

#[allow(clippy::len_without_is_empty)]
impl<T> TypeDefArray<T>
where
    T: Form,
{
    /// Returns the length of the array type.
    pub fn len(&self) -> u32 {
        self.len
    }

    /// Returns the element type of the array type.
    pub fn type_param(&self) -> &T::Type {
        &self.type_param
    }
}

/// A type to refer to tuple types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefTuple<T: Form = MetaForm> {
    /// The types of the tuple fields.
    fields: Vec<T::Type>,
}

impl IntoPortable for TypeDefTuple {
    type Output = TypeDefTuple<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefTuple {
            fields: registry.register_types(self.fields),
        }
    }
}

impl TypeDefTuple {
    /// Creates a new tuple type definition from the given types.
    pub fn new<T>(type_params: T) -> Self
    where
        T: IntoIterator<Item = MetaType>,
    {
        Self {
            fields: type_params
                .into_iter()
                .filter(|ty| !ty.is_phantom())
                .collect(),
        }
    }

    /// Creates a new unit tuple to represent the unit type, `()`.
    pub fn unit() -> Self {
        Self::new(vec![])
    }
}

impl<T> TypeDefTuple<T>
where
    T: Form,
{
    /// Returns the types of the tuple fields.
    pub fn fields(&self) -> &[T::Type] {
        &self.fields
    }
}

/// A type to refer to a sequence of elements of the same type.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefSequence<T: Form = MetaForm> {
    /// The element type of the sequence type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    type_param: T::Type,
}

impl IntoPortable for TypeDefSequence {
    type Output = TypeDefSequence<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefSequence {
            type_param: registry.register_type(&self.type_param),
        }
    }
}

impl TypeDefSequence {
    /// Creates a new sequence type.
    ///
    /// Use this constructor if you want to instantiate from a given meta type.
    pub fn new(type_param: MetaType) -> Self {
        Self { type_param }
    }

    /// Creates a new sequence type.
    ///
    /// Use this constructor if you want to instantiate from a given
    /// compile-time type.
    pub fn of<T>() -> Self
    where
        T: TypeInfo + 'static,
    {
        Self::new(MetaType::new::<T>())
    }
}

impl<T> TypeDefSequence<T>
where
    T: Form,
{
    /// Returns the element type of the sequence type.
    pub fn type_param(&self) -> &T::Type {
        &self.type_param
    }
}

/// A type wrapped in [`Compact`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefCompact<T: Form = MetaForm> {
    /// The type wrapped in [`Compact`], i.e. the `T` in `Compact<T>`.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    type_param: T::Type,
}

impl IntoPortable for TypeDefCompact {
    type Output = TypeDefCompact<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefCompact {
            type_param: registry.register_type(&self.type_param),
        }
    }
}

impl TypeDefCompact {
    /// Creates a new type wrapped in [`Compact`].
    pub fn new(type_param: MetaType) -> Self {
        Self { type_param }
    }
}
impl<T> TypeDefCompact<T>
where
    T: Form,
{
    /// Returns the [`Compact`] wrapped type, i.e. the `T` in `Compact<T>`.
    pub fn type_param(&self) -> &T::Type {
        &self.type_param
    }
}

/// Type describing a [`bitvec::vec::BitVec`].
///
/// # Note
///
/// This can only be constructed for `TypeInfo` in the `MetaForm` with the `bit-vec` feature
/// enabled, but can be decoded or deserialized into the `PortableForm` without this feature.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Debug)]
pub struct TypeDefBitSequence<T: Form = MetaForm> {
    /// The type implementing [`bitvec::store::BitStore`].
    bit_store_type: T::Type,
    /// The type implementing [`bitvec::order::BitOrder`].
    bit_order_type: T::Type,
}

impl IntoPortable for TypeDefBitSequence {
    type Output = TypeDefBitSequence<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefBitSequence {
            bit_store_type: registry.register_type(&self.bit_store_type),
            bit_order_type: registry.register_type(&self.bit_order_type),
        }
    }
}

impl<T> TypeDefBitSequence<T>
where
    T: Form,
{
    /// Returns the type of the bit ordering of the [`::bitvec::vec::BitVec`].
    pub fn bit_order_type(&self) -> &T::Type {
        &self.bit_order_type
    }

    /// Returns underlying type used to store the [`::bitvec::vec::BitVec`].
    pub fn bit_store_type(&self) -> &T::Type {
        &self.bit_store_type
    }
}

#[cfg(feature = "bit-vec")]
impl TypeDefBitSequence {
    /// Creates a new [`TypeDefBitSequence`] for the supplied bit order and bit store types.
    pub fn new<O, T>() -> Self
    where
        O: bitvec::order::BitOrder + TypeInfo + 'static,
        T: bitvec::store::BitStore + TypeInfo + 'static,
    {
        Self {
            bit_order_type: MetaType::new::<O>(),
            bit_store_type: MetaType::new::<T>(),
        }
    }
}

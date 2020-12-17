// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
        FrozenForm,
        Form,
        MetaForm,
    },
    IntoFrozen,
    MetaType,
    Registry,
    TypeInfo,
};
use derive_more::From;
use scale::{
    Decode,
    Encode,
};
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode, Decode)]
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
    type_params: Vec<T::Type>,
    /// The actual type definition
    #[cfg_attr(feature = "serde", serde(rename = "def"))]
    type_def: TypeDef<T>,
}

impl IntoFrozen for Type {
    type Output = Type<FrozenForm>;

    fn into_frozen(self, registry: &mut Registry) -> Self::Output {
        Type {
            path: self.path.into_frozen(registry),
            type_params: registry.register_types(self.type_params),
            type_def: self.type_def.into_frozen(registry),
        }
    }
}

impl From<TypeDefPrimitive> for Type {
    fn from(primitive: TypeDefPrimitive) -> Self {
        Self::new(Path::voldemort(), Vec::new(), primitive)
    }
}

impl From<TypeDefArray> for Type {
    fn from(array: TypeDefArray) -> Self {
        Self::new(Path::voldemort(), Vec::new(), array)
    }
}

impl From<TypeDefSequence> for Type {
    fn from(sequence: TypeDefSequence) -> Self {
        Self::new(Path::voldemort(), Vec::new(), sequence)
    }
}

impl From<TypeDefTuple> for Type {
    fn from(tuple: TypeDefTuple) -> Self {
        Self::new(Path::voldemort(), Vec::new(), tuple)
    }
}

impl Type {
    /// Create a [`TypeBuilder`](`crate::build::TypeBuilder`) the public API for constructing a [`Type`]
    pub fn builder() -> TypeBuilder {
        TypeBuilder::default()
    }

    pub(crate) fn new<I, D>(path: Path, type_params: I, type_def: D) -> Self
    where
        I: IntoIterator<Item = MetaType>,
        D: Into<TypeDef>,
    {
        Self {
            path,
            type_params: type_params.into_iter().collect(),
            type_def: type_def.into(),
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
    pub fn type_params(&self) -> &[T::Type] {
        &self.type_params
    }

    /// Returns the definition of the type
    pub fn type_def(&self) -> &TypeDef<T> {
        &self.type_def
    }
}

/// The possible types a SCALE encodable Rust value could have.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Encode, Decode)]
pub enum TypeDef<T: Form = MetaForm> {
    /// A composite type (e.g. a struct or a tuple)
    Composite(TypeDefComposite<T>),
    /// A variant type (e.g. an enum)
    Variant(TypeDefVariant<T>),
    /// A sequence type with runtime known length.
    Sequence(TypeDefSequence<T>),
    /// An array type with compile-time known length.
    Array(TypeDefArray<T>),
    /// A tuple type.
    Tuple(TypeDefTuple<T>),
    /// A Rust primitive type.
    Primitive(TypeDefPrimitive),
}

impl IntoFrozen for TypeDef {
    type Output = TypeDef<FrozenForm>;

    fn into_frozen(self, registry: &mut Registry) -> Self::Output {
        match self {
            TypeDef::Composite(composite) => composite.into_frozen(registry).into(),
            TypeDef::Variant(variant) => variant.into_frozen(registry).into(),
            TypeDef::Sequence(sequence) => sequence.into_frozen(registry).into(),
            TypeDef::Array(array) => array.into_frozen(registry).into(),
            TypeDef::Tuple(tuple) => tuple.into_frozen(registry).into(),
            TypeDef::Primitive(primitive) => primitive.into(),
        }
    }
}

/// A primitive Rust type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum TypeDefPrimitive {
    /// `bool` type
    Bool,
    /// `char` type
    Char,
    /// `str` type
    Str,
    /// `u8`
    U8,
    /// `u16`
    U16,
    /// `u32`
    U32,
    /// `u64`
    U64,
    /// `u128`
    U128,
    /// 256 bits unsigned int (no rust equivalent)
    U256,
    /// `i8`
    I8,
    /// `i16`
    I16,
    /// `i32`
    I32,
    /// `i64`
    I64,
    /// `i128`
    I128,
    /// 256 bits signed int (no rust equivalent)
    I256,
}

/// An array type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
pub struct TypeDefArray<T: Form = MetaForm> {
    /// The length of the array type.
    len: u32,
    /// The element type of the array type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    type_param: T::Type,
}

impl IntoFrozen for TypeDefArray {
    type Output = TypeDefArray<FrozenForm>;

    fn into_frozen(self, registry: &mut Registry) -> Self::Output {
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub struct TypeDefTuple<T: Form = MetaForm> {
    /// The types of the tuple fields.
    fields: Vec<T::Type>,
}

impl IntoFrozen for TypeDefTuple {
    type Output = TypeDefTuple<FrozenForm>;

    fn into_frozen(self, registry: &mut Registry) -> Self::Output {
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
            fields: type_params.into_iter().collect(),
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
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Debug)]
pub struct TypeDefSequence<T: Form = MetaForm> {
    /// The element type of the sequence type.
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    type_param: T::Type,
}

impl IntoFrozen for TypeDefSequence {
    type Output = TypeDefSequence<FrozenForm>;

    fn into_frozen(self, registry: &mut Registry) -> Self::Output {
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

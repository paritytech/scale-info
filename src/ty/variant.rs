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

use crate::prelude::vec::Vec;

use crate::{
    build::FieldsBuilder,
    form::{
        Form,
        MetaForm,
        PortableForm,
    },
    Field,
    IntoPortable,
    Registry,
};
use derive_more::From;
use scale::Encode;
#[cfg(feature = "serde")]
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

/// A Enum type (consisting of variants).
///
/// # Examples
///
/// ## A Rust enum, aka tagged union.
///
/// ```
/// enum MyEnum {
///     RustAllowsForClikeVariants,
///     AndAlsoForTupleStructs(i32, bool),
///     OrStructs {
///         with: i32,
///         named: bool,
///         fields: [u8; 32],
///     },
///     ItIsntPossibleToSetADiscriminantThough,
/// }
/// ```
///
/// ## A C-like enum type.
///
/// ```
/// enum Days {
///     Monday,
///     Tuesday,
///     Wednesday,
///     Thursday = 42, // Allows setting the discriminant explicitly
///     Friday,
///     Saturday,
///     Sunday,
/// }
/// ```
///
/// ## An empty enum (for marker purposes)
///
/// ```
/// enum JustAMarker {}
/// ```
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From, Encode)]
pub struct TypeDefVariant<T: Form = MetaForm> {
    /// The variants of a variant type
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    variants: Vec<Variant<T>>,
}

impl IntoPortable for TypeDefVariant {
    type Output = TypeDefVariant<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        TypeDefVariant {
            variants: registry.map_into_portable(self.variants),
        }
    }
}

impl TypeDefVariant {
    /// Create a new `TypeDefVariant` with the given variants
    pub fn new<I>(variants: I) -> Self
    where
        I: IntoIterator<Item = Variant>,
    {
        Self {
            variants: variants.into_iter().collect(),
        }
    }
}

impl<T> TypeDefVariant<T>
where
    T: Form,
{
    /// Returns the variants of a variant type
    pub fn variants(&self) -> &[Variant<T>] {
        &self.variants
    }
}

/// A struct enum variant with either named (struct) or unnamed (tuple struct)
/// fields.
///
/// # Example
///
/// ```
/// enum Operation {
///     Zero,
/// //  ^^^^ this is a unit struct enum variant
///     Add(i32, i32),
/// //  ^^^^^^^^^^^^^ this is a tuple-struct enum variant
///     Minus { source: i32 }
/// //  ^^^^^^^^^^^^^^^^^^^^^ this is a struct enum variant
/// }
/// ```
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Encode)]
pub struct Variant<T: Form = MetaForm> {
    /// The name of the variant.
    name: T::String,
    /// The fields of the variant.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Vec::is_empty", default)
    )]
    fields: Vec<Field<T>>,
    /// Index of the variant, used in `parity-scale-codec`
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    index: Option<u8>,
    /// The discriminant of the variant.
    ///
    /// # Note
    ///
    /// Even though setting the discriminant is optional
    /// every C-like enum variant has a discriminant specified
    /// upon compile-time.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    discriminant: Option<u64>,
}

impl IntoPortable for Variant {
    type Output = Variant<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        Variant {
            name: self.name.into_portable(registry),
            fields: registry.map_into_portable(self.fields),
            index: self.index,
            discriminant: self.discriminant,
        }
    }
}

impl Variant {
    /// Creates a new variant with the given fields.
    pub fn with_fields<F>(name: &'static str, fields: FieldsBuilder<F>) -> Self {
        Self {
            name,
            fields: fields.finalize(),
            index: None,
            discriminant: None,
        }
    }

    /// Creates a new indexed variant with the given fields.
    pub fn indexed_with_fields<F>(
        name: &'static str,
        index: u8,
        fields: FieldsBuilder<F>,
    ) -> Self {
        Self {
            name,
            fields: fields.finalize(),
            index: Some(index),
            discriminant: None,
        }
    }

    /// Creates a new variant with the given discriminant.
    pub fn with_discriminant(name: &'static str, discriminant: u64) -> Self {
        Self {
            name,
            fields: Vec::new(),
            index: None,
            discriminant: Some(discriminant),
        }
    }
}

impl<T> Variant<T>
where
    T: Form,
{
    /// Returns the name of the variant
    pub fn name(&self) -> &T::String {
        &self.name
    }

    /// Returns the fields of the struct variant.
    pub fn fields(&self) -> &[Field<T>] {
        &self.fields
    }

    /// Returns the discriminant of the variant.
    pub fn discriminant(&self) -> Option<u64> {
        self.discriminant
    }
}

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

use crate::tm_std::*;

use crate::{
    build::{
        FieldBuilder,
        NamedFields,
        UnnamedFields,
    },
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    IntoCompact,
    MetaType,
    Path,
    Registry,
    TypeInfo,
};
use scale::{
    Decode,
    Encode,
};
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

/// A field of a struct like data type.
///
/// Name is optional so it can represent both named and unnamed fields.
///
/// This can be a named field of a struct type or a struct variant.
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, Deserialize, Encode, Decode,
)]
#[serde(bound(
    serialize = "T::Type: Serialize, T::String: Serialize",
    deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned"
))]
#[serde(rename_all = "camelCase")]
pub struct Field<T: Form = MetaForm> {
    /// The name of the field. None for unnamed fields.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    name: Option<T::String>,
    /// The type of the field.
    #[serde(rename = "type")]
    ty: T::Type,
    /// The compile-time known displayed representation of the type of the field. This will be the
    /// actual name of the type or an alias of it.
    ///
    /// Will be `None` if the type has a qualified type path e.g. `<T as Trait>::AssociatedItem`.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    display_name: Option<Path<T>>,
}

impl IntoCompact for Field {
    type Output = Field<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        Field {
            name: self.name.map(|name| name.into_compact(registry)),
            ty: registry.register_type(&self.ty),
            display_name: self
                .display_name
                .map(|display_name| display_name.into_compact(registry)),
        }
    }
}

impl Field {
    /// Creates a new field.
    ///
    /// Use this constructor if you want to instantiate from a given meta type.
    pub fn new(
        name: Option<&'static str>,
        ty: MetaType,
        display_name: Option<Path>,
    ) -> Self {
        Self {
            name,
            ty,
            display_name,
        }
    }

    /// Creates a new named field.
    ///
    /// Use this constructor if you want to instantiate from a given
    /// compile-time type.
    pub fn named_of<T>(name: &'static str) -> FieldBuilder<NamedFields>
    where
        T: TypeInfo + ?Sized + 'static,
    {
        FieldBuilder::<NamedFields>::new(MetaType::new::<T>()).with_name(name)
    }

    /// Creates a new unnamed field.
    ///
    /// Use this constructor if you want to instantiate an unnamed field from a
    /// given compile-time type.
    pub fn unnamed_of<T>() -> FieldBuilder<UnnamedFields>
    where
        T: TypeInfo + ?Sized + 'static,
    {
        FieldBuilder::<UnnamedFields>::new(MetaType::new::<T>())
    }
}

impl<T> Field<T>
where
    T: Form,
{
    /// Returns the name of the field. None for unnamed fields.
    pub fn name(&self) -> Option<&T::String> {
        self.name.as_ref()
    }

    /// Returns the type of the field.
    pub fn ty(&self) -> &T::Type {
        &self.ty
    }
}

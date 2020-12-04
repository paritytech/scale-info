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

use crate::{
    form::{
        CompactForm,
        Form,
        MetaForm,
    },
    IntoCompact,
    MetaType,
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
/// This can be a named field of a struct type or an enum struct variant.
///
/// # Type name
///
/// The `type_name` field contains a string which is the name of the type of the
/// field as it appears in the source code. The exact contents and format of the
/// type name are not specified, but in practice will be the name of any valid
/// type for a field e.g.
///
///   - Concrete types e.g `"u32"`, `"bool"`, `"Foo"` etc.
///   - Type parameters e.g `"T"`, `"U"`
///   - Generic types e.g `"Vec<u32>"`, `"Vec<T>"`
///   - Associated types e.g. `"T::MyType"`, `"<T as MyTrait>::MyType"`
///   - Type aliases e.g. `"MyTypeAlias"`, `"MyTypeAlias<T>"`
///   - Other built in Rust types e.g. arrays, references etc.
///
/// Note that the type name doesn't correspond to the underlying type of the
/// field, unless using a concrete type directly. Any given type may be referred
/// to by multiple field type names, when using generic type parameters and type
/// aliases.
///
/// This is intended for informational and diagnostic purposes only. Although it
/// is possible to infer certain properties e.g. whether a type name is a type alias,
/// there are no guarantees provided, and the type name representation may change.
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
    /// The name of the type of the field as it appears in the source code.
    type_name: T::String,
}

impl IntoCompact for Field {
    type Output = Field<CompactForm>;

    fn into_compact(self, registry: &mut Registry) -> Self::Output {
        Field {
            name: self.name.map(|name| name.into_compact(registry)),
            ty: registry.register_type(&self.ty),
            type_name: self.type_name.into_compact(registry),
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
        type_name: &'static str,
    ) -> Self {
        Self {
            name,
            ty,
            type_name,
        }
    }

    /// Creates a new named field.
    ///
    /// Use this constructor if you want to instantiate from a given
    /// compile-time type.
    pub fn named_of<T>(name: &'static str, type_name: &'static str) -> Field
    where
        T: TypeInfo + ?Sized + 'static,
    {
        Self::new(Some(name), MetaType::new::<T>(), type_name)
    }

    /// Creates a new unnamed field.
    ///
    /// Use this constructor if you want to instantiate an unnamed field from a
    /// given compile-time type.
    pub fn unnamed_of<T>(type_name: &'static str) -> Field
    where
        T: TypeInfo + ?Sized + 'static,
    {
        Self::new(None, MetaType::new::<T>(), type_name)
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

    /// Returns a string which is the name of the type of the field as it
    /// appears in the source code. The exact contents and format of the type
    /// name are not specified, but in practice will be the name of any valid
    /// type for a field. This is intended for informational and diagnostic
    /// purposes only.
    pub fn type_name(&self) -> &T::String {
        &self.type_name
    }
}

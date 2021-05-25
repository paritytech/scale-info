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

//! Builders for defining metadata for variant types (enums), and composite types (structs).
//! They are designed to allow only construction of valid definitions.
//!
//! In most cases we recommend using the `scale-info-derive` crate to auto generate the builder
//! constructions.
//!
//! # Examples
//!
//! ## Generic struct
//! ```
//! # use scale_info::{build::Fields, MetaType, Path, Type, TypeInfo};
//! struct Foo<T> {
//!     bar: T,
//!     data: u64,
//! }
//!
//! impl<T> TypeInfo for Foo<T>
//! where
//!     T: TypeInfo + 'static,
//! {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!             .type_params(vec![MetaType::new::<T>()])
//!             .composite(Fields::named()
//!                 .field_of::<T>("bar", "T", &[])
//!                 .field_of::<u64>("data", "u64", &[])
//!             )
//!     }
//! }
//! ```
//! ## Tuple struct
//! ```
//! # use scale_info::{build::Fields, MetaType, Path, Type, TypeInfo};
//! struct Foo(u32, bool);
//!
//! impl TypeInfo for Foo {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!             .composite(Fields::unnamed()
//!                 .field_of::<u32>("u32", &[])
//!                 .field_of::<bool>("bool", &[])
//!             )
//!     }
//! }
//! ```
//! ## Enum with fields
//! ```
//! # use scale_info::{build::{Fields, Variants}, MetaType, Path, Type, TypeInfo, Variant};
//! enum Foo<T>{
//!     A(T),
//!     B { f: u32 },
//!     C,
//! }
//!
//! impl<T> TypeInfo for Foo<T>
//! where
//!     T: TypeInfo + 'static,
//! {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!                .type_params(vec![MetaType::new::<T>()])
//!             .variant(
//!                 Variants::new()
//!                     .variant(
//!                         Variant::builder("A")
//!                             .fields(Fields::unnamed().field_of::<T>("T", &[]))
//!                     )
//!                     .variant(
//!                         Variant::builder("B")
//!                             .fields(Fields::named().field_of::<u32>("f", "u32", &[]))
//!                     )
//!                     .variant(
//!                         Variant::builder("A")
//!                             .fields(Fields::unit())
//!                     )
//!             )
//!     }
//! }
//! ```
//! ## Enum without fields
//! ```
//! # use scale_info::{build::{Fields, Variants}, MetaType, Path, Type, TypeInfo, Variant};
//! enum Foo {
//!     A,
//!     B,
//!     C = 33,
//! }
//!
//! impl TypeInfo for Foo {
//!     type Identity = Self;
//!
//!     fn type_info() -> Type {
//!         Type::builder()
//!             .path(Path::new("Foo", module_path!()))
//!             .variant(
//!                 Variants::new()
//!                     .variant(Variant::builder("A").index(1))
//!                     .variant(Variant::builder("B").index(2))
//!                     .variant(Variant::builder("C").index(33))
//!             )
//!     }
//! }
//! ```

use crate::prelude::{
    marker::PhantomData,
    vec::Vec,
};

use crate::{
    form::MetaForm,
    Field,
    MetaType,
    Path,
    Type,
    TypeDef,
    TypeDefComposite,
    TypeDefVariant,
    TypeInfo,
    Variant,
};

/// State types for type builders which require a Path
pub mod state {
    /// State where the builder has not assigned a Path to the type
    pub enum PathNotAssigned {}
    /// State where the builder has assigned a Path to the type
    pub enum PathAssigned {}
}

/// Builds a [`Type`](`crate::Type`)
pub struct TypeBuilder<S = state::PathNotAssigned> {
    path: Option<Path>,
    type_params: Vec<MetaType>,
    docs: Vec<&'static str>,
    marker: PhantomData<fn() -> S>,
}

impl<S> Default for TypeBuilder<S> {
    fn default() -> Self {
        TypeBuilder {
            path: Default::default(),
            type_params: Default::default(),
            docs: Default::default(),
            marker: Default::default(),
        }
    }
}

impl TypeBuilder<state::PathNotAssigned> {
    /// Set the Path for the type
    pub fn path(self, path: Path) -> TypeBuilder<state::PathAssigned> {
        TypeBuilder {
            path: Some(path),
            type_params: self.type_params,
            docs: self.docs,
            marker: Default::default(),
        }
    }
}

impl TypeBuilder<state::PathAssigned> {
    fn build<D>(self, type_def: D) -> Type
    where
        D: Into<TypeDef>,
    {
        let path = self.path.expect("Path not assigned");
        Type::new(path, self.type_params, type_def, self.docs)
    }

    /// Construct a "variant" type i.e an `enum`
    pub fn variant(self, builder: Variants) -> Type {
        self.build(builder.finalize())
    }

    /// Construct a "composite" type i.e. a `struct`
    pub fn composite<F>(self, fields: FieldsBuilder<F>) -> Type {
        self.build(TypeDefComposite::new(fields.finalize()))
    }
}

impl<S> TypeBuilder<S> {
    /// Set the type parameters if it's a generic type
    pub fn type_params<I>(mut self, type_params: I) -> Self
    where
        I: IntoIterator<Item = MetaType>,
    {
        self.type_params = type_params.into_iter().collect();
        self
    }

    /// Set the type documentation
    pub fn docs(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }
}

/// A fields builder has no fields (e.g. a unit struct)
pub enum NoFields {}
/// A fields builder only allows named fields (e.g. a struct)
pub enum NamedFields {}
/// A fields builder only allows unnamed fields (e.g. a tuple)
pub enum UnnamedFields {}

/// Provides FieldsBuilder constructors
pub enum Fields {}

impl Fields {
    /// The type construct has no fields
    pub fn unit() -> FieldsBuilder<NoFields> {
        FieldsBuilder::<NoFields>::default()
    }

    /// Fields for a type construct with named fields
    pub fn named() -> FieldsBuilder<NamedFields> {
        FieldsBuilder::default()
    }

    /// Fields for a type construct with unnamed fields
    pub fn unnamed() -> FieldsBuilder<UnnamedFields> {
        FieldsBuilder::default()
    }
}

/// Build a set of either all named (e.g. for a struct) or all unnamed (e.g. for a tuple struct)
pub struct FieldsBuilder<T> {
    fields: Vec<Field>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for FieldsBuilder<T> {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            marker: Default::default(),
        }
    }
}

impl<T> FieldsBuilder<T> {
    /// Complete building and return the set of fields
    pub fn finalize(self) -> Vec<Field<MetaForm>> {
        self.fields
    }
}

impl FieldsBuilder<NamedFields> {
    /// Add a named field with the type of the type parameter `T`
    pub fn field_of<T>(
        mut self,
        name: &'static str,
        type_name: &'static str,
        docs: &[&'static str],
    ) -> Self
    where
        T: TypeInfo + ?Sized + 'static,
    {
        self.fields
            .push(Field::named_of::<T>(name, type_name, docs));
        self
    }

    /// Add a named, [`Compact`] field of type `T`.
    pub fn compact_of<T>(
        mut self,
        name: &'static str,
        type_name: &'static str,
        docs: &[&'static str],
    ) -> Self
    where
        T: scale::HasCompact,
        <T as scale::HasCompact>::Type: TypeInfo + 'static,
    {
        self.fields
            .push(Field::compact_of::<T>(Some(name), type_name, docs));
        self
    }
}

impl FieldsBuilder<UnnamedFields> {
    /// Add an unnamed field with the type of the type parameter `T`
    pub fn field_of<T>(mut self, type_name: &'static str, docs: &[&'static str]) -> Self
    where
        T: TypeInfo + ?Sized + 'static,
    {
        self.fields.push(Field::unnamed_of::<T>(type_name, docs));
        self
    }

    /// Add an unnamed, [`Compact`] field of type `T`.
    pub fn compact_of<T>(mut self, type_name: &'static str, docs: &[&'static str]) -> Self
    where
        T: scale::HasCompact,
        <T as scale::HasCompact>::Type: TypeInfo + 'static,
    {
        self.fields
            .push(Field::compact_of::<T>(None, type_name, docs));
        self
    }
}

/// Builds a definition of a variant type i.e an `enum`
#[derive(Default)]
pub struct Variants {
    variants: Vec<Variant>,
}

impl Variants {
    /// Create a new [`VariantsBuilder`].
    pub fn new() -> Self {
        Variants {
            variants: Vec::new(),
        }
    }

    /// Add a variant with the
    pub fn variant(mut self, builder: VariantBuilder) -> Self {
        self.variants.push(builder.finalize());
        self
    }

    /// Construct a new [`TypeDefVariant`] from the initialized builder variants.
    pub fn finalize(self) -> TypeDefVariant {
        TypeDefVariant::new(self.variants)
    }
}

/// Build a [`Variant`].
pub struct VariantBuilder {
    name: &'static str,
    fields: Vec<Field<MetaForm>>,
    index: Option<u64>,
    docs: Vec<&'static str>,
}

impl VariantBuilder {
    /// Create a new [`VariantBuilder`].
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            fields: Vec::new(),
            index: None,
            docs: Vec::new(),
        }
    }

    /// Initialize the variant's index.
    pub fn index(mut self, index: u64) -> Self {
        self.index = Some(index);
        self
    }

    /// Initialize the variant's fields.
    pub fn fields<F>(mut self, fields_builder: FieldsBuilder<F>) -> Self {
        self.fields = fields_builder.finalize();
        self
    }

    /// Initialize the variant's documentation.
    pub fn docs(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }

    /// Complete building and create final [`Variant`] instance.
    pub fn finalize(self) -> Variant<MetaForm> {
        Variant::new(self.name, self.fields, self.index, self.docs)
    }
}

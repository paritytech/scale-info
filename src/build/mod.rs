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
//! # use scale_info::{build::{Fields, Variants}, MetaType, Path, Type, TypeInfo};
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
//!                     .variant("A", Fields::unnamed().field_of::<T>("T", &[]), &[])
//!                     .variant("B", Fields::named().field_of::<u32>("f", "u32", &[]), &[])
//!                     .variant("C", Fields::unit(), &[]),
//!             )
//!     }
//! }
//! ```
//! ## Enum without fields
//! ```
//! # use scale_info::{build::{Fields, Variants}, MetaType, Path, Type, TypeInfo};
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
//!                 Variants::fieldless()
//!                     .variant("A", 1, &[])
//!                     .variant("B", 2, &[])
//!                     .variant("C", 33, &[])
//!             )
//!     }
//! }
//! ```

mod fields;
mod variant;

pub use self::fields::*;
pub use self::variant::*;

use crate::prelude::{
    marker::PhantomData,
    vec::Vec,
};

use crate::{
    MetaType,
    Path,
    Type,
    TypeDef,
    TypeDefComposite,
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

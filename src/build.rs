// Copyright 2019-2020
//     and Parity Technologies (UK) Ltd.
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

use crate::{
	form::{Form, MetaForm},
	tm_std::*,
	Field, MetaType, Metadata, Path, Type, TypeDef, TypeDefComposite, TypeDefVariant, Variant,
};

/// State types for type builders which require a Path
pub mod state {
	/// State where the builder has not assigned a Path to the type
	pub enum PathNotAssigned {}
	/// State where the builder has assigned a Path to the type
	pub enum PathAssigned {}
}

/// Builds a [`scale_info::Type`]
pub struct TypeBuilder<S = state::PathNotAssigned> {
	path: Option<Path>,
	type_params: Vec<MetaType>,
	marker: PhantomData<fn() -> S>,
}

impl<S> Default for TypeBuilder<S> {
	fn default() -> Self {
		TypeBuilder {
			path: Default::default(),
			type_params: Default::default(),
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
		Type::new(path, self.type_params, type_def)
	}

	/// Construct a "variant" type i.e an `enum`
	pub fn variant<V>(self, builder: VariantsBuilder<V>) -> Type {
		self.build(builder.done())
	}

	/// Construct a "composite" type i.e. a `struct`
	pub fn composite<F>(self, fields: FieldsBuilder<F>) -> Type {
		self.build(TypeDefComposite::new(fields.done()))
	}
}

impl<S> TypeBuilder<S> {
	/// Set the type parameters if it's a generic type
	pub fn type_params<I>(self, type_params: I) -> Self
	where
		I: IntoIterator<Item = MetaType>,
	{
		let mut this = self;
		this.type_params = type_params.into_iter().collect();
		this
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
	fields: Vec<Field<MetaForm>>,
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
	pub fn done(self) -> Vec<Field<MetaForm>> {
		self.fields
	}
}

impl FieldsBuilder<NamedFields> {
	/// Add a named field with the given [`scale_info::MetaType`] instance
	pub fn field(self, name: &'static str, ty: MetaType) -> Self {
		let mut this = self;
		this.fields.push(Field::named(name, ty));
		this
	}

	/// Add a named field with the type of the type parameter `T`
	pub fn field_of<T>(self, name: &'static str) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		let mut this = self;
		this.fields.push(Field::named_of::<T>(name));
		this
	}
}

impl FieldsBuilder<UnnamedFields> {
	/// Add an unnamed field with the given [`scale_info::MetaType`] instance
	pub fn field(self, ty: MetaType) -> Self {
		let mut this = self;
		this.fields.push(Field::unnamed(ty));
		this
	}

	/// Add an unnamed field with the type of the type parameter `T`
	pub fn field_of<T>(self) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		let mut this = self;
		this.fields.push(Field::unnamed_of::<T>());
		this
	}
}

/// Build a type with no variants.
pub enum NoVariants {}
/// Build a type where at least one variant has fields.
pub enum VariantFields {}
/// Build a type where *all* variants have no fields and the discriminant can
/// be directly chosen or accessed
pub enum Fieldless {}

/// Empty enum for VariantsBuilder constructors for the type builder DSL.
pub enum Variants {}

impl Variants {
	/// Build a set of variants, at least one of which will have fields.
	pub fn with_fields() -> VariantsBuilder<VariantFields> {
		VariantsBuilder::new()
	}

	/// Build a set of variants, none of which will have fields, and the discriminant can
	/// be directly chosen or accessed
	pub fn fieldless() -> VariantsBuilder<Fieldless> {
		VariantsBuilder::new()
	}
}

/// Builds a definition of a variant type i.e an `enum`
#[derive(Default)]
pub struct VariantsBuilder<T> {
	variants: Vec<Variant>,
	marker: PhantomData<fn() -> T>,
}

impl VariantsBuilder<VariantFields> {
	/// Add a variant with fields constructed by the supplied [`scale_info::build::FieldsBuilder`]
	pub fn variant<F>(self, name: <MetaForm as Form>::String, fields: FieldsBuilder<F>) -> Self {
		let mut this = self;
		this.variants.push(Variant::with_fields(name, fields));
		this
	}

	/// Add a variant with no fields i.e. a unit variant
	pub fn variant_unit(self, name: <MetaForm as Form>::String) -> Self {
		self.variant::<NoFields>(name, Fields::unit())
	}
}

impl VariantsBuilder<Fieldless> {
	/// Add a fieldless variant, explicitly setting the discriminant
	pub fn variant(self, name: <MetaForm as Form>::String, discriminant: u64) -> Self {
		let mut this = self;
		this.variants.push(Variant::with_discriminant(name, discriminant));
		this
	}
}

impl<T> VariantsBuilder<T> {
	fn new() -> Self {
		VariantsBuilder {
			variants: Vec::new(),
			marker: Default::default(),
		}
	}

	fn done(self) -> TypeDefVariant {
		TypeDefVariant::new(self.variants)
	}
}

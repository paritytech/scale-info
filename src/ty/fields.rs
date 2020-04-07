// Copyright 2019-2020
//     by  Centrality Investments Ltd.
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

use crate::tm_std::*;

use crate::{
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Metadata, Registry,
};
use serde::Serialize;

#[derive(derive_more::From)]
pub enum FieldType<F: Form = MetaForm> {
	/// The type of the field is concrete
	Concrete(F::Type),
	/// The type of the field is specified by a parameter of the parent type
	Parameter(F::String),
	/// The type of the field is a generic type with the given type params
	Generic(GenericFieldType<F>),
}

impl IntoCompact for FieldType {
	type Output = FieldType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			FieldType::Concrete(ref ty) => registry.register_type(ty).into(),
			FieldType::Parameter(ref name) => registry.register_string(name).into(),
			FieldType::Generic(generic) => generic.into_compact(registry).into(),
		}
	}
}

pub struct GenericFieldType<F: Form = MetaForm> {
	ty: F::Type, // this has to be the same for all instances of generic types
	params: Vec<F::Type>,
}

impl IntoCompact for GenericFieldType {
	type Output = GenericFieldType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		GenericFieldType {
			ty: registry.register_type(&self.ty),
			params: registry.register_types(self.params),
		}
	}
}

/// A field of a struct like data type.
///
/// Name is optional so it can represent both named and unnamed fields.
///
/// This can be a named field of a struct type or a struct variant.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub struct Field<F: Form = MetaForm> {
	/// The name of the field. None for unnamed fields.
	#[serde(skip_serializing_if = "Option::is_none")]
	name: Option<F::String>,
	/// The type of the field.
	#[serde(rename = "type")]
	ty: FieldType<F>,
}

impl IntoCompact for Field {
	type Output = Field<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		Field {
			name: self.name.map(|name| registry.register_string(name)),
			ty: self.ty.into_compact(registry),
		}
	}
}

impl Field {
	/// Creates a new field.
	///
	/// Use this constructor if you want to instantiate from a given meta type.
	pub fn new(name: Option<<MetaForm as Form>::String>, ty: FieldType) -> Self {
		Self { name, ty }
	}

	/// Creates a new named field
	pub fn named(name: <MetaForm as Form>::String, ty: k) -> Self {
		Self::new(Some(name), ty)
	}

	/// Creates a new unnamed field.
	///
	/// Use this constructor if you want to instantiate an unnamed field from a
	/// given meta type.
	pub fn unnamed(ty: FieldType) -> Self {
		Self::new(None, ty)
	}
}

/// A fields builder has no fields (e.g. a unit struct)
pub enum NoFields {}
/// A fields builder only allows named fields (e.g. a struct)
pub enum NamedFields {}
/// A fields builder only allows unnamed fields (e.g. a tuple)
pub enum UnnamedFields {}

/// Empty enum for FieldsBuilder constructors
pub enum Fields {}

impl Fields {
	pub fn unit() -> FieldsBuilder<NoFields> {
		FieldsBuilder::default()
	}

	pub fn named() -> FieldsBuilder<NamedFields> {
		FieldsBuilder::default()
	}

	pub fn unnamed() -> FieldsBuilder<UnnamedFields> {
		FieldsBuilder::default()
	}
}

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
	pub fn done(self) -> Vec<Field<MetaForm>> {
		self.fields
	}
}

impl FieldsBuilder<NamedFields> {
	pub fn field(self, name: &'static str, ty: MetaType) -> Self {
		let mut this = self;
		this.fields.push(Field::named(name, ty));
		this
	}

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
	pub fn field(self, ty: MetaType) -> Self {
		let mut this = self;
		this.fields.push(Field::unnamed(ty));
		this
	}

	pub fn field_of<T>(self) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		let mut this = self;
		this.fields.push(Field::unnamed_of::<T>());
		this
	}
}

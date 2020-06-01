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

use super::{IntoCompact, Registry};
use crate::tm_std::*;
use crate::{
	form::{CompactForm, Form, MetaForm},
	Path, Type,
};
use derive_more::From;
use serde::Serialize;

/// Represents a node in a tree of type definitions and concrete instances.
///
/// todo: more
#[derive(PartialEq, Eq, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum NormalizedType<F: Form = MetaForm> {
	/// The definition of the type
	Definition(NormalizedTypeDef<F>),
	/// The type is specified by a parameter of the parent type
	Parameter(NormalizedTypeParameter<F>),
	/// The type of the field is a generic type with the given type params
	Generic(NormalizedGenericType),
}

impl IntoCompact for NormalizedType<MetaForm> {
	type Output = NormalizedType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			NormalizedType::Definition(definition) => definition.into_compact(registry).into(),
			NormalizedType::Parameter(parameter) => parameter.into_compact(registry).into(),
			NormalizedType::Generic(generic) => generic.into_compact(registry).into(),
		}
	}
}

impl IntoCompact for NormalizedType<CompactForm> {
	type Output = NormalizedType<CompactForm>;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self
	}
}

impl<F> NormalizedType<F>
where
	F: Form,
{
	pub fn definition(path: Path<F>, ty: Type<F>) -> Self {
		NormalizedTypeDef::new(path, ty).into()
	}
}

#[derive(PartialEq, Eq, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct NormalizedTypeDef<F: Form = MetaForm> {
	#[serde(skip_serializing_if = "Path::is_empty")]
	path: Path<F>,
	ty: Type<F>,
}

impl IntoCompact for NormalizedTypeDef<MetaForm> {
	type Output = NormalizedTypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		NormalizedTypeDef {
			path: self.path.into_compact(registry),
			ty: self.ty.into_compact(registry),
		}
	}
}

impl<F> NormalizedTypeDef<F>
where
	F: Form,
{
	pub fn new(path: Path<F>, ty: Type<F>) -> Self {
		Self { path, ty }
	}
}

/// A generic parameter of a parameterized MetaType.
///
/// e.g. the `T` in `Option<T>`
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct NormalizedTypeParameter<F: Form = MetaForm> {
	name: F::String,
	parent: <CompactForm as Form>::Type,
}

impl IntoCompact for NormalizedTypeParameter<MetaForm> {
	type Output = NormalizedTypeParameter<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		NormalizedTypeParameter {
			name: registry.register_string(self.name),
			parent: self.parent,
		}
	}
}

impl NormalizedTypeParameter {
	pub fn new(name: <MetaForm as Form>::String, parent: <CompactForm as Form>::Type) -> Self {
		Self { name, parent }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct NormalizedGenericType {
	ty: <CompactForm as Form>::Type,
	params: Vec<<CompactForm as Form>::Type>,
}

impl IntoCompact for NormalizedGenericType {
	type Output = NormalizedGenericType;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self
	}
}

impl NormalizedGenericType {
	pub fn new<P>(ty: <CompactForm as Form>::Type, params: P) -> Self
	where
		P: IntoIterator<Item = <CompactForm as Form>::Type>,
	{
		NormalizedGenericType {
			ty,
			params: params.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From)]
pub enum NormalizedTypeId {
	/// Concrete type id
	Concrete(any::TypeId),
	/// Use a type's path as its unique id
	Path(Path),
	/// Generic type parameter Path + Name
	Parameter(NormalizedTypeParameter<CompactForm>),
	/// Generic type instance
	Generic(NormalizedGenericType),
}

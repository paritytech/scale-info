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
use crate::meta_type::MetaTypeGeneric;
use crate::tm_std::*;
use crate::{
	form::{CompactForm, Form, MetaForm},
	meta_type::{MetaType, MetaTypeConcrete},
	MetaTypeParameter, Path, Type,
};
use derive_more::From;
use serde::Serialize;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum InternedType<F: Form = MetaForm> {
	/// The definition of the type
	Definition(InternedTypeDef<F>),
	/// The type is specified by a parameter of the parent type
	Parameter(InternedTypeParameter<F>),
	/// The type of the field is a generic type with the given type params
	Generic(InternedGenericType<F>),
}

impl IntoCompact for InternedType<MetaForm> {
	type Output = InternedType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			InternedType::Definition(definition) => definition.into_compact(registry).into(),
			InternedType::Parameter(parameter) => parameter.into_compact(registry).into(),
			InternedType::Generic(generic) => generic.into_compact(registry).into(),
		}
	}
}

impl IntoCompact for InternedType<CompactForm> {
	type Output = InternedType<CompactForm>;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self
	}
}

impl<F> InternedType<F>
where
	F: Form,
{
	pub fn definition(path: Path<F>, ty: Type<F>) -> Self {
		InternedTypeDef::new(path, ty).into()
	}

	pub fn generic<P>(ty: F::Type, params: P) -> Self
	where
		P: IntoIterator<Item = F::Type>,
	{
		InternedGenericType::new(ty, params).into()
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, From, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct InternedTypeDef<F: Form = MetaForm> {
	#[serde(skip_serializing_if = "Path::is_empty")]
	path: Path<F>,
	ty: Type<F>,
}

impl IntoCompact for InternedTypeDef<MetaForm> {
	type Output = InternedTypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		InternedTypeDef {
			path: self.path.into_compact(registry),
			ty: self.ty.into_compact(registry),
		}
	}
}

impl<F> InternedTypeDef<F>
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
pub struct InternedTypeParameter<F: Form = MetaForm> {
	name: F::String,
	parent: F::Type,
}

impl IntoCompact for InternedTypeParameter<MetaForm> {
	type Output = InternedTypeParameter<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		InternedTypeParameter {
			name: registry.register_string(self.name),
			parent: registry.register_type(&self.parent),
		}
	}
}

impl InternedTypeParameter {
	pub fn new(name: <MetaForm as Form>::String, parent: MetaTypeGeneric) -> Self {
		Self {
			name,
			parent: MetaType::Generic(parent),
		}
	}
}

impl From<&MetaTypeParameter> for InternedTypeParameter {
	fn from(meta_param: &MetaTypeParameter) -> Self {
		Self::new(meta_param.name.clone(), meta_param.parent.clone())
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::Type: Serialize")]
pub struct InternedGenericType<F: Form = MetaForm> {
	ty: F::Type, // this has to be the same for all instances of generic types
	params: Vec<F::Type>,
}

impl From<&MetaTypeConcrete> for InternedGenericType {
	fn from(concrete: &MetaTypeConcrete) -> Self {
		Self {
			ty: MetaType::Generic(concrete.clone().into()),
			params: concrete.params.iter().map(|p| p.concrete.clone().into()).collect(),
		}
	}
}

impl IntoCompact for InternedGenericType<MetaForm> {
	type Output = InternedGenericType<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		InternedGenericType {
			ty: registry.register_type(&self.ty),
			params: registry.register_types(self.params),
		}
	}
}

impl IntoCompact for InternedGenericType<CompactForm> {
	type Output = InternedGenericType<CompactForm>;

	fn into_compact(self, _registry: &mut Registry) -> Self::Output {
		self
	}
}

impl<F> InternedGenericType<F>
where
	F: Form,
{
	pub fn new<P>(ty: F::Type, params: P) -> Self
	where
		P: IntoIterator<Item = F::Type>,
	{
		InternedGenericType {
			ty,
			params: params.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From)]
pub enum InternedTypeId {
	/// Any type id
	Any(any::TypeId),
	/// Use a type's path as its unique id
	Path(Path),
	/// Generic type parameter Path + Name
	Parameter(InternedTypeParameter<CompactForm>),
	/// Generic type instance
	Generic(InternedGenericType<CompactForm>),
}

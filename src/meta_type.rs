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
use crate::{Path, Type, TypeInfo};
use derive_more::From;

/// A metatype abstraction.
///
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, From)]
pub enum MetaType {
	Parameter(MetaTypeParameter),
	Concrete(MetaTypeConcrete),
}

impl MetaType {
	pub fn concrete<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		MetaType::Concrete(MetaTypeConcrete::new::<T>())
	}

	pub fn parameter<T, P>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: 'static + ?Sized + TypeInfo,
	{
		MetaType::Parameter(MetaTypeParameter::new::<T, P>(name))
	}

	pub fn parameterized<T>(params: Vec<MetaTypeParameterValue>) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		MetaType::Concrete(MetaTypeConcrete::parameterized::<T, _>(params))
	}
}

#[derive(Clone, Debug)]
pub struct MetaTypeConcrete {
	type_id: any::TypeId,
	type_def: MetaTypeDefinition,
	params: Vec<MetaTypeConcrete>,
	param_values: Vec<MetaTypeParameterValue>,
}

impl From<MetaTypeParameter> for MetaTypeConcrete {
	fn from(param: MetaTypeParameter) -> MetaTypeConcrete {
		param.concrete
	}
}

impl PartialEq for MetaTypeConcrete {
	fn eq(&self, other: &Self) -> bool {
		self.type_id == other.type_id
	}
}

impl Eq for MetaTypeConcrete {}

impl PartialOrd for MetaTypeConcrete {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.type_id.partial_cmp(&other.type_id)
	}
}

impl Ord for MetaTypeConcrete {
	fn cmp(&self, other: &Self) -> Ordering {
		self.type_id.cmp(&other.type_id)
	}
}

impl MetaTypeConcrete {
	pub fn new<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		Self {
			type_id: any::TypeId::of::<T>(),
			type_def: MetaTypeDefinition::new::<T>(),
			params: T::params(),
			param_values: T::params()
				.iter()
				.map(|p| MetaTypeParameterValue::Concrete(p.clone()))
				.collect(),
		}
	}

	pub fn parameterized<T, P>(param_values: P) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: IntoIterator<Item = MetaTypeParameterValue>,
	{
		Self {
			type_id: any::TypeId::of::<T>(),
			type_def: MetaTypeDefinition::new::<T>(),
			params: T::params(),
			param_values: param_values.into_iter().collect(),
		}
	}

	pub fn concrete_type_id(&self) -> any::TypeId {
		self.type_id
	}

	pub fn type_def(&self) -> &MetaTypeDefinition {
		&self.type_def
	}

	pub fn type_info(&self) -> Type {
		(self.type_def.fn_type_info)()
	}

	pub fn path(&self) -> &Path {
		&self.type_def.path
	}

	pub fn has_params(&self) -> bool {
		!self.params.is_empty()
	}

	pub fn params(&self) -> impl Iterator<Item = &MetaTypeConcrete> {
		self.params.iter()
	}

	pub fn parameter_values(&self) -> impl DoubleEndedIterator<Item = &MetaTypeParameterValue> {
		self.param_values.iter()
	}
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct MetaTypeParameter {
	pub name: &'static str, // todo: make private
	pub parent: MetaTypeDefinition, // todo: make private
	concrete: MetaTypeConcrete,
}

impl MetaTypeParameter {
	pub fn new<T, P>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: 'static + ?Sized + TypeInfo,
	{
		MetaTypeParameter {
			name,
			parent: MetaTypeDefinition::new::<T>(),
			concrete: MetaTypeConcrete::new::<P>(),
		}
	}

	pub fn concrete_type_id(&self) -> any::TypeId {
		self.concrete.concrete_type_id()
	}

	/// Returns true if the concrete type of the parameter itself has parameters
	pub fn has_params(&self) -> bool {
		self.concrete.has_params()
	}
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, From)]
pub enum MetaTypeParameterValue {
	Concrete(MetaTypeConcrete),
	Parameter(MetaTypeParameter),
}

impl MetaTypeParameterValue {
	pub fn parameter<T, P>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: 'static + ?Sized + TypeInfo,
	{
		MetaTypeParameterValue::Parameter(MetaTypeParameter::new::<T, P>(name))
	}

	pub fn concrete<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		MetaTypeParameterValue::Concrete(MetaTypeConcrete::new::<T>())
	}

	pub fn concrete_type_id(&self) -> any::TypeId {
		match self {
			MetaTypeParameterValue::Concrete(concrete) => concrete.type_id,
			MetaTypeParameterValue::Parameter(param) => param.concrete.type_id,
		}
	}
}

impl From<MetaTypeParameterValue> for MetaType {
	fn from(p: MetaTypeParameterValue) -> Self {
		match p {
			MetaTypeParameterValue::Concrete(c) => MetaType::Concrete(c),
			MetaTypeParameterValue::Parameter(p) => MetaType::Parameter(p),
		}
	}
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct MetaTypeDefinition {
	fn_type_info: fn() -> Type,
	path: Path,
}

impl MetaTypeDefinition {
	fn new<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		Self {
			fn_type_info: T::type_info,
			path: T::path(),
		}
	}

	pub fn path(&self) -> Path {
		self.path.clone()
	}

	pub fn type_info(&self) -> Type {
		(self.fn_type_info)()
	}
}

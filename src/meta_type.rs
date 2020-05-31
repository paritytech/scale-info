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
#[derive(Clone, Debug)]
pub struct MetaType {
	type_id: any::TypeId,
	/// The value of `any::type_name::<T>()`.
	/// This should *only* be used for debugging purposes
	type_name: &'static str,
	type_def: MetaTypeDefinition,
	params: Vec<MetaType>,
	kind: MetaTypeKind,
}

#[derive(Clone, Debug, From)]
pub enum MetaTypeKind {
	Concrete,
	Parameterized(Vec<MetaType>),
	Parameter(MetaTypeParameter),
}

impl MetaType {
	pub fn new<T>(kind: MetaTypeKind) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		Self {
			type_id: any::TypeId::of::<T>(),
			type_name: any::type_name::<T>(),
			type_def: MetaTypeDefinition::new::<T>(),
			params: T::params(),
			kind,
		}
	}

	pub fn concrete<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		Self::new::<T>(MetaTypeKind::Concrete)
	}

	pub fn parameter<P, T>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: 'static + ?Sized + TypeInfo,
	{
		Self::new::<T>(MetaTypeKind::Parameter(MetaTypeParameter::new::<P>(name)))
	}

	pub fn parameterized<T, I>(params: I) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		I: IntoIterator<Item = MetaType>,
	{
		Self::new::<T>(MetaTypeKind::Parameterized(params.into_iter().collect()))
	}

	pub fn kind(&self) -> &MetaTypeKind {
		&self.kind
	}

	/// Get the concrete type name for this `MetaType`.
	/// e.g. `core::option::Option<bool>`
	///
	/// This should *only* be used for debugging purposes.
	pub fn concrete_type_name(&self) -> &'static str {
		self.type_name
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

	pub fn params(&self) -> impl DoubleEndedIterator<Item = &MetaType> {
		self.params.iter()
	}
}

impl PartialEq for MetaType {
	fn eq(&self, other: &Self) -> bool {
		self.type_id == other.type_id
	}
}

impl Eq for MetaType {}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug)]
pub struct MetaTypeParameter {
	pub name: &'static str,         // todo: make private
	pub parent: MetaTypeDefinition, // todo: make private
}

impl MetaTypeParameter {
	pub fn new<T>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		MetaTypeParameter {
			name,
			parent: MetaTypeDefinition::new::<T>(),
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

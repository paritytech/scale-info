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
use crate::{form::MetaForm, Metadata, Type, TypeInfo, Path};
use crate::registry::TypeParameter;
use derive_more::From;

/// A metatype abstraction.
///
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone)]
pub struct MetaTypeConcrete {
	path: Path,
	/// The concrete type parameters of an instance of a generic type
	concrete_params: Vec<MetaType>,
	/// The params for this type which might be generic todo: improve comment
	// generic_params: Vec<MetaType>,
	/// Function pointer to get type information.
	fn_type_info: fn() -> Type<MetaForm>,
	/// The type identifier
	type_id: any::TypeId,
}

impl MetaTypeConcrete {
	pub fn new<T>() -> Self
		where
			T: Metadata + ?Sized + 'static,
	{
		MetaTypeConcrete {
			fn_type_info: <T as TypeInfo>::type_info,
			type_id: any::TypeId::of::<T>(),
			path: <T as TypeInfo>::path(),
			concrete_params: <T as TypeInfo>::params(),
		}
	}

	pub fn of<T>() -> Self
		where
			T: Metadata + ?Sized + 'static,
	{
		Self::new::<T>()
	}

	/// The parameters to the type
	pub fn type_params(&self) -> Vec<MetaTypeParameter> {
		// &self.params
		todo!()
	}

	pub fn path(&self) -> Path {
		self.path.clone()
	}

	/// Returns the meta type information.
	pub fn type_info(&self) -> Type<MetaForm> {
		(self.fn_type_info)()
	}
}

pub struct MetaTypeGeneric {
	pub params: Vec<MetaTypeParameter>
}

#[derive(Clone, From)]
pub enum MetaType {
	Concrete(MetaTypeConcrete),
	Parameter(MetaTypeParameter),
	Generic(MetaTypeGeneric)
}

impl PartialEq for MetaType {
	fn eq(&self, other: &Self) -> bool {
		self.type_id == other.type_id
	}
}

impl Eq for MetaType {}

impl PartialOrd for MetaType {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.type_id.partial_cmp(&other.type_id)
	}
}

impl Ord for MetaType {
	fn cmp(&self, other: &Self) -> Ordering {
		self.type_id.cmp(&other.type_id)
	}
}

impl Hash for MetaType {
	fn hash<H>(&self, state: &mut H)
	where
		H: Hasher,
	{
		self.type_id.hash(state)
	}
}

impl Debug for MetaType {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		self.type_id.fmt(f)
	}
}

impl MetaType {
	// pub fn new<T>() -> Self
	// where
	// 	T: Metadata + ?Sized + 'static,
	// {
	// 	MetaTypeInstance {
	// 		fn_type_info: <T as TypeInfo>::type_info,
	// 		type_id: any::TypeId::of::<T>(),
	// 		path: <T as TypeInfo>::path(),
	// 		concrete_params: <T as TypeInfo>::params(),
	// 	}.into()
	// }

	pub fn parameter(name: &'static str, ty: MetaType) -> Self
	{
		MetaTypeParameter {
			name,
			parent: ty
		}.into()
	}

	pub fn parameter_of<T>(name: &'static str) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::parameter(name, Self::of::<T>())
	}

	pub fn of<T>() -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new::<T>()
	}

	/// The instance params
	pub fn params(&self) -> Vec<MetaType> {

	}

	/// Returns the type identifier provided by `core::any`.
	pub fn type_id(&self) -> any::TypeId {
		self.type_id
	}
}

/// A generic parameter of a parameterized MetaType.
///
/// e.g. the `T` in `Option<T>`
pub struct MetaTypeParameter {
	pub name: &'static str,
	pub instance_type_id: any::TypeId,
	pub parent: MetaTypeConcrete,
}

impl MetaTypeParameter {
	pub fn new<T>(name: &'static str) -> Self
	where
		T: TypeInfo + 'static,
	{
		Self {
			name,
			parent: MetaTypeConcrete::of::<T>(),
		}
	}
}

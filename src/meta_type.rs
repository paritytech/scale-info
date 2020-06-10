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
use crate::registry::{NormalizedTypeParameter, NormalizedType};

/// A metatype abstraction.
///
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone, Debug, Eq, PartialEq, From)]
pub enum MetaType {
	Concrete(MetaTypeInfo),
	Parameterized(MetaTypeParameterized),
	Parameter(MetaTypeParameter),
}

impl From<MetaTypeParameterValue> for MetaType {
	fn from(param_value: MetaTypeParameterValue) -> MetaType {
		match param_value {
			MetaTypeParameterValue::Concrete(meta_type) => {
				MetaType::Concrete(meta_type)
			},
			MetaTypeParameterValue::TypeParameter(param) => {
				MetaType::Parameter(param)
			}
		}
	}
}

impl MetaType {
	pub fn concrete<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		MetaType::Concrete(MetaTypeInfo::new::<T>())
	}

	pub fn parameter<P, T>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: 'static + ?Sized + TypeInfo,
	{
		MetaType::Parameter(MetaTypeParameter::new::<T, P>(name))
	}

	pub fn parameterized<T, I>(params: I) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		I: IntoIterator<Item = MetaTypeParameterValue>,
	{
		MetaTypeParameterized::of::<T, I>(params).into()
	}

//	/// Get the concrete type name for this `MetaType`.
//	/// e.g. `core::option::Option<bool>`
//	///
//	/// This should *only* be used for debugging purposes.
	// pub fn concrete_type_name(&self) -> &'static str {
	// 	self.type_info().type_name
	// }

	// pub fn concrete_type_id(&self) -> any::TypeId {
	// 	self.type_id
	// }

	// pub fn type_def(&self) -> &MetaTypeInfo {
	// 	&self.type_info
	// }
	//
	// pub fn type_info(&self) -> Type {
	// 	(self.type_info.fn_type_info)()
	// }
	//
	// pub fn path(&self) -> Path {
	// 	self.type_info.path()
	// }

	// pub fn has_params(&self) -> bool {
	// 	!self.type_info.params.is_empty()
	// }
	//
	// pub fn params(&self) -> impl DoubleEndedIterator<Item = &MetaType> {
	// 	self.type_info().params.iter()
	// }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetaTypeParameterized {
	type_info: MetaTypeInfo,
	params: Vec<MetaTypeParameterValue>,
}

impl MetaTypeParameterized {
	pub fn new<I>(type_info: MetaTypeInfo, params: I) -> Self
	where
		I: IntoIterator<Item = MetaTypeParameterValue>
	{
		Self {
			type_info,
			params: params.into_iter().collect(),
		}
	}

	pub fn of<T, I>(params: I) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		I: IntoIterator<Item = MetaTypeParameterValue>
	{
		Self::new(MetaTypeInfo::new::<T>(), params)
	}

	pub fn type_info(&self) -> &MetaTypeInfo {
		&self.type_info
	}

	pub fn concrete_params(&self) -> impl Iterator<Item = &MetaTypeParameter> {
		self.type_info.params.iter()
	}

	pub fn param_values(&self) -> impl DoubleEndedIterator<Item = &MetaTypeParameterValue> {
		self.params.iter()
	}
}

#[derive(Clone, Debug, Eq, PartialEq, From)]
pub enum MetaTypeParameterValue {
	/// A concrete parameter value for a generic type.
	///
	/// # Example
	///
	/// ```
	/// struct A {
	/// 	a: Option<u32>,
	/// 	// 		   ^------ Concrete parameter value
	/// }
	/// ```
	Concrete(MetaTypeInfo),
	/// A type parameter value from the parent type.
	///
	/// # Example
	///
	/// ```
	/// struct B<T> {
	/// 	a: Option<T>,
	/// 	// 		  ^----- Generic parameter value
	/// }
	/// ```
	TypeParameter(MetaTypeParameter),
}

impl MetaTypeParameterValue {
	pub fn is_value_for(&self, param: &MetaTypeParameter) -> bool {
		match self {
			MetaTypeParameterValue::Concrete(meta_type) => {
				*meta_type == param.concrete
			},
			MetaTypeParameterValue::TypeParameter(generic_param) => {
				generic_param.concrete == param.concrete
			}
		}
	}
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MetaTypeParameter {
	name: &'static str,
	concrete: MetaTypeInfo,
	parent: MetaTypeInfo,
}

impl From<MetaTypeParameter> for NormalizedTypeParameter {
	fn from(param: MetaTypeParameter) -> NormalizedTypeParameter {
		NormalizedTypeParameter::new(param.name, param.parent)
	}
}

impl MetaTypeParameter {
	pub fn new<T, P>(name: &'static str) -> Self
	where
		T: 'static + ?Sized + TypeInfo,
		P: 'static + ?Sized + TypeInfo,
	{
		MetaTypeParameter {
			name,
			concrete: MetaTypeInfo::new::<T>(),
			parent: MetaTypeInfo::new::<P>(),
		}
	}

	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn parent(&self) -> &MetaTypeInfo {
		&self.parent
	}

	pub fn concrete(&self) -> &MetaTypeInfo {
		&self.concrete
	}

	/// True if this parameter has parameters itself.
	///
	/// # Example
	///
	/// Constructing a type with the parameter `T` having the value `Option<bool>` which has a
	/// parameter itself.
	///
	/// ```
	/// struct A<T> {
	/// 	f: T
	/// }
	///
	/// type B = A<Option<bool>>;
	/// ```
	pub fn has_params(&self) -> bool {
		!self.concrete.has_params()
	}
}

#[derive(Clone, Debug)]
pub struct MetaTypeInfo {
	type_id: any::TypeId,
	/// The value of `any::type_name::<T>()`.
	/// This should *only* be used for debugging purposes
	type_name: &'static str,
	fn_type_info: fn() -> Type,
	params: Vec<MetaTypeParameter>,
	path: Path,
}

impl PartialEq for MetaTypeInfo {
	fn eq(&self, other: &Self) -> bool {
		self.type_id == other.type_id
	}
}

impl Eq for MetaTypeInfo {}

impl From<MetaTypeInfo> for NormalizedType {
	fn from(type_info: MetaTypeInfo) -> NormalizedType {
		let params = type_info.params.iter().cloned().map(Into::into);
		NormalizedType::definition(type_info.path(), type_info.type_info(), params)
	}
}

impl MetaTypeInfo {
	fn new<T>() -> Self
	where
		T: 'static + ?Sized + TypeInfo,
	{
		Self {
			type_id: any::TypeId::of::<T>(),
			type_name: any::type_name::<T>(),
			fn_type_info: T::type_info,
			params: T::params(),
			path: T::path(),
		}
	}

	pub fn path(&self) -> Path {
		self.path.clone()
	}

	pub fn concrete_type_id(&self) -> any::TypeId {
		self.type_id
	}

	pub fn type_info(&self) -> Type {
		(self.fn_type_info)()
	}

	pub fn has_params(&self) -> bool {
		!self.params.is_empty()
	}

	pub fn params(&self) -> impl Iterator<Item = &MetaTypeParameter> {
		self.params.iter()
	}
}

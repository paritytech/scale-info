// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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
use crate::{form::MetaForm, Type, TypeInfo};

/// A metatype abstraction.
///
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone, Copy)]
pub struct MetaType {
	/// Function pointer to get type information.
	fn_type_info: fn() -> Type<MetaForm>,
	// The standard type ID (ab)used in order to provide
	// cheap implementations of the standard traits
	// such as `PartialEq`, `PartialOrd`, `Debug` and `Hash`.
	type_id: TypeId,
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
	/// Creates a new meta type from the given compile-time known type.
	pub fn new<T>() -> Self
	where
		T: TypeInfo + ?Sized + 'static,
	{
		Self {
			fn_type_info: <T as TypeInfo>::type_info,
			type_id: TypeId::of::<T>(),
		}
	}

	/// Creates a new meta type which is a transparent wrapper of another compile-time known type.
	///
	/// This should be used where [`EncodeLike`](scale::WrapperTypeEncode) is implemented.
	///
	/// # Example
	///
	/// ```
	/// # use scale_info::MetaType;
	/// // u32 and Box<u32> are encoded to the same SCALE representation
	/// MetaType::new_wrapper::<u32, Box<u32>>();
	/// ```
	pub fn new_wrapper<T, U>() -> Self
	where
		T: TypeInfo + ?Sized + 'static,
		U: TypeInfo + ?Sized + 'static,
	{
		Self {
			fn_type_info: <T as TypeInfo>::type_info,
			type_id: TypeId::of::<U>(),
		}
	}

	/// Creates a new meta types from the type of the given reference.
	pub fn of<T>(_elem: &T) -> Self
	where
		T: TypeInfo + ?Sized + 'static,
	{
		Self::new::<T>()
	}

	/// Returns the meta type information.
	pub fn type_info(&self) -> Type<MetaForm> {
		(self.fn_type_info)()
	}

	/// Returns the type identifier provided by `core::any`.
	pub fn type_id(&self) -> TypeId {
		self.type_id
	}
}

///
pub struct TypeInfoTag<T>(PhantomData<T>);

/// TODO: docs
pub trait TypeInfoKind {
	/// TODO: docs
	type Type: TypeInfo + 'static;

	/// TODO: docs
	#[inline]
	fn kind(&self) -> TypeInfoTag<Self::Type> {
		TypeInfoTag(PhantomData)
	}
}

impl<T: TypeInfo + 'static> TypeInfoTag<T> {
	/// TODO: docs
	#[inline]
	pub fn new(self) -> MetaType {
		MetaType::new::<T>()
	}
}

// Requires one extra autoref to call! Lower priority than WrapperTypeKind.
impl<T: TypeInfo + 'static> TypeInfoKind for &PhantomData<T> {
	type Type = T;
}

/// TODO: docs
pub struct WrapperTypeTag<T>(PhantomData<T>);

/// TODO: docs
pub trait WrapperTypeKind {
	/// TODO: docs
	type Type: scale::WrapperTypeEncode<Target = Self::Target>;
	/// TODO: docs
	type Target: TypeInfo + 'static;

	/// TODO: docs
	#[inline]
	fn kind(&self) -> WrapperTypeTag<Self::Type> {
		WrapperTypeTag(PhantomData)
	}
}

// Does not require any autoref if called as (&error).anyhow_kind().
impl<T: scale::WrapperTypeEncode<Target = U>, U: TypeInfo + 'static> WrapperTypeKind for PhantomData<T> {
	type Type = T;
	type Target = U;
}

impl<T, U> WrapperTypeTag<T>
where
	T: TypeInfo + scale::WrapperTypeEncode<Target = U> + 'static,
	U: TypeInfo + 'static
{
	/// TODO: docs
	#[inline]
	pub fn new(self) -> MetaType {
		MetaType::new_wrapper::<T, U>()
	}
}

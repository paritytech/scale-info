// Copyright 2019
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
use crate::{form::MetaForm, TypeInfo, Metadata, Type};

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
	fn_get_type: fn() -> Type<MetaForm>,
	// The standard type ID (ab)used in order to provide
	// cheap implementations of the standard traits
	// such as `PartialEq`, `PartialOrd`, `Debug` and `Hash`.
	any_id: AnyTypeId,
}

impl PartialEq for MetaType {
	fn eq(&self, other: &Self) -> bool {
		self.any_id == other.any_id
	}
}

impl Eq for MetaType {}

impl PartialOrd for MetaType {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.any_id.partial_cmp(&other.any_id)
	}
}

impl Ord for MetaType {
	fn cmp(&self, other: &Self) -> Ordering {
		self.any_id.cmp(&other.any_id)
	}
}

impl Hash for MetaType {
	fn hash<H>(&self, state: &mut H)
	where
		H: Hasher,
	{
		self.any_id.hash(state)
	}
}

impl Debug for MetaType {
	fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
		self.any_id.fmt(f)
	}
}

impl MetaType {
	/// Creates a new meta type from the given compile-time known type.
	pub fn new<T>() -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self {
			fn_get_type: <T as TypeInfo>::type_info,
			any_id: AnyTypeId::of::<T>(),
		}
	}

	/// Creates a new meta types from the type of the given reference.
	pub fn of<T>(_elem: &T) -> Self
	where
		T: Metadata + ?Sized + 'static,
	{
		Self::new::<T>()
	}

	/// Returns the meta type identifier.
	pub fn type_id(&self) -> Type<MetaForm> {
		(self.fn_get_type)()
	}

	/// Returns the type identifier provided by `core::any`.
	pub fn any_id(&self) -> AnyTypeId {
		self.any_id
	}
}

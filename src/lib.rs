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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[macro_export]
macro_rules! tuple_meta_type {
    ( $($ty:ty),* ) => {
        {
			#[cfg(not(feature = "std"))]
			extern crate alloc as _alloc;
			#[cfg(not(feature = "std"))]
			#[allow(unused_mut)]
			let mut v = _alloc::vec![];

			#[cfg(feature = "std")]
			#[allow(unused_mut)]
			let mut v = std::vec![];

            $(
				v.push(MetaType::new::<$ty>());
            )*
            v
        }
    }
}

mod tm_std;

pub mod form;
mod impls;
pub mod interner;
mod meta_type;
mod registry;
mod type_def;
mod type_id;
mod utils;

#[cfg(test)]
mod tests;

pub use self::{
	meta_type::MetaType,
	registry::{IntoCompact, Registry},
	type_def::*,
	type_id::*,
};

#[cfg(feature = "derive")]
pub use type_metadata_derive::{Metadata, TypeDef, TypeId};

pub trait Metadata: HasTypeId + HasTypeDef {
	fn meta_type() -> MetaType;
}

impl<T> Metadata for T
where
	T: ?Sized + HasTypeId + HasTypeDef + 'static,
{
	fn meta_type() -> MetaType {
		MetaType::new::<T>()
	}
}

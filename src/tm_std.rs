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

//! Exports from `std`, `core` and `alloc` crates.

mod core {
	#[cfg(not(feature = "std"))]
	pub use core::*;

	#[cfg(feature = "std")]
	pub use std::*;
}

#[rustfmt::skip]
pub use self::core::{
	i8, i16, i32, i64, i128,
	u8, u16, u32, u64, u128,

	marker::PhantomData,
	num::NonZeroU32,
	option::Option,
	result::Result,

	any::TypeId,

	clone::{Clone},
	cmp::{Eq, PartialEq, Ordering},
	convert::{From, Into},
	fmt::{Debug, Error as FmtError, Formatter},
	hash::{Hash, Hasher},
	iter,
};

mod alloc {
	#[cfg(not(feature = "std"))]
	pub use ::alloc::*;

	#[cfg(feature = "std")]
	pub use std::*;
}

#[rustfmt::skip]
pub use self::alloc::{
	boxed::Box,
	collections::btree_map::{BTreeMap, Entry},
	string::{String, ToString},
	vec, vec::Vec,
};

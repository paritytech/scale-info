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

use crate::{interner::UntrackedSymbol, meta_type::MetaType};
use crate::tm_std::*;
use serde::Serialize;

/// Trait to control the internal structures of type identifiers and definitions.
///
/// This allows for type-level separation between free forms that can be instantiated
/// out of the flux and compact forms that require some sort of interning data structures.
pub trait Form {
	/// The string type.
	type String: Serialize + PartialEq + Eq + PartialOrd + Ord + Clone + core::fmt::Debug;
	/// The type identifier type.
	type TypeId: PartialEq + Eq + PartialOrd + Ord + Clone + core::fmt::Debug;
	/// A type identifier with indirection.
	///
	/// # Note
	///
	/// This is an optimization for the compact forms.
	type IndirectTypeId: PartialEq + Eq + PartialOrd + Ord + Clone + core::fmt::Debug;
}

/// A meta meta-type.
///
/// Allows to be converted into other forms such as compact form
/// through the registry and `IntoCompact`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Debug)]
pub enum MetaForm {}

impl Form for MetaForm {
	type String = &'static str;
	type TypeId = MetaType;
	type IndirectTypeId = MetaType;
}

/// Compact form that has its lifetime untracked in association to its interner.
///
/// # Note
///
/// This resolves some lifetime issues with self-referential structs (such as
/// the registry itself) but can no longer be used to resolve to the original
/// underlying data.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Debug)]
pub enum CompactForm {}

impl Form for CompactForm {
	type String = UntrackedSymbol<&'static str>;
	type TypeId = UntrackedSymbol<AnyTypeId>;
	type IndirectTypeId = Self::TypeId;
}

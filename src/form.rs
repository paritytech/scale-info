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

//! Provides some form definitions.
//!
//! The forms provided here are used to generically communicate the
//! compaction mode a type identifier, type definition or structures
//! that are using these.
//!
//! The default form is the `MetaForm`.
//! It uses `MetaType` for communicating type identifiers and thus acts as
//! a bridge from runtime to compile time type information.
//!
//! The compact form is `CompactForm` and represents a compact form
//! that no longer has any connections to the interning registry and thus
//! can no longer be used in order to retrieve information from the
//! original registry easily. Its sole purpose is for compact serialization.
//!
//! Other forms, such as a compact form that is still bound to the registry
//! (also via lifetime tracking) are possible but current not needed.

use crate::tm_std::*;
use crate::{interner::UntrackedSymbol, meta_type::MetaType};
use serde::Serialize;

/// Trait to control the internal structures of type definitions.
///
/// This allows for type-level separation between free forms that can be
/// instantiated out of the flux and compact forms that require some sort of
/// interning data structures.
pub trait Form {
	/// The type identifier type.
	type TypeId: PartialEq + Eq + PartialOrd + Ord + Clone + core::fmt::Debug;
}

/// A meta meta-type.
///
/// Allows to be converted into other forms such as compact form
/// through the registry and `IntoCompact`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Debug)]
pub enum MetaForm {}

impl Form for MetaForm {
	type TypeId = MetaType;
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
	type TypeId = UntrackedSymbol<TypeId>;
}

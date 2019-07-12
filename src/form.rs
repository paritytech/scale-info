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

use crate::{
	interner::{Symbol, UntrackedSymbol},
	meta_type::MetaType,
	TypeId,
};
use serde::Serialize;
use core::{
	marker::PhantomData,
	any::TypeId as AnyTypeId,
};

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Debug)]
pub enum MetaForm {}

impl Form for MetaForm {
	type String = &'static str;
	type TypeId = MetaType;
	type IndirectTypeId = MetaType;
}

/// Free form is not depending on any interner data structure
/// to compact symbols and type identifiers. This means it requires
/// more space but can also be created in flux.
///
/// # Note
///
/// The free form is the default for all type identifiers and definitions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Debug)]
pub enum FreeForm {}

impl Form for FreeForm {
	type String = &'static str;
	type TypeId = TypeId;
	type IndirectTypeId = Box<TypeId>;
}

/// Compact form that is lifetime tracked in association to its interner.
///
/// # Note
///
/// This ensures safe resolution and thus allows to transform this back into
/// the free form. Can also be transformed into the untracked form, however,
/// doing so is ireversible.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Debug)]
pub struct TrackedForm<'a> {
	interner: PhantomData<fn () -> &'a ()>,
}

impl<'a> Form for TrackedForm<'a> {
	type String = Symbol<'a, &'static str>;
	type TypeId = Symbol<'a, AnyTypeId>;
	type IndirectTypeId = Self::TypeId;
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

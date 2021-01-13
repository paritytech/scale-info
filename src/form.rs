// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

//! Provides form definitions.
//!
//! The forms provided here are used to generically communicate the mode a type
//! identifier, type definition or structure is using.
//!
//! The default form is the `MetaForm`.
//! It uses `MetaType` for communicating type identifiers and thus acts as
//! a bridge from runtime to compile time type information.
//!
//! The `PortableForm` is a space-efficient representation
//! that no longer has any connections to the interning registry and thus
//! can no longer be used to retrieve information from the
//! original registry. Its sole purpose is for space-efficient serialization.
//!
//! Other forms, such as a portable form that is still bound to the registry
//! (also via lifetime tracking) are possible but current not needed.

use crate::prelude::{
    any::TypeId,
    fmt::Debug,
    marker::PhantomData,
    string::String,
};

use crate::{
    interner::UntrackedSymbol,
    meta_type::MetaType,
};

use scale::Decode;

#[cfg(feature = "serde")]
use serde::Serialize;

/// Trait to control the internal structures of type definitions.
///
/// This allows for type-level separation between free forms that can be
/// instantiated out of the flux and portable forms that require some sort of
/// interning data structures.
pub trait Form {
    /// The type representing the type.
    type Type: PartialEq + Eq + PartialOrd + Ord + Clone + Debug;
    /// The string type.
    type String: FormString;
}

/// Trait for types which can be used to represent strings in type definitions.
pub trait FormString:
    AsRef<str> + PartialEq + Eq + PartialOrd + Ord + Clone + Debug
{
}

impl FormString for &'static str {}
impl FormString for String {}

/// A meta meta-type.
///
/// Allows to be converted into other forms such as portable form
/// through the registry and `IntoPortable`.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum MetaForm {}

impl Form for MetaForm {
    type Type = MetaType;
    type String = &'static str;
}

/// Portable form that has its lifetime untracked in association to its interner.
///
/// # Note
///
/// This resolves some lifetime issues with self-referential structs (such as
/// the registry itself) but can no longer be used to resolve to the original
/// underlying data.
///
/// `type String` is owned in order to enable decoding
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Decode)]
pub struct PortableForm<S = &'static str>(PhantomData<S>);

impl<S> Form for PortableForm<S>
where
    S: FormString,
{
    type Type = UntrackedSymbol<TypeId>;
    type String = S;
}

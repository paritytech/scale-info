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

use crate::{
	form::{CompactForm, Form, MetaForm},
	IntoCompact, MetaType, Metadata, Registry,
};
use derive_more::From;
use serde::Serialize;

/// A type definition represents the internal structure of a concrete type.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize, From)]
#[serde(bound = "F::Type: Serialize")]
#[serde(rename_all = "lowercase")]
pub enum TypeDef<F: Form = MetaForm> {
	/// A struct with named fields.
	Struct(TypeDefStruct<F>),
	/// A tuple-struct with unnamed fields.
	TupleStruct(TypeDefTupleStruct<F>),
	/// A C-like enum with simple named variants.
	ClikeEnum(TypeDefClikeEnum<F>),
	/// A Rust enum with different kinds of variants.
	Enum(TypeDefEnum<F>),
	/// An unsafe Rust union type.
	Union(TypeDefUnion<F>),
}

impl IntoCompact for TypeDef {
	type Output = TypeDef<CompactForm>;

	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		match self {
			TypeDef::Struct(r#struct) => r#struct.into_compact(registry).into(),
			TypeDef::TupleStruct(tuple_struct) => tuple_struct.into_compact(registry).into(),
			TypeDef::ClikeEnum(clike_enum) => clike_enum.into_compact(registry).into(),
			TypeDef::Enum(r#enum) => r#enum.into_compact(registry).into(),
			TypeDef::Union(union) => union.into_compact(registry).into(),
		}
	}
}

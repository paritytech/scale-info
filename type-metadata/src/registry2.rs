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
	interner::{StringInterner, StringSymbol, TypeIdInterner, TypeIdSymbol},
	Form, CompactForm,
	Metadata, Namespace, TypeDef, TypeId,
};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TypeIdDef {
	id: TypeId,
	def: TypeDef,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Registry {
	string_table: StringInterner,
	typeid_table: TypeIdInterner,
	typedefs: Vec<TypeDef>,
}

impl Registry {
	pub fn register_name(&mut self, name: &'static str) -> StringSymbol {
		unimplemented!()
	}

	pub fn register_namespace(&mut self, namespace: Namespace) -> CompactNamespace {
		unimplemented!()
	}

	pub fn register_type<T>(&mut self) -> TypeIdSymbol
	where
		T: Metadata,
	{
		unimplemented!()
	}
}

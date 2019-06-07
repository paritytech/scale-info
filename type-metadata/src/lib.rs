// Copyright 2019 Centrality Investments Limited
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

//! Implements type metadata information handling to enable self-descriptive codec

mod registry;
pub use registry::Registry;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct TypeIdent {
	namespace: Vec<String>,
	ident: IdentKind,
	type_args: Vec<String>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum IdentKind {
	Custom(String),

	// primitives, including build-in and common core/std primitives
	Bool,
	Str,
	Unit,
	U8, U16, U32, Usize, U64, U128,
	I8, I16, I32, Isize, I64, I128,
	Array(u16, Box<TypeIdent>),
	Vector(Box<TypeIdent>),
	Tuple(Vec<TypeIdent>),
	Option(Box<TypeIdent>),
	Result(Box<TypeIdent>, Box<TypeIdent>),
}

pub enum TypeDef {
	Primitive,
	Struct(StructDef),
	EnumDef(EnumDef),
}

pub enum FieldName {
	Unnamed(u16),
	Named(String),
}
pub struct Field {
	pub name: FieldName,
	pub ident: TypeIdent,
}
pub type StructDef = Vec<Field>;

pub struct EnumVariant {
	pub name: String,
	pub index: u16,
	pub fields: Vec<Field>,
}
pub type EnumDef = Vec<EnumVariant>;

pub trait TypeMetadata {
	fn type_ident() -> TypeIdent;

	/// `type_def` would do registration if needed
	fn type_def(registry: &Registry) -> TypeDef;
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}

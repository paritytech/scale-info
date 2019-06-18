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
	form::{Form, FreeForm},
	HasTypeId, TypeId,
};
use derive_more::From;
use serde::Serialize;

/// Types implementing this trait can communicate their type structure.
///
/// If the current type contains any other types, `type_def` would register their metadata into the given
/// `registry`. For instance, `<Option<MyStruct>>::type_def()` would register `MyStruct` metadata. All
/// implementation must register these contained types' metadata.
pub trait HasTypeDef {
	fn type_def() -> TypeDef;
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDef<F: Form = FreeForm> {
	/// Stores count and names of all generic parameters.
	///
	/// This can be used to verify that type id's refer to
	/// correct instantiations of a generic type.
	generic_params: GenericParams<F>,
	/// The underlying structure of the type definition.
	kind: TypeDefKind<F>,
}

impl TypeDef<FreeForm> {
	pub fn new<G, K>(generic_params: G, kind: K) -> Self
	where
		G: IntoIterator<Item = <FreeForm as Form>::String>,
		K: Into<TypeDefKind>,
	{
		Self {
			generic_params: generic_params
				.into_iter()
				.map(|name| GenericArg::from(name))
				.collect::<Vec<_>>()
				.into(),
			kind: kind.into(),
		}
	}
}

impl<K> From<K> for TypeDef
where
	K: Into<TypeDefKind>,
{
	fn from(kind: K) -> Self {
		Self {
			generic_params: GenericParams::empty(),
			kind: kind.into(),
		}
	}
}

impl TypeDef {
	pub fn builtin() -> Self {
		Self {
			generic_params: GenericParams::empty(),
			kind: TypeDefKind::Builtin,
		}
	}

	pub fn kind(&self) -> &TypeDefKind {
		&self.kind
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
pub struct GenericParams<F: Form = FreeForm> {
	params: Vec<GenericArg<F>>,
}

impl GenericParams {
	pub fn empty() -> Self {
		Self { params: vec![] }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GenericArg<F: Form = FreeForm> {
	name: F::String,
}

impl From<<FreeForm as Form>::String> for GenericArg {
	fn from(name: <FreeForm as Form>::String) -> Self {
		Self { name }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
pub enum TypeDefKind<F: Form = FreeForm> {
	Builtin,
	Struct(TypeDefStruct<F>),
	TupleStruct(TypeDefTupleStruct),
	ClikeEnum(TypeDefClikeEnum),
	Enum(TypeDefEnum),
	Union(TypeDefUnion),
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefStruct<F: Form = FreeForm> {
	fields: Vec<NamedField<F>>,
}

impl TypeDefStruct {
	pub fn new<F>(fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			fields: fields.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct NamedField<F: Form = FreeForm> {
	name: F::String,
	#[serde(rename = "type")]
	ty: F::TypeId,
}

impl NamedField {
	pub fn new<T>(name: <FreeForm as Form>::String, ty: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self { name, ty: ty.into() }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefTupleStruct<F: Form = FreeForm> {
	fields: Vec<UnnamedField<F>>,
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct UnnamedField<F: Form = FreeForm> {
	#[serde(rename = "type")]
	ty: F::TypeId,
}

impl UnnamedField {
	pub fn new<T>() -> Self
	where
		T: HasTypeId,
	{
		Self { ty: T::type_id() }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefClikeEnum<F: Form = FreeForm> {
	variants: Vec<ClikeEnumVariant<F>>,
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct ClikeEnumVariant<F: Form = FreeForm> {
	name: F::String,
	discriminant: u64,
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefEnum<F: Form = FreeForm> {
	variants: Vec<EnumVariant<F>>,
}

impl TypeDefEnum {
	pub fn new<V>(variants: V) -> Self
	where
		V: IntoIterator<Item = EnumVariant>,
	{
		Self {
			variants: variants.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
pub enum EnumVariant<F: Form = FreeForm> {
	Unit(EnumVariantUnit<F>),
	Struct(EnumVariantStruct<F>),
	TupleStruct(EnumVariantTupleStruct<F>),
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantUnit<F: Form = FreeForm> {
	name: F::String,
}

impl EnumVariantUnit {
	pub fn new(name: &'static str) -> Self {
		Self { name }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantStruct<F: Form = FreeForm> {
	name: F::String,
	fields: Vec<NamedField<F>>,
}

impl EnumVariantStruct {
	pub fn new<F>(name: <FreeForm as Form>::String, fields: F) -> Self
	where
		F: IntoIterator<Item = NamedField>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantTupleStruct<F: Form = FreeForm> {
	name: F::String,
	fields: Vec<UnnamedField<F>>,
}

impl EnumVariantTupleStruct {
	pub fn new<F>(name: <FreeForm as Form>::String, fields: F) -> Self
	where
		F: IntoIterator<Item = UnnamedField>,
	{
		Self {
			name,
			fields: fields.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefUnion<F: Form = FreeForm> {
	fields: Vec<NamedField<F>>,
}

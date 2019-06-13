// TODO: This file contents have not yet been modified thoroughly.

use crate::{Registry, TypeId};

/// If the current type contains any other types, `type_def` would register their metadata into the given
/// `registry`. For instance, `<Option<MyStruct>>::type_def()` would register `MyStruct` metadata. All
/// implementation must register these contained types' metadata.
pub trait HasTypeDef {
	fn type_def(registry: &mut Registry) -> TypeDef;
}

#[derive(PartialEq, Eq, Debug)]
pub enum TypeDef {
	None,
	Struct(StructDef),
	TupleStruct(TupleStructDef),
	Enum(EnumDef),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Field {
	pub name: &'static str,
	pub ident: TypeId,
}
#[derive(PartialEq, Eq, Debug)]
pub struct StructDef(pub Vec<Field>);

#[derive(PartialEq, Eq, Debug)]
pub struct TupleStructDef(pub Vec<TypeId>);

#[derive(PartialEq, Eq, Debug)]
pub struct DataVariant<T> {
	pub name: &'static str,
	pub index: u16,
	pub struct_def: T,
}

#[derive(PartialEq, Eq, Debug)]
pub struct NoDataVariant {
	pub name: &'static str,
	pub index: u16,
}

#[derive(PartialEq, Eq, Debug)]
pub enum VariantKind {
	NoData(NoDataVariant),
	Struct(DataVariant<StructDef>),
	TupleStruct(DataVariant<TupleStructDef>),
}
#[derive(PartialEq, Eq, Debug)]
pub struct EnumDef(Vec<VariantKind>);

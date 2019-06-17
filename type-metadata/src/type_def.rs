use crate::{Registry, TypeId, HasTypeId};
use derive_more::From;
use serde::Serialize;

/// Types implementing this trait can communicate their type structure.
///
/// If the current type contains any other types, `type_def` would register their metadata into the given
/// `registry`. For instance, `<Option<MyStruct>>::type_def()` would register `MyStruct` metadata. All
/// implementation must register these contained types' metadata.
pub trait HasTypeDef {
	fn type_def(registry: &mut Registry) -> TypeDef;
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDef {
	generic_params: GenericParams,
	kind: TypeDefKind,
}

impl From<TypeDefKind> for TypeDef {
	fn from(kind: TypeDefKind) -> Self {
		Self {
			generic_params: GenericParams::empty(),
			kind,
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

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GenericParams {
	params: Vec<GenericArg>,
}

impl GenericParams {
	pub fn empty() -> Self {
		Self { params: vec![] }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct GenericArg {
	name: &'static str,
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
pub enum TypeDefKind {
	Builtin,
	Struct(TypeDefStruct),
	TupleStruct(TypeDefTupleStruct),
	ClikeEnum(TypeDefClikeEnum),
	Enum(TypeDefEnum),
	Union(TypeDefUnion),
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefStruct {
	fields: Vec<NamedField>,
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
pub struct NamedField {
	name: &'static str,
	#[serde(rename = "type")]
	ty: TypeId,
}

impl NamedField {
	pub fn new<T>(name: &'static str, ty: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self { name, ty: ty.into() }
	}
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefTupleStruct {
	fields: Vec<UnnamedField>,
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct UnnamedField {
	#[serde(rename = "type")]
	ty: TypeId,
}

impl UnnamedField {
    pub fn new<T>() -> Self
    where
        T: HasTypeId,
    {
        Self {
            ty: T::type_id(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefClikeEnum {
	variants: Vec<ClikeEnumVariant>,
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct ClikeEnumVariant {
	name: &'static str,
	discriminant: u64,
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct TypeDefEnum {
	variants: Vec<EnumVariant>,
}

impl TypeDefEnum {
    pub fn new<V>(variants: V) -> Self
    where
        V: IntoIterator<Item = EnumVariant>,
    {
        Self { variants: variants.into_iter().collect(), }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, From)]
pub enum EnumVariant {
	Unit(EnumVariantUnit),
    Struct(EnumVariantStruct),
    TupleStruct(EnumVariantTupleStruct),
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantUnit {
	name: &'static str,
}

impl EnumVariantUnit {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize)]
pub struct EnumVariantStruct {
    name: &'static str,
    fields: Vec<NamedField>,
}

impl EnumVariantStruct {
    pub fn new<F>(name: &'static str, fields: F) -> Self
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
pub struct EnumVariantTupleStruct {
    name: &'static str,
    fields: Vec<UnnamedField>,
}

impl EnumVariantTupleStruct {
    pub fn new<F>(name: &'static str, fields: F) -> Self
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
pub struct TypeDefUnion {
	fields: Vec<NamedField>,
}

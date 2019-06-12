use derive_more::From;
use serde::Serialize;

pub trait HasTypeId {
	fn type_id() -> TypeId;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct Namespace {
	segments: Vec<&'static str>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct InvalidNamespace;

impl Namespace {
	/// Creates a new namespace from the given segments.
	pub fn new<S>(segments: S) -> Result<Self, InvalidNamespace>
	where
		S: IntoIterator<Item = &'static str>,
	{
		let segments = segments.into_iter().collect::<Vec<_>>();
		if segments.len() == 0 {
			return Err(InvalidNamespace);
		}
		Ok(Self { segments })
	}

	/// Creates a new namespace from the given module path.
	///
	/// # Note
	///
	/// Module path is generally obtained from the `module_path!` Rust macro.
	pub fn from_str(module_path: &'static str) -> Result<Self, InvalidNamespace> {
		Self::new(module_path.split("::"))
	}

	/// Creates the prelude namespace.
	pub fn prelude() -> Self {
		Self { segments: vec![] }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, From, Serialize)]
pub enum TypeId {
	Custom(TypeIdCustom),
	Slice(TypeIdSlice),
	Array(TypeIdArray),
	Tuple(TypeIdTuple),
	Primitive(TypeIdPrimitive),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub enum TypeIdPrimitive {
	#[serde(rename = "bool")]
	Bool,
	#[serde(rename = "str")]
	Str,
	#[serde(rename = "u8")]
	U8,
	#[serde(rename = "u16")]
	U16,
	#[serde(rename = "u32")]
	U32,
	#[serde(rename = "u64")]
	U64,
	#[serde(rename = "u128")]
	U128,
	#[serde(rename = "i8")]
	I8,
	#[serde(rename = "i16")]
	I16,
	#[serde(rename = "i32")]
	I32,
	#[serde(rename = "i64")]
	I64,
	#[serde(rename = "i128")]
	I128,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdCustom {
	name: &'static str,
	namespace: Namespace,
	#[serde(rename = "type")]
	type_params: Vec<TypeId>,
}

impl TypeIdCustom {
	pub fn new<T>(name: &'static str, namespace: Namespace, type_params: T) -> Self
	where
		T: IntoIterator<Item = TypeId>,
	{
		Self {
			name,
			namespace,
			type_params: type_params.into_iter().collect(),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdArray {
	pub len: u16,
	#[serde(rename = "type")]
	pub type_param: Box<TypeId>,
}

impl TypeIdArray {
	pub fn new<T>(len: u16, type_param: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self {
			len,
			type_param: Box::new(type_param.into()),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdTuple {
	#[serde(rename = "type")]
	pub type_params: Vec<TypeId>,
}

impl TypeIdTuple {
	pub fn new<T>(type_params: T) -> Self
	where
		T: IntoIterator<Item = TypeId>,
	{
		Self {
			type_params: type_params.into_iter().collect(),
		}
	}

	pub fn unit() -> Self {
		Self::new(vec![])
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
pub struct TypeIdSlice {
	#[serde(rename = "type")]
	type_param: Box<TypeId>,
}

impl TypeIdSlice {
	pub fn new<T>(type_param: T) -> Self
	where
		T: Into<TypeId>,
	{
		Self {
			type_param: Box::new(type_param.into()),
		}
	}
}

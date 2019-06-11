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

mod registry;
pub use registry::Registry;

mod tests;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub enum IdentKind {
	Custom(CustomIdent),

	// primitives, including build-in types and common precludes
	Bool,
	Str,
	U8,
	U16,
	U32,
	U64,
	U128,
	I8,
	I16,
	I32,
	I64,
	I128,
	Array(ArrayIdent),
	Slice(SliceIdent),
	Tuple(TupleIdent),
	Option(OptionIdent),
	Result(ResultIdent),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Namespace {
	segments: Vec<&'static str>,
}
impl Namespace {
	fn new(segments: Vec<&'static str>) -> Self {
		Namespace { segments }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct CustomIdent {
	pub name: &'static str,
	pub namespace: Namespace,
	pub type_params: Vec<IdentKind>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ArrayIdent {
	pub len: u16,
	pub type_param: Box<IdentKind>,
}
impl ArrayIdent {
	fn new(len: u16, type_param: IdentKind) -> Self {
		ArrayIdent {
			len,
			type_param: Box::new(type_param),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct TupleIdent {
	pub type_params: Vec<IdentKind>,
}
impl TupleIdent {
	fn new(type_params: Vec<IdentKind>) -> Self {
		TupleIdent { type_params }
	}
	fn unit() -> Self {
		TupleIdent { type_params: vec![] }
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SliceIdent {
	pub type_param: Box<IdentKind>,
}
impl SliceIdent {
	fn new(type_param: IdentKind) -> Self {
		SliceIdent {
			type_param: Box::new(type_param),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct OptionIdent {
	pub type_param: Box<IdentKind>,
}
impl OptionIdent {
	fn new(type_param: IdentKind) -> Self {
		OptionIdent {
			type_param: Box::new(type_param),
		}
	}
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ResultIdent {
	pub type_params: (Box<IdentKind>, Box<IdentKind>),
}
impl ResultIdent {
	fn new(type_params: (IdentKind, IdentKind)) -> Self {
		ResultIdent {
			type_params: (Box::new(type_params.0), Box::new(type_params.1)),
		}
	}
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
	pub ident: IdentKind,
}
#[derive(PartialEq, Eq, Debug)]
pub struct StructDef(Vec<Field>);

#[derive(PartialEq, Eq, Debug)]
pub struct TupleStructDef(Vec<IdentKind>);

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

pub trait Metadata {
	fn type_ident() -> IdentKind;

	/// If the current type contains any other types, `type_def` would register their metadata into the given
	/// `registry`. For instance, `<Option<MyStruct>>::type_def()` would register `MyStruct` metadata. All
	/// implementation must register these contained types' metadata.
	fn type_def(registry: &mut Registry) -> TypeDef;

	fn register(registry: &mut Registry) {
		registry.register(Self::type_ident(), Self::type_def);
	}
}

macro_rules! impl_metadata_for_primitives {
	( $( $t:ty => $ident_kind:expr, )* ) => { $(
		impl Metadata for $t {
			fn type_ident() -> IdentKind {
				$ident_kind
			}

			fn type_def(_registry: &mut Registry) -> TypeDef {
				TypeDef::None
			}
		}
	)* }
}

impl_metadata_for_primitives!(
	bool => IdentKind::Bool,
	u8 => IdentKind::U8,
	u16 => IdentKind::U16,
	u32 => IdentKind::U32,
	u64 => IdentKind::U64,
	u128 => IdentKind::U128,
	i8 => IdentKind::I8,
	i16 => IdentKind::I16,
	i32 => IdentKind::I32,
	i64 => IdentKind::I64,
	i128 => IdentKind::I128,
);

macro_rules! impl_metadata_for_array {
	( $( $n:expr )* ) => { $(
		impl<T: Metadata> Metadata for [T; $n] {
			fn type_ident() -> IdentKind {
				IdentKind::Array(ArrayIdent::new($n, T::type_ident()))
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(T::type_ident(), T::type_def);
				TypeDef::None
			}
		}
	)* }
}

impl_metadata_for_array!(1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
	40 48 56 64 72 96 128 160 192 224 256);

macro_rules! impl_metadata_for_tuple {
	($one:ident,) => {
		impl<$one: Metadata> Metadata for ($one,) {
			fn type_ident() -> IdentKind {
				IdentKind::Tuple(TupleIdent::new(vec![<$one>::type_ident()]))
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(<$one>::type_ident(), <$one>::type_def);
				TypeDef::None
			}
		}
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first: Metadata, $($rest: Metadata),+> Metadata for ($first, $($rest),+) {
			fn type_ident() -> IdentKind {
				IdentKind::Tuple(TupleIdent::new(
					vec![<$first>::type_ident(), $( <$rest>::type_ident(), )+],
				))
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(<$first>::type_ident(), <$first>::type_def);
				$({ registry.register(<$rest>::type_ident(), <$rest>::type_def); })+
				TypeDef::None
			}
		}

		impl_metadata_for_tuple!($($rest,)+);
	}
}

impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K,);

impl<T: Metadata> Metadata for Vec<T> {
	fn type_ident() -> IdentKind {
		IdentKind::Slice(SliceIdent::new(T::type_ident()))
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		TypeDef::None
	}
}

impl<T: Metadata> Metadata for Option<T> {
	fn type_ident() -> IdentKind {
		IdentKind::Option(OptionIdent::new(T::type_ident()))
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		TypeDef::None
	}
}

impl<T: Metadata, E: Metadata> Metadata for Result<T, E> {
	fn type_ident() -> IdentKind {
		IdentKind::Result(ResultIdent::new((T::type_ident(), E::type_ident())))
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		registry.register(E::type_ident(), E::type_def);
		TypeDef::None
	}
}

impl<T: Metadata> Metadata for Box<T> {
	fn type_ident() -> IdentKind {
		T::type_ident()
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)
	}
}

impl<T: Metadata> Metadata for &T {
	fn type_ident() -> IdentKind {
		T::type_ident()
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)
	}
}

impl<T: Metadata> Metadata for [T] {
	fn type_ident() -> IdentKind {
		<Vec<T>>::type_ident()
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		<Vec<T>>::type_def(registry)
	}
}

impl Metadata for () {
	fn type_ident() -> IdentKind {
		IdentKind::Tuple(TupleIdent::unit())
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::None
	}
}

impl Metadata for &str {
	fn type_ident() -> IdentKind {
		IdentKind::Str
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::None
	}
}

impl Metadata for String {
	fn type_ident() -> IdentKind {
		IdentKind::Str
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::None
	}
}

impl<T: Metadata> Metadata for std::marker::PhantomData<T> {
	fn type_ident() -> IdentKind {
		IdentKind::Tuple(TupleIdent::new(vec![T::type_ident()]))
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::None
	}
}

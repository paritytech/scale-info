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
	Array(Box<ArrayIdent>),
	Slice(Box<SliceIdent>),
	Tuple(Box<TupleIdent>),
	Option(Box<OptionIdent>),
	Result(Box<ResultIdent>),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Namespace(Vec<&'static str>);
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct CustomIdent {
	pub name: &'static str,
	pub namespace: Namespace,
	pub type_params: Vec<IdentKind>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ArrayIdent {
	pub len: u16,
	pub type_param: IdentKind,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct TupleIdent {
	pub type_params: Option<Vec<IdentKind>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct SliceIdent {
	pub type_param: IdentKind,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct OptionIdent {
	pub type_param: IdentKind,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct ResultIdent {
	pub type_params: (IdentKind, IdentKind),
}

#[derive(PartialEq, Eq, Debug)]
pub enum TypeDef {
	Primitive,
	Struct(StructDef),
	EnumDef(EnumDef),
}

#[derive(PartialEq, Eq, Debug)]
pub enum FieldName {
	Unnamed(u16),
	Named(String),
}
#[derive(PartialEq, Eq, Debug)]
pub struct Field {
	pub name: FieldName,
	pub ident: IdentKind,
}
pub type StructDef = Vec<Field>;

#[derive(PartialEq, Eq, Debug)]
pub struct EnumVariant {
	pub name: String,
	pub index: u16,
	pub fields: Vec<Field>,
}
pub type EnumDef = Vec<EnumVariant>;

pub trait Metadata {
	fn type_ident() -> IdentKind;

	/// `type_def` would do registration if needed
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
				TypeDef::Primitive
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
				IdentKind::Array(Box::new(ArrayIdent {
					len: $n,
					type_param: T::type_ident(),
				}))
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(T::type_ident(), T::type_def);
				TypeDef::Primitive
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
				IdentKind::Tuple(Box::new(TupleIdent {
					type_params: Some(vec![<$one>::type_ident()]),
				}))
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(<$one>::type_ident(), <$one>::type_def);
				TypeDef::Primitive
			}
		}
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first: Metadata, $($rest: Metadata),+> Metadata for ($first, $($rest),+) {
			fn type_ident() -> IdentKind {
				IdentKind::Tuple(Box::new(TupleIdent {
					type_params: Some(vec![<$first>::type_ident(), $( <$rest>::type_ident(), )+])
				}))
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(<$first>::type_ident(), <$first>::type_def);
				$({ registry.register(<$rest>::type_ident(), <$rest>::type_def); })+
				TypeDef::Primitive
			}
		}

		impl_metadata_for_tuple!($($rest,)+);
	}
}

impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K,);

impl<T: Metadata> Metadata for Vec<T> {
	fn type_ident() -> IdentKind {
		IdentKind::Slice(Box::new(SliceIdent {
			type_param: T::type_ident(),
		}))
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		TypeDef::Primitive
	}
}

impl<T: Metadata> Metadata for Option<T> {
	fn type_ident() -> IdentKind {
		IdentKind::Option(Box::new(OptionIdent {
			type_param: T::type_ident(),
		}))
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		TypeDef::Primitive
	}
}

impl<T: Metadata, E: Metadata> Metadata for Result<T, E> {
	fn type_ident() -> IdentKind {
		IdentKind::Result(Box::new(ResultIdent {
			type_params: (T::type_ident(), E::type_ident()),
		}))
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		registry.register(E::type_ident(), E::type_def);
		TypeDef::Primitive
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
		IdentKind::Tuple(Box::new(TupleIdent { type_params: None }))
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

impl Metadata for &str {
	fn type_ident() -> IdentKind {
		IdentKind::Str
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

impl Metadata for String {
	fn type_ident() -> IdentKind {
		IdentKind::Str
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

impl<T: Metadata> Metadata for std::marker::PhantomData<T> {
	fn type_ident() -> IdentKind {
		IdentKind::Tuple(Box::new(TupleIdent { type_params: None }))
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

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

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct TypeIdent {
	namespace: Vec<String>,
	ident: IdentKind,
	args: Vec<TypeIdent>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum IdentKind {
	Custom(String),

	// primitives, including build-in and common core/std primitives
	Bool,
	Str,
	Unit,
	U8,
	U16,
	U32,
	Usize,
	U64,
	U128,
	I8,
	I16,
	I32,
	Isize,
	I64,
	I128,
	Array(u16),
	Vector,
	Tuple,
	Option,
	Result,
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

pub trait Metadata {
	fn type_ident() -> TypeIdent;

	/// `type_def` would do registration if needed
	fn type_def(registry: &mut Registry) -> TypeDef;

	fn register(registry: &mut Registry) {
		registry.register(Self::type_ident(), Self::type_def);
	}
}

macro_rules! impl_metadata_for_primitives {
	( $( $t:ty => $ident_kind:expr, )* ) => { $(
		impl Metadata for $t {
			fn type_ident() -> TypeIdent {
				TypeIdent {
					namespace: vec![],
					ident: $ident_kind,
					args: vec![],
				}
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
	usize => IdentKind::Usize,
	u64 => IdentKind::U64,
	u128 => IdentKind::U128,
	i8 => IdentKind::I8,
	i16 => IdentKind::I16,
	i32 => IdentKind::I32,
	isize => IdentKind::Isize,
	i64 => IdentKind::I64,
	i128 => IdentKind::I128,
);

macro_rules! impl_metadata_for_array {
	( $( $n:expr )* ) => { $(
		impl<T: Metadata> Metadata for [T; $n] {
			fn type_ident() -> TypeIdent {
				TypeIdent {
					namespace: vec![],
					ident: IdentKind::Array($n),
					args: vec![T::type_ident()],
				}
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
			fn type_ident() -> TypeIdent {
				TypeIdent {
					namespace: vec![],
					ident: IdentKind::Tuple,
					args: vec![<$one>::type_ident()],
				}
			}
			fn type_def(registry: &mut Registry) -> TypeDef {
				registry.register(<$one>::type_ident(), <$one>::type_def);
				TypeDef::Primitive
			}
		}
	};
	($first:ident, $($rest:ident,)+) => {
		impl<$first: Metadata, $($rest: Metadata),+> Metadata for ($first, $($rest),+) {
			fn type_ident() -> TypeIdent {
				TypeIdent {
					namespace: vec![],
					ident: IdentKind::Tuple,
					args: vec![<$first>::type_ident(), $( <$rest>::type_ident(), )+]
				}
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
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Vector,
			args: vec![T::type_ident()],
		}
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		TypeDef::Primitive
	}
}

impl<T: Metadata> Metadata for Option<T> {
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Option,
			args: vec![T::type_ident()],
		}
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		TypeDef::Primitive
	}
}

impl<T: Metadata, E: Metadata> Metadata for Result<T, E> {
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Result,
			args: vec![T::type_ident(), E::type_ident()],
		}
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register(T::type_ident(), T::type_def);
		registry.register(E::type_ident(), E::type_def);
		TypeDef::Primitive
	}
}

impl<T: Metadata> Metadata for Box<T> {
	fn type_ident() -> TypeIdent {
		T::type_ident()
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)
	}
}

impl<T: Metadata> Metadata for &T {
	fn type_ident() -> TypeIdent {
		T::type_ident()
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)
	}
}

impl<T: Metadata> Metadata for [T] {
	fn type_ident() -> TypeIdent {
		<Vec<T>>::type_ident()
	}

	fn type_def(registry: &mut Registry) -> TypeDef {
		<Vec<T>>::type_def(registry)
	}
}

impl<T: Metadata> Metadata for std::marker::PhantomData<T> {
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Unit,
			args: vec![T::type_ident()],
		}
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

impl Metadata for () {
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Unit,
			args: vec![],
		}
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

impl Metadata for &str {
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Str,
			args: vec![],
		}
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

impl Metadata for String {
	fn type_ident() -> TypeIdent {
		TypeIdent {
			namespace: vec![],
			ident: IdentKind::Str,
			args: vec![],
		}
	}

	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::Primitive
	}
}

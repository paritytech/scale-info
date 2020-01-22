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
use crate::*;

macro_rules! impl_metadata_for_primitives {
	( $( $t:ty => $ident_kind:expr, )* ) => { $(
		impl HasTypeId for $t {
			fn type_id() -> TypeId {
				TypeId::Primitive($ident_kind)
			}
		}

		impl HasTypeDef for $t {
			fn type_def() -> TypeDef {
				TypeDef::builtin()
			}
		}
	)* }
}

impl_metadata_for_primitives!(
	bool => TypeIdPrimitive::Bool,
	char => TypeIdPrimitive::Char,
	u8 => TypeIdPrimitive::U8,
	u16 => TypeIdPrimitive::U16,
	u32 => TypeIdPrimitive::U32,
	u64 => TypeIdPrimitive::U64,
	u128 => TypeIdPrimitive::U128,
	i8 => TypeIdPrimitive::I8,
	i16 => TypeIdPrimitive::I16,
	i32 => TypeIdPrimitive::I32,
	i64 => TypeIdPrimitive::I64,
	i128 => TypeIdPrimitive::I128,
);

macro_rules! impl_metadata_for_array {
	( $( $n:expr )* ) => {
		$(
			impl<T: Metadata + 'static> HasTypeId for [T; $n] {
				fn type_id() -> TypeId {
					TypeIdArray::new($n, MetaType::new::<T>()).into()
				}
			}

			impl<T: Metadata> HasTypeDef for [T; $n] {
				fn type_def() -> TypeDef {
					TypeDef::builtin()
				}
			}
		)*
	}
}

#[rustfmt::skip]
impl_metadata_for_array!(
        1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
	40 48 56 64 72 96 128 160 192 224 256
);

macro_rules! impl_metadata_for_tuple {
    ( $($ty:ident),* ) => {
		impl<$($ty),*> HasTypeId for ($($ty,)*)
		where
			$(
				$ty: Metadata + 'static,
			)*
		{
			fn type_id() -> TypeId {
				TypeIdTuple::new(tuple_meta_type!($($ty),*)).into()
			}
		}

		impl<$($ty),*> HasTypeDef for ($($ty,)*)
		where
			$(
				$ty: Metadata,
			)*
		{
			fn type_def() -> TypeDef {
				TypeDef::builtin()
			}
		}
    }
}

impl_metadata_for_tuple!();
impl_metadata_for_tuple!(A);
impl_metadata_for_tuple!(A, B);
impl_metadata_for_tuple!(A, B, C);
impl_metadata_for_tuple!(A, B, C, D);
impl_metadata_for_tuple!(A, B, C, D, E);
impl_metadata_for_tuple!(A, B, C, D, E, F);
impl_metadata_for_tuple!(A, B, C, D, E, F, G);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J);

impl<T> HasTypeId for Vec<T>
where
	T: Metadata + 'static,
{
	fn type_id() -> TypeId {
		TypeIdCollection::new("Vec", tuple_meta_type![T]).into()
	}
}

impl<T> HasTypeDef for Vec<T>
where
	T: Metadata + 'static,
{
	fn type_def() -> TypeDef {
		TypeDef::builtin()
	}
}

impl<T> HasTypeId for Option<T>
where
	T: Metadata + 'static,
{
	fn type_id() -> TypeId {
		TypeIdCustom::new("Option", Namespace::prelude(), tuple_meta_type![T]).into()
	}
}

impl<T> HasTypeDef for Option<T>
where
	T: Metadata + 'static,
{
	fn type_def() -> TypeDef {
		TypeDefEnum::new(vec![
			EnumVariantUnit::new("None").into(),
			EnumVariantTupleStruct::new("Some", vec![UnnamedField::of::<T>()]).into(),
		])
		.into()
	}
}

impl<T, E> HasTypeId for Result<T, E>
where
	T: Metadata + 'static,
	E: Metadata + 'static,
{
	fn type_id() -> TypeId {
		TypeIdCustom::new("Result", Namespace::prelude(), tuple_meta_type!(T, E)).into()
	}
}

impl<T, E> HasTypeDef for Result<T, E>
where
	T: Metadata + 'static,
	E: Metadata + 'static,
{
	fn type_def() -> TypeDef {
		TypeDefEnum::new(vec![
			EnumVariantTupleStruct::new("Ok", vec![UnnamedField::of::<T>()]).into(),
			EnumVariantTupleStruct::new("Err", vec![UnnamedField::of::<E>()]).into(),
		])
		.into()
	}
}

impl<K, V> HasTypeId for BTreeMap<K, V>
where
	K: Metadata + 'static,
	V: Metadata + 'static,
{
	fn type_id() -> TypeId {
		TypeIdCollection::new("BTreeMap", tuple_meta_type!(K, V)).into()
	}
}

impl<K, V> HasTypeDef for BTreeMap<K, V>
where
	K: Metadata + 'static,
	V: Metadata + 'static,
{
	fn type_def() -> TypeDef {
		TypeDef::builtin()
	}
}

impl<T> HasTypeId for Box<T>
where
	T: HasTypeId + ?Sized,
{
	fn type_id() -> TypeId {
		T::type_id()
	}
}

impl<T> HasTypeDef for Box<T>
where
	T: Metadata + ?Sized,
{
	fn type_def() -> TypeDef {
		T::type_def()
	}
}

impl<T> HasTypeId for &T
where
	T: HasTypeId + ?Sized,
{
	fn type_id() -> TypeId {
		T::type_id()
	}
}

impl<T> HasTypeDef for &T
where
	T: Metadata + ?Sized,
{
	fn type_def() -> TypeDef {
		T::type_def()
	}
}

impl<T> HasTypeId for &mut T
where
	T: HasTypeId + ?Sized,
{
	fn type_id() -> TypeId {
		T::type_id()
	}
}

impl<T> HasTypeDef for &mut T
where
	T: Metadata + ?Sized,
{
	fn type_def() -> TypeDef {
		T::type_def()
	}
}

impl<T> HasTypeId for [T]
where
	T: Metadata + 'static,
{
	fn type_id() -> TypeId {
		TypeIdSlice::of::<T>().into()
	}
}

impl<T> HasTypeDef for [T]
where
	T: Metadata,
{
	fn type_def() -> TypeDef {
		TypeDef::builtin()
	}
}

impl HasTypeId for str {
	fn type_id() -> TypeId {
		TypeIdPrimitive::Str.into()
	}
}

impl HasTypeDef for str {
	fn type_def() -> TypeDef {
		TypeDef::builtin()
	}
}

impl HasTypeId for String {
	fn type_id() -> TypeId {
		<str>::type_id()
	}
}

impl HasTypeDef for String {
	fn type_def() -> TypeDef {
		TypeDefStruct::new(vec![NamedField::new("vec", MetaType::new::<Vec<u8>>())]).into()
	}
}

impl<T> HasTypeId for PhantomData<T>
where
	T: Metadata + ?Sized,
{
	fn type_id() -> TypeId {
		TypeIdCustom::new("PhantomData", Namespace::prelude(), vec![T::meta_type()]).into()
	}
}

impl<T> HasTypeDef for PhantomData<T>
where
	T: Metadata + ?Sized,
{
	fn type_def() -> TypeDef {
		TypeDefTupleStruct::new(vec![]).into()
	}
}

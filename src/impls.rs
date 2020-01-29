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
		impl HasType for $t {
			fn get_type() -> Type {
				Type::Primitive($ident_kind)
			}
		}
	)* }
}

impl_metadata_for_primitives!(
	bool => TypePrimitive::Bool,
	char => TypePrimitive::Char,
	u8 => TypePrimitive::U8,
	u16 => TypePrimitive::U16,
	u32 => TypePrimitive::U32,
	u64 => TypePrimitive::U64,
	u128 => TypePrimitive::U128,
	i8 => TypePrimitive::I8,
	i16 => TypePrimitive::I16,
	i32 => TypePrimitive::I32,
	i64 => TypePrimitive::I64,
	i128 => TypePrimitive::I128,
);

macro_rules! impl_metadata_for_array {
	( $( $n:expr )* ) => {
		$(
			impl<T: Metadata + 'static> HasType for [T; $n] {
				fn get_type() -> Type {
					TypeArray::new($n, MetaType::new::<T>()).into()
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
		impl<$($ty),*> HasType for ($($ty,)*)
		where
			$(
				$ty: Metadata + 'static,
			)*
		{
			fn get_type() -> Type {
				TypeTuple::new(tuple_meta_type!($($ty),*)).into()
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

impl<T> HasType for Vec<T>
where
	T: Metadata + 'static,
{
	fn get_type() -> Type {
		product_type(
			"Vec",
			Namespace::prelude(),
			tuple_meta_type![T],
			TypeProductStruct::new(vec![NamedField::new("elems", MetaType::new::<[T]>())]),
		)
	}
}

impl<T> HasType for Option<T>
where
	T: Metadata + 'static,
{
	fn get_type() -> Type {
		sum_type(
			"Option",
			Namespace::prelude(),
			tuple_meta_type![T],
			TypeSumEnum::new(vec![
				EnumVariantUnit::new("None").into(),
				EnumVariantTupleStruct::new("Some", vec![UnnamedField::of::<T>()]).into(),
			]),
		)
	}
}

impl<T, E> HasType for Result<T, E>
where
	T: Metadata + 'static,
	E: Metadata + 'static,
{
	fn get_type() -> Type {
		sum_type(
			"Result",
			Namespace::prelude(),
			tuple_meta_type!(T, E),
			TypeSumEnum::new(vec![
				EnumVariantTupleStruct::new("Ok", vec![UnnamedField::of::<T>()]).into(),
				EnumVariantTupleStruct::new("Err", vec![UnnamedField::of::<E>()]).into(),
			]),
		)
	}
}

impl<K, V> HasType for BTreeMap<K, V>
where
	K: Metadata + 'static,
	V: Metadata + 'static,
{
	fn get_type() -> Type {
		product_type(
			"BTreeMap",
			Namespace::prelude(),
			tuple_meta_type![(K, V)],
			TypeProductStruct::new(vec![NamedField::new("elems", MetaType::new::<[(K, V)]>())]),
		)
	}
}

impl<T> HasType for Box<T>
where
	T: HasType + ?Sized,
{
	fn get_type() -> Type {
		T::get_type()
	}
}

impl<T> HasType for &T
where
	T: HasType + ?Sized,
{
	fn get_type() -> Type {
		T::get_type()
	}
}

impl<T> HasType for &mut T
where
	T: HasType + ?Sized,
{
	fn get_type() -> Type {
		T::get_type()
	}
}

impl<T> HasType for [T]
where
	T: Metadata + 'static,
{
	fn get_type() -> Type {
		TypeSlice::of::<T>().into()
	}
}

impl HasType for str {
	fn get_type() -> Type {
		TypePrimitive::Str.into()
	}
}

impl HasType for String {
	fn get_type() -> Type {
		product_type(
			"String",
			Namespace::prelude(),
			Vec::new(),
			TypeProductStruct::new(vec![NamedField::new("vec", MetaType::new::<Vec<u8>>())]),
		)
	}
}

impl<T> HasType for PhantomData<T>
where
	T: Metadata + ?Sized,
{
	fn get_type() -> Type {
		product_type(
			"PhantomData",
			Namespace::prelude(),
			vec![T::meta_type()],
			TypeProductTupleStruct::new(vec![]),
		)
	}
}

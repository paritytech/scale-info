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
		impl TypeInfo for $t {
			fn type_info() -> Type {
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
			impl<T: Metadata + 'static> TypeInfo for [T; $n] {
				fn type_info() -> Type {
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
		impl<$($ty),*> TypeInfo for ($($ty,)*)
		where
			$(
				$ty: Metadata + 'static,
			)*
		{
			fn type_info() -> Type {
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

impl<T> TypeInfo for Vec<T>
where
	T: Metadata + 'static,
{
	fn type_info() -> Type {
		TypeComposite::new("Vec", Namespace::prelude())
			.type_params(tuple_meta_type![T])
			.named_fields(Fields::field_of::<T>("elems"))
			.into()
	}
}

impl<T> TypeInfo for Option<T>
where
	T: Metadata + 'static,
{
	fn type_info() -> Type {
		TypeVariant::new("Option", Namespace::prelude())
			.type_params(tuple_meta_type![T])
			.variants_with_fields(|variants| {
				variants
					.unit_variant("None")
					.composite_variant("Some", Fields::unnamed().of::<T>())
			})
			.into()
	}
}

impl<T, E> TypeInfo for Result<T, E>
where
	T: Metadata + 'static,
	E: Metadata + 'static,
{
	fn type_info() -> Type {
		TypeVariant::new("Result", Namespace::prelude())
			.type_params(tuple_meta_type!(T, E))
			.variants_with_fields(|variants| {
				variants
					.unit_variant("None")
					.composite_variant("Ok", Fields::unnamed().field_of::<T>())
					.composite_variant("Err", Fields::unnamed().field_of::<E>())
			})
			.into()
	}
}

impl<K, V> TypeInfo for BTreeMap<K, V>
where
	K: Metadata + 'static,
	V: Metadata + 'static,
{
	fn type_info() -> Type {
		TypeComposite::new("BTreeMap", Namespace::prelude())
			.type_params(tuple_meta_type![(K, V)])
			.named_fields(Fields::field_of::<[(K, V)]>("elems"))
			.into()
	}
}

impl<T> TypeInfo for Box<T>
where
	T: TypeInfo + ?Sized,
{
	fn type_info() -> Type {
		T::type_info()
	}
}

impl<T> TypeInfo for &T
where
	T: TypeInfo + ?Sized,
{
	fn type_info() -> Type {
		T::type_info()
	}
}

impl<T> TypeInfo for &mut T
where
	T: TypeInfo + ?Sized,
{
	fn type_info() -> Type {
		T::type_info()
	}
}

impl<T> TypeInfo for [T]
where
	T: Metadata + 'static,
{
	fn type_info() -> Type {
		TypeSlice::of::<T>().into()
	}
}

impl TypeInfo for str {
	fn type_info() -> Type {
		TypePrimitive::Str.into()
	}
}

impl TypeInfo for String {
	fn type_info() -> Type {
		TypeComposite::new("String", Namespace::prelude())
			.named_fields(Fields::field_of::<Vec<u8>>("vec"))
			.into()
	}
}

impl<T> TypeInfo for PhantomData<T>
where
	T: Metadata + ?Sized,
{
	fn type_info() -> Type {
		TypeComposite::unit("PhantomData", Namespace::prelude())
			.type_params(vec![T::meta_type()])
			.into()
	}
}

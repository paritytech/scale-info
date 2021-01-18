// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use crate::prelude::{
    boxed::Box,
    collections::BTreeMap,
    marker::PhantomData,
    string::String,
    vec,
    vec::Vec,
};

use crate::{
    build::*,
    meta_type,
    MetaType,
    Path,
    Type,
    TypeDefArray,
    TypeDefPrimitive,
    TypeDefSequence,
    TypeDefTuple,
    TypeInfo,
};

macro_rules! impl_metadata_for_primitives {
    ( $( $t:ty => $ident_kind:expr, )* ) => { $(
        impl TypeInfo for $t {
            type Identity = Self;

            fn type_info() -> Type {
                $ident_kind.into()
            }
        }
    )* }
}

impl_metadata_for_primitives!(
    bool => TypeDefPrimitive::Bool,
    char => TypeDefPrimitive::Char,
    u8 => TypeDefPrimitive::U8,
    u16 => TypeDefPrimitive::U16,
    u32 => TypeDefPrimitive::U32,
    u64 => TypeDefPrimitive::U64,
    u128 => TypeDefPrimitive::U128,
    i8 => TypeDefPrimitive::I8,
    i16 => TypeDefPrimitive::I16,
    i32 => TypeDefPrimitive::I32,
    i64 => TypeDefPrimitive::I64,
    i128 => TypeDefPrimitive::I128,
);

macro_rules! impl_metadata_for_array {
    ( $( $n:expr )* ) => {
        $(
            impl<T: TypeInfo + 'static> TypeInfo for [T; $n] {
                type Identity = Self;

                fn type_info() -> Type {
                    TypeDefArray::new($n, MetaType::new::<T>()).into()
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
                $ty: TypeInfo+ 'static,
            )*
        {
            type Identity = Self;

            fn type_info() -> Type {
                TypeDefTuple::new(tuple_meta_type!($($ty),*)).into()
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
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_metadata_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

impl<T> TypeInfo for Vec<T>
where
    T: TypeInfo + 'static,
{
    type Identity = [T];

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for Option<T>
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("Option"))
            .type_params(tuple_meta_type![T])
            .variant(
                Variants::with_fields()
                    .variant_unit("None")
                    .variant("Some", Fields::unnamed().field_of::<T>("T")),
            )
    }
}

impl<T, E> TypeInfo for Result<T, E>
where
    T: TypeInfo + 'static,
    E: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("Result"))
            .type_params(tuple_meta_type!(T, E))
            .variant(
                Variants::with_fields()
                    .variant("Ok", Fields::unnamed().field_of::<T>("T"))
                    .variant("Err", Fields::unnamed().field_of::<E>("E")),
            )
    }
}

impl<K, V> TypeInfo for BTreeMap<K, V>
where
    K: TypeInfo + 'static,
    V: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("BTreeMap"))
            .type_params(tuple_meta_type![(K, V)])
            .composite(Fields::unnamed().field_of::<[(K, V)]>("[(K, V)]"))
    }
}

impl<T> TypeInfo for Box<T>
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for &T
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for &mut T
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = T;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for [T]
where
    T: TypeInfo + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        TypeDefSequence::of::<T>().into()
    }
}

impl TypeInfo for str {
    type Identity = Self;

    fn type_info() -> Type {
        TypeDefPrimitive::Str.into()
    }
}

impl TypeInfo for String {
    type Identity = str;

    fn type_info() -> Type {
        Self::Identity::type_info()
    }
}

impl<T> TypeInfo for PhantomData<T>
where
    T: TypeInfo + ?Sized + 'static,
{
    type Identity = Self;

    fn type_info() -> Type {
        Type::builder()
            .path(Path::prelude("PhantomData"))
            .type_params(vec![meta_type::<T>()])
            .composite(Fields::unit())
    }
}

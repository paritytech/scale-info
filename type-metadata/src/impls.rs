use crate::*;

macro_rules! impl_metadata_for_primitives {
	( $( $t:ty => $ident_kind:expr, )* ) => { $(
        impl HasTypeId for $t {
            fn type_id() -> TypeId {
                TypeId::Primitive($ident_kind)
            }
        }

        impl HasTypeDef for $t {
            fn type_def(_registry: &mut Registry) -> TypeDef {
                TypeDef::builtin()
            }
        }

		impl RegisterSubtypes for $t {}
	)* }
}

impl_metadata_for_primitives!(
	bool => TypeIdPrimitive::Bool,
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
            impl<T: HasTypeId> HasTypeId for [T; $n] {
                fn type_id() -> TypeId {
                    TypeIdArray::new($n, T::type_id()).into()
                }
            }

            impl<T: Metadata> HasTypeDef for [T; $n] {
                fn type_def(registry: &mut Registry) -> TypeDef {
                    registry.register_type::<T>();
                    TypeDef::builtin()
                }
            }

			impl<T: Metadata> RegisterSubtypes for [T; $n] {
				fn register_subtypes(registry: &mut Registry) {
                    registry.register_type::<T>();
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
                $ty: HasTypeId,
            )*
        {
            fn type_id() -> TypeId {
                TypeIdTuple::new(tuple_type_id!($($ty),*)).into()
            }
        }

        impl<$($ty),*> HasTypeDef for ($($ty,)*)
        where
            $(
                $ty: Metadata,
            )*
        {

		impl<$($ty),*> RegisterSubtypes for ($($ty,)*)
        where
            $(
                $ty: Metadata,
            )*
		{
            #[allow(unused)]
			fn register_subtypes(registry: &mut Registry) {
                $(
                    registry.register_type::<$ty>();
                )*
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
	T: HasTypeId,
{
	fn type_id() -> TypeId {
		TypeIdCustom::new("Vec", Namespace::prelude(), tuple_type_id![T]).into()
	}
}

impl<T> HasTypeDef for Vec<T>
where
	T: Metadata,
{
	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register_type::<T>();
		TypeDefStruct::new(vec![NamedField::new("elems", <[T]>::type_id())]).into()
	}
}

impl<T> RegisterSubtypes for Vec<T>
where
	T: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
	}
}

impl<T> HasTypeId for Option<T>
where
	T: HasTypeId,
{
	fn type_id() -> TypeId {
		TypeIdCustom::new("Option", Namespace::prelude(), tuple_type_id![T]).into()
	}
}

impl<T> HasTypeDef for Option<T>
where
	T: Metadata,
{
	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register_type::<T>();

		TypeDefEnum::new(vec![
			EnumVariantUnit::new("None").into(),
			EnumVariantTupleStruct::new("Some", vec![UnnamedField::new::<T>()]).into(),
		])
		.into()
	}
}

impl<T> RegisterSubtypes for Option<T>
where
	T: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
	}
}

impl<T, E> HasTypeId for Result<T, E>
where
	T: HasTypeId,
	E: HasTypeId,
{
	fn type_id() -> TypeId {
		TypeIdCustom::new("Result", Namespace::prelude(), tuple_type_id!(T, E)).into()
	}
}

impl<T, E> HasTypeDef for Result<T, E>
where
	T: Metadata,
	E: Metadata,
{
	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register_type::<T>();
		registry.register_type::<E>();

		TypeDefEnum::new(vec![
			EnumVariantTupleStruct::new("Ok", vec![UnnamedField::new::<T>()]).into(),
			EnumVariantTupleStruct::new("Err", vec![UnnamedField::new::<E>()]).into(),
		])
		.into()
	}
}

impl<T, E> RegisterSubtypes for Result<T, E>
where
	T: Metadata,
	E: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
		registry.register_type::<E>();
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
	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)

impl<T> RegisterSubtypes for Box<T>
where
	T: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
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
	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)

impl<T> RegisterSubtypes for &T
where
	T: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
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
	fn type_def(registry: &mut Registry) -> TypeDef {
		T::type_def(registry)

impl<T> RegisterSubtypes for &mut T
where
	T: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
	}
}

impl<T> HasTypeId for [T]
where
	T: HasTypeId,
{
	fn type_id() -> TypeId {
		TypeIdSlice::new(T::type_id()).into()
	}
}

impl<T> HasTypeDef for [T]
where
	T: Metadata,
{
	fn type_def(registry: &mut Registry) -> TypeDef {
		registry.register_type::<T>();
		TypeDef::builtin()
	}
}

impl<T> RegisterSubtypes for [T]
where
	T: Metadata,
{
	fn register_subtypes(registry: &mut Registry) {
		registry.register_type::<T>();
	}
}

impl HasTypeId for str {
	fn type_id() -> TypeId {
		TypeIdPrimitive::Str.into()
	}
}

impl HasTypeDef for str {
	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::builtin()
	}
}

impl RegisterSubtypes for str {}

impl HasTypeId for String {
	fn type_id() -> TypeId {
		<str>::type_id()
	}
}

impl HasTypeDef for String {
	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDefStruct::new(vec![NamedField::new("vec", Vec::<u8>::type_id())]).into()
	}
}

impl RegisterSubtypes for String {}

use core::marker::PhantomData;

impl<T> HasTypeId for PhantomData<T>
where
	T: HasTypeId + ?Sized,
{
	fn type_id() -> TypeId {
		<T>::type_id()
	}
}

impl<T> HasTypeDef for PhantomData<T>
where
	T: Metadata + ?Sized,
{
	fn type_def(_registry: &mut Registry) -> TypeDef {
		TypeDef::builtin()
	}
}

impl<T> RegisterSubtypes for PhantomData<T> where T: Metadata {}

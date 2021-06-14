use scale_info::TypeInfo;
use core::marker::PhantomData;

#[allow(unused)]
#[derive(TypeInfo)]
#[scale_info(bounds(T: Default + TypeInfo + 'static, N: TypeInfo + 'static))]
struct Hey<T, N> {
    ciao: Greet<T>,
    ho: N,
}

#[derive(TypeInfo)]
#[scale_info(bounds(T: TypeInfo + 'static))]
struct Greet<T> {
    marker: PhantomData<T>,
}

#[derive(TypeInfo, Default)]
struct SomeType;

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<Hey<SomeType, u16>>();
}
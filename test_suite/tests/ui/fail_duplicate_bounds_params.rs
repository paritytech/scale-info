use scale_info::TypeInfo;
use core::marker::PhantomData;

#[derive(TypeInfo)]
#[scale_info(bounds(), bounds())]
struct A<T> {
    marker: PhantomData<T>,
}

fn main() {}
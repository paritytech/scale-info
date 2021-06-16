use scale_info::TypeInfo;

#[allow(unused)]
#[derive(TypeInfo)]
#[scale_info(bounds(), skip_type_params(T))]
struct A<T> {
    marker: core::marker::PhantomData<T>,
}

#[allow(unused)]
#[derive(TypeInfo)]
#[scale_info(bounds())]
#[scale_info(skip_type_params(T))]
struct B<T> {
    marker: core::marker::PhantomData<T>,
}

fn main() { }
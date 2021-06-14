use scale_info::TypeInfo;

#[allow(unused)]
#[derive(TypeInfo)]
// Without this we get `overflow evaluating the requirement `Vec<B<()>>: TypeInfo``.
// The custom bounds replace the auto generated bounds.
#[scale_info(bounds(T: TypeInfo + 'static))]
struct A<T> {
    a: Vec<B<T>>,
    b: Vec<B<()>>,
    marker: core::marker::PhantomData<T>,
}

#[allow(unused)]
#[derive(TypeInfo)]
struct B<T>(A<T>);

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<A<bool>>();
}
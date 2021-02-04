use scale_info::TypeInfo;
use scale::{Codec, Encode, Decode};

#[derive(TypeInfo, Encode, Decode)]
struct Foo<T> {
    // When the below attribute is uncommented it will trigger the error
    //   `overflow evaluating the requirement `_::_parity_scale_codec::Compact<_>: Decode`
    // It will expand to a call of `.compact_of::<T>()` which triggers the overflow.
    //
    #[codec(compact)]
    b: T,
}

trait Trait {
    type Type: TypeInfo + 'static;
}

#[derive(TypeInfo, Encode, Decode)]
struct Bar<T>(std::marker::PhantomData<T>);

impl<T> Trait for Bar<T>
where
    T: TypeInfo + 'static,
    // Foo<T>: TypeInfo + 'static, // adding this bound will fix the overflow error:
{
    type Type = Foo<T>;
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<Foo<u16>>();
}

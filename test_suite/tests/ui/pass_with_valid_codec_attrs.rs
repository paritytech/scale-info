use scale_info::TypeInfo;
use scale::{Encode, HasCompact};

#[derive(TypeInfo, Encode)]
struct ValidStruct {
    #[codec(skip)]
    a: u8,
    #[codec(compact)]
    b: u16,
    #[codec(encoded_as = "<u32 as HasCompact>::Type")]
    c: u32,
}

#[derive(TypeInfo, Encode)]
enum ValidEnum {
    #[allow(unused)]
    #[codec(index = 3)]
    Thing(u32),
    #[allow(unused)]
    #[codec(skip)]
    Thong(bool),
    #[allow(unused)]
    Theng(ValidStruct),
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<ValidStruct>();
    assert_type_info::<ValidEnum>();
}

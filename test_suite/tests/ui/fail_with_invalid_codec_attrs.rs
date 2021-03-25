use scale_info::TypeInfo;
use scale::Encode;

#[derive(TypeInfo, Encode)]
struct AttrValidation {
    a: u8,
    #[codec(skip, compact)]
    b: u16,
}

#[derive(TypeInfo, Encode)]
enum EnumsAttrValidation {
    Thing(#[codec(index = 3, compact)] u32),
    #[codec(encode_as = u8, compact)]
    Thong(bool),
    Theng(AttrValidation),
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<AttrValidation>();
}

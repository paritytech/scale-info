use scale_info::TypeInfo;
use scale::Encode;

#[derive(TypeInfo, Encode)]
struct NoMultipleAttrs {
    a: u8,
    #[codec(skip, compact)]
    b: u16,
}

#[derive(Encode, TypeInfo)]
enum NoIndexOnVariantFields {
    Thing(#[codec(index = 3)] u32),
}

#[derive(Encode, TypeInfo)]
enum IndexMustBeNumber {
    #[codec(index = a)]
    Thing(u32),
}

#[derive(Encode, TypeInfo)]
enum EncodeAsAttrMustBeLiteral {
    #[codec(encode_as = u8, compact)]
    Thong(bool),
}

fn main() {}

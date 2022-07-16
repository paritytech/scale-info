use info::{self as scale_info};
use scale_info::TypeInfo;
use scale::Encode;

#[derive(TypeInfo, Encode)]
#[scale_info(foo)]
struct InvalidKeywordInScaleInfoAttr {
    a: u8,
    b: u16,
}

fn main() {}

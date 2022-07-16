// scale_info only exists as "::info", so this test
// helps ensure that we never point to `scale_info::*`
// and only use our renamed crate path.

use info::TypeInfo;

#[derive(TypeInfo)]
#[scale_info(crate = info)]
struct MyStruct {
    bar: bool
}

#[derive(TypeInfo)]
#[scale_info(crate = info)]
enum MyEnum {
    Variant { s: MyStruct }
}

fn main() {}

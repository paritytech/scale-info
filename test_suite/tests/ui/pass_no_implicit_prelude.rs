#![no_implicit_prelude]

use info::{self as scale_info};
use scale_info::TypeInfo;

#[allow(dead_code)]
#[derive(TypeInfo)]
struct S { a: bool }

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<S>();
}

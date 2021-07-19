use scale_info::TypeInfo;

#[derive(TypeInfo)]
struct AttrValidation {
    a: u8,
    #[codec(skip)]
    b: u16,
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<AttrValidation>();
}

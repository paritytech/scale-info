use scale_info::TypeInfo;

#[derive(TypeInfo)]
#[repr(C)]
union Commonwealth {
    a: u8,
    b: f32,
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<Commonwealth>();
}

use info::{self as scale_info};
use scale_info::TypeInfo;

#[allow(dead_code, non_camel_case_types)]
mod r#mod {
    use super::*;
    #[derive(TypeInfo)]
    pub enum r#enum {
        r#true,
    }
    #[derive(TypeInfo)]
    pub struct r#struct {
        r#try: r#enum,
    }
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<r#mod::r#struct>();
}

use scale_info::TypeInfo;
use scale::Encode;

#[derive(TypeInfo)]
struct Color<Hue>{hue: Hue}
#[derive(TypeInfo)]
struct Texture<Bump, Hump>{bump: Bump, hump: Hump}

#[allow(unused)]
#[derive(Encode, TypeInfo)]
struct Apple<T, U> {
    #[codec(compact)]
    one: Color<U>,   // <– works with a "naked" generic, `U`, but not like this
    two: Texture<T, U>,
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    // When this test fails it could mean that https://github.com/rust-lang/rust/issues/81785 is fixed
    assert_type_info::<Apple<u8, u16>>();
}

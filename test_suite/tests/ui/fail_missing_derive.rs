use info::{self as scale_info};
use scale_info::TypeInfo;

enum PawType<Paw> {
    Big(Paw),
    Small(Paw),
}
#[derive(TypeInfo)]
#[scale_info(crate = info)]
struct Cat<Tail, Ear, Paw> {
    tail: Tail,
    ears: [Ear; 3],
    paws: PawType<Paw>,
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<Cat<bool, u8, u16>>();
}

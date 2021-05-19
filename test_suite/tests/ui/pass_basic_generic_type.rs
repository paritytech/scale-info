use scale_info::TypeInfo;

#[allow(dead_code)]
#[derive(TypeInfo)]
enum PawType<Paw> {
    Big(Paw),
    Small(Paw),
}
#[derive(TypeInfo)]
struct Cat<Tail, Ear, Paw> {
    _tail: Tail,
    _ears: [Ear; 3],
    _paws: PawType<Paw>,
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<Cat<bool, u8, u16>>();
}

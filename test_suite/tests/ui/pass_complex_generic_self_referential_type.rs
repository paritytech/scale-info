use scale_info::TypeInfo;


#[derive(TypeInfo)]
struct Nested<P> {
    _pos: P,
}

#[derive(TypeInfo)]
struct Is<N> {
    _nested: N,
}

#[derive(TypeInfo)]
struct That<I, S> {
    _is: I,
    _selfie: S,
}

#[derive(TypeInfo)]
struct Thing<T> {
    _that: T,
}

#[derive(TypeInfo)]
struct Other<T> {
    _thing: T,
}

#[derive(TypeInfo)]
struct Selfie<Pos> {
    _another: Box<Selfie<Pos>>,
    _pos: Pos,
    _nested: Box<Other<Thing<That<Is<Nested<Pos>>, Selfie<Pos>>>>>,
}

fn assert_type_info<T: TypeInfo + 'static>() {}

fn main() {
    assert_type_info::<Selfie<bool>>();
}

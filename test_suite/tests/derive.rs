// Copyright 2019-2020 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

use scale_info::prelude::boxed::Box;

use pretty_assertions::assert_eq;
use scale_info::{
    build::*,
    tuple_meta_type,
    Path,
    Type,
    TypeInfo,
};

fn assert_type<T, E>(expected: E)
where
    T: TypeInfo + ?Sized,
    E: Into<Type>,
{
    assert_eq!(T::type_info(), expected.into());
}

macro_rules! assert_type {
    ( $ty:ty, $expected:expr ) => {{
        assert_type::<$ty, _>($expected)
    }};
}

#[test]
fn struct_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct S<T, U> {
        pub t: T,
        pub u: U,
    }

    let struct_type = Type::builder()
        .path(Path::new("S", "derive"))
        .type_params(tuple_meta_type!(bool, u8))
        .composite(
            Fields::named()
                .field_of::<bool>("t", "T")
                .field_of::<u8>("u", "U"),
        );

    assert_type!(S<bool, u8>, struct_type);

    // With "`Self` typed" fields

    type SelfTyped = S<Box<S<bool, u8>>, bool>;

    let self_typed_type = Type::builder()
        .path(Path::new("S", "derive"))
        .type_params(tuple_meta_type!(Box<S<bool, u8>>, bool))
        .composite(
            Fields::named()
                .field_of::<Box<S<bool, u8>>>("t", "T")
                .field_of::<bool>("u", "U"),
        );
    assert_type!(SelfTyped, self_typed_type);
}

#[test]
fn tuple_struct_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct S<T>(T);

    let ty = Type::builder()
        .path(Path::new("S", "derive"))
        .type_params(tuple_meta_type!(bool))
        .composite(Fields::unnamed().field_of::<bool>("T"));

    assert_type!(S<bool>, ty);
}

#[test]
fn unit_struct_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct S;

    let ty = Type::builder()
        .path(Path::new("S", "derive"))
        .composite(Fields::unit());

    assert_type!(S, ty);
}

#[test]
fn c_like_enum_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    enum E {
        A,
        B = 10,
    }

    let ty = Type::builder()
        .path(Path::new("E", "derive"))
        .variant(Variants::fieldless().variant("A", 0u64).variant("B", 10u64));

    assert_type!(E, ty);
}

#[test]
fn enum_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    enum E<T> {
        A(T),
        B { b: T },
        C,
    }

    let ty = Type::builder()
        .path(Path::new("E", "derive"))
        .type_params(tuple_meta_type!(bool))
        .variant(
            Variants::with_fields()
                .variant("A", Fields::unnamed().field_of::<bool>("T"))
                .variant("B", Fields::named().field_of::<bool>("b", "T"))
                .variant_unit("C"),
        );

    assert_type!(E<bool>, ty);
}

#[test]
fn recursive_type_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    pub enum Tree {
        Leaf { value: i32 },
        Node { right: Box<Tree>, left: Box<Tree> },
    }

    let ty = Type::builder().path(Path::new("Tree", "derive")).variant(
        Variants::with_fields()
            .variant("Leaf", Fields::named().field_of::<i32>("value", "i32"))
            .variant(
                "Node",
                Fields::named()
                    .field_of::<Box<Tree>>("right", "Box<Tree>")
                    .field_of::<Box<Tree>>("left", "Box<Tree>"),
            ),
    );

    assert_type!(Tree, ty);
}

#[test]
fn fields_with_type_alias() {
    type BoolAlias = bool;

    #[allow(unused)]
    #[derive(TypeInfo)]
    struct S {
        a: BoolAlias,
    }

    let ty = Type::builder()
        .path(Path::new("S", "derive"))
        .composite(Fields::named().field_of::<BoolAlias>("a", "BoolAlias"));

    assert_type!(S, ty);
}

#[test]
fn associated_types_derive_without_bounds() {
    trait Types {
        type A;
    }
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Assoc<T: Types> {
        a: T::A,
    }

    #[derive(TypeInfo)]
    enum ConcreteTypes {}
    impl Types for ConcreteTypes {
        type A = bool;
    }

    let struct_type = Type::builder()
        .path(Path::new("Assoc", "derive"))
        .type_params(tuple_meta_type!(ConcreteTypes))
        .composite(Fields::named().field_of::<bool>("a", "T::A"));

    assert_type!(Assoc<ConcreteTypes>, struct_type);
}

#[test]
fn adds_proper_trait_bounds() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Cat {
        tail: bool,
        ears: u8,
    }
    fn assert_type_info<T: TypeInfo + 'static>() {};

    assert_type_info::<Cat>();
}

#[test]
fn adds_type_info_trait_bounds_for_all_generics() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    enum PawType<Paw> {
        Big(Paw),
        Small(Paw),
    }
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Cat<Tail, Ear, Paw> {
        tail: Tail,
        ears: [Ear; 3],
        paws: PawType<Paw>,
    }

    fn assert_type_info<T: TypeInfo + 'static>() {};
    assert_type_info::<Cat<bool, u8, u16>>();
}

#[test]
fn adds_correct_bounds_to_self_referential_types() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Nested<P> {
        pos: P,
    };
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Is<N> {
        nexted: N,
    };
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct That<I, S> {
        is: I,
        selfie: S,
    };
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Thing<T> {
        that: T,
    };
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Other<T> {
        thing: T,
    };
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Selfie<Pos> {
        another: Box<Selfie<Pos>>,
        pos: Pos,
        nested: Box<Other<Thing<That<Is<Nested<Pos>>, Selfie<Pos>>>>>,
    }

    fn assert_type_info<T: TypeInfo + 'static>() {};
    assert_type_info::<Selfie<bool>>();
}
#[test]
fn self_referential() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Meee {
        me: Box<Meee>,
    }
    fn assert_type_info<T: TypeInfo + 'static>() {};
    assert_type_info::<Meee>();
}

#[test]
fn user_error_is_compilation_error() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_missing_derive.rs");
    t.compile_fail("tests/ui/fail_non_static_lifetime.rs");
    t.compile_fail("tests/ui/fail_unions.rs");
}

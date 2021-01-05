// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

use pretty_assertions::assert_eq;
use scale::Encode;
use scale_info::{
    build::*,
    prelude::boxed::Box,
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
fn c_like_enum_derive_with_scale_index_set() {
    #[allow(unused)]
    #[derive(TypeInfo, Encode)]
    enum E {
        A,
        B = 10,
        #[codec(index = 13)]
        C,
    }

    let ty = Type::builder().path(Path::new("E", "derive")).variant(
        Variants::fieldless()
            .variant("A", 0u64)
            .variant("B", 10u64)
            .variant("C", 13u64),
    );

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
fn scale_compact_types_work_in_structs() {
    #[allow(unused)]
    #[derive(Encode, TypeInfo)]
    struct Dense {
        a: u8,
        #[codec(compact)]
        b: u16,
    }

    let dense = Type::builder()
        .path(Path::new("Dense", "derive"))
        .composite(
            Fields::named()
                .field_of::<u8>("a", "u8")
                .field_of::<u16>("b", "u16")
                .compact(),
        );

    assert_type!(Dense, dense);
}

#[test]
fn scale_compact_types_work_in_enums() {
    #[allow(unused)]
    #[derive(Encode, TypeInfo)]
    enum MutilatedMultiAddress<AccountId, AccountIndex> {
        Id(AccountId),
        Index(#[codec(compact)] AccountIndex),
        Address32([u8; 32]),
    }

    let ty = Type::builder()
        .path(Path::new("MutilatedMultiAddress", "derive"))
        .type_params(tuple_meta_type!(u8, u16))
        .variant(
            Variants::with_fields()
                .variant("Id", Fields::unnamed().field_of::<u8>("AccountId"))
                .variant(
                    "Index",
                    Fields::unnamed().field_of::<u16>("AccountIndex").compact(),
                )
                .variant(
                    "Address32",
                    Fields::unnamed().field_of::<[u8; 32]>("[u8; 32]"),
                ),
        );

    assert_type!(MutilatedMultiAddress<u8, u16>, ty);
}

#[test]
fn struct_fields_marked_scale_skip_are_skipped() {
    #[allow(unused)]
    #[derive(TypeInfo, Encode)]
    struct Skippy {
        a: u8,
        #[codec(skip)]
        b: u16,
        c: u32,
    }

    let ty = Type::builder()
        .path(Path::new("Skippy", "derive"))
        .composite(
            Fields::named()
                .field_of::<u8>("a", "u8")
                .field_of::<u32>("c", "u32"),
        );
    assert_type!(Skippy, ty);
}

#[test]
fn enum_variants_marked_scale_skip_are_skipped() {
    #[allow(unused)]
    #[derive(TypeInfo, Encode)]
    enum Skippy {
        A,
        #[codec(skip)]
        B,
        C,
    }

    let ty = Type::builder()
        .path(Path::new("Skippy", "derive"))
        .variant(Variants::fieldless().variant("A", 0).variant("C", 2));
    assert_type!(Skippy, ty);
}

#[test]
fn enum_variants_with_fields_marked_scale_skip_are_skipped() {
    #[allow(unused)]
    #[derive(TypeInfo, Encode)]
    enum Skippy {
        #[codec(skip)]
        Apa,
        Bajs {
            #[codec(skip)]
            a: u8,
            b: bool,
        },
        Coo(bool),
    }

    let ty = Type::builder().path(Path::new("Skippy", "derive")).variant(
        Variants::with_fields()
            .variant("Bajs", Fields::named().field_of::<bool>("b", "bool"))
            .variant("Coo", Fields::unnamed().field_of::<bool>("bool")),
    );
    assert_type!(Skippy, ty);
}

#[rustversion::nightly]
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_missing_derive.rs");
    t.compile_fail("tests/ui/fail_non_static_lifetime.rs");
    t.compile_fail("tests/ui/fail_unions.rs");
    t.pass("tests/ui/pass_self_referential.rs");
    t.pass("tests/ui/pass_basic_generic_type.rs");
    t.pass("tests/ui/pass_complex_generic_self_referential_type.rs");
}

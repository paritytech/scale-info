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
    prelude::{
        boxed::Box,
        marker::PhantomData,
        vec::Vec,
    },
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
fn phantom_data_is_part_of_the_type_info() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct P<T> {
        a: u8,
        m: PhantomData<T>,
    }

    let ty = Type::builder()
        .path(Path::new("P", "derive"))
        .type_params(tuple_meta_type!(bool))
        .composite(
            Fields::named()
                .field_of::<u8>("a", "u8")
                .field_of::<PhantomData<bool>>("m", "PhantomData<T>"),
        );

    assert_type!(P<bool>, ty);
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
        D,
        #[codec(index = 14)]
        E = 15,
    }

    let ty = Type::builder().path(Path::new("E", "derive")).variant(
        Variants::fieldless()
            .variant("A", 0)
            .variant("B", 10)
            .variant("C", 13)
            .variant("D", 3)
            .variant("E", 14),
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
    struct Assoc<'bar, T: Types> {
        a: T::A,
        b: &'bar u64,
    }

    #[derive(TypeInfo)]
    enum ConcreteTypes {}
    impl Types for ConcreteTypes {
        type A = bool;
    }

    let struct_type = Type::builder()
        .path(Path::new("Assoc", "derive"))
        .type_params(tuple_meta_type!(ConcreteTypes))
        .composite(
            Fields::named()
                .field_of::<bool>("a", "T::A")
                .field_of::<u64>("b", "&'static u64"),
        );

    assert_type!(Assoc<ConcreteTypes>, struct_type);
}

#[test]
fn associated_types_named_like_the_derived_type_works() {
    trait Types {
        type Assoc;
    }
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Assoc<T: Types> {
        a: Vec<T::Assoc>,
        b: Vec<<T>::Assoc>,
        c: T::Assoc,
        d: <T>::Assoc,
    }

    #[derive(TypeInfo)]
    enum ConcreteTypes {}
    impl Types for ConcreteTypes {
        type Assoc = bool;
    }

    let struct_type = Type::builder()
        .path(Path::new("Assoc", "derive"))
        .type_params(tuple_meta_type!(ConcreteTypes))
        .composite(
            Fields::named()
                .field_of::<Vec<bool>>("a", "Vec<T::Assoc>")
                .field_of::<Vec<bool>>("b", "Vec<<T>::Assoc>")
                .field_of::<bool>("c", "T::Assoc")
                .field_of::<bool>("d", "<T>::Assoc"),
        );

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

    let ty_alt = Type::builder()
        .path(Path::new("Dense", "derive"))
        .composite(
            Fields::named()
                .field_of::<u8>("a", "u8")
                .compact_of::<u16>("b", "u16"),
        );
    assert_type!(Dense, ty_alt);
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
                .variant("Index", Fields::unnamed().compact_of::<u16>("AccountIndex"))
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

#[test]
fn type_parameters_with_default_bound_works() {
    trait Formy {
        type Tip;
    }
    #[derive(TypeInfo)]
    pub enum MetaFormy {}
    impl Formy for MetaFormy {
        type Tip = u8;
    }

    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Bat<TTT: Formy = MetaFormy> {
        one: TTT,
    }

    let ty = Type::builder()
        .path(Path::new("Bat", "derive"))
        .type_params(tuple_meta_type!(MetaFormy))
        .composite(Fields::named().field_of::<MetaFormy>("one", "TTT"));

    assert_type!(Bat<MetaFormy>, ty);
}

#[test]
fn whitespace_scrubbing_works() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct A {
        a: (u8, (bool, u8)),
    }

    let ty = Type::builder()
        .path(Path::new("A", "derive"))
        .composite(Fields::named().field_of::<(u8, (bool, u8))>("a", "(u8, (bool, u8))"));

    assert_type!(A, ty);
}

#[test]
fn custom_bounds() {
    // TODO: this test is dumb. It's a copy of Basti's equivalent in `parity-scale-codec` but I
    // don't think it can work for us. I need a proper example of when custom bounds are needed.
    // As-is, this test is simply setting the same bounds as the derive would have, which is pretty
    // pointless.
    #[allow(unused)]
    #[derive(TypeInfo)]
    #[scale_info(bounds(T: Default + TypeInfo + 'static, N: TypeInfo + 'static))]
    struct Hey<T, N> {
        ciao: Greet<T>,
        ho: N,
    }

    #[derive(TypeInfo)]
    #[scale_info(bounds(T: TypeInfo + 'static))]
    struct Greet<T> {
        marker: PhantomData<T>,
    }

    #[derive(TypeInfo, Default)]
    struct SomeType;

    let ty = Type::builder()
        .path(Path::new("Hey", "derive"))
        .type_params(tuple_meta_type!(SomeType, u16))
        .composite(
            Fields::named()
                .field_of::<Greet<SomeType>>("ciao", "Greet<T>")
                .field_of::<u16>("ho", "N"),
        );

    assert_type!(Hey<SomeType, u16>, ty);
}

#[rustversion::nightly]
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_missing_derive.rs");
    t.compile_fail("tests/ui/fail_unions.rs");
    t.compile_fail("tests/ui/fail_use_codec_attrs_without_deriving_encode.rs");
    t.compile_fail("tests/ui/fail_with_invalid_codec_attrs.rs");
    t.pass("tests/ui/pass_with_valid_codec_attrs.rs");
    t.pass("tests/ui/pass_non_static_lifetime.rs");
    t.pass("tests/ui/pass_self_referential.rs");
    t.pass("tests/ui/pass_basic_generic_type.rs");
    t.pass("tests/ui/pass_complex_generic_self_referential_type.rs");
}

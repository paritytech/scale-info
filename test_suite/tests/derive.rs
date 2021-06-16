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
    /// Type docs.
    /// Multiline.
    struct S<T, U> {
        /// Field docs.
        pub t: T,
        pub u: U,
    }

    let struct_type = Type::builder()
        .path(Path::new("S", "derive"))
        .type_params(tuple_meta_type!(bool, u8))
        .docs(&["Type docs.", "Multiline."])
        .composite(
            Fields::named()
                .field(|f| {
                    f.ty::<bool>()
                        .name("t")
                        .type_name("T")
                        .docs(&["Field docs."])
                })
                .field(|f| f.ty::<u8>().name("u").type_name("U")),
        );

    assert_type!(S<bool, u8>, struct_type);

    // With "`Self` typed" fields

    type SelfTyped = S<Box<S<bool, u8>>, bool>;

    let self_typed_type = Type::builder()
        .path(Path::new("S", "derive"))
        .type_params(tuple_meta_type!(Box<S<bool, u8>>, bool))
        .docs(&["Type docs.", "Multiline."])
        .composite(
            Fields::named()
                .field(|f| {
                    f.ty::<Box<S<bool, u8>>>()
                        .name("t")
                        .type_name("T")
                        .docs(&["Field docs."])
                })
                .field(|f| f.ty::<bool>().name("u").type_name("U")),
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
                .field(|f| f.ty::<u8>().name("a").type_name("u8"))
                .field(|f| {
                    f.ty::<PhantomData<bool>>()
                        .name("m")
                        .type_name("PhantomData<T>")
                }),
        );

    assert_type!(P<bool>, ty);
}

#[test]
fn tuple_struct_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    /// Type docs.
    struct S<T>(
        /// Unnamed field docs.
        T,
    );

    let ty = Type::builder()
        .path(Path::new("S", "derive"))
        .type_params(tuple_meta_type!(bool))
        .docs(&["Type docs."])
        .composite(
            Fields::unnamed()
                .field(|f| f.ty::<bool>().type_name("T").docs(&["Unnamed field docs."])),
        );

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
    /// Enum docs.
    enum E {
        /// Unit variant.
        A,
        /// Variant with discriminant.
        B = 10,
    }

    let ty = Type::builder()
        .path(Path::new("E", "derive"))
        .docs(&["Enum docs."])
        .variant(
            Variants::new()
                .variant("A", |v| v.discriminant(0).docs(&["Unit variant."]))
                .variant("B", |v| {
                    v.discriminant(10).docs(&["Variant with discriminant."])
                }),
        );

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
        Variants::new()
            .variant("A", |v| v.discriminant(0))
            .variant("B", |v| v.discriminant(10))
            .variant("C", |v| v.discriminant(13))
            .variant("D", |v| v.discriminant(3))
            .variant("E", |v| v.discriminant(14)),
    );

    assert_type!(E, ty);
}

#[test]
fn enum_derive() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    /// Enum docs.
    enum E<T> {
        /// Unnamed fields variant.
        A(
            /// Unnamed field.
            T,
        ),
        /// Named fields variant.
        B {
            /// Named field.
            b: T,
        },
        /// Unit variant.
        C,
    }

    let ty = Type::builder()
        .path(Path::new("E", "derive"))
        .type_params(tuple_meta_type!(bool))
        .docs(&["Enum docs."])
        .variant(
            Variants::new()
                .variant("A", |v| {
                    v.fields(Fields::unnamed().field(|f| {
                        f.ty::<bool>().type_name("T").docs(&["Unnamed field."])
                    }))
                    .docs(&["Unnamed fields variant."])
                })
                .variant("B", |v| {
                    v.fields(Fields::named().field(|f| {
                        f.ty::<bool>()
                            .name("b")
                            .type_name("T")
                            .docs(&["Named field."])
                    }))
                    .docs(&["Named fields variant."])
                })
                .variant("C", |v| v.docs(&["Unit variant."])),
        );

    assert_type!(E<bool>, ty);
}

#[test]
fn enum_derive_with_codec_index() {
    #[allow(unused)]
    #[derive(TypeInfo, Encode)]
    enum E<T> {
        #[codec(index = 5)]
        A(T),
        #[codec(index = 0)]
        B { b: T },
        #[codec(index = 13)]
        C,
    }

    let ty = Type::builder()
        .path(Path::new("E", "derive"))
        .type_params(tuple_meta_type!(bool))
        .variant(
            Variants::new()
                .variant("A", |v| {
                    v.index(5).fields(
                        Fields::unnamed().field(|f| f.ty::<bool>().type_name("T")),
                    )
                })
                .variant("B", |v| {
                    v.index(0).fields(
                        Fields::named()
                            .field(|f| f.ty::<bool>().name("b").type_name("T")),
                    )
                })
                .variant("C", |v| v.index(13)),
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
        Variants::new()
            .variant("Leaf", |v| {
                v.fields(
                    Fields::named()
                        .field(|f| f.ty::<i32>().name("value").type_name("i32")),
                )
            })
            .variant("Node", |v| {
                v.fields(
                    Fields::named()
                        .field(|f| {
                            f.ty::<Box<Tree>>().name("right").type_name("Box<Tree>")
                        })
                        .field(|f| {
                            f.ty::<Box<Tree>>().name("left").type_name("Box<Tree>")
                        }),
                )
            }),
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

    let ty = Type::builder().path(Path::new("S", "derive")).composite(
        Fields::named().field(|f| f.ty::<BoolAlias>().name("a").type_name("BoolAlias")),
    );

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
                .field(|f| f.ty::<bool>().name("a").type_name("T::A"))
                .field(|f| f.ty::<u64>().name("b").type_name("&'static u64")),
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
                .field(|f| f.ty::<Vec<bool>>().name("a").type_name("Vec<T::Assoc>"))
                .field(|f| f.ty::<Vec<bool>>().name("b").type_name("Vec<<T>::Assoc>"))
                .field(|f| f.ty::<bool>().name("c").type_name("T::Assoc"))
                .field(|f| f.ty::<bool>().name("d").type_name("<T>::Assoc")),
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
                .field(|f| f.ty::<u8>().name("a").type_name("u8"))
                .field(|f| f.compact::<u16>().name("b").type_name("u16")),
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
            Variants::new()
                .variant("Id", |v| {
                    v.fields(
                        Fields::unnamed().field(|f| f.ty::<u8>().type_name("AccountId")),
                    )
                })
                .variant("Index", |v| {
                    v.fields(
                        Fields::unnamed()
                            .field(|f| f.compact::<u16>().type_name("AccountIndex")),
                    )
                })
                .variant("Address32", |v| {
                    v.fields(
                        Fields::unnamed()
                            .field(|f| f.ty::<[u8; 32]>().type_name("[u8; 32]")),
                    )
                }),
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
                .field(|f| f.ty::<u8>().name("a").type_name("u8"))
                .field(|f| f.ty::<u32>().name("c").type_name("u32")),
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

    let ty = Type::builder().path(Path::new("Skippy", "derive")).variant(
        Variants::new()
            .variant("A", |v| v.discriminant(0))
            .variant("C", |v| v.discriminant(2)),
    );
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
        Variants::new()
            .variant("Bajs", |v| {
                v.fields(
                    Fields::named().field(|f| f.ty::<bool>().name("b").type_name("bool")),
                )
            })
            .variant("Coo", |v| {
                v.fields(Fields::unnamed().field(|f| f.ty::<bool>().type_name("bool")))
            }),
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
        .composite(
            Fields::named().field(|f| f.ty::<MetaFormy>().name("one").type_name("TTT")),
        );

    assert_type!(Bat<MetaFormy>, ty);
}

#[test]
fn whitespace_scrubbing_works() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct A {
        a: (u8, (bool, u8)),
    }

    let ty =
        Type::builder()
            .path(Path::new("A", "derive"))
            .composite(Fields::named().field(|f| {
                f.ty::<(u8, (bool, u8))>()
                    .name("a")
                    .type_name("(u8, (bool, u8))")
            }));

    assert_type!(A, ty);
}

#[test]
fn doc_capture_works() {
    //! Que pasa
    #[allow(unused)]
    #[derive(TypeInfo)]
    #[doc(hidden)]
    struct S {
        #[doc = " Field a"]
        a: bool,
        #[doc(primitive)]
        b: u8,
        ///     Indented
        c: u16,
    }

    let ty = Type::builder().path(Path::new("S", "derive")).composite(
        Fields::named()
            .field(|f| {
                f.ty::<bool>()
                    .name("a")
                    .type_name("bool")
                    .docs(&["Field a"])
            })
            .field(|f| f.ty::<u8>().name("b").type_name("u8").docs(&[]))
            .field(|f| {
                f.ty::<u16>()
                    .name("c")
                    .type_name("u16")
                    .docs(&["    Indented"])
            }),
    );

    assert_type!(S, ty);
}

#[rustversion::nightly]
#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail_*");
    t.pass("tests/ui/pass_*");
}

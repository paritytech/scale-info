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
// #![feature(box_syntax)]
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

// TODO: dp Make useful or remove
#[test]
fn substrate_example() {
    use scale::{
        // Decode,
        Encode, Compact,
    };
    use scale_info::prelude::vec::Vec;
    // #[allow(unused)]
    // type AccountIndex = u32;
    /// A multi-format address wrapper for on-chain accounts.
    #[allow(unused)]
    // #[derive(Encode, Decode, PartialEq, Eq, Clone, TypeInfo)]
    #[derive(Encode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(Hash))]
    pub enum MultiAddress<AccountId, AccountIndex> {
        /// It's an account ID (pubkey).
        Id(AccountId),
        /// It's an account index.
        // Index(#[codec(compact)] AccountIndex),
        Index(Compact<AccountIndex>),
        /// It's some arbitrary raw bytes.
        Raw(Vec<u8>),
        /// It's a 32 byte representation.
        Address32([u8; 32]),
        /// Its a 20 byte representation.
        Address20([u8; 20]),
    }

    let _ma = MultiAddress::<u64, u32>::Id(32);
}

// #[ignore]
// fn substrate_example_expanded() {
//     use scale::{Decode, Encode, Compact};
//     use scale_info::prelude::vec::Vec;
//     /// A multi-format address wrapper for on-chain accounts.
//     #[allow(unused)]
//     pub enum MultiAddress<AccountId, AccountIndex> {
//         /// It's an account ID (pubkey).
//         Id(AccountId),
//         /// It's an account index.
//         // Index(#[codec(compact)] AccountIndex),
//         Index(Compact<AccountIndex>),
//         /// It's some arbitrary raw bytes.
//         Raw(Vec<u8>),
//         /// It's a 32 byte representation.
//         Address32([u8; 32]),
//         /// Its a 20 byte representation.
//         Address20([u8; 20]),
//     }

//     const _IMPL_TYPE_INFO_FOR_MultiAddress: () = {
//         impl<
//                 AccountId: ::scale_info::TypeInfo + 'static,
//                 AccountIndex: ::scale_info::TypeInfo + 'static,
//             > ::scale_info::TypeInfo for MultiAddress<AccountId, AccountIndex>
//         where
//             AccountId: ::scale_info::TypeInfo + 'static,
//             AccountIndex: ::scale_info::TypeInfo + 'static,
//         {
//             type Identity = Self;
//             fn type_info() -> ::scale_info::Type {
//                 ::scale_info::Type::builder()
//                     .path(::scale_info::Path::new("MultiAddress", "derive"))
//                     .type_params(<[_]>::into_vec(box [
//                         ::scale_info::meta_type::<AccountId>(),
//                         ::scale_info::meta_type::<AccountIndex>(),
//                     ]))
//                     .variant(
//                         ::scale_info::build::Variants::with_fields()
//                             .variant(
//                                 "Id",
//                                 ::scale_info::build::Fields::unnamed()
//                                     .field_of::<AccountId>("AccountId"),
//                             )
//                             .variant(
//                                 "Index",
//                                 ::scale_info::build::Fields::unnamed()
//                                     .field_of::<AccountIndex>("AccountIndex"),
//                             )
//                             .variant(
//                                 "Raw",
//                                 ::scale_info::build::Fields::unnamed()
//                                     .field_of::<Vec<u8>>("Vec<u8>"),
//                             )
//                             .variant(
//                                 "Address32",
//                                 ::scale_info::build::Fields::unnamed()
//                                     .field_of::<[u8; 32]>("[u8; 32]"),
//                             )
//                             .variant(
//                                 "Address20",
//                                 ::scale_info::build::Fields::unnamed()
//                                     .field_of::<[u8; 20]>("[u8; 20]"),
//                             ),
//                     )
//                     .into()
//             }
//         };
//     };

//     const _: () = {
//         #[allow(unknown_lints)]
//         #[allow(rust_2018_idioms)]
//         extern crate scale as _parity_scale_codec;
//         impl<AccountId, AccountIndex> _parity_scale_codec::Encode for MultiAddress<AccountId, AccountIndex>
//         where
//             AccountId: _parity_scale_codec::Encode,
//             AccountId: _parity_scale_codec::Encode,
//             AccountIndex: _parity_scale_codec::HasCompact,
//         {
//             fn encode_to<__CodecOutputEdqy: _parity_scale_codec::Output>(
//                 &self,
//                 __codec_dest_edqy: &mut __CodecOutputEdqy,
//             ) {
//                 match *self {
//                     MultiAddress::Id(ref aa) => {
//                         __codec_dest_edqy.push_byte(0usize as u8);
//                         __codec_dest_edqy.push(aa);
//                     }
//                     MultiAddress::Index(ref aa) => {
//                         __codec_dest_edqy.push_byte(1usize as u8);
//                         {
//                             __codec_dest_edqy.push (&<<AccountIndex as _parity_scale_codec::HasCompact>::Type as _parity_scale_codec::EncodeAsRef< '_ , AccountIndex >>::RefType::from(aa));
//                         }
//                     }
//                     MultiAddress::Raw(ref aa) => {
//                         __codec_dest_edqy.push_byte(2usize as u8);
//                         __codec_dest_edqy.push(aa);
//                     }
//                     MultiAddress::Address32(ref aa) => {
//                         __codec_dest_edqy.push_byte(3usize as u8);
//                         __codec_dest_edqy.push(aa);
//                     }
//                     MultiAddress::Address20(ref aa) => {
//                         __codec_dest_edqy.push_byte(4usize as u8);
//                         __codec_dest_edqy.push(aa);
//                     }
//                     _ => (),
//                 }
//             }
//         }
//         impl<AccountId, AccountIndex> _parity_scale_codec::EncodeLike
//             for MultiAddress<AccountId, AccountIndex>
//         where
//             AccountId: _parity_scale_codec::Encode,
//             AccountId: _parity_scale_codec::Encode,
//             AccountIndex: _parity_scale_codec::HasCompact,
//         {
//         }
//     };
//     let _ma = MultiAddress::<u64, u32>::Id(32);
// }

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

use scale_info::prelude::{
    marker::PhantomData,
    boxed::Box,
};

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
fn no_phantom_types_are_derived_in_structs() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct P<T> {
        pub a: u8,
        pub marker: PhantomData<T>,
    }

    let ty = Type::builder()
        .path(Path::new("P", "derive"))
        .composite(Fields::named().field_of::<u8>("a", "u8"));

    assert_type!(P<bool>, ty);
}

#[test]
fn no_phantom_types_are_derived_in_tuple_structs() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Tuppy<T>(u8, PhantomData<T>);

    let tuppy = Type::builder()
        .path(Path::new("Tuppy", "derive"))
        .composite(Fields::unnamed().field_of::<u8>("u8"));

    assert_type!(Tuppy<()>, tuppy);
}

#[test]
fn no_phantoms_are_derived_in_struct_with_tuple_members() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct WithTuples<TT, UU> {
        a: (u8, PhantomData<TT>, u32, PhantomData<UU>),
    }

    let ty =
        Type::builder()
            .path(Path::new("WithTuples", "derive"))
            .composite(Fields::named().field_of::<(u8, u32)>(
                "a",
                "(u8, PhantomData<TT>, u32, PhantomData<UU>)",
            ));

    assert_type!(WithTuples<u16, u64>, ty);
}

#[test]
fn no_phantom_types_are_derived_in_enums() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Chocolate<Flavour> {
        flavour: PhantomData<Flavour>,
    }
    #[allow(unused)]
    #[derive(TypeInfo)]
    enum Choices<F> {
        Nutella,
        RealThing(Chocolate<F>),
        Marshmallow(PhantomData<F>),
    };

    let ty = Type::builder()
        .path(Path::new("Choices", "derive"))
        .variant(
            Variants::with_fields()
                .variant_unit("Nutella")
                .variant(
                    "RealThing",
                    Fields::unnamed().field_of::<Chocolate<bool>>("Chocolate<F>"),
                )
                .variant_unit("Marshmallow"),
        );

    assert_type!(Choices<bool>, ty);
}

#[test]
fn complex_enum_with_phantoms() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    enum Cake<Icing, Topping, Filling> {
        A((PhantomData<Icing>, u8, PhantomData<Topping>, Filling)),
        B,
    }

    let ty = Type::builder()
        .path(Path::new("Cake", "derive"))
        .type_params(tuple_meta_type!(u16))
        .variant(
            Variants::with_fields()
                .variant(
                    "A",
                    Fields::unnamed().field_of::<(u8, u16)>(
                        "(PhantomData<Icing>, u8, PhantomData<Topping>, Filling)",
                    ),
                )
                .variant_unit("B"),
        );
    assert_type!(Cake<bool, bool, u16>, ty);
}

#[test]
fn no_nested_phantom_types_are_derived_structs() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct Door<Size> {
        size: PhantomData<Size>,
        b: u16,
    };
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct House<TDoor> {
        a: u8,
        door: TDoor,
    };

    let house = Type::builder()
        .path(Path::new("House", "derive"))
        .type_params(tuple_meta_type!(Door<bool>))
        .composite(
            Fields::named()
                .field_of::<u8>("a", "u8")
                .field_of::<Door<bool>>("door", "TDoor"),
        );

    assert_type!(House<Door<bool>>, house);
}

#[test]
fn no_phantoms_in_nested_tuples() {
    #[allow(unused)]
    #[derive(TypeInfo)]
    struct A<T> {
        is_a: bool,
        teeth: (u8, u16, (u32, (PhantomData<T>, bool))),
    }

    let ty = Type::builder().path(Path::new("A", "derive")).composite(
        Fields::named()
            .field_of::<bool>("is_a", "bool")
            .field_of::<(u8, u16, (u32, (bool)))>(
                "teeth",
                "(u8, u16, (u32, (PhantomData<T>, bool)))",
            ),
    );

    assert_type!(A<u64>, ty);
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

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

use std::collections::HashSet;

use alloc::vec::Vec;
use proc_macro2::Ident;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::Visit,
    Generics,
    Result,
    Type,
    WhereClause,
};

/// Generates a where clause for a `TypeInfo` impl, adding `TypeInfo + 'static` bounds to all
/// relevant generic types including associated types (e.g. `T::A: TypeInfo`), correctly dealing
/// with self-referential types.
pub fn make_where_clause<'a>(
    input_ident: &'a Ident, // The type we're deriving TypeInfo for
    generics: &'a Generics, // The type params to our type
    data: &'a syn::Data,    // The body of the type
    scale_info: &Ident,     // Crate name
    parity_scale_codec: &Ident, // Crate name
) -> Result<(WhereClause, HashSet<Ident>)> {
    let mut where_clause = generics.where_clause.clone().unwrap_or_else(|| {
        WhereClause {
            where_token: <syn::Token![where]>::default(),
            predicates: Punctuated::new(),
        }
    });

    let type_params = generics.type_params();
    let ty_params_ids = type_params
        .map(|type_param| type_param.ident.clone())
        .collect::<Vec<Ident>>();

    if ty_params_ids.is_empty() {
        return Ok((where_clause, HashSet::new()))
    }

    let types = collect_types_to_bind(input_ident, data, &ty_params_ids)?;
    // println!("[DDD] nr types appearing in source={:?}", types.len());
    let mut need_compact_bounds: HashSet<Ident> = HashSet::new();
    let mut need_normal_bounds: HashSet<Ident> = HashSet::new();
    use syn::WherePredicate;
    let mut where_predicates: HashSet<WherePredicate> = HashSet::new();
    types.into_iter().for_each(|(ty, is_compact, _)| {
        // Compact types need two bounds: `T: HasCompact` and `<T as
        // HasCompact>::Type: TypeInfo + 'static`

        // TODO: should pass in a `&mut HashSet` from outside the loop to ensure we avoid repetitions
        let type_params_in_type = collect_generic_arguments(&ty);
        if is_compact {
            println!("[make_where_clause]   is Compact, ty={:?}  adding HasCompact bounds", ty);
            println!("[make_where_clause]   is Compact, type_params_in_type={:?}", type_params_in_type);
            where_predicates.insert(parse_quote!(#ty : :: #parity_scale_codec ::HasCompact));
            where_predicates.insert(parse_quote!(<#ty as :: #parity_scale_codec ::HasCompact>::Type : :: #scale_info ::TypeInfo + 'static));

            // where_clause
            //     .predicates
            //     .push(parse_quote!(#ty : :: #parity_scale_codec ::HasCompact));
            // where_clause
            //     .predicates
            //     .push(parse_quote!(<#ty as :: #parity_scale_codec ::HasCompact>::Type : :: #scale_info ::TypeInfo + 'static));
            // for tp in type_params_in_type {
            //     where_clause.predicates.push(parse_quote!( #tp : 'static ))
            // }
            need_compact_bounds.extend(type_params_in_type);
        } else {
            // println!("[DDD] ty={:?} is NOT Compact, adding TypeInfo bound", ty);
            where_predicates.insert(parse_quote!(#ty : :: #scale_info ::TypeInfo + 'static));

            // where_clause
            //     .predicates
            //     .push(parse_quote!(#ty : :: #scale_info ::TypeInfo + 'static));
            // for tp in type_params_in_type {
            //     where_clause.predicates.push(parse_quote!( #tp : :: #scale_info ::TypeInfo + 'static ))
            // }
            need_normal_bounds.extend(type_params_in_type);
        }
    });
    println!(
        "[make_where_clause]   need_compact_bounds={:?}",
        need_compact_bounds
    );
    println!(
        "[make_where_clause]   need_normal_bounds={:?}",
        need_normal_bounds
    );
    // TODO: make issue:
    // Loop over the type params given to the type and add `TypeInfo` and
    // `'static` bounds. Type params that are used in fields/variants that are
    // `Compact` need only the `'static` bound.
    // Note that if a type parameter appears "naked" in a field/variant, e.g.
    // `struct A<T> { one: T }`, we will end up adding the bounds twice, once
    // above as it appears in a field and once again below as it's one of the
    // type params of `A`. Type params that appear as parameters to other types,
    // e.g. `struct A<T> { one: PhantomData<T> }` do not get double bounds.

    // The reason we do this "double looping" over first generics in used in
    // fields/variants and then generics is that a type can have a type
    // parameter bound to a trait, e.g. `T: SomeTrait`, that is not used as-is
    // in the fields/variants but whose associated type **is**. Something like
    // this:
    // `struct A<T: SomeTrait> { one: T::SomeAssoc, }`
    // When deriving `TypeInfo` for `A`, the first loop above adds
    // `T::SomeAssoc: TypeInfo + 'static`, but we also need
    // `T: TypeInfo + 'static`.
    // Hence the second loop.
    generics.type_params().into_iter().for_each(|type_param| {
        let ident = type_param.ident.clone();
        // Find the type parameter in the list of types that appear in the
        // fields/variants and check if it is used in a `Compact` field/variant.
        // If yes, only add the `'static` bound, else add both `TypeInfo` and
        // `'static` bounds.

        let mut bounds = type_param.bounds.clone();
        if need_compact_bounds.contains(&ident) {
            bounds.push(parse_quote!('static));
        } else {
            bounds.push(parse_quote!(:: #scale_info ::TypeInfo));
            bounds.push(parse_quote!('static));
        }
        where_predicates.insert(parse_quote!( #ident : #bounds));

        // let mut bounds = type_param.bounds.clone();
        // if need_compact_bounds.contains(&ident) {
        //     bounds.push(parse_quote!('static));
        //     where_clause
        //         .predicates
        //         .push(parse_quote!( #ident : #bounds));
        // } else {
        //     // let mut bounds = type_param.bounds.clone();
        //     bounds.push(parse_quote!(:: #scale_info ::TypeInfo));
        //     bounds.push(parse_quote!('static));
        //     where_clause
        //         .predicates
        //         .push(parse_quote!( #ident : #bounds));
        // }

        // let is_compact = types2
        //     .iter()
        //     .filter(|ty| {
        //         if let Some(i) = &ty.2 {
        //             i == &ident
        //         } else {
        //             false
        //         }
        //     })
        //     .any(|ty| ty.1 );
        // if is_compact {
        //     // println!("[DDD] ident {:?} is used for a field that is Compact; not adding TypeInfo bound", ident);
        //     let mut bounds = type_param.bounds.clone();
        //     bounds.push(parse_quote!('static));
        //     where_clause
        //         .predicates
        //         .push(parse_quote!( #ident : #bounds));

        // } else {
        //     // println!("[DDD] ident {:?} is used for a field that is NOT Compact. Adding bounds", ident);
        //     let mut bounds = type_param.bounds.clone();
        //     bounds.push(parse_quote!(:: #scale_info ::TypeInfo));
        //     bounds.push(parse_quote!('static));
        //     where_clause
        //         .predicates
        //         .push(parse_quote!( #ident : #bounds));
        // }
    });

    where_predicates.extend(
        need_compact_bounds
            .iter()
            .map(|tp_ident| parse_quote!( #tp_ident : 'static )),
    );
    where_predicates.extend(
        need_normal_bounds.iter().map(
            |tp_ident| parse_quote!( #tp_ident : :: #scale_info ::TypeInfo + 'static ),
        ),
    );
    where_clause.predicates.extend(where_predicates);

    Ok((where_clause, need_compact_bounds))
}

/// Visits the ast and checks if the given type contains one of the given
/// idents.
// TODO: what if the type contains more than one of the idents?
// TODO: return just `Option<Ident>`?
fn type_contains_idents(ty: &Type, idents: &[Ident]) -> (bool, Option<Ident>) {
    struct ContainIdents<'a> {
        result: (bool, Option<Ident>),
        idents: &'a [Ident],
    }
    impl<'a, 'ast> Visit<'ast> for ContainIdents<'a> {
        fn visit_ident(&mut self, i: &'ast Ident) {
            if self.idents.iter().any(|id| id == i) {
                self.result = (true, Some(i.clone()));
            }
        }
    }

    let mut visitor = ContainIdents {
        result: (false, None),
        idents,
    };
    visitor.visit_type(ty);
    visitor.result
}

// Should call this on a syn:Field instead so we have access to the Attributes, that way we could collect both `Compact` and generics usage?
// TODO: Can this be done when visiting the type in `type_contains_ident` instead?
// Visit a `Type` and collect all generics used.
// example: given `struct A<T> { thing: SomeType<T> }` will return `T`.
// example: given `struct A<T> { thing: T }` will **not** return `T`.
// TODO: can it be changed to return both `T` in both cases? I think not, proc macros work on the syntax level.
fn collect_generic_arguments(ty: &Type) -> HashSet<Ident> {
    use syn::GenericArgument;
    println!("[collect_generic_arguments]  ty={:?}", ty);
    struct Bla {
        result: HashSet<Ident>,
    }
    impl<'ast> Visit<'ast> for Bla {
        fn visit_generic_argument(&mut self, g: &'ast GenericArgument) {
            println!("collect_generic_arguments={:?}", g);
            if let GenericArgument::Type(syn::Type::Path(syn::TypePath {
                path: t, ..
            })) = g
            {
                println!("collect_generic_arguments, type={:?}", t.get_ident());

                if let Some(ident) = t.get_ident() {
                    self.result.insert(ident.clone());
                }
            }
        }
    }
    let mut visitor = Bla {
        result: HashSet::new(),
    };
    visitor.visit_type(ty);
    println!(
        "[collect_generic_arguments]  found type params={:?}",
        visitor.result
    );

    visitor.result
}

fn visit_fields(ident: &Ident, data: &syn::Data) {
    use syn::{
        Attribute,
        Field,
        Fields,
    };
    struct Bla {
        result: HashSet<Ident>,
    }

    impl<'ast> Visit<'ast> for Bla {
        fn visit_fields(&mut self, fs: &'ast Fields) {
            println!("visit_fields={:#?}", fs);
            for f in fs {
                self.visit_field(f)
            }
        }
        // Nope
        fn visit_attribute(&mut self, node: &'ast Attribute) {
            println!("visit_attribute={:#?}", node);
        }
        // Nope
        fn visit_field(&mut self, i: &'ast Field) {
            println!("visit_field (sing)={:#?}", i);
            for attr in &i.attrs {
                self.visit_attribute(&attr);
            }
        }
    }

    let mut visitor = Bla {
        result: HashSet::new(),
    };
    visitor.visit_data(data);
}

/// Returns all types that must be added to the where clause with a boolean
/// indicating if the field is [`scale::Compact`] or not.
fn collect_types_to_bind(
    input_ident: &Ident,
    data: &syn::Data,
    ty_params: &[Ident],
) -> Result<Vec<(Type, bool, Option<Ident>)>> {
    // Experiment:
    // visit_fields(input_ident, data);
    let types_from_fields =
        |fields: &Punctuated<syn::Field, _>| -> Vec<(Type, bool, Option<Ident>)> {
            fields
                .iter()
                .filter(|field| {
                    // Only add a bound if the type uses a generic.
                    type_contains_idents(&field.ty, &ty_params).0
                &&
                // Remove all remaining types that start/contain the input ident
                // to not have them in the where clause.
                !type_contains_idents(&field.ty, &[input_ident.clone()]).0
                })
                .map(|f| {
                    (
                        f.ty.clone(),
                        super::is_compact(f),
                        type_contains_idents(&f.ty, &ty_params).1,
                    )
                })
                .collect()
        };

    let types = match *data {
        syn::Data::Struct(ref data) => {
            match &data.fields {
                syn::Fields::Named(syn::FieldsNamed { named: fields, .. })
                | syn::Fields::Unnamed(syn::FieldsUnnamed {
                    unnamed: fields, ..
                }) => types_from_fields(fields),
                syn::Fields::Unit => Vec::new(),
            }
        }

        syn::Data::Enum(ref data) => {
            data.variants
                .iter()
                .flat_map(|variant| {
                    match &variant.fields {
                        syn::Fields::Named(syn::FieldsNamed {
                            named: fields, ..
                        })
                        | syn::Fields::Unnamed(syn::FieldsUnnamed {
                            unnamed: fields,
                            ..
                        }) => types_from_fields(fields),
                        syn::Fields::Unit => Vec::new(),
                    }
                })
                .collect()
        }

        syn::Data::Union(ref data) => {
            return Err(syn::Error::new(
                data.union_token.span(),
                "Union types are not supported.",
            ))
        }
    };

    Ok(types)
}

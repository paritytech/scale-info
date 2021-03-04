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

use alloc::vec::Vec;
use proc_macro2::Ident;
use std::collections::HashSet;
use syn::{
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::Visit,
    GenericArgument,
    Generics,
    Result,
    Type,
    WhereClause,
    WherePredicate,
};

/// Generates a where clause for a `TypeInfo` impl, adding `TypeInfo + 'static` bounds to all
/// relevant generic types including associated types (e.g. `T::A: TypeInfo`), correctly dealing
/// with self-referential types.
/// Returns a tuple with a where clause and the set of type params that appear
/// in `Compact` fields/variants in the form `Something<T>` (type params that
/// appear as just `T` are not included)
pub fn make_where_clause<'a>(
    input_ident: &'a Ident,
    generics: &'a Generics,
    data: &'a syn::Data,
    scale_info: &Ident,
    parity_scale_codec: &Ident,
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
    let mut need_compact_bounds: HashSet<Ident> = HashSet::new();
    let mut need_normal_bounds: HashSet<Ident> = HashSet::new();
    let mut where_predicates: HashSet<WherePredicate> = HashSet::new();
    types.into_iter().for_each(|(ty, is_compact, _)| {
        // Compact types need two bounds:
        //  `T: HasCompact` and
        //  `<T as HasCompact>::Type: TypeInfo + 'static`
        let generic_arguments = collect_generic_arguments(&ty);
        if is_compact {
            where_predicates.insert(parse_quote!(#ty : :: #parity_scale_codec ::HasCompact));
            where_predicates.insert(parse_quote!(<#ty as :: #parity_scale_codec ::HasCompact>::Type : :: #scale_info ::TypeInfo + 'static));
            need_compact_bounds.extend(generic_arguments);
        } else {
            where_predicates.insert(parse_quote!(#ty : :: #scale_info ::TypeInfo + 'static));
            need_normal_bounds.extend(generic_arguments);
        }
    });
    // Loop over the type params given to the type and add `TypeInfo` and
    // `'static` bounds. Type params that are used in fields/variants that are
    // `Compact` need only the `'static` bound.
    // The reason we do this "double looping", first over generics used in
    // fields/variants and then over the generics given in the type definition
    // itself, is that a type can have a type parameter bound to a trait, e.g.
    // `T: SomeTrait`, that is not used as-is in the fields/variants but whose
    // associated type **is**. Something like this:
    // `struct A<T: SomeTrait> { one: T::SomeAssoc, }`
    //
    // When deriving `TypeInfo` for `A`, the first loop above adds
    // `T::SomeAssoc: TypeInfo + 'static`, but we also need
    // `T: TypeInfo + 'static`.
    // Hence the second loop.
    generics.type_params().into_iter().for_each(|type_param| {
        let ident = type_param.ident.clone();
        // Find the type parameter in the list of types that appear in any of the
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

/// Visits the ast for a [`syn::Type`] and checks if it contains one of the given
/// idents.
fn type_contains_idents(ty: &Type, idents: &[Ident]) -> Option<Ident> {
    struct ContainIdents<'a> {
        idents: &'a [Ident],
        result: Option<Ident>,
    }
    impl<'a, 'ast> Visit<'ast> for ContainIdents<'a> {
        fn visit_ident(&mut self, i: &'ast Ident) {
            if self.idents.iter().any(|id| id == i) {
                self.result = Some(i.clone());
            }
        }
    }

    let mut visitor = ContainIdents {
        idents,
        result: None,
    };
    visitor.visit_type(ty);
    visitor.result
}

/// Visit a `Type` and collect generic type params used.
/// Given `struct A<T> { thing: SomeType<T> }`, will include `T` in the set.
/// Given `struct A<T> { thing: T }`, will **not** include `T` in the set.
fn collect_generic_arguments(ty: &Type) -> HashSet<Ident> {
    struct GenericsVisitor {
        found: HashSet<Ident>,
    }
    impl<'ast> Visit<'ast> for GenericsVisitor {
        fn visit_generic_argument(&mut self, g: &'ast GenericArgument) {
            if let GenericArgument::Type(syn::Type::Path(syn::TypePath {
                path: t, ..
            })) = g
            {
                if let Some(ident) = t.get_ident() {
                    self.found.insert(ident.clone());
                }
            }
        }
    }
    let mut visitor = GenericsVisitor {
        found: HashSet::new(),
    };
    visitor.visit_type(ty);
    visitor.found
}

/// Returns all types that must be added to the where clause, with a boolean
/// indicating if the field is [`scale::Compact`] or not.
fn collect_types_to_bind(
    input_ident: &Ident,
    data: &syn::Data,
    ty_params: &[Ident],
) -> Result<Vec<(Type, bool, Option<Ident>)>> {
    let types_from_fields =
        |fields: &Punctuated<syn::Field, _>| -> Vec<(Type, bool, Option<Ident>)> {
            fields.iter().fold(Vec::new(), |mut acc, field| {
                // Only add a bound if the type uses a generic.
                let uses_generic = type_contains_idents(&field.ty, &ty_params);
                // Find remaining types that start with/contain the input ident
                // to not have them in the where clause.
                let uses_input_ident =
                    type_contains_idents(&field.ty, &[input_ident.clone()]);
                if uses_generic.is_some() && uses_input_ident.is_none() {
                    acc.push((field.ty.clone(), super::is_compact(field), uses_generic));
                }
                acc
            })
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

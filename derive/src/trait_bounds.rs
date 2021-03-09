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
use syn::{
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    visit::{
        self,
        Visit,
    },
    Generics,
    Result,
    Type,
    TypePath,
    WhereClause,
};

/// Generates a where clause for a `TypeInfo` impl, adding `TypeInfo + 'static` bounds to all
/// relevant generic types including associated types (e.g. `T::A: TypeInfo`), correctly dealing
/// with self-referential types.
pub fn make_where_clause<'a>(
    input_ident: &'a Ident,
    generics: &'a Generics,
    data: &'a syn::Data,
    scale_info: &Ident,
    parity_scale_codec: &Ident,
) -> Result<WhereClause> {
    let mut where_clause = generics.where_clause.clone().unwrap_or_else(|| {
        WhereClause {
            where_token: <syn::Token![where]>::default(),
            predicates: Punctuated::new(),
        }
    });
    for lifetime in generics.lifetimes() {
        where_clause
            .predicates
            .push(parse_quote!(#lifetime: 'static))
    }

    let type_params = generics.type_params();
    let ty_params_ids = type_params
        .map(|type_param| type_param.ident.clone())
        .collect::<Vec<Ident>>();

    if ty_params_ids.is_empty() {
        return Ok(where_clause)
    }

    let types = collect_types_to_bind(input_ident, data, &ty_params_ids)?;

    types.into_iter().for_each(|(ty, is_compact)| {
        // Compact types need extra bounds, T: HasCompact and <T as
        // HasCompact>::Type: TypeInfo + 'static
        if is_compact {
            where_clause
                .predicates
                .push(parse_quote!(#ty : :: #parity_scale_codec ::HasCompact));
            where_clause
                .predicates
                .push(parse_quote!(<#ty as :: #parity_scale_codec ::HasCompact>::Type : :: #scale_info ::TypeInfo + 'static));
        } else {
            where_clause
                .predicates
                .push(parse_quote!(#ty : :: #scale_info ::TypeInfo + 'static));
        }
    });

    generics.type_params().into_iter().for_each(|type_param| {
        let ident = type_param.ident.clone();
        let mut bounds = type_param.bounds.clone();
        bounds.push(parse_quote!(:: #scale_info ::TypeInfo));
        bounds.push(parse_quote!('static));
        where_clause
            .predicates
            .push(parse_quote!( #ident : #bounds));
    });

    Ok(where_clause)
}

/// Visits the ast and checks if the given type contains one of the given
/// idents.
fn type_contains_idents(ty: &Type, idents: &[Ident]) -> bool {
    struct ContainIdents<'a> {
        result: bool,
        idents: &'a [Ident],
    }

    impl<'a, 'ast> Visit<'ast> for ContainIdents<'a> {
        fn visit_ident(&mut self, i: &'ast Ident) {
            if self.idents.iter().any(|id| id == i) {
                self.result = true;
            }
        }
    }

    let mut visitor = ContainIdents {
        result: false,
        idents,
    };
    visitor.visit_type(ty);
    visitor.result
}

/// Checks if the given type or any containing type path starts with the given ident.
fn type_or_sub_type_path_starts_with_ident(ty: &Type, ident: &Ident) -> bool {
    // Visits the ast and checks if the a type path starts with the given ident.
    struct TypePathStartsWithIdent<'a> {
        result: bool,
        ident: &'a Ident,
    }

    impl<'a, 'ast> Visit<'ast> for TypePathStartsWithIdent<'a> {
        fn visit_type_path(&mut self, i: &'ast TypePath) {
            if i.qself.is_none() {
                if let Some(segment) = i.path.segments.first() {
                    if &segment.ident == self.ident {
                        self.result = true;
                        return
                    }
                }
            }
            visit::visit_type_path(self, i);
        }
    }

    let mut visitor = TypePathStartsWithIdent {
        result: false,
        ident,
    };
    visitor.visit_type(ty);
    visitor.result
}

/// Returns all types that must be added to the where clause with a boolean
/// indicating if the field is [`scale::Compact`] or not.
fn collect_types_to_bind(
    input_ident: &Ident,
    data: &syn::Data,
    ty_params: &[Ident],
) -> Result<Vec<(Type, bool)>> {
    let types_from_fields = |fields: &Punctuated<syn::Field, _>| -> Vec<(Type, bool)> {
        fields
            .iter()
            .filter(|field| {
                // Only add a bound if the type uses a generic.
                type_contains_idents(&field.ty, &ty_params)
                &&
                // Remove all remaining types that start/contain the input ident
                // to not have them in the where clause.
                !type_or_sub_type_path_starts_with_ident(&field.ty, &input_ident)
            })
            .map(|f| (f.ty.clone(), super::is_compact(f)))
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

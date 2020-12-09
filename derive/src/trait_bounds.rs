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

use alloc::vec::Vec;
use proc_macro2::Ident;
use syn::{
    parse_quote,
    spanned::Spanned,
    visit::Visit,
    Generics,
    Result,
    Type,
};

/// Visits the ast and checks if one of the given idents is found.
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

/// Checks if the given type contains one of the given idents.
fn type_contains_idents(ty: &Type, idents: &[Ident]) -> bool {
    let mut visitor = ContainIdents {
        result: false,
        idents,
    };
    visitor.visit_type(ty);
    visitor.result
}

/// Adds a `TypeInfo + 'static` bound to all relevant generic types, correctly
/// dealing with self-referential types.
pub fn add(input_ident: &Ident, generics: &mut Generics, data: &syn::Data) -> Result<()> {
    let ty_params =
        generics.type_params_mut().fold(Vec::new(), |mut acc, p| {
            p.bounds.push(parse_quote!(::scale_info::TypeInfo));
            p.bounds.push(parse_quote!('static));
            acc.push(p.ident.clone());
            acc
        });

    if ty_params.is_empty() {
        return Ok(())
    }

    let types = collect_types_to_bind(input_ident, data, &ty_params)?;

    if !types.is_empty() {
        let where_clause = generics.make_where_clause();

        types.into_iter().for_each(|ty| {
            where_clause
                .predicates
                .push(parse_quote!(#ty : ::scale_info::TypeInfo + 'static))
        });
    }

    Ok(())
}

/// Returns all types that must be added to the where clause with the respective trait bound.
fn collect_types_to_bind(
    input_ident: &Ident,
    data: &syn::Data,
    ty_params: &[Ident],
) -> Result<Vec<Type>> {
    let type_filter = |field: &syn::Field| {
        // Only add a bound if the type uses a generic
        type_contains_idents(&field.ty, &ty_params)
        &&
        // Remove all remaining types that start/contain the input ident
        // to not have them in the where clause.
        !type_contains_idents(&field.ty, &[input_ident.clone()])
    };
    collect_types(
        &data,
        type_filter,
        // TODO: dp double check we don't need to filter anything here
        |_| true
    )
}

fn collect_types(
    data: &syn::Data,
    type_filter: impl Fn(&syn::Field) -> bool,
    variant_filter: impl Fn(&syn::Variant) -> bool,
) -> Result<Vec<syn::Type>> {
    use syn::*;

    let types = match *data {
        Data::Struct(ref data) => {
            match &data.fields {
                Fields::Named(FieldsNamed { named: fields, .. })
                | Fields::Unnamed(FieldsUnnamed {
                    unnamed: fields, ..
                }) => {
                    fields
                        .iter()
                        .filter(|f| type_filter(f))
                        .map(|f| f.ty.clone())
                        .collect()
                }

                Fields::Unit => Vec::new(),
            }
        }

        Data::Enum(ref data) => {
            data.variants
                .iter()
                .filter(|variant| variant_filter(variant))
                .flat_map(|variant| {
                    match &variant.fields {
                        Fields::Named(FieldsNamed { named: fields, .. })
                        | Fields::Unnamed(FieldsUnnamed {
                            unnamed: fields, ..
                        }) => {
                            fields
                                .iter()
                                .filter(|f| type_filter(f))
                                .map(|f| f.ty.clone())
                                .collect()
                        }

                        Fields::Unit => Vec::new(),
                    }
                })
                .collect()
        }

        Data::Union(ref data) => {
            return Err(Error::new(
                data.union_token.span(),
                "Union types are not supported.",
            ))
        }
    };

    Ok(types)
}

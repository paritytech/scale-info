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
    punctuated::Punctuated,
    spanned::Spanned,
    visit::Visit,
    Generics,
    Result,
    Type,
};

/// Adds a `TypeInfo + 'static` bound to all relevant generic types including
/// associated types (e.g. `T::A: TypeInfo`), correctly dealing with
/// self-referential types.
pub fn add(input_ident: &Ident, generics: &mut Generics, data: &syn::Data) -> Result<()> {
    let ty_params = generics.type_params_mut().fold(Vec::new(), |mut acc, p| {
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

/// Returns all types that must be added to the where clause with the respective
/// trait bound.
fn collect_types_to_bind(
    input_ident: &Ident,
    data: &syn::Data,
    ty_params: &[Ident],
) -> Result<Vec<Type>> {
    let types_from_fields = |fields: &Punctuated<syn::Field, _>| -> Vec<syn::Type> {
        fields
            .iter()
            .filter(|field| {
                // Only add a bound if the type uses a generic.
                type_contains_idents(&field.ty, &ty_params)
                &&
                // Remove all remaining types that start/contain the input ident
                // to not have them in the where clause.
                !type_contains_idents(&field.ty, &[input_ident.clone()])
            })
            .map(|f| f.ty.clone())
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

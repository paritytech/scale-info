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

extern crate alloc;
extern crate proc_macro;

mod impl_wrapper;

use alloc::{
    string::{
        String,
        ToString,
    },
    vec::Vec,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{
        Error,
        Result,
    },
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    AngleBracketedGenericArguments,
    Data,
    DataEnum,
    DataStruct,
    DeriveInput,
    Expr,
    ExprLit,
    Field,
    Fields,
    GenericArgument,
    Ident,
    Lit,
    PathArguments,
    TypePath,
    Variant,
};

#[proc_macro_derive(TypeInfo)]
pub fn type_info(input: TokenStream) -> TokenStream {
    match generate(input.into()) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate(input: TokenStream2) -> Result<TokenStream2> {
    let mut tokens = quote! {};
    tokens.extend(generate_type(input)?);
    Ok(tokens)
}

fn generate_type(input: TokenStream2) -> Result<TokenStream2> {
    let mut ast: DeriveInput = syn::parse2(input.clone())?;

    ast.generics.type_params_mut().for_each(|p| {
        p.bounds.push(parse_quote!(_scale_info::TypeInfo));
        p.bounds.push(parse_quote!('static));
    });

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let ast: DeriveInput = syn::parse2(input.clone())?;
    let (build_type, phantom_params) = match &ast.data {
        Data::Struct(ref s) => generate_composite_type(s),
        Data::Enum(ref e) => generate_variant_type(e),
        Data::Union(_) => return Err(Error::new_spanned(input, "Unions not supported")),
    };

    let generic_type_ids = ast.generics.type_params().fold(Vec::new(), |mut acc, ty| {
        let ty_ident = &ty.ident;
        if !phantom_params.contains(ty_ident) {
            acc.push(quote! {
                _scale_info::meta_type::<#ty_ident>()
            });
        }
        acc
    });

    let type_info_impl = quote! {
        impl #impl_generics _scale_info::TypeInfo for #ident #ty_generics #where_clause {
            type Identity = Self;
            fn type_info() -> _scale_info::Type {
                _scale_info::Type::builder()
                    .path(_scale_info::Path::new(stringify!(#ident), module_path!()))
                    .type_params(__core::vec![ #( #generic_type_ids ),* ])
                    .#build_type
                    .into()
            }
        }
    };

    Ok(impl_wrapper::wrap(ident, "TYPE_INFO", type_info_impl))
}

type FieldsList = Punctuated<Field, Comma>;

/// Given a type `Path`, find all generics params that are used in a
/// `PhantomData` and return them as a `Vec` of `Ident`.
/// E.g. given `utils::stuff::Thing<PhantomData<P>, Q, PhantomData<R>>`, return
/// `vec!["P", "R"]`
fn find_phantoms_in_path(path: &syn::Path) -> Vec<Ident> {
    path.segments
        .iter()
        .filter(|seg| seg.ident == "PhantomData")
        .fold(Vec::new(), |mut acc, seg| {
            if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                args,
                ..
            }) = &seg.arguments
            {
                for arg in args {
                    if let GenericArgument::Type(syn::Type::Path(TypePath {
                        path, ..
                    })) = arg
                    {
                        for segment in path.segments.iter() {
                            acc.push(segment.ident.clone())
                        }
                    }
                }
            }
            acc
        })
}

/// Given a list of tuple elements, removes all `PhantomData` and returns a new
/// tuple along with any type parameters used in `PhantomData`.
fn scrub_phantoms_from_tuple(
    tuple_elements: &Punctuated<syn::Type, Comma>,
    paren_token: &syn::token::Paren
) -> (syn::Type, Vec<Ident>)
{
    let mut phantom_params = Vec::new();
    let mut punctuated: Punctuated<_, Comma> = Punctuated::new();
    for tuple_element in tuple_elements {
        match tuple_element {
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let phantoms = find_phantoms_in_path(path);
                if phantoms.is_empty() {
                    punctuated.push(tuple_element.clone());
                } else {
                    phantom_params.extend(phantoms);
                }
            }
            syn::Type::Tuple(syn::TypeTuple { elems, paren_token }) => {
                let (sub_tuple, phantoms) = scrub_phantoms_from_tuple(elems, paren_token);
                punctuated.push(sub_tuple);
                phantom_params.extend(phantoms);
            },
            // TODO: (dp) can there be anything but types and tuples in a tuple?
            _ => unreachable!("Only types and tuples can appear in tuples")
        }
    }
    let tuple = syn::Type::Tuple(syn::TypeTuple {
        elems: punctuated,
        paren_token: *paren_token,
    });

    (tuple, phantom_params)
}

/// Generate code for each field of a struct (named or unnamed).
/// Filters out `PhantomData` fields and returns the type parameters used as a
/// `Vec` of `Ident` so that other code can match up the type parameters in the
/// generics section to `PhantomData` (and omit them).
fn generate_fields(fields: &FieldsList) -> (Vec<TokenStream2>, Vec<Ident>) {
    // Collect all type params used with `PhantomData` anywhere in the type.
    let mut phantom_params = Vec::new();
    let field_tokens = fields.iter().fold(Vec::new(), |mut acc, field| {
        let (ty, ident) = (&field.ty, &field.ident);
        let type_name = clean_type_string(&quote!(#ty).to_string());
        match ty {
            // Regular types, e.g. `struct A<T> { a: PhantomData<T>, b: u8 }`
            // Check for phantom types and skip them; record any
            // PhantomData type params.
            syn::Type::Path(syn::TypePath { path, .. }) => {
                let phantoms = find_phantoms_in_path(path);
                if phantoms.is_empty() {
                    let tokens = if let Some(i) = ident {
                        quote! {.field_of::<#ty>(stringify!(#i), #type_name) }
                    } else {
                        quote! {.field_of::<#ty>(#type_name) }
                    };
                    acc.push(tokens);
                } else {
                    phantom_params.extend(phantoms);
                }
            }
            // If the type is a tuple, check it for any `PhantomData` elements and rebuild a new tuple without them.
            syn::Type::Tuple(syn::TypeTuple { elems, paren_token }) => {
                let (tuple, phantoms) = scrub_phantoms_from_tuple(elems, paren_token);
                let tokens = if let Some(i) = ident {
                    quote! { .field_of::<#tuple>(stringify!(#i), #type_name) }
                } else {
                    quote! { .field_of::<#tuple>(#type_name)}
                };
                phantom_params.extend(phantoms);
                acc.push(tokens)
            }
            _ => {
                let tokens = if let Some(i) = ident {
                    quote! { .field_of::<#ty>(stringify!(#i), #type_name) }
                } else {
                    quote! { .field_of::<#ty>(#type_name)}
                };
                acc.push(tokens)
            }
        }
        acc
    });
    (field_tokens, phantom_params)
}

fn clean_type_string(input: &str) -> String {
    input
        .replace(" ::", "::")
        .replace(":: ", "::")
        .replace(" ,", ",")
        .replace(" ;", ";")
        .replace(" [", "[")
        .replace("[ ", "[")
        .replace(" ]", "]")
        .replace(" (", "(")
        // put back a space so that `a: u8, (bool, u8)` isn't turned into `a: u8,(bool, u8)`
        .replace(",(", ", (")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
}

/// Generate code for a composite type. Returns the token stream and all the
/// generic types used in `PhantomData` members.
fn generate_composite_type(data_struct: &DataStruct) -> (TokenStream2, Vec<Ident>) {
    let (fields, phantom_params) = match data_struct.fields {
        Fields::Named(ref fs) => {
            let (fields, phantoms) = generate_fields(&fs.named);
            (quote! { named()#( #fields )* }, phantoms)
        }
        Fields::Unnamed(ref fs) => {
            let (fields, phantoms) = generate_fields(&fs.unnamed);
            (quote! { unnamed()#( #fields )* }, phantoms)
        }
        Fields::Unit => (quote! { unit() }, Vec::new()),
    };
    (
        quote! { composite(_scale_info::build::Fields::#fields) },
        phantom_params,
    )
}

type VariantList = Punctuated<Variant, Comma>;

fn generate_c_like_enum_def(variants: &VariantList) -> TokenStream2 {
    let variants = variants.into_iter().enumerate().map(|(i, v)| {
        let name = &v.ident;
        let discriminant = if let Some((
            _,
            Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int),
                ..
            }),
        )) = &v.discriminant
        {
            match lit_int.base10_parse::<u64>() {
                Ok(i) => i,
                Err(err) => return err.to_compile_error(),
            }
        } else {
            i as u64
        };
        quote! {
            .variant(stringify!(#name), #discriminant)
        }
    });
    quote! {
        variant(
            _scale_info::build::Variants::fieldless()
                #( #variants )*
        )
    }
}

fn is_c_like_enum(variants: &VariantList) -> bool {
    // any variant has an explicit discriminant
    variants.iter().any(|v| v.discriminant.is_some()) ||
        // all variants are unit
        variants.iter().all(|v| match v.fields {
            Fields::Unit => true,
            _ => false,
        })
}

/// Build enum variants while keeping track of any `PhantomData` members so we can skip them later.
fn generate_variant_type(data_enum: &DataEnum) -> (TokenStream2, Vec<Ident>) {
    let variants = &data_enum.variants;

    if is_c_like_enum(&variants) {
        return (generate_c_like_enum_def(variants), Vec::new())
    }

    let mut phantom_params = Vec::new();
    let variants = variants.into_iter().map(|v| {
        let ident = &v.ident;
        let v_name = quote! {stringify!(#ident) };
        match v.fields {
            Fields::Named(ref fs) => {
                let (fields, phantoms) = generate_fields(&fs.named);
                phantom_params.extend(phantoms);
                quote! {
                    .variant(
                        #v_name,
                        _scale_info::build::Fields::named()
                            #( #fields)*
                    )
                }
            }
            Fields::Unnamed(ref fs) => {
                let (fields, phantoms) = generate_fields(&fs.unnamed);
                phantom_params.extend(phantoms);
                quote! {
                    .variant(
                        #v_name,
                        _scale_info::build::Fields::unnamed()
                            #( #fields)*
                    )
                }
            }
            Fields::Unit => {
                quote! {
                    .variant_unit(#v_name)
                }
            }
        }
    });
    (
        quote! {
            variant(
                _scale_info::build::Variants::with_fields()
                    #( #variants)*
            )
        },
        phantom_params,
    )
}

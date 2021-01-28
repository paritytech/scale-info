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

extern crate alloc;
extern crate proc_macro;

mod impl_wrapper;
mod trait_bounds;

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
    visit_mut::VisitMut,
    Data,
    DataEnum,
    DataStruct,
    DeriveInput,
    Expr,
    ExprLit,
    Field,
    Fields,
    Lifetime,
    Lit,
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

    let ident = &ast.ident;

    ast.generics
        .lifetimes_mut()
        .for_each(|l| *l = parse_quote!('static));

    let (_, ty_generics, _) = ast.generics.split_for_impl();
    let where_clause = trait_bounds::make_where_clause(ident, &ast.generics, &ast.data)?;

    let generic_type_ids = ast.generics.type_params().map(|ty| {
        let ty_ident = &ty.ident;
        quote! {
            ::scale_info::meta_type::<#ty_ident>()
        }
    });

    let ast: DeriveInput = syn::parse2(input.clone())?;
    let build_type = match &ast.data {
        Data::Struct(ref s) => generate_composite_type(s),
        Data::Enum(ref e) => generate_variant_type(e),
        Data::Union(_) => return Err(Error::new_spanned(input, "Unions not supported")),
    };
    let generic_types = ast.generics.type_params();
    let type_info_impl = quote! {
        impl <#( #generic_types ),*> ::scale_info::TypeInfo for #ident #ty_generics #where_clause {
            type Identity = Self;
            fn type_info() -> ::scale_info::Type {
                ::scale_info::Type::builder()
                    .path(::scale_info::Path::new(stringify!(#ident), module_path!()))
                    .type_params(::scale_info::prelude::vec![ #( #generic_type_ids ),* ])
                    .#build_type
                    .into()
            }
        }
    };

    Ok(impl_wrapper::wrap(ident, "TYPE_INFO", type_info_impl))
}

type FieldsList = Punctuated<Field, Comma>;

fn generate_fields(fields: &FieldsList) -> Vec<TokenStream2> {
    fields
        .iter()
        .map(|f| {
            let (ty, ident) = (&f.ty, &f.ident);
            // Replace any field lifetime params with `static to prevent "unnecessary lifetime parameter"
            // warning. Any lifetime parameters are specified as 'static in the type of the impl.
            struct StaticLifetimesReplace;
            impl VisitMut for StaticLifetimesReplace {
                fn visit_lifetime_mut(&mut self, lifetime: &mut Lifetime) {
                    *lifetime = parse_quote!('static)
                }
            }
            let mut ty = ty.clone();
            StaticLifetimesReplace.visit_type_mut(&mut ty);

            let type_name = clean_type_string(&quote!(#ty).to_string());

            if let Some(i) = ident {
                quote! {
                    .field_of::<#ty>(stringify!(#i), #type_name)
                }
            } else {
                quote! {
                    .field_of::<#ty>(#type_name)
                }
            }
        })
        .collect()
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
        // put back a space so that `a: (u8, (bool, u8))` isn't turned into `a: (u8,(bool, u8))`
        .replace(",(", ", (")
        .replace("( ", "(")
        .replace(" )", ")")
        .replace(" <", "<")
        .replace("< ", "<")
        .replace(" >", ">")
        .replace("& \'", "&'")
}

fn generate_composite_type(data_struct: &DataStruct) -> TokenStream2 {
    let fields = match data_struct.fields {
        Fields::Named(ref fs) => {
            let fields = generate_fields(&fs.named);
            quote! { named()#( #fields )* }
        }
        Fields::Unnamed(ref fs) => {
            let fields = generate_fields(&fs.unnamed);
            quote! { unnamed()#( #fields )* }
        }
        Fields::Unit => {
            quote! {
                unit()
            }
        }
    };
    quote! {
        composite(::scale_info::build::Fields::#fields)
    }
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
            ::scale_info::build::Variants::fieldless()
                #( #variants )*
        )
    }
}

fn is_c_like_enum(variants: &VariantList) -> bool {
    // any variant has an explicit discriminant
    variants.iter().any(|v| v.discriminant.is_some()) ||
        // all variants are unit
        variants.iter().all(|v| matches!(v.fields, Fields::Unit))
}

fn generate_variant_type(data_enum: &DataEnum) -> TokenStream2 {
    let variants = &data_enum.variants;

    if is_c_like_enum(&variants) {
        return generate_c_like_enum_def(variants)
    }

    let variants = variants.into_iter().map(|v| {
        let ident = &v.ident;
        let v_name = quote! {stringify!(#ident) };
        match v.fields {
            Fields::Named(ref fs) => {
                let fields = generate_fields(&fs.named);
                quote! {
                    .variant(
                        #v_name,
                        ::scale_info::build::Fields::named()
                            #( #fields)*
                    )
                }
            }
            Fields::Unnamed(ref fs) => {
                let fields = generate_fields(&fs.unnamed);
                quote! {
                    .variant(
                        #v_name,
                        ::scale_info::build::Fields::unnamed()
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
    quote! {
        variant(
            ::scale_info::build::Variants::with_fields()
                #( #variants)*
        )
    }
}

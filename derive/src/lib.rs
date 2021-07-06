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

extern crate alloc;
extern crate proc_macro;

mod attr;
mod trait_bounds;
mod utils;

use proc_macro::TokenStream;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
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
    Field,
    Fields,
    Ident,
    Lifetime,
    Variant,
};

#[proc_macro_derive(TypeInfo, attributes(scale_info))]
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
    let ast: DeriveInput = syn::parse2(input.clone())?;

    let attrs = attr::Attributes::from_ast(&ast)?;

    let scale_info = crate_name_ident("scale-info")?;

    let ident = &ast.ident;

    let where_clause = trait_bounds::make_where_clause(
        &attrs,
        ident,
        &ast.generics,
        &ast.data,
        &scale_info,
    )?;

    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let type_params = ast.generics.type_params().map(|tp| {
        let ty_ident = &tp.ident;
        let ty = if attrs.skip_type_params().map_or(true, |skip| !skip.skip(tp)) {
            quote! { ::core::Option::Some(:: #scale_info ::meta_type::<#ty_ident>()) }
        } else {
            quote! { ::core::Option::None }
        };
        quote! {
            :: #scale_info ::TypeParameter::new(::core::stringify!(#ty_ident), #ty)
        }
    });

    let build_type = match &ast.data {
        Data::Struct(ref s) => generate_composite_type(s, &scale_info),
        Data::Enum(ref e) => generate_variant_type(e, &scale_info),
        Data::Union(_) => return Err(Error::new_spanned(input, "Unions not supported")),
    };
    let docs = generate_docs(&ast.attrs);

    let type_info_impl = quote! {
        impl #impl_generics :: #scale_info ::TypeInfo for #ident #ty_generics #where_clause {
            type Identity = Self;
            fn type_info() -> :: #scale_info ::Type {
                :: #scale_info ::Type::builder()
                    .path(:: #scale_info ::Path::new(::core::stringify!(#ident), ::core::module_path!()))
                    .type_params(:: #scale_info ::prelude::vec![ #( #type_params ),* ])
                    #docs
                    .#build_type
            }
        }
    };

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #type_info_impl;
        };
    })
}

/// Get the name of a crate, to be robust against renamed dependencies.
fn crate_name_ident(name: &str) -> Result<Ident> {
    proc_macro_crate::crate_name(name)
        .map(|crate_name| {
            use proc_macro_crate::FoundCrate::*;
            match crate_name {
                Itself => Ident::new("self", Span::call_site()),
                Name(name) => Ident::new(&name, Span::call_site()),
            }
        })
        .map_err(|e| syn::Error::new(Span::call_site(), &e))
}

type FieldsList = Punctuated<Field, Comma>;

fn generate_fields(fields: &FieldsList) -> Vec<TokenStream2> {
    fields
        .iter()
        .filter(|f| !utils::should_skip(&f.attrs))
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
            let docs = generate_docs(&f.attrs);
            let type_of_method = if utils::is_compact(f) {
                quote!(compact)
            } else {
                quote!(ty)
            };
            let name = if let Some(ident) = ident {
                quote!(.name(::core::stringify!(#ident)))
            } else {
                quote!()
            };
            quote!(
                .field(|f| f
                    .#type_of_method::<#ty>()
                    #name
                    .type_name(#type_name)
                    #docs
                )
            )
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
        .replace("&\'", "&'")
}

fn generate_composite_type(data_struct: &DataStruct, scale_info: &Ident) -> TokenStream2 {
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
        composite(:: #scale_info ::build::Fields::#fields)
    }
}

type VariantList = Punctuated<Variant, Comma>;

fn generate_c_like_enum_def(variants: &VariantList, scale_info: &Ident) -> TokenStream2 {
    let variants = variants
        .into_iter()
        .enumerate()
        .filter(|(_, v)| !utils::should_skip(&v.attrs))
        .map(|(i, v)| {
            let name = &v.ident;
            let discriminant = utils::variant_index(v, i);
            let docs = generate_docs(&v.attrs);
            quote! {
                .variant(::core::stringify!(#name), |v|
                    v
                        .discriminant(#discriminant as ::core::primitive::u64)
                        #docs
                )
            }
        });
    quote! {
        variant(
            :: #scale_info ::build::Variants::new()
                #( #variants )*
        )
    }
}

fn is_c_like_enum(variants: &VariantList) -> bool {
    // One of the variants has an explicit discriminant, or…
    variants.iter().any(|v| v.discriminant.is_some()) ||
        // …all variants are unit
        variants.iter().all(|v| matches!(v.fields, Fields::Unit))
}

fn generate_variant_type(data_enum: &DataEnum, scale_info: &Ident) -> TokenStream2 {
    let variants = &data_enum.variants;

    if is_c_like_enum(variants) {
        return generate_c_like_enum_def(variants, scale_info)
    }

    let variants = variants
        .into_iter()
        .filter(|v| !utils::should_skip(&v.attrs))
        .map(|v| {
            let ident = &v.ident;
            let v_name = quote! {::core::stringify!(#ident) };
            let docs = generate_docs(&v.attrs);
            let index = utils::maybe_index(v).map(|i| quote!(.index(#i)));

            let fields = match v.fields {
                Fields::Named(ref fs) => {
                    let fields = generate_fields(&fs.named);
                    quote! {
                        :: #scale_info::build::Fields::named()
                            #( #fields )*
                    }
                }
                Fields::Unnamed(ref fs) => {
                    let fields = generate_fields(&fs.unnamed);
                    quote! {
                        :: #scale_info::build::Fields::unnamed()
                            #( #fields )*
                    }
                }
                Fields::Unit => {
                    quote! {
                        :: #scale_info::build::Fields::unit()
                    }
                }
            };

            quote! {
                .variant(#v_name, |v|
                    v
                        .fields(#fields)
                        #docs
                        #index
                )
            }
        });
    quote! {
        variant(
            :: #scale_info ::build::Variants::new()
                #( #variants )*
        )
    }
}

#[cfg(feature = "docs")]
fn generate_docs(attrs: &[syn::Attribute]) -> Option<TokenStream2> {
    let docs = attrs
        .iter()
        .filter_map(|attr| {
            if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
                if meta.path.get_ident().map_or(false, |ident| ident == "doc") {
                    let lit = &meta.lit;
                    let doc_lit = quote!(#lit).to_string();
                    let trimmed_doc_lit =
                        doc_lit.trim_start_matches(r#"" "#).trim_end_matches('"');
                    let lit: syn::Lit = parse_quote!(#trimmed_doc_lit);
                    Some(lit)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Some(quote! {
        .docs(&[ #( #docs ),* ])
    })
}

#[cfg(not(feature = "docs"))]
fn generate_docs(_: &[syn::Attribute]) -> Option<TokenStream2> {
    None
}

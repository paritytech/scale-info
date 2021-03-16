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

mod internals;
mod trait_bounds;
#[macro_use]
mod fragment;
mod bound;

use alloc::{
    string::{
        String,
        ToString,
    },
    vec::Vec,
};
use internals::{
    ast::{
        self,
        Container,
        Data,
        Style,
    },
    attr,
    replace_receiver,
    Ctxt,
};
use proc_macro::TokenStream;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::quote;
use syn::{
    parse::Error,
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token::Comma,
    visit_mut::VisitMut,
    AttrStyle,
    DataEnum,
    DataStruct,
    DeriveInput,
    Expr,
    ExprLit,
    Field,
    Fields,
    Ident,
    Lifetime,
    Lit,
    Meta,
    MetaList,
    NestedMeta,
    Variant,
};

use fragment::{
    Fragment,
    Stmts,
};

#[proc_macro_derive(TypeInfo, attributes(scale_info))]
pub fn type_info(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    derive_type_info(&mut input)
        .unwrap_or_else(to_compile_errors)
        .into()
    // match generate(input.into()) {
    //     Ok(output) => output.into(),
    //     Err(err) => err.to_compile_error().into(),
    // }
}

fn derive_type_info(input: &mut DeriveInput) -> Result<TokenStream2, Vec<syn::Error>> {
    replace_receiver(input);

    let ctxt = Ctxt::new();
    let cont = match Container::from_ast(&ctxt, input) {
        Some(cont) => cont,
        // None => return Err(vec![ctxt.check().unwrap_err()]), /* TODO: check how serde does the wrapping in Vec */
        None => return Err(ctxt.check().unwrap_err()),
    };
    // TODO: what exactly do we want `check()` to check for our case?
    ctxt.check()?;

    let ident = &cont.ident;
    let params = Parameters::new(&cont);
    // TODO: This was a `Path` in serde, is that a problem?
    let scale_info = crate_name_ident("scale-info").expect("TODO!!");
    // TODO: make this into a method on `Parameters`?
    let generic_type_ids = params.generics.type_params().map(|ty| {
        let ty_ident = &ty.ident;
        quote! {
            :: #scale_info ::meta_type::<#ty_ident>()
        }
    });

    let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();
    let body = Stmts(type_info_body(&cont, &params));

    let impl_block = quote! {
        #[automatically_derived]
        impl #impl_generics :: #scale_info ::TypeInfo for #ident #ty_generics #where_clause {
            type Identity = Self;
            fn type_info() -> :: #scale_info ::Type {
                :: #scale_info ::Type::builder()
                    .path(:: #scale_info ::Path::new(stringify!(#ident), module_path!()))
                    .type_params(:: #scale_info ::prelude::vec![ #( #generic_type_ids ),* ])
                    .#body
            }
        }
    };

    Ok(quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #impl_block;
        };
    })
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}

struct Parameters {
    // TODO: figure out what this means in a `scale_info` context
    /// Variable holding the value being derived. Either `self` for local
    /// types or `__self` for remote types.
    self_var: Ident,

    /// Path to the type the impl is for. Does not include generic parameters.
    this: syn::Path,

    /// Generics including any explicit and inferred bounds for the impl.
    generics: syn::Generics,
    /* TODO: serde has `#[serde(remote = "…")]` – would that be useful to us? Serde sets up a "shadow" type for such types. */

    /* TODO: maybe we need a `is_compact` here? */
}

impl Parameters {
    fn new(cont: &Container) -> Self {
        let self_var = Ident::new("self", Span::call_site());
        let this = cont.ident.clone().into();
        let generics = build_generics(cont);

        Parameters {
            self_var,
            this,
            generics,
        }
    }

    /// Type name to use in error messages and `&'static str` arguments to
    /// various Serializer methods.
    fn type_name(&self) -> String {
        self.this.segments.last().unwrap().ident.to_string()
    }
}

// All the generics in the input, plus a bound `T: Serialize` for each generic
// field type that will be serialized by us.
fn build_generics(cont: &Container) -> syn::Generics {
    // TODO: all the `with_` methods
    let generics = bound::without_defaults(cont.generics);

    let generics =
        bound::with_where_predicates_from_fields(cont, &generics, attr::Field::bound);

    let generics =
        bound::with_where_predicates_from_variants(cont, &generics, attr::Variant::bound);

    match cont.attrs.bound() {
        Some(predicates) => bound::with_where_predicates(&generics, predicates),
        None => {
            bound::with_bound(
                cont,
                &generics,
                needs_bound,
                // TODO: what goes here?
                // TODO: need the proper crate name yes?
                &parse_quote!(scale_info::TypeInfo),
            )
        }
    }
}

fn needs_bound(field: &attr::Field, variant: Option<&attr::Variant>) -> bool {
    field.bound().is_none() && variant.map_or(true, |variant| variant.bound().is_none())
}

fn type_info_body(cont: &Container, params: &Parameters) -> Fragment {
    // TODO: port all the derive_for_* (was serialize_*)
    // This is where we plug in the actual TypeInfo generating code.
    // Each derive_for_* call should return a `Fragment`, which is either an `Expr` or a `Block`

    match &cont.data {
        Data::Enum(variants) => derive_for_enum(params, variants, &cont.attrs),
        Data::Struct(_, _) => todo!()
        // Data::Struct(Style::Struct, fields) => derive_for_struct(params, fields, &cont.attrs),
        // Data::Struct(Style::Tuple, fields) => {
        //     derive_for_tuple_struct(params, fields, &cont.attrs)
        // }
        // Data::Struct(Style::Newtype, fields) => {
        //     derive_for_newtype_struct(params, &fields[0], &cont.attrs)
        // }
        // Data::Struct(Style::Unit, _) => derive_for_unit_struct(&cont.attrs),
    }
}

fn derive_for_enum(
    params: &Parameters,
    variants: &[ast::Variant],
    cattrs: &attr::Container,
) -> Fragment {
    assert!(variants.len() as u64 <= u64::from(u32::max_value()));

    let self_var = &params.self_var;

    let variant_tokens: Vec<_> = variants
        .iter()
        .enumerate()
        .map(|(variant_index, variant)| {
            derive_for_variant(params, variant, variant_index as u32, cattrs)
        })
        .collect();

    // TODO:
    quote_expr! {}
    // quote_expr! {
    //     match *#self_var {
    //         #(#arms)*
    //     }
    // }
}

fn derive_for_variant(
    params: &Parameters,
    variant: &ast::Variant,
    variant_index: u32,
    cattrs: &attr::Container,
) -> TokenStream2 {
    let this = &params.this;
    let variant_ident = &variant.ident;
    quote! {}
    // let case = match variant.style {
    //     Style::Unit => {
    //         quote! {
    //             #this::#variant_ident
    //         }
    //     }
    //     Style::Newtype => {
    //         quote! {
    //             #this::#variant_ident(ref __field0)
    //         }
    //     }
    //     Style::Tuple => {
    //         let field_names = (0..variant.fields.len())
    //             .map(|i| Ident::new(&format!("__field{}", i), Span::call_site()));
    //         quote! {
    //             #this::#variant_ident(#(ref #field_names),*)
    //         }
    //     }
    //     Style::Struct => {
    //         let members = variant.fields.iter().map(|f| &f.member);
    //         quote! {
    //             #this::#variant_ident { #(ref #members),* }
    //         }
    //     }
    // };

    // let body = Match(match cattrs.tag() {
    //     attr::TagType::External => {
    //         serialize_externally_tagged_variant(params, variant, variant_index, cattrs)
    //     }
    //     attr::TagType::Internal { tag } => {
    //         serialize_internally_tagged_variant(params, variant, cattrs, tag)
    //     }
    //     attr::TagType::Adjacent { tag, content } => {
    //         serialize_adjacently_tagged_variant(params, variant, cattrs, tag, content)
    //     }
    //     attr::TagType::None => serialize_untagged_variant(params, variant, cattrs),
    // });

    // quote! {
    //     #case => #body
    // }
}

// OLD

fn generate(input: TokenStream2) -> Result<TokenStream2, syn::Error> {
    let mut tokens = quote! {};
    tokens.extend(generate_type(input)?);
    Ok(tokens)
}

fn generate_type(input: TokenStream2) -> Result<TokenStream2, syn::Error> {
    let ast: DeriveInput = syn::parse2(input.clone())?;

    let scale_info = crate_name_ident("scale-info")?;
    let parity_scale_codec = crate_name_ident("parity-scale-codec")?;

    let ident = &ast.ident;

    let cx = Ctxt::new();
    let cont = attr::Container::from_ast(&cx, &ast);
    Ctxt::check(cx).expect("oh noes");
    println!(
        "[generate_type, {}] attributes container: {:#?}",
        ident, cont
    );

    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();
    let where_clause = trait_bounds::make_where_clause(
        ident,
        &ast.generics,
        &ast.data,
        &scale_info,
        &parity_scale_codec,
        &cont,
    )?;

    let generic_type_ids = ast.generics.type_params().map(|ty| {
        let ty_ident = &ty.ident;
        quote! {
            :: #scale_info ::meta_type::<#ty_ident>()
        }
    });

    let build_type = match &ast.data {
        syn::Data::Struct(ref s) => generate_composite_type(s, &scale_info),
        syn::Data::Enum(ref e) => generate_variant_type(e, &scale_info),
        syn::Data::Union(_) => {
            return Err(Error::new_spanned(input, "Unions not supported"))
        }
    };
    let type_info_impl = quote! {
        impl #impl_generics :: #scale_info ::TypeInfo for #ident #ty_generics #where_clause {
            type Identity = Self;
            fn type_info() -> :: #scale_info ::Type {
                :: #scale_info ::Type::builder()
                    .path(:: #scale_info ::Path::new(stringify!(#ident), module_path!()))
                    .type_params(:: #scale_info ::prelude::vec![ #( #generic_type_ids ),* ])
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
fn crate_name_ident(name: &str) -> Result<Ident, syn::Error> {
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
            let method_call = if is_compact(f) {
                quote!(.compact_of::<#ty>)
            } else {
                quote!(.field_of::<#ty>)
            };
            if let Some(ident) = ident {
                quote!(#method_call(stringify!(#ident), #type_name))
            } else {
                quote!(#method_call(#type_name))
            }
        })
        .collect()
}

/// Look for a `#[codec(compact)]` outer attribute.
fn is_compact(f: &Field) -> bool {
    f.attrs.iter().any(|attr| {
        let mut is_compact = false;
        if attr.style == AttrStyle::Outer && attr.path.is_ident("codec") {
            if let Ok(Meta::List(MetaList { nested, .. })) = attr.parse_meta() {
                if let Some(NestedMeta::Meta(Meta::Path(path))) = nested.iter().next() {
                    if path.is_ident("compact") {
                        is_compact = true;
                    }
                }
            }
        }
        is_compact
    })
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
            :: #scale_info ::build::Variants::fieldless()
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

fn generate_variant_type(data_enum: &DataEnum, scale_info: &Ident) -> TokenStream2 {
    let variants = &data_enum.variants;

    if is_c_like_enum(&variants) {
        return generate_c_like_enum_def(variants, scale_info)
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
                        :: #scale_info ::build::Fields::named()
                            #( #fields)*
                    )
                }
            }
            Fields::Unnamed(ref fs) => {
                let fields = generate_fields(&fs.unnamed);
                quote! {
                    .variant(
                        #v_name,
                        :: #scale_info ::build::Fields::unnamed()
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
            :: #scale_info ::build::Variants::with_fields()
                #( #variants)*
        )
    }
}

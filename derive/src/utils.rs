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

//! Utility methods to work with `SCALE` attributes relevant for the `TypeInfo` derive..
//!
//! NOTE: The code here is copied verbatim from `parity-scale-codec-derive`.

use alloc::{
    string::ToString,
    vec::Vec,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse,
    parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    token,
    AttrStyle,
    Attribute,
    DeriveInput,
    Lit,
    Meta,
    NestedMeta,
    Variant,
};

/// Return all doc attributes literals found.
pub fn get_doc_literals(attrs: &[syn::Attribute]) -> Vec<syn::Lit> {
    attrs
        .iter()
        .filter_map(|attr| {
            if let Ok(syn::Meta::NameValue(meta)) = attr.parse_meta() {
                if meta.path.get_ident().map_or(false, |ident| ident == "doc") {
                    let lit = &meta.lit;
                    let doc_lit = quote!(#lit).to_string();
                    let trimmed_doc_lit =
                        doc_lit.trim_start_matches(r#"" "#).trim_end_matches('"');
                    Some(parse_quote!(#trimmed_doc_lit))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Trait bounds.
pub type TraitBounds = Punctuated<syn::WherePredicate, token::Comma>;

/// Parse `name(T: Bound, N: Bound)` as a custom trait bound.
struct CustomTraitBound<N> {
    _name: N,
    _paren_token: token::Paren,
    bounds: TraitBounds,
}

impl<N: Parse> Parse for CustomTraitBound<N> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        let _name = input.parse()?;
        let _paren_token = syn::parenthesized!(content in input);
        let bounds = content.parse_terminated(syn::WherePredicate::parse)?;
        Ok(Self {
            _name,
            _paren_token,
            bounds,
        })
    }
}

syn::custom_keyword!(bounds);

/// Look for a `#[scale_info(bounds(…))]`in the given attributes.
///
/// If found, use the given trait bounds when deriving the `TypeInfo` trait.
pub fn custom_trait_bounds(attrs: &[Attribute]) -> Option<TraitBounds> {
    scale_info_meta_item(attrs.iter(), |meta: CustomTraitBound<bounds>| {
        Some(meta.bounds)
    })
}

/// Look for a `#[codec(index = $int)]` attribute on a variant. If no attribute
/// is found, fall back to the discriminant or just the variant index.
pub fn variant_index(v: &Variant, i: usize) -> TokenStream {
    // first look for an `index` attribute…
    let index = maybe_index(v);
    // …then fallback to discriminant or just index
    index.map(|i| quote! { #i }).unwrap_or_else(|| {
        v.discriminant
            .as_ref()
            .map(|&(_, ref expr)| quote! { #expr })
            .unwrap_or_else(|| quote! { #i })
    })
}

/// Look for a `#[codec(index = $int)]` outer attribute on a variant.
/// If found, it is expected to be a parseable as a `u8` (panics otherwise).
pub fn maybe_index(variant: &Variant) -> Option<u8> {
    let outer_attrs = variant
        .attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer);

    codec_meta_item(outer_attrs, |meta| {
        if let NestedMeta::Meta(Meta::NameValue(ref nv)) = meta {
            if nv.path.is_ident("index") {
                if let Lit::Int(ref v) = nv.lit {
                    let byte = v
                        .base10_parse::<u8>()
                        .expect("Internal error. `#[codec(index = …)]` attribute syntax must be checked in `parity-scale-codec`. This is a bug.");
                    return Some(byte)
                }
            }
        }

        None
    })
}

/// Look for a `#[codec(compact)]` outer attribute on the given `Field`.
pub fn is_compact(field: &syn::Field) -> bool {
    let outer_attrs = field
        .attrs
        .iter()
        .filter(|attr| attr.style == AttrStyle::Outer);
    codec_meta_item(outer_attrs, |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("compact") {
                return Some(())
            }
        }

        None
    })
    .is_some()
}

/// Look for a `#[codec(skip)]` in the given attributes.
pub fn should_skip(attrs: &[Attribute]) -> bool {
    codec_meta_item(attrs.iter(), |meta| {
        if let NestedMeta::Meta(Meta::Path(ref path)) = meta {
            if path.is_ident("skip") {
                return Some(path.span())
            }
        }

        None
    })
    .is_some()
}

fn codec_meta_item<'a, F, R, I, M>(itr: I, pred: F) -> Option<R>
where
    F: FnMut(M) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
    M: Parse,
{
    find_meta_item("codec", itr, pred)
}

fn scale_info_meta_item<'a, F, R, I, M>(itr: I, pred: F) -> Option<R>
where
    F: FnMut(M) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
    M: Parse,
{
    find_meta_item("scale_info", itr, pred)
}

fn find_meta_item<'a, F, R, I, M>(kind: &str, mut itr: I, mut pred: F) -> Option<R>
where
    F: FnMut(M) -> Option<R> + Clone,
    I: Iterator<Item = &'a Attribute>,
    M: Parse,
{
    itr.find_map(|attr| {
        attr.path
            .is_ident(kind)
            .then(|| pred(attr.parse_args().ok()?))
            .flatten()
    })
}

/// Ensure attributes are correctly applied. This *must* be called before using
/// any of the attribute finder methods or the macro may panic if it encounters
/// misapplied attributes.
/// `#[scale_info(bounds())]` is the only accepted attribute.
pub fn check_attributes(input: &DeriveInput) -> syn::Result<()> {
    for attr in &input.attrs {
        check_top_attribute(attr)?;
    }
    Ok(())
}

// Only `#[scale_info(bounds())]` is a valid top attribute.
fn check_top_attribute(attr: &Attribute) -> syn::Result<()> {
    if attr.path.is_ident("scale_info") {
        match attr.parse_args::<CustomTraitBound<bounds>>() {
            Ok(_) => Ok(()),
            Err(e) => Err(syn::Error::new(attr.span(), format!("Invalid attribute: {:?}. Only `#[scale_info(bounds(…))]` is a valid top attribute", e)))
        }
    } else {
        Ok(())
    }
}

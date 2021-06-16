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

use syn::{
    parse::{
        Parse,
        ParseBuffer,
    },
    punctuated::Punctuated,
    Token,
};

const SCALE_INFO: &str = "scale_info";

mod keywords {
    syn::custom_keyword!(scale_info);
    syn::custom_keyword!(bounds);
    syn::custom_keyword!(skip_type_params);
}

/// List of `#[scale_info(...)]` attributes for an item.
pub struct ScaleInfoAttrList {
    attrs: Punctuated<ScaleInfoAttr, Token![,]>,
}

impl ScaleInfoAttrList {
    /// Extract out `#[scale_info(...)]` attributes from an item.
    pub fn from_ast(item: &syn::DeriveInput) -> syn::Result<Self> {
        let mut attrs = Punctuated::new();
        for attr in &item.attrs {
            if !attr.path.is_ident(SCALE_INFO) {
                continue
            }
            let scale_info_attr_list = attr.parse_args_with(ScaleInfoAttrList::parse)?;
            for scale_info_attr in scale_info_attr_list.attrs {
                attrs.push(scale_info_attr);
            }
        }
        // todo: [AJ] check for duplicates
        Ok(Self { attrs })
    }

    /// Get the `#[scale_info(bounds(...))]` attribute, if present.
    pub fn bounds(&self) -> Option<&BoundsAttr> {
        self.attrs.iter().find_map(|attr| {
            match attr {
                ScaleInfoAttr::Bounds(bounds) => Some(bounds),
                _ => None,
            }
        })
    }

    /// Get the `#[scale_info(skip_type_params(...))]` attribute, if present.
    pub fn skip_type_params(&self) -> Option<&SkipTypeParamsAttr> {
        self.attrs.iter().find_map(|attr| {
            match attr {
                ScaleInfoAttr::SkipTypeParams(type_params) => Some(type_params),
                _ => None,
            }
        })
    }
}

impl Parse for ScaleInfoAttrList {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let attrs = input.parse_terminated(ScaleInfoAttr::parse)?;
        Ok(Self { attrs })
    }
}

/// Parsed representation of the `#[scale_info(bounds(...))]` attribute.
pub struct BoundsAttr {
    predicates: Punctuated<syn::WherePredicate, Token![,]>,
}

impl Parse for BoundsAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        input.parse::<keywords::bounds>()?;
        let content;
        syn::parenthesized!(content in input);
        let predicates = content.parse_terminated(syn::WherePredicate::parse)?;
        Ok(Self { predicates })
    }
}

impl BoundsAttr {
    /// Add the predicates defined in this attribute to the given `where` clause.
    pub fn extend_where_clause(&self, where_clause: &mut syn::WhereClause) {
        where_clause.predicates.extend(self.predicates.clone());
    }
}

/// Parsed representation of the `#[scale_info(skip_type_params(...))]` attribute.
pub struct SkipTypeParamsAttr {
    type_params: Punctuated<syn::TypeParam, Token![,]>,
}

impl Parse for SkipTypeParamsAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        input.parse::<keywords::skip_type_params>()?;
        let content;
        syn::parenthesized!(content in input);
        let type_params = content.parse_terminated(syn::TypeParam::parse)?;
        Ok(Self { type_params })
    }
}

impl SkipTypeParamsAttr {
    /// Returns `true` if the given type parameter should be skipped.
    pub fn skip(&self, type_param: &syn::TypeParam) -> bool {
        self.type_params
            .iter()
            .any(|tp| tp.ident == type_param.ident)
    }
}

/// Parsed representation of one of the `#[scale_info(..)]` attributes.
pub enum ScaleInfoAttr {
    Bounds(BoundsAttr),
    SkipTypeParams(SkipTypeParamsAttr),
}

impl Parse for ScaleInfoAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keywords::bounds) {
            let bounds = input.parse()?;
            Ok(Self::Bounds(bounds))
        } else if lookahead.peek(keywords::skip_type_params) {
            let skip_type_params = input.parse()?;
            Ok(Self::SkipTypeParams(skip_type_params))
        } else {
            Err(input.error("Expected either `bounds` or `skip_type_params`"))
        }
    }
}

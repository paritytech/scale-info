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
    punctuated::Punctuated,
    Token,
};
use syn::{
    parse::Parse,
    spanned::Spanned,
    NestedMeta::Meta,
};
use std::iter::Scan;
use syn::token::parsing::keyword;
use syn::parse::ParseBuffer;

const SCALE_INFO: &str = "scale_info";

mod keywords {
    syn::custom_keyword!(scale_info);
    syn::custom_keyword!(bounds);
    syn::custom_keyword!(skip_type_params);
}

pub struct ScaleInfoAttrList {
    attrs: Vec<ScaleInfoAttr>,
}

impl ScaleInfoAttrList {
    pub fn bounds(&self) -> Option<Vec<syn::WherePredicate>> {
        self.attrs.iter().find_map(|attr| match attr {
            ScaleInfoAttr::Bounds(bounds) => Some(bounds.iter().collect()),
            _ => None,
        })
    }

    pub fn skip_type_params(&self) -> Option<Vec<syn::TypeParam>> {
        self.attrs.iter().find_map(|attr| match attr {
            ScaleInfoAttr::SkipTypeParams(type_params) => Some(type_params.iter().collect()),
            _ => None,
        })
    }
}

impl Parse for ScaleInfoAttrList {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keywords::scale_info) {
            let content;
            syn::parenthesized!(content in input);
            let attrs = content.parse_terminated(ScaleInfoAttr::parse)?;
            Ok(Self { attrs: attrs.iter().collect() })
        } else {
            Err(input.error("Expected a `#[scale_info(..)]` attribute"))
        }
    }
}

pub enum ScaleInfoAttr {
    Bounds(Punctuated<syn::WherePredicate, Token![,]>),
    SkipTypeParams(Punctuated<syn::TypeParam, Token![,]>),
}

impl Parse for ScaleInfoAttr {
    fn parse(input: &ParseBuffer) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keywords::bounds) {
            input.parse::<keywords::bounds>()?;
            let content;
            syn::parenthesized!(content in input);
            content.parse_terminated(syn::WherePredicate::parse).map(Self::Bounds)
        } else if lookahead.peek(keywords::skip_type_params) {
            input.parse::<keywords::skip_type_params>()?;
            let content;
            syn::parenthesized!(content in input);
            content.parse_terminated(syn::TypeParam::parse).map(Self::SkipTypeParams)
        } else {
            Err(input.error("Expected either `bounds` or `skip_type_params`"))
        }
    }
}

impl ScaleInfoAttrList {
    fn from_ast(item: &syn::DeriveInput) -> syn::Result<Self> {
        let mut attrs = Vec::new();
        for attr in &item.attrs {
            if attr.path.is_ident("scale_info") {
                continue;
            }
            let scale_info_attr_list = attr.parse_args_with(ScaleInfoAttrList::parse)?;
            for scale_info_attr in scale_info_attr_list.attrs {
                attrs.push(scale_info_attr);
            }
        }
        // todo: [AJ] check for duplicates
        Ok(Self { attrs })
    }
}



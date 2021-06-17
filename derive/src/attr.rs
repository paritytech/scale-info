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
    spanned::Spanned,
    Token,
};

const SCALE_INFO: &str = "scale_info";

mod keywords {
    syn::custom_keyword!(scale_info);
    syn::custom_keyword!(bounds);
    syn::custom_keyword!(skip_type_params);
}

/// Parsed and validated set of `#[scale_info(...)]` attributes for an item.
pub struct Attributes {
    bounds: Option<BoundsAttr>,
    skip_type_params: Option<SkipTypeParamsAttr>,
}

impl Attributes {
    /// Extract out `#[scale_info(...)]` attributes from an item.
    pub fn from_ast(item: &syn::DeriveInput) -> syn::Result<Self> {
        let mut bounds = None;
        let mut skip_type_params = None;

        for attr in &item.attrs {
            if !attr.path.is_ident(SCALE_INFO) {
                continue
            }
            let scale_info_attrs = attr.parse_args_with(|input: &ParseBuffer| {
                let attrs: Punctuated<ScaleInfoAttr, Token![,]> =
                    input.parse_terminated(ScaleInfoAttr::parse)?;
                Ok(attrs)
            })?;

            for scale_info_attr in scale_info_attrs {
                // check for duplicates
                match scale_info_attr {
                    ScaleInfoAttr::Bounds(parsed_bounds) => {
                        if bounds.is_some() {
                            return Err(syn::Error::new(
                                attr.span(),
                                "Duplicate `bounds` attributes",
                            ))
                        }
                        bounds = Some(parsed_bounds);
                    }
                    ScaleInfoAttr::SkipTypeParams(parsed_skip_type_params) => {
                        if skip_type_params.is_some() {
                            return Err(syn::Error::new(
                                attr.span(),
                                "Duplicate `skip_type_params` attributes",
                            ))
                        }
                        skip_type_params = Some(parsed_skip_type_params);
                    }
                }
            }
        }

        // validate type params which do not appear in custom bounds but are not skipped.
        if let Some(ref bounds) = bounds {
            for type_param in item.generics.type_params() {
                if !bounds.contains_type_param(type_param) {
                    let type_param_skipped = skip_type_params
                        .as_ref()
                        .map(|skip| skip.skip(type_param))
                        .unwrap_or(false);
                    if !type_param_skipped {
                        let msg = format!(
                            "Type parameter requires a `TypeInfo` bound, so either: \n \
                        - add it to `#[scale_info(bounds({}: TypeInfo))]` \n \
                        - skip it with `#[scale_info(skip_type_params({}))]`",
                            type_param.ident, type_param.ident
                        );
                        return Err(syn::Error::new(type_param.span(), msg))
                    }
                }
            }
        }

        Ok(Self {
            bounds,
            skip_type_params,
        })
    }

    /// Get the `#[scale_info(bounds(...))]` attribute, if present.
    pub fn bounds(&self) -> Option<&BoundsAttr> {
        self.bounds.as_ref()
    }

    /// Get the `#[scale_info(skip_type_params(...))]` attribute, if present.
    pub fn skip_type_params(&self) -> Option<&SkipTypeParamsAttr> {
        self.skip_type_params.as_ref()
    }
}

/// Parsed representation of the `#[scale_info(bounds(...))]` attribute.
#[derive(Clone)]
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

    /// Returns true if the given type parameter appears in the custom bounds attribute.
    pub fn contains_type_param(&self, type_param: &syn::TypeParam) -> bool {
        self.predicates.iter().any(|p| {
            if let syn::WherePredicate::Type(ty) = p {
                if let syn::Type::Path(ref path) = ty.bounded_ty {
                    path.path.get_ident() == Some(&type_param.ident)
                } else {
                    false
                }
            } else {
                false
            }
        })
    }
}

/// Parsed representation of the `#[scale_info(skip_type_params(...))]` attribute.
#[derive(Clone)]
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

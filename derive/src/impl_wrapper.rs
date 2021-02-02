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

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::ToString,
};

use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::quote;
use syn::Ident;

pub fn wrap(
    ident: &Ident,
    trait_name: &'static str,
    impl_quote: TokenStream2,
) -> TokenStream2 {
    let include_scale_info = include_crate("scale-info", "_scale_info");
    let include_parity_scale_codec = include_crate("parity_scale_codec", "_scale");

    quote! {
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #include_scale_info
            #include_parity_scale_codec

            #impl_quote;
        };
    }
}

/// Include a crate under a known alias, to be robust against renamed dependencies.
fn include_crate(name: &str, alias: &str) -> proc_macro2::TokenStream {
    match proc_macro_crate::crate_name(name) {
        Ok(crate_name) => {
            let crate_name_ident = Ident::new(&crate_name, Span::call_site());
            let crate_alias_ident = Ident::new(&alias, Span::call_site());
            quote!( extern crate #crate_name_ident as #crate_alias_ident; )
        },
        Err(e) => syn::Error::new(Span::call_site(), &e).to_compile_error(),
    }
}

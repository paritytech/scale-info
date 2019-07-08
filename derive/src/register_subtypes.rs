// Copyright 2019
//     by  Centrality Investments Ltd.
//     and Parity Technologies (UK) Ltd.
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

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{self, parse::Result, parse_quote, DeriveInput, Ident};

pub fn generate(input: TokenStream2) -> TokenStream2 {
	match generate_impl(input.into()) {
		Ok(output) => output.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

pub fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
	let mut ast: DeriveInput = syn::parse2(input)?;

	// add bound
	ast.generics.type_params_mut().for_each(|p| {
		p.bounds.push(parse_quote!(_type_metadata::Metadata));
	});

	let ident = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
	let register_types = ast.generics.type_params().into_iter().map(|ty| {
		let ty_ident = ty.ident.clone();
		quote! { registry.register_type::<#ty_ident>(); }
	});

	let register_subtypes_impl = quote! {
		impl #impl_generics _type_metadata::RegisterSubtypes for #ident #ty_generics #where_clause {
			fn register_subtypes(registry: &mut _type_metadata::Registry) {
				#( #register_types )*
			}
		}
	};

	let mut renamed = String::from("_IMPL_REGISTER_SUBTYPES_FOR_");
	renamed.push_str(ident.to_string().trim_start_matches("r#"));
	let dummy_const = Ident::new(&renamed, Span::call_site());
	let output = quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			#[allow(unknown_lints)]
			#[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
			#[allow(rust_2018_idioms)]
			use type_metadata as _type_metadata;
			#register_subtypes_impl;
		};
	};

	Ok(output.into())
}

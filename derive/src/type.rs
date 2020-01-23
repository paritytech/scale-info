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

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Result, parse_quote, DeriveInput};

use crate::{impl_wrapper::wrap, type_def};

pub fn generate(input: TokenStream2) -> TokenStream2 {
	match generate_impl(input) {
		Ok(output) => output,
		Err(err) => err.to_compile_error(),
	}
}

pub fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
	let mut ast: DeriveInput = syn::parse2(input.clone())?;

	ast.generics.type_params_mut().for_each(|p| {
		p.bounds.push(parse_quote!(_type_metadata::Metadata));
		p.bounds.push(parse_quote!('static));
	});

	let ident = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
	let generic_type_ids = ast.generics.type_params().map(|ty| {
		let ty_ident = &ty.ident;
		quote! {
			<#ty_ident as _type_metadata::Metadata>::meta_type()
		}
	});
	let type_def = type_def::generate_impl(input)?;
	let has_type_id_impl = quote! {
		impl #impl_generics _type_metadata::HasType for #ident #ty_generics #where_clause {
			fn type_id() -> _type_metadata::Type {
				_type_metadata::TypeCustom::new(
					stringify!(#ident),
					_type_metadata::Namespace::from_module_path(module_path!())
						.expect("namespace from module path cannot fail"),
					__core::vec![ #( #generic_type_ids ),* ],
					#type_def.into(),
				).into()
			}
		}
	};

	Ok(wrap(ident, "HAS_TYPE_ID", has_type_id_impl))
}

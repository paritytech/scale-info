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
use syn::{self, parse::Result, parse_quote, DeriveInput, GenericParam, TypeParamBound};

pub fn generate(input: TokenStream2) -> TokenStream2 {
	match generate_impl(input.into()) {
		Ok(output) => output.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

pub fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
	let mut ast: DeriveInput = syn::parse2(input)?;

	// add bound
	let type_id_bound: TypeParamBound = parse_quote!(_type_metadata::HasTypeId);
	ast.generics.params.iter_mut().for_each(|param| {
		if let GenericParam::Type(ref mut type_param) = param {
			type_param.bounds.push(type_id_bound.clone());
		}
	});

	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

	// impl `HasTypeId`
	let generic_type_params = ast
		.generics
		.params
		.iter()
		.filter_map(|param| {
			if let GenericParam::Type(ref type_param) = param {
				Some(type_param)
			} else {
				None
			}
		})
		.collect::<Vec<_>>();
	let ident = &ast.ident;
	let generic_type_ids = generic_type_params.into_iter().map(|ty| {
		quote! {
			<#ty as _type_metadata::HasTypeId>::type_id()
		}
	});

	let impl_has_type_id = quote! {
		impl _type_metadata::HasTypeId for #ident #ty_generics #where_clause {
			fn type_id() -> _type_metadata::TypeId {
				_type_metadata::TypeIdCustom::new(
					stringify!(#ident),
					// namespace from module path cannot fail
					_type_metadata::Namespace::from_str(module_path!()).unwrap(),
					vec![ #( #generic_type_ids ),* ],
				).into()
			}
		}
	};

	let output = quote! {
		use type_metadata as _type_metadata;
		#impl_has_type_id;
	};

	Ok(output.into())
}

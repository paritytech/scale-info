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
use quote::{quote, quote_spanned};
use syn::{self, Data, DataStruct, DataEnum, Field, Fields, Ident, parse::Result, parse_quote, DeriveInput, spanned::Spanned, punctuated::Punctuated, token::Comma};

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
		p.bounds.push(parse_quote!(_type_metadata::HasTypeId));
		p.bounds.push(parse_quote!(_type_metadata::HasTypeDef));
	});

	let ident = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

	let type_kind = match &ast.data {
		Data::Struct(ref s) => generate_struct_def_kind(s),
		Data::Enum(ref e) => generate_enum_def_kind(e),
		// TODO: handle union
		_ => quote! {
			_type_metadata::TypeDefKind::Builtin
		}
	};

	let has_type_def_impl = quote! {
		impl #impl_generics _type_metadata::HasTypeDef for #ident #ty_generics #where_clause {
			fn type_def() -> _type_metadata::TypeDef {
				let annotated: _type_metadata::TypeDefKind = #type_kind;
				_type_metadata::TypeDef::new(
					// TODO: generic arg name are like `T` or for instance `u128`?
					vec![],
					annotated,
				)
			}
		}
	};

	let mut renamed = String::from("_IMPL_HAS_TYPE_DEF_FOR_");
	renamed.push_str(ident.to_string().trim_start_matches("r#"));
	let dummy_const = Ident::new(&renamed, Span::call_site());
	let output = quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			#[allow(unknown_lints)]
			#[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
			#[allow(rust_2018_idioms)]
			use type_metadata as _type_metadata;
			#has_type_def_impl;
		};
	};

	Ok(output.into())
}

type FieldsList = Punctuated<Field, Comma>;

fn generate_fields_def(fields: FieldsList) -> TokenStream2 {
	let fields_def = fields.iter().map(|f| {
		let (ty, ident) = (f.ty.clone(), f.ident.clone());
		if let Some(i) = ident {
			let type_id = quote! { <#ty as _type_metadata::HasTypeId>::type_id() };
			quote! { _type_metadata::NamedField::new(stringify!(#i), #type_id) }
		} else {
			quote! { _type_metadata::UnnamedField::new::<#ty>() }
		}
	});
	quote! { vec![#( #fields_def, )*] }
}

fn generate_struct_def_kind(data_struct: &DataStruct) -> TokenStream2 {
	match data_struct.fields {
		Fields::Named(ref fs) => {
			let fields = generate_fields_def(fs.named.clone());
			quote! { _type_metadata::TypeDefStruct::new(#fields).into() }
		},
		Fields::Unnamed(ref fs) => {
			let fields = generate_fields_def(fs.unnamed.clone());
			quote! { _type_metadata::TypeDefTupleStruct::new(#fields).into() }
		},
		Fields::Unit => quote! { _type_metadata::TypeDefTupleStruct::unit().into() },
	}
}

fn generate_enum_def_kind(data_enum: &DataEnum) -> TokenStream2 {
	quote! {
		_type_metadata::TypeDefKind::Builtin
	}
}

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

use crate::impl_wrapper::wrap;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	self, parse::Result, parse_quote, punctuated::Punctuated, token::Comma, Data, DataEnum, DataStruct, DataUnion,
	DeriveInput, Field, Fields,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
	match generate_impl(input.into()) {
		Ok(output) => output.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

pub fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
	let mut ast: DeriveInput = syn::parse2(input)?;

	ast.generics.type_params_mut().for_each(|p| {
		p.bounds.push(parse_quote!(_type_metadata::Metadata));
	});

	let ident = &ast.ident;
	let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
	let register_subtypes = match &ast.data {
		Data::Struct(ref s) => register_struct_subtypes(s),
		Data::Enum(ref e) => register_enum_subtypes(e),
		Data::Union(ref u) => register_union_subtypes(u),
	};

	let register_subtypes_impl = quote! {
		impl #impl_generics _type_metadata::RegisterSubtypes for #ident #ty_generics #where_clause {
			fn register_subtypes(registry: &mut _type_metadata::Registry) {
				#register_subtypes
			}
		}
	};

	Ok(wrap(ident, "REGISTER_SUBTYPES", register_subtypes_impl).into())
}

/// Register subtypes of `Punctuated<Field, Comma>`.
fn register_fields_subtypes(fields: &Punctuated<Field, Comma>) -> TokenStream2 {
	let registers = fields.iter().map(|f| {
		let ty = &f.ty;
		quote! { registry.register_type::<#ty>(); }
	});
	quote! { #( #registers )* }
}

/// Register subtypes of `syn::Fields`.
fn register_fields_kind_subtypes(fields_kind: &Fields) -> TokenStream2 {
	match fields_kind {
		Fields::Named(ref fs) => register_fields_subtypes(&fs.named),
		Fields::Unnamed(ref fs) => register_fields_subtypes(&fs.unnamed),
		Fields::Unit => quote! {},
	}
}

fn register_struct_subtypes(data_struct: &DataStruct) -> TokenStream2 {
	register_fields_kind_subtypes(&data_struct.fields)
}

fn register_enum_subtypes(data_enum: &DataEnum) -> TokenStream2 {
	let variants = &data_enum.variants;
	let register = variants.into_iter().map(|v| register_fields_kind_subtypes(&v.fields));
	quote! { #( #register )* }
}

fn register_union_subtypes(data_union: &DataUnion) -> TokenStream2 {
	register_fields_subtypes(&data_union.fields.named)
}

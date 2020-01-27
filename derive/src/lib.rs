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

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
extern crate proc_macro;

mod impl_wrapper;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	parse_quote,
	parse::{Result, Error}, punctuated::Punctuated, token::Comma, Data, DataEnum, DataStruct, DeriveInput, Expr,
	ExprLit, Field, Fields, Lit, Variant,
};

#[proc_macro_derive(Metadata)]
pub fn metadata(input: TokenStream) -> TokenStream {
	match generate(input.into()) {
		Ok(output) => output.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

fn generate(input: TokenStream2) -> Result<TokenStream2> {
	let mut tokens = quote! {};
	tokens.extend(generate_type(input)?);
	Ok(tokens)
}

fn generate_type(input: TokenStream2) -> Result<TokenStream2> {
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
	let type_path = quote! {
		_type_metadata::TypePath::new(
			stringify!(#ident),
			_type_metadata::Namespace::from_module_path(module_path!())
				.expect("namespace from module path cannot fail"),
			__core::vec![ #( #generic_type_ids ),* ],
		)
	};
	let type_def = generate_impl(input, type_path)?;

	let has_type_id_impl = quote! {
		impl #impl_generics _type_metadata::HasType for #ident #ty_generics #where_clause {
			fn get_type() -> _type_metadata::Type {
				#type_def.into()
			}
		}
	};

	Ok(impl_wrapper::wrap(ident, "HAS_TYPE", has_type_id_impl))
}

fn generate_impl(input: TokenStream2, type_path: TokenStream2) -> Result<TokenStream2> {
	let ast: DeriveInput = syn::parse2(input.clone())?;

	let def = match &ast.data {
		Data::Struct(ref s) => generate_struct_def(s, type_path),
		Data::Enum(ref e) => generate_enum_def(e, type_path),
		Data::Union(_) => return Err(Error::new_spanned(input, "Unions not supported")),
	};

	Ok(def)
}

type FieldsList = Punctuated<Field, Comma>;

fn generate_fields_def(fields: &FieldsList) -> TokenStream2 {
	let fields_def = fields.iter().map(|f| {
		let (ty, ident) = (&f.ty, &f.ident);
		let meta_type = quote! {
			<#ty as _type_metadata::Metadata>::meta_type()
		};
		if let Some(i) = ident {
			quote! {
				_type_metadata::NamedField::new(stringify!(#i), #meta_type)
			}
		} else {
			quote! {
				_type_metadata::UnnamedField::new(#meta_type)
			}
		}
	});
	quote! { __core::vec![#( #fields_def, )*] }
}

fn generate_struct_def(data_struct: &DataStruct, type_path: TokenStream2) -> TokenStream2 {
	match data_struct.fields {
		Fields::Named(ref fs) => {
			let fields = generate_fields_def(&fs.named);
			quote! {
				_type_metadata::TypeProductStruct::new(#type_path, #fields)
			}
		}
		Fields::Unnamed(ref fs) => {
			let fields = generate_fields_def(&fs.unnamed);
			quote! {
				_type_metadata::TypeProductTupleStruct::new(#type_path, #fields)
			}
		}
		Fields::Unit => quote! {
			_type_metadata::TypeProductTupleStruct::unit(#type_path)
		},
	}
}

type VariantList = Punctuated<Variant, Comma>;

fn generate_c_like_enum_def(variants: &VariantList, type_path: TokenStream2) -> TokenStream2 {
	let variants_def = variants.into_iter().enumerate().map(|(i, v)| {
		let name = &v.ident;
		let discriminant = if let Some((
										   _,
										   Expr::Lit(ExprLit {
														 lit: Lit::Int(lit_int), ..
													 }),
									   )) = &v.discriminant
		{
			match lit_int.base10_parse::<u64>() {
				Ok(i) => i,
				Err(err) => return err.to_compile_error(),
			}
		} else {
			i as u64
		};
		quote! {
			_type_metadata::ClikeEnumVariant::new(stringify!(#name), #discriminant)
		}
	});
	quote! {
		_type_metadata::TypeSumClikeEnum::new(#type_path, __core::vec![#( #variants_def, )*])
	}
}

fn is_c_like_enum(variants: &VariantList) -> bool {
	// any variant has an explicit discriminant
	variants.iter().any(|v| v.discriminant.is_some()) ||
		// all variants are unit
		variants.iter().all(|v| match v.fields {
			Fields::Unit => true,
			_ => false,
		})
}

fn generate_enum_def(data_enum: &DataEnum, type_path: TokenStream2) -> TokenStream2 {
	let variants = &data_enum.variants;

	if is_c_like_enum(&variants) {
		return generate_c_like_enum_def(variants, type_path);
	}

	let variants_def = variants.into_iter().map(|v| {
		let ident = &v.ident;
		let v_name = quote! {stringify!(#ident) };
		match v.fields {
			Fields::Named(ref fs) => {
				let fields = generate_fields_def(&fs.named);
				quote! {
					_type_metadata::EnumVariantStruct::new(#v_name, #fields).into()
				}
			}
			Fields::Unnamed(ref fs) => {
				let fields = generate_fields_def(&fs.unnamed);
				quote! {
					_type_metadata::EnumVariantTupleStruct::new(#v_name, #fields).into()
				}
			}
			Fields::Unit => quote! {
				_type_metadata::EnumVariantUnit::new(#v_name).into()
			},
		}
	});
	quote! {
		_type_metadata::TypeSumEnum::new(#type_path, __core::vec![#( #variants_def, )*])
	}
}


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
use syn::{self, Data, DataStruct, DataEnum, Expr, ExprLit, Field, Fields, Ident, Lit, parse::Result, parse_quote, DeriveInput, punctuated::Punctuated, token::Comma, Variant};

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
		Data::Struct(ref s) => generate_struct_def(s),
		Data::Enum(ref e) => generate_enum_def(e),
		// TODO: handle union
		_ => quote! {
			_type_metadata::TypeDefKind::Builtin
		}
	};

	let has_type_def_impl = quote! {
		impl #impl_generics _type_metadata::HasTypeDef for #ident #ty_generics #where_clause {
			fn type_def() -> _type_metadata::TypeDef {
				#type_kind.into()
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

fn generate_struct_def(data_struct: &DataStruct) -> TokenStream2 {
	match data_struct.fields {
		Fields::Named(ref fs) => {
			let fields = generate_fields_def(fs.named.clone());
			quote! { _type_metadata::TypeDefStruct::new(#fields) }
		},
		Fields::Unnamed(ref fs) => {
			let fields = generate_fields_def(fs.unnamed.clone());
			quote! { _type_metadata::TypeDefTupleStruct::new(#fields) }
		},
		Fields::Unit => quote! { _type_metadata::TypeDefTupleStruct::unit() },
	}
}

type VariantList = Punctuated<Variant, Comma>;

fn generate_c_like_enum_def(variants: VariantList) -> TokenStream2 {
	let variants_def = variants.into_iter().enumerate().map(|(i, v)| {
		let name = v.ident;
		let discriminant = if let Some(
			(_, Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }))
		) = v.discriminant {
			lit_int.value()
		} else {
			i as u64
		};
		quote! {
			_type_metadata::ClikeEnumVariant {
				name: stringify!(#name),
				discriminant: #discriminant,
			}
		}
	});
	quote! {
		_type_metadata::TypeDefClikeEnum::new(vec![#( #variants_def, )*])
	}
}

fn is_c_like_enum(variants: &VariantList) -> bool {
	// any viriant has a explicit discriminant
	variants.iter().any(|v| v.discriminant.is_some()) ||
	// all variants are unit
	variants.iter().all(|v| v.fields == Fields::Unit)
}

fn generate_enum_def(data_enum: &DataEnum) -> TokenStream2 {
	let variants = data_enum.variants.clone();

	// C-Like enum
	if is_c_like_enum(&variants) {
		return generate_c_like_enum_def(variants);
	}

	// not C-Like
	let variants_def = variants.into_iter().map(|v| {
		let ident = v.ident;
		let v_name = quote! {stringify!(#ident) };
		match v.fields {
			Fields::Named(ref fs) => {
				let fields = generate_fields_def(fs.named.clone());
				quote! {
					_type_metadata::EnumVariantStruct::new(#v_name, #fields).into()
				}
			},
			Fields::Unnamed(ref fs) => {
				let fields = generate_fields_def(fs.unnamed.clone());
				quote! {
					_type_metadata::EnumVariantTupleStruct::new(#v_name, #fields).into()
				}
			},
			Fields::Unit => quote! {
				_type_metadata::EnumVariantUnit::new(#v_name).into()
			},
		}
	});
	quote! {
		_type_metadata::TypeDefEnum::new(vec![#( #variants_def, )*])
	}
}

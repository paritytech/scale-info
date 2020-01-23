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
use syn::{
	parse::Result, punctuated::Punctuated, token::Comma, Data, DataEnum, DataStruct, DataUnion,
	DeriveInput, Expr, ExprLit, Field, Fields, Lit, Variant,
};

pub fn generate(input: TokenStream2) -> TokenStream2 {
	match generate_impl(input) {
		Ok(output) => output,
		Err(err) => err.to_compile_error(),
	}
}

pub fn generate_impl(input: TokenStream2) -> Result<TokenStream2> {
	let ast: DeriveInput = syn::parse2(input)?;

	let def = match &ast.data {
		Data::Struct(ref s) => generate_struct_def(s),
		Data::Enum(ref e) => generate_enum_def(e),
		Data::Union(ref u) => generate_union_def(u),
	};

//	Ok(quote!(#def))
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

fn generate_struct_def(data_struct: &DataStruct) -> TokenStream2 {
	match data_struct.fields {
		Fields::Named(ref fs) => {
			let fields = generate_fields_def(&fs.named);
			quote! {
				_type_metadata::TypeDefStruct::new(#fields)
			}
		}
		Fields::Unnamed(ref fs) => {
			let fields = generate_fields_def(&fs.unnamed);
			quote! {
				_type_metadata::TypeDefTupleStruct::new(#fields)
			}
		}
		Fields::Unit => quote! {
			_type_metadata::TypeDefTupleStruct::unit()
		},
	}
}

type VariantList = Punctuated<Variant, Comma>;

fn generate_c_like_enum_def(variants: &VariantList) -> TokenStream2 {
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
		_type_metadata::TypeDefClikeEnum::new(__core::vec![#( #variants_def, )*])
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

fn generate_enum_def(data_enum: &DataEnum) -> TokenStream2 {
	let variants = &data_enum.variants;

	if is_c_like_enum(&variants) {
		return generate_c_like_enum_def(variants);
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
		_type_metadata::TypeDefEnum::new(__core::vec![#( #variants_def, )*])
	}
}

fn generate_union_def(data_union: &DataUnion) -> TokenStream2 {
	let fields = generate_fields_def(&data_union.fields.named);
	quote! {
		_type_metadata::TypeDefUnion::new(#fields)
	}
}

// Copyright 2019-2020 Parity Technologies (UK) Ltd.
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

use alloc::vec::Vec;
use proc_macro2::Ident;
use syn::{
	spanned::Spanned,
	visit::{self, Visit},
	Generics, Result, Type, TypePath,
	parse_quote,
};

/// Visits the ast and checks if one of the given idents is found.
struct ContainIdents<'a> {
	result: bool,
	idents: &'a[Ident]
}

impl<'a, 'ast> Visit<'ast> for ContainIdents<'a> {
	fn visit_ident(&mut self, i: &'ast Ident) {
		if self.idents.iter().any(|id| id == i) {
			self.result = true;
		}
	}
}

/// Checks if the given type contains one of the given idents.
fn type_contain_idents(ty: &Type, idents: &[Ident]) -> bool {
	let mut visitor = ContainIdents { result: false, idents };
	visitor.visit_type(ty);
	visitor.result
}

/// Visits the ast and checks if the a type path starts with the given ident.
struct TypePathStartsWithIdent<'a> {
	result: bool,
	ident: &'a Ident
}

impl<'a, 'ast> Visit<'ast> for TypePathStartsWithIdent<'a> {
	fn visit_type_path(&mut self, i: &'ast TypePath) {
		if let Some(segment) = i.path.segments.first() {
			if &segment.ident == self.ident {
				self.result = true;
				return;
			}
		}

		visit::visit_type_path(self, i);
	}
}

/// Checks if the given type path or any containing type path starts with the given ident.
fn type_path_or_sub_starts_with_ident(ty: &TypePath, ident: &Ident) -> bool {
	let mut visitor = TypePathStartsWithIdent { result: false, ident };
	visitor.visit_type_path(ty);
	visitor.result
}

/// Checks if the given type or any containing type path starts with the given ident.
fn type_or_sub_type_path_starts_with_ident(ty: &Type, ident: &Ident) -> bool {
	let mut visitor = TypePathStartsWithIdent { result: false, ident };
	visitor.visit_type(ty);
	visitor.result
}

/// Visits the ast and collects all type paths that do not start or contain the given ident.
///
/// Returns `T`, `N`, `A` for `Vec<(Recursive<T, N>, A)>` with `Recursive` as ident.
struct FindTypePathsNotStartOrContainIdent<'a> {
	result: Vec<TypePath>,
	ident: &'a Ident
}

impl<'a, 'ast> Visit<'ast> for FindTypePathsNotStartOrContainIdent<'a> {
	fn visit_type_path(&mut self, i: &'ast TypePath) {
		if type_path_or_sub_starts_with_ident(i, &self.ident) {
			visit::visit_type_path(self, i);
		} else {
			self.result.push(i.clone());
		}
	}
}

/// Collects all type paths that do not start or contain the given ident in the given type.
///
/// Returns `T`, `N`, `A` for `Vec<(Recursive<T, N>, A)>` with `Recursive` as ident.
fn find_type_paths_not_start_or_contain_ident(ty: &Type, ident: &Ident) -> Vec<TypePath> {
	let mut visitor = FindTypePathsNotStartOrContainIdent { result: Vec::new(), ident };
	visitor.visit_type(ty);
	visitor.result
}

/// Add required trait bounds to all generic types.
pub fn add(
	input_ident: &Ident,
	generics: &mut Generics,
	data: &syn::Data,
) -> Result<()> {
	generics.type_params_mut().for_each(|p| {
		p.bounds.push(parse_quote!(_scale_info::TypeInfo));
		p.bounds.push(parse_quote!('static));
	});

	let ty_params = generics.type_params().map(|p| p.ident.clone()).collect::<Vec<_>>();
	if ty_params.is_empty() {
		return Ok(());
	}

	let codec_types = get_types_to_add_trait_bound(input_ident, data, &ty_params)?;

	if !codec_types.is_empty() {
		let where_clause = generics.make_where_clause();

		codec_types
			.into_iter()
			.for_each(|ty| {
				where_clause.predicates.push(parse_quote!(#ty : _scale_info::TypeInfo + 'static))
			});
	}

	Ok(())
}

/// Returns all types that must be added to the where clause with the respective trait bound.
fn get_types_to_add_trait_bound(
	input_ident: &Ident,
	data: &syn::Data,
	ty_params: &[Ident],
) -> Result<Vec<Type>> {
	let res = collect_types(&data, |_| { true }, |_| { true })?
		.into_iter()
		// Only add a bound if the type uses a generic
		.filter(|ty| type_contain_idents(ty, &ty_params))
		// If a struct is containing itself as field type, we can not add this type into the where clause.
		// This is required to work a round the following compiler bug: https://github.com/rust-lang/rust/issues/47032
		.flat_map(|ty| {
			find_type_paths_not_start_or_contain_ident(&ty, input_ident)
				.into_iter()
				.map(|ty| Type::Path(ty.clone()))
				// Remove again types that do not contain any of our generic parameters
				.filter(|ty| type_contain_idents(ty, &ty_params))
			// todo: [AJ] can I remove this
			// // Add back the original type, as we don't want to loose him.
			// .chain(core::iter::once(ty))
		})
		// Remove all remaining types that start/contain the input ident to not have them in the where clause.
		.filter(|ty| !type_or_sub_type_path_starts_with_ident(ty, input_ident))
		.collect();

	Ok(res)
}

fn collect_types(
	data: &syn::Data,
	type_filter: fn(&syn::Field) -> bool,
	variant_filter: fn(&syn::Variant) -> bool,
) -> Result<Vec<syn::Type>> {
	use syn::*;

	let types = match *data {
		Data::Struct(ref data) => match &data.fields {
			| Fields::Named(FieldsNamed { named: fields , .. })
			| Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. }) => {
				fields.iter()
					.filter(|f| type_filter(f))
					.map(|f| f.ty.clone())
					.collect()
			},

			Fields::Unit => { Vec::new() },
		},

		Data::Enum(ref data) => data.variants.iter()
			.filter(|variant| variant_filter(variant))
			.flat_map(|variant| {
				match &variant.fields {
					| Fields::Named(FieldsNamed { named: fields , .. })
					| Fields::Unnamed(FieldsUnnamed { unnamed: fields, .. }) => {
						fields.iter()
							.filter(|f| type_filter(f))
							.map(|f| f.ty.clone())
							.collect()
					},

					Fields::Unit => { Vec::new() },
				}
			}).collect(),

		Data::Union(ref data) => return Err(Error::new(
			data.union_token.span(),
			"Union types are not supported."
		)),
	};

	Ok(types)
}

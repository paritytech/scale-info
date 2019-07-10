use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::Ident;

pub fn wrap(ident: &Ident, trait_name: &'static str, impl_quote: TokenStream2) -> TokenStream2 {
	let mut renamed = String::from(format!("_IMPL_{}_FOR_", trait_name));
	renamed.push_str(ident.to_string().trim_start_matches("r#"));
	let dummy_const = Ident::new(&renamed, Span::call_site());

	quote! {
		#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
		const #dummy_const: () = {
			#[allow(unknown_lints)]
			#[cfg_attr(feature = "cargo-clippy", allow(useless_attribute))]
			#[allow(rust_2018_idioms)]
			use type_metadata as _type_metadata;
			#impl_quote;
		};
	}
}

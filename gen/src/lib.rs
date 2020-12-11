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

use scale_info::{form::CompactForm, RegistryReadOnly, TypeDef};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate_types(registry: &RegistryReadOnly) -> TokenStream2 {
	quote! {
		// required that this be placed at crate root so can do ::registry_types.
		// alternatively use relative paths? more complicated
		mod registry_types {

		}
	}
}

trait GenerateType {
	fn generate_type(&self, registry: &RegistryReadOnly); // todo add some state here
}

impl GenerateType for TypeDef<CompactForm> {
	fn generate_type(&self, registry: &RegistryReadOnly) {

	}
}

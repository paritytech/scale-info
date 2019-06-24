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

extern crate proc_macro;

mod type_id;
mod type_def;
mod register_subtypes;

use proc_macro::TokenStream;

#[proc_macro_derive(TypeId)]
pub fn type_id(input: TokenStream) -> TokenStream {
    type_id::generate(input.into()).into()
}

#[proc_macro_derive(TypeDef)]
pub fn type_def(input: TokenStream) -> TokenStream {
    type_def::generate(input.into()).into()
}

#[proc_macro_derive(RegisterSubtypes)]
pub fn register_subtypes(input: TokenStream) -> TokenStream {
    register_subtypes::generate(input.into()).into()
}

#[proc_macro_derive(Metadata)]
pub fn metadata(input: TokenStream) -> TokenStream {
    type_id::generate(input.into()).into()
}

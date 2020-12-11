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

mod types;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Result, ExprArray};
use syn::parse::{Parse, ParseStream};
use std::io::Read;

#[proc_macro]
pub fn generate_types(input: TokenStream) -> TokenStream {
    let input = input.to_string();
    let input = input.trim_matches('"');

    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into());
    let root_path = std::path::Path::new(&root);
    let path = root_path.join(input);
    println!("HHHHHHHHH {}", path.display());

    let mut file = std::fs::File::open(&path).expect("Error opening file");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).unwrap();

    let registry: scale_info::RegistryReadOnly = scale::Decode::decode(&mut &bytes[..]).unwrap();
    types::generate("root", &registry).into()
    // TokenStream::default()
}

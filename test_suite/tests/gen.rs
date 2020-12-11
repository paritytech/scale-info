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

#[test]
fn gen() {
    // todo: make the module name the same name as the file prefix e.g types in this case
    // todo: prove that encoding the source types and decoding into the dest types works
    scale_info_gen::generate_types!("encoded/types.scale");

    let _ = root::Combined (
        root::S { a: true, b: 10 },
        root::Parent {
            a: false,
            b: root::Child { a: 3 },
        }
    );
}

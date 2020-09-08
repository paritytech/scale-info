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

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const BOUND: Symbol = Symbol("bound");
pub const SERDE: Symbol = Symbol("scale_info");

impl PartialEq<Symbol> for Ident {
	fn eq(&self, word: &Symbol) -> bool {
		self == word.0
	}
}

impl<'a> PartialEq<Symbol> for &'a Ident {
	fn eq(&self, word: &Symbol) -> bool {
		*self == word.0
	}
}

impl PartialEq<Symbol> for Path {
	fn eq(&self, word: &Symbol) -> bool {
		self.is_ident(word.0)
	}
}

impl<'a> PartialEq<Symbol> for &'a Path {
	fn eq(&self, word: &Symbol) -> bool {
		self.is_ident(word.0)
	}
}

impl Display for Symbol {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str(self.0)
	}
}

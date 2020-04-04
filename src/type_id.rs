// Copyright 2019-2020
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

use crate::tm_std::*;
use crate::Path;
use crate::form::{Form, MetaForm};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Serialize)]
#[serde(bound = "F::TypeId: Serialize")]
pub enum TypeId<F: Form = MetaForm> {
	/// The type of the field is concrete
	Concrete(F::Type),
	/// The type of the field is specified by a parameter of the parent type
	Parameter(F::String),
	/// The type of the field is a generic type with the given type params
	Generic {
		ty: F::Type, // this has to be the same for all instances of generic types
		params: Vec<F::Type>,
	}
}

// need to be able to

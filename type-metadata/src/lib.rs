// Copyright 2019 Centrality Investments Limited
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

extern crate alloc;

macro_rules! tuple_type_id {
    ( $($ty:ident),* ) => {
        {
            #[allow(unused_mut)]
            let mut v = vec![];
            $(
                v.push(<$ty as $crate::HasTypeId>::type_id());
            )*
            v
        }
    }
}

mod impls;
pub mod interner;
mod registry;
mod type_def;
mod type_id;

#[cfg(test)]
mod tests;

pub use self::{
	registry::{RegisterSubtypes, Registry, Tables},
	type_def::*,
	type_id::*,
};

pub trait Metadata: HasTypeId + HasTypeDef + RegisterSubtypes {}

impl<T> Metadata for T where T: ?Sized + HasTypeId + HasTypeDef + RegisterSubtypes {}

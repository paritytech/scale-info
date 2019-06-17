// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of type-metadata.
//
// type-metadata is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// type-metadata is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with type-metadata.  If not, see <http://www.gnu.org/licenses/>.

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

mod form;
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

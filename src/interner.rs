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

use alloc::collections::btree_map::{BTreeMap, Entry};
use core::{marker::PhantomData, num::NonZeroU32};
use serde::Serialize;

/// A symbol that is not lifetime tracked.
///
/// This can be used by self-referential types but
/// can no longer be used to resolve instances.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UntrackedSymbol<T> {
	id: NonZeroU32,
	marker: PhantomData<fn() -> T>,
}

/// A symbol from an interner.
///
/// Can be used to resolve to the associated instance.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Symbol<'a, T> {
	id: NonZeroU32,
	marker: PhantomData<fn() -> &'a T>,
}

impl<T> Symbol<'_, T> {
	/// Removes the lifetime tracking for this symbol.
	///
	/// # Note
	///
	/// - This can be useful in situations where a data structure
	///   owns all symbols and interners and can verify accesses by itself.
	/// - For further safety reasons an untracked symbol
	///   can no longer be used to resolve from an interner.
	///   It is still useful for serialization purposes.
	///
	/// # Safety
	///
	/// Although removing lifetime constraints this operation can be
	/// considered to be safe since untracked symbols can no longer be
	/// used to resolve their associated instance from the interner.
	pub fn into_untracked(self) -> UntrackedSymbol<T> {
		UntrackedSymbol {
			id: self.id,
			marker: PhantomData,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Interner<T> {
	#[serde(skip)]
	map: BTreeMap<T, usize>,
	vec: Vec<T>,
}

impl<T> Interner<T>
where
	T: Ord,
{
	/// Creates a new empty interner.
	pub fn new() -> Self {
		Self {
			map: BTreeMap::new(),
			vec: Vec::new(),
		}
	}
}

impl<T> Interner<T>
where
	T: Ord + Clone,
{
	pub fn intern_or_get(&mut self, s: T) -> (bool, Symbol<T>) {
		let next_id = self.vec.len();
		let (inserted, sym_id) = match self.map.entry(s.clone()) {
			Entry::Vacant(vacant) => {
				vacant.insert(next_id);
				self.vec.push(s);
				(true, next_id)
			}
			Entry::Occupied(occupied) => (false, *occupied.get()),
		};
		(
			inserted,
			Symbol {
				id: NonZeroU32::new((sym_id + 1) as u32).unwrap(),
				marker: PhantomData,
			},
		)
	}

	pub fn get(&self, s: &T) -> Option<Symbol<T>> {
		self.map.get(s).map(|&id| Symbol {
			id: NonZeroU32::new(id as u32).unwrap(),
			marker: PhantomData,
		})
	}

	pub fn resolve(&self, sym: Symbol<T>) -> Option<&T> {
		let idx = (sym.id.get() - 1) as usize;
		if idx >= self.vec.len() {
			return None;
		}
		self.vec.get((sym.id.get() - 1) as usize)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	type StringInterner = Interner<&'static str>;

	fn assert_id(interner: &mut StringInterner, new_symbol: &'static str, expected_id: u32) {
		let actual_id = interner.intern_or_get(new_symbol).1.id.get();
		assert_eq!(actual_id, expected_id,);
	}

	fn assert_resolve<E>(interner: &mut StringInterner, symbol_id: u32, expected_str: E)
	where
		E: Into<Option<&'static str>>,
	{
		let actual_str = interner.resolve(Symbol {
			id: NonZeroU32::new(symbol_id).unwrap(),
			marker: PhantomData,
		});
		assert_eq!(actual_str.cloned(), expected_str.into(),);
	}

	#[test]
	fn simple() {
		let mut interner = StringInterner::new();
		assert_id(&mut interner, "Hello", 1);
		assert_id(&mut interner, ", World!", 2);
		assert_id(&mut interner, "1 2 3", 3);
		assert_id(&mut interner, "Hello", 1);

		assert_resolve(&mut interner, 1, "Hello");
		assert_resolve(&mut interner, 2, ", World!");
		assert_resolve(&mut interner, 3, "1 2 3");
		assert_resolve(&mut interner, 4, None);
	}
}

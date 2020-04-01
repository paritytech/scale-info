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

use crate::tm_std::*;

use crate::{
	form::{CompactForm, Form, MetaForm},
	utils::is_rust_identifier,
	IntoCompact, MetaType, Registry,
};
use serde::Serialize;

/// Represents the path of a type definition.
///
/// This consists of several segments that each have to be a valid Rust
/// identifier. The first segment represents the crate name in which the type
/// has been defined. The last
///
/// Rust prelude type may have an empty namespace definition.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug, Default)]
#[serde(transparent)]
pub struct Path<F: Form = MetaForm> {
	/// The segments of the namespace.
	segments: Vec<F::String>,
}

impl IntoCompact for Path {
	type Output = Path<CompactForm>;

	/// Compacts this path using the given registry.
	fn into_compact(self, registry: &mut Registry) -> Self::Output {
		Path {
			segments: self
				.segments
				.into_iter()
				.map(|seg| registry.register_string(seg))
				.collect::<Vec<_>>(),
		}
	}
}

impl Path {
	/// Start building a Path with PathBuilder
	#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_ret_no_self))]
	pub fn new() -> PathBuilder {
		PathBuilder::new()
	}

	/// Creates a new path from the given segments.
	pub fn from_segments<S>(segments: S) -> Result<Self, PathError>
	where
		S: IntoIterator<Item = <MetaForm as Form>::String>,
	{
		Self::new().segments(segments).done()
	}

	/// Creates a new empty path
	pub fn empty() -> Self {
		Self::new().done()
	}
}

impl<F> Path<F>
where
	F: Form
{
	pub fn is_empty(&self) -> bool {
		self.segments.is_empty()
	}
}

/// Begin building a path
pub enum BeginPath {}
/// The PathBuilder has a module path, not valid until a type identifier is added
pub enum ModulePath {}
/// The PathBuilder is ready to attempt to build a Path
pub enum CompletePath {}

#[derive(Default)]
pub struct PathBuilder<S = BeginPath> {
	segments: Vec<<MetaForm as Form>::String>,
	marker: PhantomData<fn() -> S>,
}

impl PathBuilder<BeginPath> {
	/// Create a new PathBuilder
	pub fn new() -> Self {
		Self::default()
	}

	/// Starts to build a path from the given module path
	///
	/// # Note
	///
	/// Module path is generally obtained from the `module_path!` Rust macro.
	pub fn module_path(self, module_path: <MetaForm as Form>::String) -> PathBuilder<ModulePath> {
		PathBuilder { segments: module_path.split("::") }
	}

	/// Build a new path from the given segments.
	pub fn segments<S>(self, segments: S) -> PathBuilder<CompletePath>
	where
		S: IntoIterator<Item = <MetaForm as Form>::String>,
	{
		PathBuilder { segments }
	}

	/// Build an empty path, which is valid for so-called Voldermort types
	pub fn empty(self) -> PathBuilder<CompletePath> {
		PathBuilder { segments: Vec::new() }
	}
}

impl<S> PathBuilder<S> {
	/// Add a type identifier segment to the path
	pub fn type_ident(self, ident: <MetaForm as Form>::String) -> PathBuilder<CompletePath> {
		PathBuilder { segments: this.path.segments.chain([ident]) }
	}
}

impl PathBuilder<CompletePath> {
	pub fn done(self) -> Result<Path, PathError> {
		if self.segments.is_empty() {
			return Err(PathError::MissingSegments);
		}
		if let Some(err_at) = self.segments.iter().position(|seg| !is_rust_identifier(seg)) {
			return Err(PathError::InvalidIdentifier { segment: err_at });
		}
		Ok(Path { segments: self.segments })
	}
}

/// An error that may be encountered upon constructing namespaces.
#[derive(PartialEq, Eq, Debug)]
pub enum PathError {
	/// If the module path does not at least have one segment.
	MissingSegments,
	/// If a segment within a module path is not a proper Rust identifier.
	InvalidIdentifier {
		/// The index of the errorneous segment.
		segment: usize,
	},
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn path_ok() {
		assert_eq!(
			Path::from_segments(vec!["hello"]),
			Ok(Path {
				segments: vec!["hello"]
			})
		);
		assert_eq!(
			Path::from_segments(vec!["Hello", "World"]),
			Ok(Path {
				segments: vec!["Hello", "World"]
			})
		);
		assert_eq!(Path::from_segments(vec!["_"]), Ok(Path { segments: vec!["_"] }));
	}

	#[test]
	fn path_err() {
		assert_eq!(Path::from_segments(vec![]), Err(PathError::MissingSegments));
		assert_eq!(
			Path::from_segments(vec![""]),
			Err(PathError::InvalidIdentifier { segment: 0 })
		);
		assert_eq!(
			Path::from_segments(vec!["1"]),
			Err(PathError::InvalidIdentifier { segment: 0 })
		);
		assert_eq!(
			Path::from_segments(vec!["Hello", ", World!"]),
			Err(PathError::InvalidIdentifier { segment: 1 })
		);
	}

	#[test]
	fn path_from_module_path_and_ident() {
		assert_eq!(
			Path::new()
				.module_path("hello::world")
				.type_ident("Planet")
				.done(),
			Ok(Path {
				segments: vec!["hello", "world", "Planet"]
			})
		);
		assert_eq!(
			Path::new().module_path("::world").type_ident("Earth"),
			Err(PathError::InvalidIdentifier { segment: 0 })
		);
	}
}

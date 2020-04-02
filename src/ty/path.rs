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

use crate::{
	form::{CompactForm, Form, MetaForm},
	utils::is_rust_identifier,
	IntoCompact, Registry,
};
use serde::Serialize;

/// Represents the path of a type definition.
///
/// This consists of several segments that each have to be a valid Rust
/// identifier. The first segment represents the crate name in which the type
/// has been defined. The last
///
/// Rust prelude type may have an empty namespace definition.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Serialize, Debug)]
#[serde(transparent)]
pub struct Path<F: Form = MetaForm> {
	/// The segments of the namespace.
	segments: Vec<F::String>,
}

impl<F> Default for Path<F>
where
	F: Form,
{
	fn default() -> Self {
		Path { segments: Vec::new() }
	}
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
	pub fn new(module_path: <MetaForm as Form>::String, ident: <MetaForm as Form>::String) -> Result<Path, PathError>
	{
		let mut segments = module_path.split("::").into_iter().collect::<Vec<_>>();
		segments.push(ident);
		Self::from_segments(segments)
	}

	/// Create an empty path for types which shall not be named
	///
	/// # Note
	///
	/// Returns an always `Ok` Result to match the other constructor signatures
	#[allow(unused)]
	pub(crate) fn voldermort() -> Result<Path, PathError> {
		Ok(Path { segments: Vec::new() })
	}

	/// Crate a Path for types in the Prelude namespace
	///
	/// # Errors
	///
	/// - If the supplied ident is an invalid Rust identifier
	pub fn prelude(ident: <MetaForm as Form>::String) -> Result<Path, PathError> {
		Self::from_segments(vec![ident])
	}

	/// Create a Path from the given segments
	///
	/// # Errors
	///
	/// - If no segments are supplied
	/// - If any of the segments are invalid Rust identifiers
	pub fn from_segments<I>(segments: I) -> Result<Path, PathError>
	where
		I: IntoIterator<Item = <MetaForm as Form>::String>,
	{
		let segments = segments.into_iter().collect::<Vec<_>>();
		if segments.is_empty() {
			return Err(PathError::MissingSegments);
		}
		if let Some(err_at) = segments.iter().position(|seg| !is_rust_identifier(seg)) {
			return Err(PathError::InvalidIdentifier { segment: err_at });
		}
		Ok(Path { segments })
	}
}

impl<F> Path<F>
where
	F: Form,
{
	pub fn is_empty(&self) -> bool {
		self.segments.is_empty()
	}

	/// Get the ident segment of the Path
	pub fn ident(&self) -> Option<&F::String> {
		self.segments.iter().last()
	}

	/// Get the namespace segments of the Path
	pub fn namespace(&self) -> &[F::String] {
		self.segments.split_last().map(|(_, ns)| ns).unwrap_or(&[])
	}
}

/// An error that may be encountered upon constructing namespaces.
#[derive(PartialEq, Eq, Debug)]
pub enum PathError {
	/// If the module path does not at least have one segment.
	MissingSegments,
	/// If a segment within a module path is not a proper Rust identifier.
	InvalidIdentifier {
		/// The index of the erroneous segment.
		segment: usize,
	},
}

/// State types for type builders which require a Path
pub mod state {
	/// State where the builder has not assigned a Path to the type
	pub enum PathNotAssigned {}
	/// State where the builder has assigned a Path to the type
	pub enum PathAssigned {}
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
			Path::new("hello::world", "Planet"),
			Ok(Path {
				segments: vec!["hello", "world", "Planet"]
			})
		);
		assert_eq!(
			Path::new("::world", "Earth"),
			Err(PathError::InvalidIdentifier { segment: 0 })
		);
	}

	#[test]
	fn path_get_namespace_and_ident() {
		let path = Path::new("hello::world", "Planet").unwrap();
		assert_eq!(path.namespace(), &["hello", "world"]);
		assert_eq!(path.ident(), Some(&"Planet"));
	}
}

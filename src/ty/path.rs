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

use crate::prelude::{
    fmt::{
        Display,
        Error as FmtError,
        Formatter,
    },
    vec,
    vec::Vec,
};

use crate::{
    form::{
        Form,
        PortableForm,
        MetaForm,
    },
    utils::is_rust_identifier,
    IntoPortable,
    Registry,
};
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "serde")]
use serde::{
    de::DeserializeOwned,
    Deserialize,
    Serialize,
};

/// Represents the path of a type definition.
///
/// This consists of several segments that each have to be a valid Rust
/// identifier. The first segment represents the crate name in which the type
/// has been defined. The last
///
/// Rust prelude type may have an empty namespace definition.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T::Type: Serialize, T::String: Serialize",
        deserialize = "T::Type: DeserializeOwned, T::String: DeserializeOwned",
    ))
)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Path<T: Form = MetaForm> {
    /// The segments of the namespace.
    segments: Vec<T::String>,
}

impl<T> Default for Path<T>
where
    T: Form,
{
    fn default() -> Self {
        Path {
            segments: Vec::new(),
        }
    }
}

impl IntoPortable for Path {
    type Output = Path<PortableForm>;

    fn into_portable(self, registry: &mut Registry) -> Self::Output {
        Path {
            segments: registry.map_into_portable(self.segments),
        }
    }
}

impl Display for Path<PortableForm> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "{}", self.segments.join("::"))
    }
}

impl Path {
    /// Create a new Path
    ///
    /// # Panics
    ///
    /// - If the type identifier or module path contain invalid Rust identifiers
    pub fn new(ident: &'static str, module_path: &'static str) -> Path {
        let mut segments = module_path.split("::").collect::<Vec<_>>();
        segments.push(ident);
        Self::from_segments(segments)
            .expect("All path segments should be valid Rust identifiers")
    }

    /// Create an empty path for types which shall not be named
    #[allow(unused)]
    pub(crate) fn voldemort() -> Path {
        Path {
            segments: Vec::new(),
        }
    }

    /// Crate a Path for types in the Prelude namespace
    ///
    /// # Panics
    ///
    /// - If the supplied ident is not a valid Rust identifier
    pub(crate) fn prelude(ident: &'static str) -> Path {
        Self::from_segments(vec![ident])
            .unwrap_or_else(|_| panic!("{} is not a valid Rust identifier", ident))
    }

    /// Create a Path from the given segments
    ///
    /// # Errors
    ///
    /// - If no segments are supplied
    /// - If any of the segments are invalid Rust identifiers
    pub fn from_segments<I>(segments: I) -> Result<Path, PathError>
    where
        I: IntoIterator<Item = &'static str>,
    {
        let segments = segments.into_iter().collect::<Vec<_>>();
        if segments.is_empty() {
            return Err(PathError::MissingSegments)
        }
        if let Some(err_at) = segments.iter().position(|seg| !is_rust_identifier(seg)) {
            return Err(PathError::InvalidIdentifier { segment: err_at })
        }
        Ok(Path { segments })
    }
}

impl<T> Path<T>
where
    T: Form,
{
    /// Returns the segments of the Path
    pub fn segments(&self) -> &[T::String] {
        &self.segments
    }

    /// Returns `true` if the path is empty
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Get the ident segment of the Path
    pub fn ident(&self) -> Option<T::String> {
        self.segments.iter().last().cloned()
    }

    /// Get the namespace segments of the Path
    pub fn namespace(&self) -> &[T::String] {
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
        assert_eq!(
            Path::from_segments(vec!["_"]),
            Ok(Path {
                segments: vec!["_"]
            })
        );
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
            Path::new("Planet", "hello::world"),
            Path {
                segments: vec!["hello", "world", "Planet"]
            }
        );
        assert_eq!(
            Path::from_segments(vec!["Earth", "::world"]),
            Err(PathError::InvalidIdentifier { segment: 1 })
        );
    }

    #[test]
    fn path_get_namespace_and_ident() {
        let path = Path::new("Planet", "hello::world");
        assert_eq!(path.namespace(), &["hello", "world"]);
        assert_eq!(path.ident(), Some("Planet"));
    }

    #[test]
    #[should_panic]
    fn path_new_panics_with_invalid_identifiers() {
        Path::new("Planet", "hello$!@$::world");
    }

    #[test]
    fn path_display() {
        let path =
            Path::new("Planet", "hello::world").into_portable(&mut Default::default());
        assert_eq!("hello::world::Planet", format!("{}", path))
    }
}

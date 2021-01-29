// Copyright 2019-2021 Parity Technologies (UK) Ltd.
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

//! The registry stores type definitions in a space-efficient manner.
//!
//! This is done by deduplicating common types in order to reuse their
//! definitions which otherwise can grow arbitrarily large. A type is uniquely
//! identified by its type identifier that is therefore used to refer to types
//! and their definitions.
//!
//! Types with the same name are uniquely identifiable by introducing
//! namespaces. The normal Rust namespace of a type is used, except for the Rust
//! prelude types that live in the so-called root namespace which is empty.

use crate::prelude::{
    any::TypeId,
    collections::BTreeMap,
    fmt::Debug,
    num::NonZeroU32,
    vec::Vec,
};

use crate::{
    form::{
        Form,
        PortableForm,
    },
    interner::{
        Interner,
        UntrackedSymbol,
    },
    meta_type::MetaType,
    Type,
};
use scale::{
    Decode,
    Encode,
};
#[cfg(feature = "serde")]
use serde::{
    Deserialize,
    Serialize,
};

/// Convert the type definition into the portable form using a registry.
pub trait IntoPortable {
    /// The portable version of `Self`.
    type Output;

    /// Convert `self` to the portable form by using the registry for caching.
    fn into_portable(self, registry: &mut Registry) -> Self::Output;
}

impl IntoPortable for &'static str {
    type Output = <PortableForm as Form>::String;

    fn into_portable(self, _registry: &mut Registry) -> Self::Output {
        self.into()
    }
}

/// The registry for space-efficient storage of type identifiers and
/// definitions.
///
/// The registry consists of a cache for type identifiers and definitions.
///
/// When adding a type to  the registry, all of its sub-types are registered
/// recursively as well. A type is considered a sub-type of another type if it
/// is used by its identifier or structure.
///
/// # Note
///
/// A type can be a sub-type of itself. In this case the registry has a builtin
/// mechanism to stop recursion and avoid going into an infinite loop.
#[derive(Debug, PartialEq, Eq)]
pub struct Registry {
    /// The cache for already registered types.
    ///
    /// This is just an accessor to the actual database
    /// for all types found in the `types` field.
    type_table: Interner<TypeId>,
    /// The database where registered types reside.
    ///
    /// The contents herein is used for serlialization.
    types: BTreeMap<UntrackedSymbol<core::any::TypeId>, Type<PortableForm>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            type_table: Interner::new(),
            types: BTreeMap::new(),
        }
    }

    /// Registers the given type ID into the registry.
    ///
    /// Returns `false` as the first return value if the type ID has already
    /// been registered into this registry.
    /// Returns the associated type ID symbol as second return value.
    ///
    /// # Note
    ///
    /// This is an internal API and should not be called directly from the
    /// outside.
    fn intern_type_id(&mut self, type_id: TypeId) -> (bool, UntrackedSymbol<TypeId>) {
        let (inserted, symbol) = self.type_table.intern_or_get(type_id);
        (inserted, symbol.into_untracked())
    }

    /// Registers the given type into the registry and returns
    /// its associated type ID symbol.
    ///
    /// # Note
    ///
    /// Due to safety requirements the returns type ID symbol cannot
    /// be used later to resolve back to the associated type definition.
    /// However, since this facility is going to be used for serialization
    /// purposes this functionality isn't needed anyway.
    pub fn register_type(&mut self, ty: &MetaType) -> UntrackedSymbol<TypeId> {
        let (inserted, symbol) = self.intern_type_id(ty.type_id());
        if inserted {
            let portable_id = ty.type_info().into_portable(self);
            self.types.insert(symbol, portable_id);
        }
        symbol
    }

    /// Calls `register_type` for each `MetaType` in the given `iter`.
    pub fn register_types<I>(&mut self, iter: I) -> Vec<UntrackedSymbol<TypeId>>
    where
        I: IntoIterator<Item = MetaType>,
    {
        iter.into_iter()
            .map(|i| self.register_type(&i))
            .collect::<Vec<_>>()
    }

    /// Converts an iterator into a Vec of the equivalent portable
    /// representations.
    pub fn map_into_portable<I, T>(&mut self, iter: I) -> Vec<T::Output>
    where
        I: IntoIterator<Item = T>,
        T: IntoPortable,
    {
        iter.into_iter()
            .map(|i| i.into_portable(self))
            .collect::<Vec<_>>()
    }
}

/// A read-only registry containing types in their portable form for serialization.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct PortableRegistry {
    types: Vec<Type<PortableForm>>,
}

impl From<Registry> for PortableRegistry {
    fn from(registry: Registry) -> Self {
        PortableRegistry {
            types: registry.types.values().cloned().collect::<Vec<_>>(),
        }
    }
}

impl PortableRegistry {
    /// Returns the type definition for the given identifier, `None` if no type found for that ID.
    pub fn resolve(&self, id: NonZeroU32) -> Option<&Type<PortableForm>> {
        self.types.get((id.get() - 1) as usize)
    }

    /// Returns an iterator for all types paired with their associated NonZeroU32 identifier.
    pub fn enumerate(
        &self,
    ) -> impl Iterator<Item = (NonZeroU32, &Type<PortableForm>)> {
        self.types.iter().enumerate().map(|(i, ty)| {
            let id = NonZeroU32::new(i as u32 + 1).expect("i + 1 > 0; qed");
            (id, ty)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build::Fields,
        meta_type,
        Path,
        TypeDef,
        TypeInfo,
    };

    #[test]
    fn readonly_enumerate() {
        let mut registry = Registry::new();
        registry.register_type(&MetaType::new::<u32>());
        registry.register_type(&MetaType::new::<bool>());
        registry.register_type(&MetaType::new::<Option<(u32, bool)>>());

        let readonly: PortableRegistry = registry.into();

        assert_eq!(4, readonly.enumerate().count());

        let mut expected = 1;
        for (i, _) in readonly.enumerate() {
            assert_eq!(NonZeroU32::new(expected).unwrap(), i);
            expected += 1;
        }
    }

    #[test]
    fn recursive_struct_with_references() {
        #[allow(unused)]
        struct RecursiveRefs<'a> {
            boxed: Box<RecursiveRefs<'a>>,
            reference: &'a RecursiveRefs<'a>,
            mutable_reference: &'a mut RecursiveRefs<'a>,
        }

        impl TypeInfo for RecursiveRefs<'static> {
            type Identity = Self;

            fn type_info() -> Type {
                Type::builder()
                    .path(Path::new("RecursiveRefs", module_path!()))
                    .composite(
                        Fields::named()
                            .field_of::<Box<RecursiveRefs>>(
                                "boxed",
                                "Box < RecursiveRefs >",
                            )
                            .field_of::<&'static RecursiveRefs<'static>>(
                                "reference",
                                "&RecursiveRefs",
                            )
                            .field_of::<&'static mut RecursiveRefs<'static>>(
                                "mutable_reference",
                                "&mut RecursiveRefs",
                            ),
                    )
                    .into()
            }
        }

        let mut registry = Registry::new();
        let type_id = registry.register_type(&meta_type::<RecursiveRefs>());

        let recursive = registry.types.get(&type_id).unwrap();
        if let TypeDef::Composite(composite) = recursive.type_def() {
            for field in composite.fields() {
                assert_eq!(*field.ty(), type_id)
            }
        } else {
            panic!("Should be a composite type definition")
        }
    }
}

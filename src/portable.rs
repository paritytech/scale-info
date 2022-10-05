// Copyright 2019-2022 Parity Technologies (UK) Ltd.
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

use crate::{
    form::PortableForm,
    prelude::{
        fmt::Debug,
        vec::Vec,
    },
    Registry,
    Type,
};
use scale::Encode;

/// A read-only registry containing types in their portable form for serialization.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "serde", feature = "decode"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct PortableRegistry {
    types: Vec<PortableType>,
}

impl From<Registry> for PortableRegistry {
    fn from(registry: Registry) -> Self {
        PortableRegistry {
            types: registry
                .types()
                .map(|(k, v)| {
                    PortableType {
                        id: k.id(),
                        ty: v.clone(),
                    }
                })
                .collect::<Vec<_>>(),
        }
    }
}

impl PortableRegistry {
    /// Construct a new `PortableRegistry` from custom types.
    pub fn new_from_types(types: Vec<PortableType>) -> Self {
        Self { types }
    }

    /// Returns the type definition for the given identifier, `None` if no type found for that ID.
    pub fn resolve(&self, id: u32) -> Option<&Type<PortableForm>> {
        self.types.get(id as usize).map(|ty| ty.ty())
    }

    /// Returns all types with their associated identifiers.
    pub fn types(&self) -> &[PortableType] {
        &self.types
    }
}

/// Represent a type in it's portable form.
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(all(feature = "serde", feature = "decode"), derive(serde::Deserialize))]
#[cfg_attr(any(feature = "std", feature = "decode"), derive(scale::Decode))]
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct PortableType {
    #[codec(compact)]
    id: u32,
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    ty: Type<PortableForm>,
}

impl PortableType {
    /// Construct a custom `PortableType`.
    pub fn new(id: u32, ty: Type<PortableForm>) -> Self {
        Self { id, ty }
    }

    /// Returns the index of the [`PortableType`].
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the type of the [`PortableType`].
    pub fn ty(&self) -> &Type<PortableForm> {
        &self.ty
    }
}

/// Construct a [`PortableRegistry`].
pub struct PortableRegistryBuilder {

}

#[cfg(test)]
mod tests {
    use crate::{
        build::*,
        prelude::vec,
        *,
    };
    use super::*;

    #[test]
    fn type_ids_are_sequential() {
        let mut registry = Registry::new();
        registry.register_type(&MetaType::new::<u32>());
        registry.register_type(&MetaType::new::<bool>());
        registry.register_type(&MetaType::new::<Option<(u32, bool)>>());

        let readonly: PortableRegistry = registry.into();

        assert_eq!(4, readonly.types().len());

        for (expected, ty) in readonly.types().iter().enumerate() {
            assert_eq!(expected as u32, ty.id());
        }
    }

    #[test]
    fn construct_portable_registry() {
        let mut types = Vec::new();

        let u32_type_id = types.len() as u32;
        types.push(PortableType::new(
            u32_type_id,
            Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]),
        ));

        let vec_u32_type_id = types.len() as u32;
        types.push(PortableType::new(
            vec_u32_type_id,
            Type::new(
                Path::default(),
                vec![],
                TypeDefSequence::new(u32_type_id.into()),
                vec![],
            ),
        ));

        let composite_type_id = types.len() as u32;
        let composite = Type::builder_portable()
            .path(Path::new_custom(vec!["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(u32_type_id))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id))
                    .field_portable(|f| {
                        f.name("self_referential".into()).ty(composite_type_id)
                    }),
            );
        types.push(PortableType::new(composite_type_id, composite));

        let _registry = PortableRegistry::new_from_types(types);
    }
}


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
    interner::Interner,
    prelude::{
        collections::{
            HashMap,
            HashSet,
        },
        fmt::Debug,
        vec::Vec,
    },
    Registry,
    Type,
    TypeDef,
};
use core::sync::atomic::AtomicU32;
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
    /// Returns the type definition for the given identifier, `None` if no type found for that ID.
    pub fn resolve(&self, id: u32) -> Option<&Type<PortableForm>> {
        self.types.get(id as usize).map(|ty| ty.ty())
    }

    /// Returns all types with their associated identifiers.
    pub fn types(&self) -> &[PortableType] {
        &self.types
    }

    /// Retains only the portable types needed to express the provided ids.
    ///
    /// The type IDs retained are returned as key to the `HashMap`.
    /// The value of the `HashMap` represents the new ID of that type.
    ///
    /// # Note
    ///
    /// A given type ID can be expressed by nesting type IDs, such is the case
    /// of a [`TypeDef::Composite`] and others. To retain a valid [`PortableRegistry`]
    /// all the types needed to express an ID are retained. Therefore, the number of
    /// elements expressed by the result is equal or greater than the number of
    /// provided IDs.
    pub fn retain(
        &mut self,
        ids: impl IntoIterator<Item = u32>,
    ) -> Result<HashMap<u32, u32>, PortableRegistryError> {
        // Recursively visit all type ids needed to express the list of provided ids.
        let resolver = TypeIdResolver::new(self);
        // Map of "old id" to "new id".
        let ids_map = resolver.resolve(ids)?;

        // Sort the ids by their order in the new registry.
        let mut ids_order: Vec<_> = ids_map.clone().into_iter().collect();
        ids_order.sort_by(|(_, lhs_new), (_, rhs_new)| lhs_new.cmp(rhs_new));

        // We cannot construct directly a new `PortableRegistry` by registering
        // the current types because they may contain recursive type ids
        // that must be updated.
        let mut types = Vec::with_capacity(ids_order.len());
        for (old_id, new_id) in ids_order.iter() {
            let ty = self
                .types
                .get_mut(*old_id as usize)
                .ok_or(PortableRegistryError::MissingTypeID { id: *old_id })?;
            let mut ty = ty.clone();
            ty.id = *new_id;
            self.update_type(&ids_map, &mut ty.ty)?;
            types.push(ty);
        }

        self.types = types;

        Ok(ids_map)
    }

    /// Update all the type IDs composting the given type.
    fn update_type(
        &self,
        ids_map: &HashMap<u32, u32>,
        ty: &mut Type<PortableForm>,
    ) -> Result<(), PortableRegistryError> {
        for param in ty.type_params.iter_mut() {
            let Some(ty) = param.ty() else {
                continue
            };

            let new_id = ids_map
                .get(&ty.id())
                .ok_or(PortableRegistryError::MissingTypeID { id: ty.id() })?;
            param.ty = Some(*new_id).map(Into::into);
        }

        match &mut ty.type_def {
            TypeDef::Composite(composite) => {
                for field in composite.fields.iter_mut() {
                    let new_id = ids_map.get(&field.ty().id()).ok_or(
                        PortableRegistryError::MissingTypeID {
                            id: field.ty().id(),
                        },
                    )?;
                    field.ty = (*new_id).into();
                }
            }
            TypeDef::Variant(variant) => {
                for var in variant.variants.iter_mut() {
                    for field in var.fields.iter_mut() {
                        let new_id = ids_map.get(&field.ty().id()).ok_or(
                            PortableRegistryError::MissingTypeID {
                                id: field.ty().id(),
                            },
                        )?;
                        field.ty = (*new_id).into();
                    }
                }
            }
            TypeDef::Sequence(sequence) => {
                let new_id = ids_map.get(&sequence.type_param().id()).ok_or(
                    PortableRegistryError::MissingTypeID {
                        id: sequence.type_param().id(),
                    },
                )?;
                sequence.type_param = (*new_id).into();
            }
            TypeDef::Array(array) => {
                let new_id = ids_map.get(&array.type_param().id()).ok_or(
                    PortableRegistryError::MissingTypeID {
                        id: array.type_param().id(),
                    },
                )?;
                array.type_param = (*new_id).into();
            }
            TypeDef::Tuple(tuple) => {
                for ty in tuple.fields.iter_mut() {
                    let new_id = ids_map
                        .get(&ty.id())
                        .ok_or(PortableRegistryError::MissingTypeID { id: ty.id() })?;
                    *ty = (*new_id).into();
                }
            }
            TypeDef::Primitive(_) => (),
            TypeDef::Compact(compact) => {
                let new_id = ids_map.get(&compact.type_param().id()).ok_or(
                    PortableRegistryError::MissingTypeID {
                        id: compact.type_param().id(),
                    },
                )?;
                compact.type_param = (*new_id).into();
            }
            TypeDef::BitSequence(bit_seq) => {
                let new_id = ids_map.get(&bit_seq.bit_order_type().id()).ok_or(
                    PortableRegistryError::MissingTypeID {
                        id: bit_seq.bit_order_type().id(),
                    },
                )?;
                bit_seq.bit_order_type = (*new_id).into();

                let new_id = ids_map.get(&bit_seq.bit_store_type().id()).ok_or(
                    PortableRegistryError::MissingTypeID {
                        id: bit_seq.bit_store_type().id(),
                    },
                )?;
                bit_seq.bit_store_type = (*new_id).into();
            }
        };

        Ok(())
    }
}

/// An error that may be encountered upon using the portable registry.
#[derive(PartialEq, Eq, Debug)]
pub enum PortableRegistryError {
    /// The type ID is not present in the registry.
    MissingTypeID {
        /// The type ID that is missing.
        id: u32,
    },
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
///
/// Guarantees that the resulting [`PortableRegistry`] has the list of types in the correct order,
/// since downstream libs assume that a `u32` type id corresponds to the index of the type
/// definition type table.
#[derive(Debug, Default)]
pub struct PortableRegistryBuilder {
    types: Interner<Type<PortableForm>>,
}

impl PortableRegistryBuilder {
    /// Create a new [`PortableRegistryBuilder`].
    pub fn new() -> Self {
        Default::default()
    }

    /// Register a type, returning the assigned ID.
    ///
    /// If the type is already registered it will return the existing ID.
    pub fn register_type(&mut self, ty: Type<PortableForm>) -> u32 {
        self.types.intern_or_get(ty).1.into_untracked().id()
    }

    /// Returns the type id that would be assigned to a newly registered type.
    pub fn next_type_id(&self) -> u32 {
        self.types.elements().len() as u32
    }

    /// Returns a reference to the type registered at the given ID (if any).
    pub fn get(&self, id: u32) -> Option<&Type<PortableForm>> {
        self.types.elements().get(id as usize)
    }

    /// Finalize and return a valid [`PortableRegistry`] instance.
    pub fn finish(&self) -> PortableRegistry {
        let types = self
            .types
            .elements()
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                PortableType {
                    id: i as u32,
                    ty: ty.clone(),
                }
            })
            .collect();
        PortableRegistry { types }
    }
}

/// Recursive resolver for the type IDs needed to express a given type ID.
struct TypeIdResolver<'a> {
    registry: &'a PortableRegistry,
    result: HashMap<u32, u32>,
    next_id: AtomicU32,
}

impl<'a> TypeIdResolver<'a> {
    /// Construct a new [`TypeIdResolver`].
    fn new(registry: &'a PortableRegistry) -> Self {
        TypeIdResolver {
            registry,
            result: Default::default(),
            next_id: Default::default(),
        }
    }

    /// Get the next unique ID.
    fn next_id(&mut self) -> u32 {
        self.next_id
            .fetch_add(1, core::sync::atomic::Ordering::Relaxed)
    }

    /// Recursively add all type ids needed to express the given identifier.
    fn visit_type_id(&mut self, id: u32) -> Result<(), PortableRegistryError> {
        if self.result.get(&id).is_some() {
            return Ok(())
        }

        let ty = self
            .registry
            .resolve(id)
            .ok_or(PortableRegistryError::MissingTypeID { id })?;

        let new_id = self.next_id();
        self.result.insert(id, new_id);

        // Add generic type params.
        for param in ty.type_params() {
            if let Some(ty) = param.ty() {
                self.visit_type_id(ty.id())?;
            }
        }

        // Recursively visit any other type ids needed to represent this type.
        match ty.type_def() {
            TypeDef::Composite(composite) => {
                for field in composite.fields() {
                    self.visit_type_id(field.ty().id())?;
                }
            }
            TypeDef::Variant(variant) => {
                for var in variant.variants() {
                    for field in var.fields() {
                        self.visit_type_id(field.ty().id())?;
                    }
                }
            }
            TypeDef::Sequence(sequence) => {
                self.visit_type_id(sequence.type_param().id())?;
            }
            TypeDef::Array(array) => {
                self.visit_type_id(array.type_param().id())?;
            }
            TypeDef::Tuple(tuple) => {
                for ty in tuple.fields() {
                    self.visit_type_id(ty.id())?;
                }
            }
            TypeDef::Primitive(_) => {}
            TypeDef::Compact(compact) => {
                self.visit_type_id(compact.type_param().id())?;
            }
            TypeDef::BitSequence(bit_sequence) => {
                self.visit_type_id(bit_sequence.bit_store_type().id())?;
                self.visit_type_id(bit_sequence.bit_order_type().id())?;
            }
        }

        Ok(())
    }

    /// Resolve all the type IDs needed to express the given type IDs.
    ///
    /// The type IDs are returned as key to the `HashMap`.
    /// The value of the `HashMap` represents the new ID of that type
    /// if only the resolved types are expressed in the [`PortableRegistry`].
    fn resolve(
        mut self,
        ids: impl IntoIterator<Item = u32>,
    ) -> Result<HashMap<u32, u32>, PortableRegistryError> {
        let ids: HashSet<_> = ids.into_iter().collect();
        for id in ids {
            self.visit_type_id(id)?;
        }

        Ok(self.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        build::*,
        prelude::vec,
        *,
    };

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
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type.clone());

        let vec_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(u32_type_id.into()),
            vec![],
        );
        let vec_u32_type_id = builder.register_type(vec_u32_type.clone());

        let self_referential_type_id = builder.next_type_id();

        let composite_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(u32_type_id))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id))
                    .field_portable(|f| {
                        f.name("self_referential".into())
                            .ty(self_referential_type_id)
                    }),
            );
        let composite_type_id = builder.register_type(composite_type.clone());

        assert_eq!(self_referential_type_id, composite_type_id);

        assert_eq!(builder.get(u32_type_id).unwrap(), &u32_type);
        assert_eq!(builder.get(vec_u32_type_id).unwrap(), &vec_u32_type);
        assert_eq!(builder.get(composite_type_id).unwrap(), &composite_type);

        let registry = builder.finish();

        assert_eq!(Some(&u32_type), registry.resolve(u32_type_id));
        assert_eq!(Some(&vec_u32_type), registry.resolve(vec_u32_type_id));
        assert_eq!(Some(&composite_type), registry.resolve(composite_type_id));
    }

    #[test]
    fn retain_ids() {
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type.clone());

        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let u64_type_id = builder.register_type(u64_type.clone());

        let mut registry = builder.finish();
        assert_eq!(registry.types.len(), 2);

        // Resolve just u64.
        let resolver = TypeIdResolver::new(&registry);
        let result = resolver.resolve(vec![u64_type_id]).unwrap();
        assert_eq!(result.len(), 1);
        // Make sure `u32_type_id` is not present.
        assert!(!result.contains_key(&u32_type_id));

        // `u64_type_id` should be mapped on id `0`.
        assert_eq!(result.get(&u64_type_id).unwrap(), &0);

        let expected_result = registry.retain(vec![u64_type_id]).unwrap();
        assert_eq!(expected_result, result);
        assert_eq!(registry.types.len(), 1);

        assert_eq!(registry.resolve(0).unwrap(), &u64_type);
    }

    #[test]
    fn retain_recursive_ids() {
        let mut builder = PortableRegistryBuilder::new();
        let u32_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U32, vec![]);
        let u32_type_id = builder.register_type(u32_type.clone());

        let u64_type = Type::new(Path::default(), vec![], TypeDefPrimitive::U64, vec![]);
        let u64_type_id = builder.register_type(u64_type.clone());

        let vec_u32_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(u32_type_id.into()),
            vec![],
        );
        let vec_u32_type_id = builder.register_type(vec_u32_type.clone());

        let composite_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(u32_type_id))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id)),
            );
        let composite_type_id = builder.register_type(composite_type.clone());

        let composite_type_second = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStructSecond".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(vec_u32_type_id))
                    .field_portable(|f| f.name("second".into()).ty(composite_type_id)),
            );
        let composite_type_second_id =
            builder.register_type(composite_type_second.clone());

        let mut registry = builder.finish();
        assert_eq!(registry.types.len(), 5);

        // Resolve just `MyStruct`.
        let resolver = TypeIdResolver::new(&registry);
        let result = resolver.resolve(vec![composite_type_second_id]).unwrap();
        assert_eq!(result.len(), 4);
        // Make sure `u64_type_id` is not present.
        assert!(!result.contains_key(&u64_type_id));

        // `MyStructSecond` is the first id visited.
        assert_eq!(result.get(&composite_type_second_id).unwrap(), &0);
        assert_eq!(result.get(&vec_u32_type_id).unwrap(), &1);
        assert_eq!(result.get(&u32_type_id).unwrap(), &2);
        assert_eq!(result.get(&composite_type_id).unwrap(), &3);

        let expected_result = registry.retain(vec![composite_type_second_id]).unwrap();
        assert_eq!(expected_result, result);
        assert_eq!(registry.types.len(), 4);

        // New type IDs are generated in DFS manner.
        let expected_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStructSecond".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(1))
                    .field_portable(|f| f.name("second".into()).ty(3)),
            );
        assert_eq!(registry.resolve(0).unwrap(), &expected_type);

        let expected_type = Type::new(
            Path::default(),
            vec![],
            TypeDefSequence::new(2.into()),
            vec![],
        );
        assert_eq!(registry.resolve(1).unwrap(), &expected_type);
        assert_eq!(registry.resolve(2).unwrap(), &u32_type);

        let expected_type = Type::builder_portable()
            .path(Path::from_segments_unchecked(["MyStruct".into()]))
            .composite(
                Fields::named()
                    .field_portable(|f| f.name("primitive".into()).ty(2))
                    .field_portable(|f| f.name("vec_of_u32".into()).ty(1)),
            );
        assert_eq!(registry.resolve(3).unwrap(), &expected_type);
    }
}

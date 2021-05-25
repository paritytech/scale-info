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

use crate::prelude::{
    marker::PhantomData,
    vec::Vec,
};

use crate::{
    Field,
    TypeInfo,
    form::MetaForm,
};

/// A fields builder has no fields (e.g. a unit struct)
pub enum NoFields {}
/// A fields builder only allows named fields (e.g. a struct)
pub enum NamedFields {}
/// A fields builder only allows unnamed fields (e.g. a tuple)
pub enum UnnamedFields {}

/// Provides FieldsBuilder constructors
pub enum Fields {}

impl Fields {
    /// The type construct has no fields
    pub fn unit() -> FieldsBuilder<NoFields> {
        FieldsBuilder::<NoFields>::default()
    }

    /// Fields for a type construct with named fields
    pub fn named() -> FieldsBuilder<NamedFields> {
        FieldsBuilder::default()
    }

    /// Fields for a type construct with unnamed fields
    pub fn unnamed() -> FieldsBuilder<UnnamedFields> {
        FieldsBuilder::default()
    }
}

/// Build a set of either all named (e.g. for a struct) or all unnamed (e.g. for a tuple struct)
pub struct FieldsBuilder<T> {
    fields: Vec<Field>,
    marker: PhantomData<fn() -> T>,
}

impl<T> Default for FieldsBuilder<T> {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            marker: Default::default(),
        }
    }
}

impl<T> FieldsBuilder<T> {
    /// Complete building and return the set of fields
    pub fn finalize(self) -> Vec<Field<MetaForm>> {
        self.fields
    }
}

impl FieldsBuilder<NamedFields> {
    /// Add a named field with the type of the type parameter `T`
    pub fn field_of<T>(
        mut self,
        name: &'static str,
        type_name: &'static str,
        docs: &[&'static str],
    ) -> Self
        where
            T: TypeInfo + ?Sized + 'static,
    {
        self.fields
            .push(Field::named_of::<T>(name, type_name, docs));
        self
    }

    /// Add a named, [`Compact`] field of type `T`.
    pub fn compact_of<T>(
        mut self,
        name: &'static str,
        type_name: &'static str,
        docs: &[&'static str],
    ) -> Self
        where
            T: scale::HasCompact,
            <T as scale::HasCompact>::Type: TypeInfo + 'static,
    {
        self.fields
            .push(Field::compact_of::<T>(Some(name), type_name, docs));
        self
    }
}

impl FieldsBuilder<UnnamedFields> {
    /// Add an unnamed field with the type of the type parameter `T`
    pub fn field_of<T>(mut self, type_name: &'static str, docs: &[&'static str]) -> Self
        where
            T: TypeInfo + ?Sized + 'static,
    {
        self.fields.push(Field::unnamed_of::<T>(type_name, docs));
        self
    }

    /// Add an unnamed, [`Compact`] field of type `T`.
    pub fn compact_of<T>(mut self, type_name: &'static str, docs: &[&'static str]) -> Self
        where
            T: scale::HasCompact,
            <T as scale::HasCompact>::Type: TypeInfo + 'static,
    {
        self.fields
            .push(Field::compact_of::<T>(None, type_name, docs));
        self
    }
}
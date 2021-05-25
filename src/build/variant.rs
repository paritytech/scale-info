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

use crate::prelude::vec::Vec;

use crate::{
    form::MetaForm,
    Field,
    TypeDefVariant,
    Variant,
};

use super::*;

/// Builds a definition of a variant type i.e an `enum`
#[derive(Default)]
pub struct Variants {
    variants: Vec<Variant>,
}

impl Variants {
    /// Create a new [`VariantsBuilder`].
    pub fn new() -> Self {
        Variants {
            variants: Vec::new(),
        }
    }

    /// Add a variant with the
    pub fn variant(mut self, builder: VariantBuilder) -> Self {
        self.variants.push(builder.finalize());
        self
    }

    /// Construct a new [`TypeDefVariant`] from the initialized builder variants.
    pub fn finalize(self) -> TypeDefVariant {
        TypeDefVariant::new(self.variants)
    }
}

/// Build a [`Variant`].
pub struct VariantBuilder {
    name: &'static str,
    fields: Vec<Field<MetaForm>>,
    index: Option<u64>,
    docs: Vec<&'static str>,
}

impl VariantBuilder {
    /// Create a new [`VariantBuilder`].
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            fields: Vec::new(),
            index: None,
            docs: Vec::new(),
        }
    }

    /// Initialize the variant's index.
    pub fn index(mut self, index: u64) -> Self {
        self.index = Some(index);
        self
    }

    /// Initialize the variant's fields.
    pub fn fields<F>(mut self, fields_builder: FieldsBuilder<F>) -> Self {
        self.fields = fields_builder.finalize();
        self
    }

    /// Initialize the variant's documentation.
    pub fn docs(mut self, docs: &[&'static str]) -> Self {
        self.docs = docs.to_vec();
        self
    }

    /// Complete building and create final [`Variant`] instance.
    pub fn finalize(self) -> Variant<MetaForm> {
        Variant::new(self.name, self.fields, self.index, self.docs)
    }
}

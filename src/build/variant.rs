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
    TypeDefVariant,
    Variant,
};

use super::*;

/// Build a type with no variants.
pub enum NoVariants {}
/// Build a type where at least one variant has fields.
pub enum VariantFields {}
/// Build a type where *all* variants have no fields and the discriminant can
/// be directly chosen or accessed
pub enum Fieldless {}

/// Empty enum for VariantsBuilder constructors for the type builder DSL.
pub enum Variants {}

impl Variants {
    /// Build a set of variants, at least one of which will have fields.
    pub fn with_fields() -> VariantsBuilder<VariantFields> {
        VariantsBuilder::new()
    }

    /// Build a set of variants, none of which will have fields, and the discriminant can
    /// be directly chosen or accessed
    pub fn fieldless() -> VariantsBuilder<Fieldless> {
        VariantsBuilder::new()
    }
}

/// Builds a definition of a variant type i.e an `enum`
#[derive(Default)]
pub struct VariantsBuilder<T> {
    variants: Vec<Variant>,
    marker: PhantomData<fn() -> T>,
}

impl VariantsBuilder<VariantFields> {
    /// Add a variant with fields constructed by the supplied [`FieldsBuilder`](`crate::build::FieldsBuilder`)
    pub fn variant<F>(
        mut self,
        name: &'static str,
        fields: FieldsBuilder<F>,
        docs: &[&'static str],
    ) -> Self {
        self.variants
            .push(Variant::with_fields(name, fields, docs.to_vec()));
        self
    }

    /// Add a variant with no fields i.e. a unit variant
    pub fn variant_unit(self, name: &'static str, docs: &[&'static str]) -> Self {
        self.variant::<NoFields>(name, Fields::unit(), docs)
    }
}

impl VariantsBuilder<Fieldless> {
    /// Add a fieldless variant, explicitly setting the discriminant
    pub fn variant(
        mut self,
        name: &'static str,
        discriminant: u64,
        docs: &[&'static str],
    ) -> Self {
        self.variants
            .push(Variant::with_discriminant(name, discriminant, docs));
        self
    }
}

impl<T> VariantsBuilder<T> {
    fn new() -> Self {
        VariantsBuilder {
            variants: Vec::new(),
            marker: Default::default(),
        }
    }

    /// Construct a new [`TypeDefVariant`] from the initialized builder variants.
    pub fn finalize(self) -> TypeDefVariant {
        TypeDefVariant::new(self.variants)
    }
}

// pub struct VariantBuilder<F> {
//     name: &'static str,
//     fields: Option<FieldsBuilder<F>>,
//     discriminant: Option<u64>,
//     docs: Vec<&'static str>,
// }
//
// impl<F> VariantBuilder<F> {
//     pub fn new(name: &'static str) -> Self {
//         Self { name, fields: None, discriminant: None, docs: Vec::new() }
//     }
// }
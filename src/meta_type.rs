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

use std::sync::{
    Arc,
    Mutex,
};

use crate::prelude::{
    cmp::Ordering,
    fmt::{
        Debug,
        Error as FmtError,
        Formatter,
    },
    hash::{
        Hash,
        Hasher,
    },
};

use crate::{
    form::MetaForm,
    Type,
    TypeId,
    TypeInfo,
};

/// A metatype abstraction.
///
/// Allows to store compile-time type information at runtime.
/// This again allows to derive type ID and type definition from it.
///
/// This needs a conversion to another representation of types
/// in order to be serializable.
#[derive(Clone)]
pub struct MetaType {
    /// Function pointer to get type information.
    fn_type_info: Arc<Mutex<dyn FnMut() -> Type<MetaForm>>>,
    // The standard type ID (ab)used in order to provide
    // cheap implementations of the standard traits
    // such as `PartialEq`, `PartialOrd`, `Debug` and `Hash`.
    type_id: TypeId,
}

impl PartialEq for MetaType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for MetaType {}

impl PartialOrd for MetaType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.type_id.partial_cmp(&other.type_id)
    }
}

impl Ord for MetaType {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl Hash for MetaType {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.type_id.hash(state)
    }
}

impl Debug for MetaType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.type_id.fmt(f)
    }
}

impl MetaType {
    /// Creates a new meta type from the given compile-time known type.
    pub fn new<T>() -> Self
    where
        T: TypeInfo + ?Sized + 'static,
    {
        Self {
            fn_type_info: Arc::new(Mutex::new(<T as TypeInfo>::type_info)),
            type_id: TypeId::of::<T::Identity>(),
        }
    }

    /// Creates a new meta type from the user supplied type id and type info function.
    ///
    /// NOTE: It is the responsibility of the caller to ensure unique type ids per custom type.
    pub fn new_custom(
        type_id: u64,
        fn_type_info: Arc<Mutex<dyn FnMut() -> Type<MetaForm>>>,
    ) -> Self {
        Self {
            fn_type_info,
            type_id: TypeId::Custom(type_id),
        }
    }

    /// Returns the meta type information.
    pub fn type_info(&self) -> Type<MetaForm> {
        (self.fn_type_info.lock().unwrap())()
    }

    /// Returns the type identifier provided by `core::any`.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Returns true if this represents a type of [`core::marker::PhantomData`].
    pub(crate) fn is_phantom(&self) -> bool {
        self == &MetaType::new::<crate::impls::PhantomIdentity>()
    }
}

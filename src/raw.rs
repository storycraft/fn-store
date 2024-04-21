/*
 * Created on Thu Jul 06 2023
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use core::{mem::ManuallyDrop, ptr, ptr::NonNull};

use bumpalo::Bump;
use hashbrown::HashMap;
use nohash_hasher::BuildNoHashHasher;
use type_key::TypeKey;

#[derive(Debug)]
/// raw FnMap
pub struct RawFnMap {
    // [`TypeId`] only hashes lower 64 bits
    map: HashMap<TypeKey, Val, BuildNoHashHasher<u64>>,

    bump: ManuallyDrop<Bump>,
}

impl RawFnMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),

            bump: ManuallyDrop::new(Bump::new()),
        }
    }

    pub fn get<T: 'static>(&self, key: &TypeKey) -> Option<NonNull<T>> {
        Some(self.map.get(key)?.inner().cast::<T>())
    }

    /// insert value
    ///
    /// Returned pointer is covariant to lifetime 'a where &'a mut self
    pub fn insert<T: 'static>(&mut self, key: TypeKey, value: T) -> NonNull<T> {
        let value = Val(NonNull::from(self.bump.alloc(value)).cast());
        let ptr = value.inner();

        self.map.insert(key, value);

        ptr.cast::<T>()
    }

    pub fn reset(&mut self) {
        self.map.clear();
        self.bump.reset();
    }
}

impl Default for RawFnMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for RawFnMap {
    fn drop(&mut self) {
        self.map.clear();

        // SAFETY: Manually dropped to ensure allocated objects to drop first
        unsafe { ManuallyDrop::drop(&mut self.bump) }
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct Val(NonNull<()>);

impl Val {
    pub const fn inner(&self) -> NonNull<()> {
        self.0
    }
}

impl Drop for Val {
    fn drop(&mut self) {
        // SAFETY: Safe to drop since it is the only unique pointer
        unsafe { ptr::drop_in_place(self.0.as_ptr()) }
    }
}

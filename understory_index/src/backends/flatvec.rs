// Copyright 2025 the Understory Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Flat vector backend with linear scans. Small and simple; good for tiny sets.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt::Debug;

use crate::backend::Backend;
use crate::types::Aabb2D;

/// Flat vector backend with linear scans.
pub struct FlatVec<T: Copy + PartialOrd + Debug, P: Copy + Debug> {
    entries: Vec<Option<Aabb2D<T>>>,
    _p: core::marker::PhantomData<P>,
}

impl<T: Copy + PartialOrd + Debug, P: Copy + Debug> Default for FlatVec<T, P> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            _p: core::marker::PhantomData,
        }
    }
}

impl<T: Copy + PartialOrd + Debug, P: Copy + Debug> Debug for FlatVec<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let total = self.entries.len();
        let alive = self.entries.iter().filter(|e| e.is_some()).count();
        f.debug_struct("FlatVec")
            .field("total_slots", &total)
            .field("alive", &alive)
            .finish_non_exhaustive()
    }
}

impl<T: Copy + PartialOrd + Debug, P: Copy + Debug> Backend<T, P> for FlatVec<T, P> {
    fn insert(&mut self, slot: usize, aabb: Aabb2D<T>) {
        if self.entries.len() <= slot {
            self.entries.resize_with(slot + 1, || None);
        }
        self.entries[slot] = Some(aabb);
    }
    fn update(&mut self, slot: usize, aabb: Aabb2D<T>) {
        if let Some(e) = self.entries.get_mut(slot) {
            *e = Some(aabb);
        }
    }
    fn remove(&mut self, slot: usize) {
        if let Some(e) = self.entries.get_mut(slot) {
            *e = None;
        }
    }
    fn clear(&mut self) {
        self.entries.clear();
    }
    fn query_point<'a>(&'a self, x: T, y: T) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut out = Vec::new();
        for (i, slot) in self.entries.iter().enumerate() {
            if let Some(a) = slot.as_ref()
                && a.contains_point(x, y)
            {
                out.push(i);
            }
        }
        Box::new(out.into_iter())
    }
    fn query_rect<'a>(&'a self, rect: Aabb2D<T>) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut out = Vec::new();
        for (i, slot) in self.entries.iter().enumerate() {
            if let Some(a) = slot.as_ref()
                && !a.intersect(&rect).is_empty()
            {
                out.push(i);
            }
        }
        Box::new(out.into_iter())
    }
}

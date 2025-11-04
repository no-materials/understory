// Copyright 2025 the Understory Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Backend trait for spatial indexing implementations.

use alloc::boxed::Box;

use crate::types::Aabb2D;
use core::fmt::Debug;

/// Spatial backend abstraction used by `IndexGeneric`.
pub trait Backend<T: Copy + PartialOrd + Debug, P: Copy + Debug> {
    /// Insert a new slot into the spatial structure.
    fn insert(&mut self, slot: usize, aabb: Aabb2D<T>);

    /// Update an existing slot's AABB.
    fn update(&mut self, slot: usize, aabb: Aabb2D<T>);

    /// Remove a slot from the spatial structure.
    fn remove(&mut self, slot: usize);

    /// Clear all spatial structures.
    fn clear(&mut self);

    /// Query slots whose AABB contains the point.
    fn query_point<'a>(&'a self, x: T, y: T) -> Box<dyn Iterator<Item = usize> + 'a>;

    /// Query slots whose AABB intersects the rectangle.
    fn query_rect<'a>(&'a self, rect: Aabb2D<T>) -> Box<dyn Iterator<Item = usize> + 'a>;
}

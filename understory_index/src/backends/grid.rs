// Copyright 2025 the Understory Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Uniform grid backends. Provide cell-based spatial indexing for common scalars.

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::fmt::Debug;

use crate::backend::Backend;
use crate::types::Aabb2D;

/// Uniform grid backend.
///
/// Uses a fixed-size cell grid to accelerate queries. Coordinates are expected to be
/// non-negative; queries and updates map AABBs to covered cells and aggregate candidates.
pub struct GridF64<P: Copy + Debug> {
    cell_w: f64,
    cell_h: f64,
    origin_x: f64,
    origin_y: f64,
    entries: Vec<Option<Aabb2D<f64>>>,
    cells: Vec<(i64, i64, Vec<usize>)>,
    _p: core::marker::PhantomData<P>,
}

impl<P: Copy + Debug> GridF64<P> {
    /// Create a grid backend with the given cell size and origin offset.
    pub fn new(cell_w: f64, cell_h: f64, origin_x: f64, origin_y: f64) -> Self {
        Self {
            cell_w,
            cell_h,
            origin_x,
            origin_y,
            entries: Vec::new(),
            cells: Vec::new(),
            _p: core::marker::PhantomData,
        }
    }

    #[inline]
    fn floor_to_i64(v: f64) -> i64 {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "GridI64 casts are intentional and documented."
        )]
        let i = v as i64;
        if (i as f64) > v { i - 1 } else { i }
    }

    fn key_for(&self, x: f64, y: f64) -> (i64, i64) {
        let cw = self.cell_w;
        let ch = self.cell_h;
        debug_assert!(cw > 0.0 && ch > 0.0, "cell widths must be positive");
        let cx = Self::floor_to_i64((x - self.origin_x) / cw);
        let cy = Self::floor_to_i64((y - self.origin_y) / ch);
        (cx, cy)
    }

    fn cells_for_aabb(&self, a: &Aabb2D<f64>) -> Vec<(i64, i64)> {
        let (minx, miny) = self.key_for(a.min_x, a.min_y);
        let (maxx, maxy) = self.key_for(a.max_x, a.max_y);
        let mut out = Vec::new();
        for y in miny..=maxy {
            for x in minx..=maxx {
                out.push((x, y));
            }
        }
        out
    }

    fn find_cell_mut(&mut self, key: (i64, i64)) -> usize {
        if let Some((idx, _)) = self
            .cells
            .iter()
            .enumerate()
            .find(|(_, (cx, cy, _))| (*cx, *cy) == key)
        {
            idx
        } else {
            self.cells.push((key.0, key.1, Vec::new()));
            self.cells.len() - 1
        }
    }

    fn remove_from_cells(&mut self, slot: usize) {
        for (_, _, slots) in &mut self.cells {
            if let Some(pos) = slots.iter().position(|&s| s == slot) {
                slots.swap_remove(pos);
            }
        }
    }
}

impl<P: Copy + Debug> Backend<f64, P> for GridF64<P> {
    fn insert(&mut self, slot: usize, aabb: Aabb2D<f64>) {
        if self.entries.len() <= slot {
            self.entries.resize_with(slot + 1, || None);
        }
        self.entries[slot] = Some(aabb);
        for key in self.cells_for_aabb(&aabb) {
            let idx = self.find_cell_mut(key);
            self.cells[idx].2.push(slot);
        }
    }
    fn update(&mut self, slot: usize, aabb: Aabb2D<f64>) {
        self.remove_from_cells(slot);
        if let Some(e) = self.entries.get_mut(slot) {
            *e = Some(aabb);
            for key in self.cells_for_aabb(&aabb) {
                let idx = self.find_cell_mut(key);
                self.cells[idx].2.push(slot);
            }
        }
    }
    fn remove(&mut self, slot: usize) {
        self.remove_from_cells(slot);
        if let Some(e) = self.entries.get_mut(slot) {
            *e = None;
        }
    }
    fn clear(&mut self) {
        self.entries.clear();
        self.cells.clear();
    }
    fn query_point<'a>(&'a self, x: f64, y: f64) -> Box<dyn Iterator<Item = usize> + 'a> {
        let key = self.key_for(x, y);
        let mut set = BTreeSet::new();
        if let Some((_, _, slots)) = self.cells.iter().find(|(cx, cy, _)| (*cx, *cy) == key) {
            for &s in slots {
                set.insert(s);
            }
        }
        Box::new(set.into_iter())
    }
    fn query_rect<'a>(&'a self, rect: Aabb2D<f64>) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut set = BTreeSet::new();
        for key in self.cells_for_aabb(&rect) {
            if let Some((_, _, slots)) = self.cells.iter().find(|(cx, cy, _)| (*cx, *cy) == key) {
                for &s in slots {
                    set.insert(s);
                }
            }
        }
        Box::new(set.into_iter())
    }
}

impl<P: Copy + Debug> Debug for GridF64<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let total = self.entries.len();
        let alive = self.entries.iter().filter(|e| e.is_some()).count();
        let cells = self.cells.len();
        f.debug_struct("GridF64")
            .field("cell_w", &self.cell_w)
            .field("cell_h", &self.cell_h)
            .field("origin_x", &self.origin_x)
            .field("origin_y", &self.origin_y)
            .field("total_slots", &total)
            .field("alive", &alive)
            .field("cells", &cells)
            .finish_non_exhaustive()
    }
}

/// Uniform grid backend for f32 coordinates.
pub struct GridF32<P: Copy + Debug> {
    cell_w: f32,
    cell_h: f32,
    origin_x: f32,
    origin_y: f32,
    entries: Vec<Option<Aabb2D<f32>>>,
    cells: Vec<(i32, i32, Vec<usize>)>,
    _p: core::marker::PhantomData<P>,
}

impl<P: Copy + Debug> GridF32<P> {
    /// Create a grid backend with the given cell size and origin offset.
    ///
    /// Coordinates are mapped to integer cell indices by floor-division of
    /// `(x - origin_x) / cell_w` and `(y - origin_y) / cell_h`.
    pub fn new(cell_w: f32, cell_h: f32, origin_x: f32, origin_y: f32) -> Self {
        Self {
            cell_w,
            cell_h,
            origin_x,
            origin_y,
            entries: Vec::new(),
            cells: Vec::new(),
            _p: core::marker::PhantomData,
        }
    }

    #[inline]
    fn floor_to_i32(v: f32) -> i32 {
        #[allow(
            clippy::cast_possible_truncation,
            reason = "GridF32->i32 casts are intentional and documented."
        )]
        let i = v as i32;
        if (i as f32) > v { i - 1 } else { i }
    }

    fn key_for(&self, x: f32, y: f32) -> (i32, i32) {
        let cw = self.cell_w;
        let ch = self.cell_h;
        debug_assert!(cw > 0.0 && ch > 0.0, "cell widths must be positive");
        let cx = Self::floor_to_i32((x - self.origin_x) / cw);
        let cy = Self::floor_to_i32((y - self.origin_y) / ch);
        (cx, cy)
    }

    fn cells_for_aabb(&self, a: &Aabb2D<f32>) -> Vec<(i32, i32)> {
        let (minx, miny) = self.key_for(a.min_x, a.min_y);
        let (maxx, maxy) = self.key_for(a.max_x, a.max_y);
        let mut out = Vec::new();
        for y in miny..=maxy {
            for x in minx..=maxx {
                out.push((x, y));
            }
        }
        out
    }

    fn find_cell_mut(&mut self, key: (i32, i32)) -> usize {
        if let Some((idx, _)) = self
            .cells
            .iter()
            .enumerate()
            .find(|(_, (cx, cy, _))| (*cx, *cy) == key)
        {
            idx
        } else {
            self.cells.push((key.0, key.1, Vec::new()));
            self.cells.len() - 1
        }
    }

    fn remove_from_cells(&mut self, slot: usize) {
        for (_, _, slots) in &mut self.cells {
            if let Some(pos) = slots.iter().position(|&s| s == slot) {
                slots.swap_remove(pos);
            }
        }
    }
}

impl<P: Copy + Debug> Backend<f32, P> for GridF32<P> {
    fn insert(&mut self, slot: usize, aabb: Aabb2D<f32>) {
        if self.entries.len() <= slot {
            self.entries.resize_with(slot + 1, || None);
        }
        self.entries[slot] = Some(aabb);
        for key in self.cells_for_aabb(&aabb) {
            let idx = self.find_cell_mut(key);
            self.cells[idx].2.push(slot);
        }
    }
    fn update(&mut self, slot: usize, aabb: Aabb2D<f32>) {
        self.remove_from_cells(slot);
        if let Some(e) = self.entries.get_mut(slot) {
            *e = Some(aabb);
            for key in self.cells_for_aabb(&aabb) {
                let idx = self.find_cell_mut(key);
                self.cells[idx].2.push(slot);
            }
        }
    }
    fn remove(&mut self, slot: usize) {
        self.remove_from_cells(slot);
        if let Some(e) = self.entries.get_mut(slot) {
            *e = None;
        }
    }
    fn clear(&mut self) {
        self.entries.clear();
        self.cells.clear();
    }
    fn query_point<'a>(&'a self, x: f32, y: f32) -> Box<dyn Iterator<Item = usize> + 'a> {
        let key = self.key_for(x, y);
        let mut set = BTreeSet::new();
        if let Some((_, _, slots)) = self.cells.iter().find(|(cx, cy, _)| (*cx, *cy) == key) {
            for &s in slots {
                set.insert(s);
            }
        }
        Box::new(set.into_iter())
    }
    fn query_rect<'a>(&'a self, rect: Aabb2D<f32>) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut set = BTreeSet::new();
        for key in self.cells_for_aabb(&rect) {
            if let Some((_, _, slots)) = self.cells.iter().find(|(cx, cy, _)| (*cx, *cy) == key) {
                for &s in slots {
                    set.insert(s);
                }
            }
        }
        Box::new(set.into_iter())
    }
}

impl<P: Copy + Debug> Debug for GridF32<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let total = self.entries.len();
        let alive = self.entries.iter().filter(|e| e.is_some()).count();
        let cells = self.cells.len();
        f.debug_struct("GridF32")
            .field("cell_w", &self.cell_w)
            .field("cell_h", &self.cell_h)
            .field("origin_x", &self.origin_x)
            .field("origin_y", &self.origin_y)
            .field("total_slots", &total)
            .field("alive", &alive)
            .field("cells", &cells)
            .finish_non_exhaustive()
    }
}

/// Uniform grid backend for i64 coordinates.
pub struct GridI64<P: Copy + Debug> {
    cell_w: i64,
    cell_h: i64,
    origin_x: i64,
    origin_y: i64,
    entries: Vec<Option<Aabb2D<i64>>>,
    cells: Vec<(i64, i64, Vec<usize>)>,
    _p: core::marker::PhantomData<P>,
}

impl<P: Copy + Debug> GridI64<P> {
    /// Create an integer grid with the given cell size and origin offset.
    ///
    /// Mapping uses Euclidean division (`div_euclid`) so negative coordinates
    /// snap consistently toward negative infinity.
    pub fn new(cell_w: i64, cell_h: i64, origin_x: i64, origin_y: i64) -> Self {
        assert!(cell_w > 0 && cell_h > 0, "cell sizes must be positive");
        Self {
            cell_w,
            cell_h,
            origin_x,
            origin_y,
            entries: Vec::new(),
            cells: Vec::new(),
            _p: core::marker::PhantomData,
        }
    }

    #[inline]
    fn key_for(&self, x: i64, y: i64) -> (i64, i64) {
        let cx = (x - self.origin_x).div_euclid(self.cell_w);
        let cy = (y - self.origin_y).div_euclid(self.cell_h);
        (cx, cy)
    }

    fn cells_for_aabb(&self, a: &Aabb2D<i64>) -> Vec<(i64, i64)> {
        let (minx, miny) = self.key_for(a.min_x, a.min_y);
        let (maxx, maxy) = self.key_for(a.max_x, a.max_y);
        let mut out = Vec::new();
        for y in miny..=maxy {
            for x in minx..=maxx {
                out.push((x, y));
            }
        }
        out
    }

    fn find_cell_mut(&mut self, key: (i64, i64)) -> usize {
        if let Some((idx, _)) = self
            .cells
            .iter()
            .enumerate()
            .find(|(_, (cx, cy, _))| (*cx, *cy) == key)
        {
            idx
        } else {
            self.cells.push((key.0, key.1, Vec::new()));
            self.cells.len() - 1
        }
    }

    fn remove_from_cells(&mut self, slot: usize) {
        for (_, _, slots) in &mut self.cells {
            if let Some(pos) = slots.iter().position(|&s| s == slot) {
                slots.swap_remove(pos);
            }
        }
    }
}

impl<P: Copy + Debug> Backend<i64, P> for GridI64<P> {
    fn insert(&mut self, slot: usize, aabb: Aabb2D<i64>) {
        if self.entries.len() <= slot {
            self.entries.resize_with(slot + 1, || None);
        }
        self.entries[slot] = Some(aabb);
        for key in self.cells_for_aabb(&aabb) {
            let idx = self.find_cell_mut(key);
            self.cells[idx].2.push(slot);
        }
    }
    fn update(&mut self, slot: usize, aabb: Aabb2D<i64>) {
        self.remove_from_cells(slot);
        if let Some(e) = self.entries.get_mut(slot) {
            *e = Some(aabb);
            for key in self.cells_for_aabb(&aabb) {
                let idx = self.find_cell_mut(key);
                self.cells[idx].2.push(slot);
            }
        }
    }
    fn remove(&mut self, slot: usize) {
        self.remove_from_cells(slot);
        if let Some(e) = self.entries.get_mut(slot) {
            *e = None;
        }
    }
    fn clear(&mut self) {
        self.entries.clear();
        self.cells.clear();
    }
    fn query_point<'a>(&'a self, x: i64, y: i64) -> Box<dyn Iterator<Item = usize> + 'a> {
        let key = self.key_for(x, y);
        let mut set = BTreeSet::new();
        if let Some((_, _, slots)) = self.cells.iter().find(|(cx, cy, _)| (*cx, *cy) == key) {
            for &s in slots {
                set.insert(s);
            }
        }
        Box::new(set.into_iter())
    }
    fn query_rect<'a>(&'a self, rect: Aabb2D<i64>) -> Box<dyn Iterator<Item = usize> + 'a> {
        let mut set = BTreeSet::new();
        for key in self.cells_for_aabb(&rect) {
            if let Some((_, _, slots)) = self.cells.iter().find(|(cx, cy, _)| (*cx, *cy) == key) {
                for &s in slots {
                    set.insert(s);
                }
            }
        }
        Box::new(set.into_iter())
    }
}

impl<P: Copy + Debug> Debug for GridI64<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let total = self.entries.len();
        let alive = self.entries.iter().filter(|e| e.is_some()).count();
        let cells = self.cells.len();
        f.debug_struct("GridI64")
            .field("cell_w", &self.cell_w)
            .field("cell_h", &self.cell_h)
            .field("origin_x", &self.origin_x)
            .field("origin_y", &self.origin_y)
            .field("total_slots", &total)
            .field("alive", &alive)
            .field("cells", &cells)
            .finish_non_exhaustive()
    }
}

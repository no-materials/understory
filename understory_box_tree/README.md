<div align="center">

# Understory Box Tree

**Kurbo-native spatially indexed box tree**

[![Latest published version.](https://img.shields.io/crates/v/understory_box_tree.svg)](https://crates.io/crates/understory_box_tree)
[![Documentation build status.](https://img.shields.io/docsrs/understory_box_tree.svg)](https://docs.rs/understory_box_tree)
[![Apache 2.0 license.](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](#license)
\
[![GitHub Actions CI status.](https://img.shields.io/github/actions/workflow/status/endoli/understory/ci.yml?logo=github&label=CI)](https://github.com/endoli/understory/actions)

</div>

<!-- We use cargo-rdme to update the README with the contents of lib.rs.
To edit the following section, update it in lib.rs, then run:
cargo rdme --workspace-project=understory_box_tree
Full documentation at https://github.com/orium/cargo-rdme -->

<!-- Intra-doc links used in lib.rs may be evaluated here. -->

<!-- cargo-rdme start -->

Understory Box Tree: a Kurbo-native, spatially indexed box tree.

Understory Box Tree is a reusable building block for UIs, canvas and vector editors, and CAD viewers.

- Represents a hierarchy of regions with local transforms, clips, z-order, and flags.
- Provides hit testing and rectangle intersection queries over world-space AABBs.
- Supports batched updates with a [`Tree::commit`] step that yields coarse damage regions.

It aims for a stable, minimal API and leaves room to evolve internals (for example a pluggable spatial index) without churn at call sites.

## Where this fits: three-tree model

We’re standardizing on a simple separation of concerns for UI stacks.
- Widget tree: interaction/state.
- Box tree: geometry/spatial indexing (this crate).
- Render tree: display list (future crate).

The box tree computes world-space AABBs from local bounds, transforms, and clips, and synchronizes them into a spatial index for fast hit testing and visibility queries.
This decouples scene structure from the spatial acceleration and makes debugging and incremental updates tractable.

## Not a layout engine

This crate does not perform layout (measurement or arrangement) or apply layout policies such as flex, grid, or stack.
Upstream code is expected to compute positions and sizes using whatever layout system you choose and then update this tree with the resulting world-space boxes, transforms, optional clips, and z-order.
Think of this as a scene and spatial index, not a layout system.

## Integration with Understory Index

This crate uses [`understory_index`] for spatial queries. You can choose the backend and scalar to
fit your workload (flat vector, uniform grid for `f32`/`f64`/`i64`, R-tree or BVH). Float inputs are
assumed to be finite (no NaNs). AABBs are conservative for non-axis transforms and rounded clips.

See [`understory_index::Index`], [`understory_index::GridF32`]/[`understory_index::GridF64`]/[`understory_index::GridI64`],
[`understory_index::RTreeF32`]/[`understory_index::RTreeF64`]/[`understory_index::RTreeI64`], and
[`understory_index::BVHF32`]/[`understory_index::BVHF64`]/[`understory_index::BVHI64`] for details.

## API overview

- [`Tree`]: container managing nodes and the spatial index synchronization.
- [`LocalNode`]: per-node local data (bounds, transform, optional clip, z, flags).
  See [`LocalNode::flags`] for visibility/picking controls.
- [`NodeFlags`]: visibility and picking controls.
- [`NodeId`]: generational handle of a node.
- [`QueryFilter`]: restricts hit/intersect results (visible/pickable).
  See [`NodeFlags::VISIBLE`] and [`NodeFlags::PICKABLE`].

Key operations:
- [`Tree::insert`](Tree::insert) → [`NodeId`]
- [`Tree::set_local_transform`](Tree::set_local_transform) / [`Tree::set_local_clip`](Tree::set_local_clip)
- [`Tree::commit`](Tree::commit) → damage summary; updates world data and the spatial index.
- [`Tree::hit_test_point`](Tree::hit_test_point) and [`Tree::intersect_rect`](Tree::intersect_rect).

## Damage and debugging notes

- [`Tree::commit`] batches adds/updates/removals and produces coarse damage (added/removed AABBs and
  old/new pairs for moved nodes). This is enough to bound a paint traversal in most UIs.
- World AABBs are conservative under rotation/shear and rounded-rect clips are approximated by
  their axis-aligned bounds for acceleration; precise hit-filtering is applied where cheap.

## Examples

- `examples/basic_box_tree.rs`: builds a trivial tree, commits, and runs a couple of queries.
- `examples/visible_list.rs`: demonstrates using `intersect_rect` to compute a visible set,
  a building block for virtualization.

### Minimal usage

```rust
use understory_box_tree::{Tree, LocalNode, QueryFilter};
use kurbo::{Rect, Affine, Vec2, Point};

// Build a tiny tree.
let mut tree = Tree::new();

let root = tree.insert(
    None,
    LocalNode { local_bounds: Rect::new(0.0, 0.0, 200.0, 200.0), ..Default::default() },
);

let child = tree.insert(
    Some(root),
    LocalNode { local_bounds: Rect::new(10.0, 10.0, 60.0, 60.0), ..Default::default() },
);

// Synchronize and compute damage.
let _ = tree.commit();

// Move and hit-test.
tree.set_local_transform(child, Affine::translate(Vec2::new(10.0, 0.0)));
let _ = tree.commit();

let filter = QueryFilter { visible_only: true, pickable_only: true };
let hit = tree.hit_test_point(Point::new(25.0, 25.0), filter).unwrap();
assert_eq!(hit.node, child);
```

### Visible set using a viewport rectangle

```rust
use understory_box_tree::{Tree, LocalNode, QueryFilter};
use kurbo::Rect;

let mut tree = Tree::new();

let root = tree.insert(
    None,
    LocalNode { local_bounds: Rect::new(0.0, 0.0, 1000.0, 1000.0), ..Default::default() },
);

// Insert rows.
for i in 0..10u32 {
    let y = i as f64 * 50.0;
    let _ = tree.insert(
        Some(root),
        LocalNode {
            local_bounds: Rect::new(0.0, y, 200.0, y + 40.0),
            z_index: i as i32,
            ..Default::default()
        },
    );
}

let _ = tree.commit();

// Compute visible set.
let filter = QueryFilter { visible_only: true, pickable_only: true };
let viewport = Rect::new(0.0, 120.0, 200.0, 220.0);
let visible: Vec<_> = tree.intersect_rect(viewport, filter).collect();
assert!(visible.len() >= 2);
```

This crate is `no_std` and uses `alloc`.

# Example

```rust
use understory_box_tree::{Tree, LocalNode, QueryFilter};
use kurbo::{Rect, Affine, Vec2, Point};

// Build a small tree.
let mut tree = Tree::new();

let root = tree.insert(
    None,
    LocalNode { local_bounds: Rect::new(0.0, 0.0, 200.0, 200.0), ..Default::default() },
);

let a = tree.insert(
    Some(root),
    LocalNode { local_bounds: Rect::new(10.0, 10.0, 60.0, 60.0), z_index: 0, ..Default::default() },
);

let b = tree.insert(
    Some(root),
    LocalNode { local_bounds: Rect::new(40.0, 40.0, 120.0, 120.0), z_index: 10, ..Default::default() },
);

let _damage0 = tree.commit();

// Move node A to the right and compute damage.
tree.set_local_transform(a, Affine::translate(Vec2::new(20.0, 0.0)));
let damage = tree.commit();
assert!(damage.union_rect().is_some());

// Hit-test prefers the higher z-index (node B).
let filter = QueryFilter { visible_only: true, pickable_only: true };
let hit = tree.hit_test_point(Point::new(50.0, 50.0), filter).unwrap();
assert_eq!(hit.node, b);
```

See the `basic` example in this crate for a runnable version with printed output.

<!-- cargo-rdme end -->

## Minimum supported Rust Version (MSRV)

This crate has been verified to compile with **Rust 1.88** and later.

## License

Licensed under the Apache License, Version 2.0 ([LICENSE] or <http://www.apache.org/licenses/LICENSE-2.0>)

<!-- Needs to be defined here for rustdoc's benefit -->
[LICENSE]: LICENSE

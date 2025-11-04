// Copyright 2025 the Understory Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![cfg(feature = "compare_rstar")]

use criterion::{BatchSize, Criterion, Throughput, black_box, criterion_group, criterion_main};
use understory_index::{Aabb2D, Index};

use rstar::primitives::Rectangle;
use rstar::{AABB, RTree};

fn gen_grid_rects(n: usize, cell: f64) -> Vec<Aabb2D<f64>> {
    let mut out = Vec::with_capacity(n * n);
    for y in 0..n {
        for x in 0..n {
            let x0 = x as f64 * cell;
            let y0 = y as f64 * cell;
            out.push(Aabb2D::<f64>::from_xywh(x0, y0, cell, cell));
        }
    }
    out
}

fn to_rstar_rects(v: &[Aabb2D<f64>]) -> Vec<Rectangle<[f64; 2]>> {
    v.iter()
        .map(|r| Rectangle::from_corners([r.min_x, r.min_y], [r.max_x, r.max_y]))
        .collect()
}

fn bench_rtree_external_compare_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("rtree_external_compare_f64");
    for &n in &[64usize, 128] {
        let rects = gen_grid_rects(n, 10.0);
        let aabb_query = Aabb2D::<f64>::from_xywh(100.0, 100.0, 400.0, 400.0);
        group.throughput(Throughput::Elements((n * n) as u64));

        group.bench_function(format!("understory_build_query_n{}", n), |b| {
            b.iter_batched(
                Index::<f64, u32>::with_rtree,
                |mut idx| {
                    for (i, r) in rects.iter().copied().enumerate() {
                        let _ = idx.insert(r, i as u32);
                    }
                    let _ = idx.commit();
                    let hits: usize = idx.query_rect(aabb_query).count();
                    black_box(hits);
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("understory_build_query_bulk_n{}", n), |b| {
            b.iter_batched(
                || {
                    let entries: Vec<_> = rects
                        .iter()
                        .copied()
                        .enumerate()
                        .map(|(i, r)| (r, i as u32))
                        .collect();
                    entries
                },
                |entries| {
                    let idx = Index::<f64, u32>::with_rtree_bulk(&entries);
                    let hits: usize = idx.query_rect(aabb_query).count();
                    black_box(hits);
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(format!("rstar_build_query_bulk_n{}", n), |b| {
            b.iter_batched(
                || to_rstar_rects(&rects),
                |rectangles| {
                    let tree = RTree::bulk_load(rectangles);
                    let aabb = AABB::from_corners(
                        [aabb_query.min_x, aabb_query.min_y],
                        [aabb_query.max_x, aabb_query.max_y],
                    );
                    let hits: usize = tree.locate_in_envelope_intersecting(&aabb).count();
                    black_box(hits);
                },
                BatchSize::SmallInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, bench_rtree_external_compare_f64);
criterion_main!(benches);

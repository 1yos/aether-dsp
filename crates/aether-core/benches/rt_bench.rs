//! Real-time performance benchmarks.
//!
//! Validates that the scheduler meets the 1.33ms deadline at 64-sample / 48kHz.
//! Run with: cargo bench -p aether-core

use aether_core::{
    arena::NodeId,
    buffer_pool::BufferPool,
    command::Command,
    graph::DspGraph,
    node::{DspNode, NodeRecord},
    param::ParamBlock,
    scheduler::Scheduler,
    state::StateBlob,
    BUFFER_SIZE, MAX_INPUTS,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ringbuf::{
    traits::{Consumer, Producer, Split},
    HeapRb,
};

/// Minimal no-op node for pure scheduler overhead measurement.
struct NoopNode;
impl DspNode for NoopNode {
    fn process(
        &mut self,
        _inputs: &[Option<&[f32; BUFFER_SIZE]>; MAX_INPUTS],
        output: &mut [f32; BUFFER_SIZE],
        _params: &mut ParamBlock,
        _sr: f32,
    ) {
        output.fill(0.0);
    }
    fn type_name(&self) -> &'static str {
        "Noop"
    }
}

fn bench_scheduler_noop(c: &mut Criterion) {
    let mut group = c.benchmark_group("scheduler");
    group.measurement_time(std::time::Duration::from_secs(10));

    for node_count in [1, 10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::new("noop_nodes", node_count),
            &node_count,
            |b, &n| {
                let mut sched = Scheduler::new(48_000.0);
                let mut prev_id: Option<NodeId> = None;
                for _ in 0..n {
                    let id = sched.graph.add_node(Box::new(NoopNode)).unwrap();
                    if let Some(p) = prev_id {
                        sched.graph.connect(p, id, 0);
                    }
                    prev_id = Some(id);
                }
                if let Some(out) = prev_id {
                    sched.graph.set_output_node(out);
                }

                let ring = HeapRb::<Command>::new(64);
                let (_prod, mut cons) = ring.split();
                let mut output = [0.0f32; BUFFER_SIZE * 2];

                b.iter(|| {
                    sched.process_block(&mut cons, black_box(&mut output));
                });
            },
        );
    }
    group.finish();
}

fn bench_arena(c: &mut Criterion) {
    use aether_core::arena::Arena;
    c.bench_function("arena_insert_remove_1000", |b| {
        b.iter(|| {
            let mut arena: Arena<u64> = Arena::with_capacity(2048);
            let ids: Vec<_> = (0..1000).map(|i| arena.insert(i).unwrap()).collect();
            for id in ids {
                arena.remove(id);
            }
        });
    });
}

fn bench_param_fill(c: &mut Criterion) {
    use aether_core::param::Param;
    c.bench_function("param_fill_buffer_64", |b| {
        let mut param = Param::new(440.0);
        param.set_target(880.0, 4410);
        let mut buf = [0.0f32; BUFFER_SIZE];
        b.iter(|| {
            param.fill_buffer(black_box(&mut buf));
        });
    });
}

criterion_group!(benches, bench_scheduler_noop, bench_arena, bench_param_fill);
criterion_main!(benches);

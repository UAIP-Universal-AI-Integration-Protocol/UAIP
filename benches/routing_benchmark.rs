//! Performance benchmarks for UAIP Router
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use uaip_core::message::{EntityType, Priority, QosLevel, UaipMessage};
use uaip_router::priority_queue::PriorityQueue;

/// Benchmark message creation
fn bench_message_creation(c: &mut Criterion) {
    c.bench_function("message_creation", |b| {
        b.iter(|| {
            UaipMessage::new(
                black_box("device_001".to_string()),
                black_box(EntityType::Device),
                black_box("ai_agent_001".to_string()),
                black_box(EntityType::AiAgent),
            )
        })
    });
}

/// Benchmark message serialization
fn bench_message_serialization(c: &mut Criterion) {
    let msg = UaipMessage::new(
        "device_001".to_string(),
        EntityType::Device,
        "ai_agent_001".to_string(),
        EntityType::AiAgent,
    )
    .with_priority(Priority::High)
    .with_qos(QosLevel::ExactlyOnce);

    c.bench_function("message_serialization", |b| {
        b.iter(|| msg.to_json().expect("Serialization failed"))
    });
}

/// Benchmark message deserialization
fn bench_message_deserialization(c: &mut Criterion) {
    let msg = UaipMessage::new(
        "device_001".to_string(),
        EntityType::Device,
        "ai_agent_001".to_string(),
        EntityType::AiAgent,
    );

    let json = msg.to_json().expect("Serialization failed");

    c.bench_function("message_deserialization", |b| {
        b.iter(|| UaipMessage::from_json(black_box(&json)).expect("Deserialization failed"))
    });
}

/// Benchmark priority queue operations
fn bench_priority_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_queue");

    // Benchmark enqueue
    group.bench_function("enqueue", |b| {
        let mut queue = PriorityQueue::new();
        let msg = UaipMessage::new(
            "device_001".to_string(),
            EntityType::Device,
            "ai_agent_001".to_string(),
            EntityType::AiAgent,
        )
        .with_priority(Priority::Normal);

        b.iter(|| {
            queue.enqueue(black_box(msg.clone()));
        })
    });

    // Benchmark dequeue with different queue sizes
    for size in [10, 100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::new("dequeue", size), size, |b, &size| {
            b.iter_batched(
                || {
                    let mut queue = PriorityQueue::new();
                    for i in 0..size {
                        let priority = match i % 4 {
                            0 => Priority::Critical,
                            1 => Priority::High,
                            2 => Priority::Normal,
                            _ => Priority::Low,
                        };
                        let msg = UaipMessage::new(
                            format!("device_{:03}", i),
                            EntityType::Device,
                            "ai_agent_001".to_string(),
                            EntityType::AiAgent,
                        )
                        .with_priority(priority);
                        queue.enqueue(msg);
                    }
                    queue
                },
                |mut queue| {
                    for _ in 0..size {
                        black_box(queue.dequeue());
                    }
                },
                criterion::BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// Benchmark throughput: messages per second
fn bench_message_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_throughput");

    for count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::new("create_and_serialize", count),
            count,
            |b, &count| {
                b.iter(|| {
                    for i in 0..count {
                        let msg = UaipMessage::new(
                            format!("device_{:03}", i),
                            EntityType::Device,
                            "ai_agent_001".to_string(),
                            EntityType::AiAgent,
                        )
                        .with_priority(Priority::Normal);

                        black_box(msg.to_json().expect("Serialization failed"));
                    }
                })
            },
        );
    }

    group.finish();
}

/// Benchmark priority ordering
fn bench_priority_ordering(c: &mut Criterion) {
    c.bench_function("priority_ordering_1000_messages", |b| {
        b.iter(|| {
            let mut queue = PriorityQueue::new();

            // Insert 1000 messages with mixed priorities
            for i in 0..1000 {
                let priority = match i % 4 {
                    0 => Priority::Critical,
                    1 => Priority::High,
                    2 => Priority::Normal,
                    _ => Priority::Low,
                };
                let msg = UaipMessage::new(
                    format!("device_{:03}", i),
                    EntityType::Device,
                    "ai_agent_001".to_string(),
                    EntityType::AiAgent,
                )
                .with_priority(priority);
                queue.enqueue(msg);
            }

            // Dequeue all messages (should be in priority order)
            let mut prev_priority = Priority::Critical;
            while let Some(msg) = queue.dequeue() {
                assert!(msg.header.priority <= prev_priority);
                prev_priority = msg.header.priority;
                black_box(msg);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_message_creation,
    bench_message_serialization,
    bench_message_deserialization,
    bench_priority_queue,
    bench_message_throughput,
    bench_priority_ordering
);

criterion_main!(benches);

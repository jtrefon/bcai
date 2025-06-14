//! Tensor Operations Benchmarks
//!
//! Comprehensive performance benchmarks for tensor operations in the enhanced VM.

use candle_core::Device;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use runtime::{
    tensor_ops::{Tensor, TensorManager},
    DataType, TensorId, VmError,
};
use std::time::Duration;

/// Benchmark tensor creation with various sizes
fn bench_tensor_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_creation");

    // Test different tensor sizes
    let sizes = vec![
        (10, 10),
        (100, 100),
        (1000, 1000),
        (32, 224, 224), // Typical image size
        (1, 512, 768),  // Typical NLP embedding
    ];

    for (i, size) in sizes.iter().enumerate() {
        let shape = if size.2.is_some() {
            vec![size.0, size.1, size.2.unwrap()]
        } else {
            vec![size.0, size.1]
        };

        let total_elements = shape.iter().product::<usize>();
        group.throughput(Throughput::Elements(total_elements as u64));

        group.bench_with_input(
            BenchmarkId::new("create", format!("{:?}", shape)),
            &shape,
            |b, shape| {
                let manager = TensorManager::new(1000, 1024);
                let device = Device::Cpu;

                b.iter(|| {
                    let tensor =
                        Tensor::new(TensorId(0), shape.clone(), DataType::Float32, &device);
                    black_box(tensor)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark tensor arithmetic operations
fn bench_tensor_arithmetic(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor_arithmetic");
    group.measurement_time(Duration::from_secs(10));

    let manager = TensorManager::new(1000, 1024);

    // Setup test tensors
    let shapes = vec![vec![100, 100], vec![1000, 1000], vec![32, 224, 224]];

    for (i, shape) in shapes.iter().enumerate() {
        let total_elements = shape.iter().product::<usize>();
        group.throughput(Throughput::Elements(total_elements as u64));

        // Create test tensors
        manager.create_tensor(TensorId(i as u64 * 2), shape.clone(), DataType::Float32).unwrap();
        manager
            .create_tensor(TensorId(i as u64 * 2 + 1), shape.clone(), DataType::Float32)
            .unwrap();

        group.bench_with_input(BenchmarkId::new("add", format!("{:?}", shape)), &i, |b, &i| {
            b.iter(|| {
                let result = manager.tensor_add(
                    TensorId(i as u64 * 2),
                    TensorId(i as u64 * 2 + 1),
                    TensorId(100 + i as u64),
                );
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark matrix multiplication
fn bench_matrix_multiplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_multiplication");
    group.measurement_time(Duration::from_secs(15));

    let manager = TensorManager::new(1000, 2048);

    // Test different matrix sizes
    let matrix_sizes =
        vec![(64, 64, 64), (128, 128, 128), (256, 256, 256), (512, 512, 512), (1024, 1024, 1024)];

    for (i, (m, n, k)) in matrix_sizes.iter().enumerate() {
        let ops = 2 * m * n * k; // Approximate FLOPs for matrix multiplication
        group.throughput(Throughput::Elements(ops as u64));

        // Create matrices A (m x k) and B (k x n)
        manager.create_tensor(TensorId(i as u64 * 2), vec![*m, *k], DataType::Float32).unwrap();
        manager.create_tensor(TensorId(i as u64 * 2 + 1), vec![*k, *n], DataType::Float32).unwrap();

        group.bench_with_input(
            BenchmarkId::new("matmul", format!("{}x{}x{}", m, n, k)),
            &i,
            |b, &i| {
                b.iter(|| {
                    let result = manager.tensor_matmul(
                        TensorId(i as u64 * 2),
                        TensorId(i as u64 * 2 + 1),
                        TensorId(200 + i as u64),
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark activation functions
fn bench_activations(c: &mut Criterion) {
    let mut group = c.benchmark_group("activations");

    let manager = TensorManager::new(1000, 1024);

    let shapes = vec![vec![1000, 1000], vec![32, 224, 224], vec![64, 512, 768]];

    for (i, shape) in shapes.iter().enumerate() {
        let total_elements = shape.iter().product::<usize>();
        group.throughput(Throughput::Elements(total_elements as u64));

        manager.create_tensor(TensorId(i as u64), shape.clone(), DataType::Float32).unwrap();

        group.bench_with_input(BenchmarkId::new("relu", format!("{:?}", shape)), &i, |b, &i| {
            b.iter(|| {
                let result = manager.tensor_relu(TensorId(i as u64), TensorId(300 + i as u64));
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark memory operations
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");

    let manager = TensorManager::new(1000, 2048);

    // Benchmark tensor creation and destruction
    group.bench_function("create_destroy_cycle", |b| {
        b.iter(|| {
            // Create tensor
            manager.create_tensor(TensorId(1000), vec![100, 100], DataType::Float32).unwrap();

            // Destroy tensor
            manager.destroy_tensor(TensorId(1000)).unwrap();
        });
    });

    // Benchmark memory usage tracking
    group.bench_function("memory_stats", |b| {
        // Create some tensors first
        for i in 0..10 {
            manager.create_tensor(TensorId(i), vec![100, 100], DataType::Float32).unwrap();
        }

        b.iter(|| {
            let stats = manager.memory_stats();
            black_box(stats)
        });
    });

    group.finish();
}

/// Benchmark concurrent tensor operations
fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(20));

    let manager = std::sync::Arc::new(TensorManager::new(10000, 4096));

    // Setup test tensors
    for i in 0..100 {
        manager.create_tensor(TensorId(i), vec![100, 100], DataType::Float32).unwrap();
    }

    group.bench_function("concurrent_adds", |b| {
        use std::sync::Arc;
        use std::thread;

        b.iter(|| {
            let handles: Vec<_> = (0..8)
                .map(|thread_id| {
                    let manager = Arc::clone(&manager);
                    thread::spawn(move || {
                        for i in 0..10 {
                            let tensor_a = TensorId(thread_id * 10 + i);
                            let tensor_b = TensorId(thread_id * 10 + i + 1);
                            let output = TensorId(1000 + thread_id * 10 + i);

                            manager.tensor_add(tensor_a, tensor_b, output).ok();
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

/// Benchmark GPU vs CPU operations (if CUDA is available)
#[cfg(feature = "cuda")]
fn bench_gpu_vs_cpu(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_vs_cpu");
    group.measurement_time(Duration::from_secs(30));

    let cpu_device = Device::Cpu;
    let shapes = vec![vec![1000, 1000], vec![2000, 2000], vec![4000, 4000]];

    // Test if CUDA is available
    if let Ok(gpu_device) = Device::new_cuda(0) {
        for shape in shapes {
            let total_elements = shape.iter().product::<usize>();
            group.throughput(Throughput::Elements(total_elements as u64));

            // CPU benchmark
            group.bench_with_input(
                BenchmarkId::new("cpu_matmul", format!("{:?}", shape)),
                &shape,
                |b, shape| {
                    b.iter(|| {
                        let tensor_a =
                            Tensor::new(TensorId(0), shape.clone(), DataType::Float32, &cpu_device)
                                .unwrap();
                        let tensor_b =
                            Tensor::new(TensorId(1), shape.clone(), DataType::Float32, &cpu_device)
                                .unwrap();

                        // Simulate matrix multiplication (placeholder)
                        black_box((tensor_a, tensor_b))
                    });
                },
            );

            // GPU benchmark
            group.bench_with_input(
                BenchmarkId::new("gpu_matmul", format!("{:?}", shape)),
                &shape,
                |b, shape| {
                    b.iter(|| {
                        let tensor_a =
                            Tensor::new(TensorId(0), shape.clone(), DataType::Float32, &gpu_device)
                                .unwrap();
                        let tensor_b =
                            Tensor::new(TensorId(1), shape.clone(), DataType::Float32, &gpu_device)
                                .unwrap();

                        // Simulate matrix multiplication (placeholder)
                        black_box((tensor_a, tensor_b))
                    });
                },
            );
        }
    }

    group.finish();
}

/// Benchmark different data types
fn bench_data_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_types");

    let manager = TensorManager::new(1000, 1024);
    let shape = vec![1000, 1000];
    let data_types = vec![DataType::Float32, DataType::Float64, DataType::Int32, DataType::Int64];

    for (i, dtype) in data_types.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("create", format!("{:?}", dtype)),
            dtype,
            |b, dtype| {
                b.iter(|| {
                    let result =
                        manager.create_tensor(TensorId(i as u64), shape.clone(), dtype.clone());
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Comprehensive benchmark suite
criterion_group!(
    benches,
    bench_tensor_creation,
    bench_tensor_arithmetic,
    bench_matrix_multiplication,
    bench_activations,
    bench_memory_operations,
    bench_concurrent_operations,
    bench_data_types,
    #[cfg(feature = "cuda")]
    bench_gpu_vs_cpu,
);
criterion_main!(benches);

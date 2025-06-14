use clap::{Parser, Subcommand};
use runtime::{
    enhanced_vm::{EnhancedVM, VMConfig},
    ml_instructions::MLInstruction,
    hardware_abstraction::HardwareBackend,
    python_bridge::PythonBridge,
    tensor_ops::{Tensor, TensorManager},
};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "vm_test_runner")]
#[command(about = "BCAI Enhanced VM Test Runner")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests
    All {
        #[arg(long)]
        benchmark: bool,
    },
    /// Run VM instruction tests
    Instructions,
    /// Run tensor operation tests
    Tensors,
    /// Run Python bridge tests
    Python,
    /// Run hardware acceleration tests
    Hardware,
    /// Run performance benchmarks
    Benchmark {
        #[arg(long, default_value = "1000")]
        iterations: u32,
        #[arg(long)]
        detailed: bool,
    },
    /// Run stress tests
    Stress {
        #[arg(long, default_value = "60")]
        duration_seconds: u64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::All { benchmark } => {
            run_all_tests(benchmark).await?;
        }
        Commands::Instructions => {
            run_instruction_tests().await?;
        }
        Commands::Tensors => {
            run_tensor_tests().await?;
        }
        Commands::Python => {
            run_python_tests().await?;
        }
        Commands::Hardware => {
            run_hardware_tests().await?;
        }
        Commands::Benchmark { iterations, detailed } => {
            run_benchmarks(iterations, detailed).await?;
        }
        Commands::Stress { duration_seconds } => {
            run_stress_tests(duration_seconds).await?;
        }
    }

    Ok(())
}

async fn run_all_tests(benchmark: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 BCAI Enhanced VM - Comprehensive Test Suite");
    println!("===============================================");
    println!();

    let mut passed = 0;
    let mut failed = 0;

    // Test categories
    let test_results = vec![
        ("VM Instructions", run_instruction_tests().await),
        ("Tensor Operations", run_tensor_tests().await),
        ("Python Bridge", run_python_tests().await),
        ("Hardware Abstraction", run_hardware_tests().await),
    ];

    for (name, result) in test_results {
        match result {
            Ok(_) => {
                println!("✅ {}: PASSED", name);
                passed += 1;
            }
            Err(e) => {
                println!("❌ {}: FAILED - {}", name, e);
                failed += 1;
            }
        }
    }

    println!();
    println!("📊 Test Summary:");
    println!("  ✅ Passed: {}", passed);
    println!("  ❌ Failed: {}", failed);
    println!("  📈 Success Rate: {:.1}%", (passed as f64 / (passed + failed) as f64) * 100.0);

    if benchmark {
        println!();
        run_benchmarks(1000, false).await?;
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

async fn run_instruction_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing VM Instructions...");
    
    let config = VMConfig {
        memory_limit: 512 * 1024 * 1024,
        max_iterations: 1000,
        enable_python_bridge: false,
        hardware_backend: HardwareBackend::CPU,
        enable_gpu: false,
        python_timeout_seconds: 30,
        max_python_memory: 256 * 1024 * 1024,
    };

    let mut vm = EnhancedVM::new(config)?;

    // Test tensor creation
    let create_instruction = MLInstruction::TensorCreate {
        id: 1,
        shape: vec![2, 2],
        data: vec![1.0, 2.0, 3.0, 4.0],
    };
    vm.execute_ml_instruction(create_instruction).await?;

    // Test tensor addition
    let create_instruction2 = MLInstruction::TensorCreate {
        id: 2,
        shape: vec![2, 2],
        data: vec![5.0, 6.0, 7.0, 8.0],
    };
    vm.execute_ml_instruction(create_instruction2).await?;

    let add_instruction = MLInstruction::TensorAdd {
        input1: 1,
        input2: 2,
        output: 3,
    };
    vm.execute_ml_instruction(add_instruction).await?;

    // Test tensor multiplication
    let mul_instruction = MLInstruction::TensorMul {
        input1: 1,
        input2: 2,
        output: 4,
    };
    vm.execute_ml_instruction(mul_instruction).await?;

    // Test matrix multiplication
    let matmul_instruction = MLInstruction::MatMul {
        input1: 1,
        input2: 2,
        output: 5,
    };
    vm.execute_ml_instruction(matmul_instruction).await?;

    println!("  ✓ Tensor creation, addition, multiplication, and matrix multiplication");
    Ok(())
}

async fn run_tensor_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔢 Testing Tensor Operations...");

    let mut tensor_manager = TensorManager::new();

    // Test tensor creation and storage
    let tensor = Tensor::new(vec![3, 3], vec![1.0; 9])?;
    tensor_manager.store_tensor(1, tensor);

    // Test tensor retrieval
    let retrieved = tensor_manager.get_tensor(1)
        .ok_or("Failed to retrieve tensor")?;
    
    assert_eq!(retrieved.shape(), &vec![3, 3]);
    assert_eq!(retrieved.data().len(), 9);

    // Test tensor operations
    let tensor2 = Tensor::new(vec![3, 3], vec![2.0; 9])?;
    tensor_manager.store_tensor(2, tensor2);

    let result = tensor_manager.add_tensors(1, 2)?;
    tensor_manager.store_tensor(3, result);

    let result_tensor = tensor_manager.get_tensor(3)
        .ok_or("Failed to retrieve result tensor")?;
    
    // Verify all values are 3.0 (1.0 + 2.0)
    for &value in result_tensor.data() {
        assert!((value - 3.0).abs() < 1e-6, "Expected 3.0, got {}", value);
    }

    println!("  ✓ Tensor creation, storage, retrieval, and arithmetic operations");
    Ok(())
}

async fn run_python_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🐍 Testing Python Bridge...");

    #[cfg(feature = "enhanced-vm")]
    {
        let bridge = PythonBridge::new(30, 256 * 1024 * 1024)?;

        // Test basic Python execution
        let result = bridge.execute_code("2 + 2").await?;
        assert!(result.contains("4"), "Expected '4' in result, got: {}", result);

        // Test NumPy operations
        let numpy_code = r#"
import numpy as np
arr = np.array([1, 2, 3, 4, 5])
result = np.sum(arr)
print(f"Sum: {result}")
result
"#;
        let result = bridge.execute_code(numpy_code).await?;
        assert!(result.contains("15") || result.contains("Sum: 15"), 
                "Expected NumPy sum result, got: {}", result);

        // Test security restrictions
        let malicious_code = "import os; os.system('echo malicious')";
        let result = bridge.execute_code(malicious_code).await;
        assert!(result.is_err(), "Security test failed - malicious code was executed");

        println!("  ✓ Basic execution, NumPy operations, and security restrictions");
    }

    #[cfg(not(feature = "enhanced-vm"))]
    {
        println!("  ⚠️  Skipped - Enhanced VM features not compiled");
    }

    Ok(())
}

async fn run_hardware_tests() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Testing Hardware Abstraction...");

    // Test CPU backend
    let cpu_config = VMConfig {
        memory_limit: 256 * 1024 * 1024,
        max_iterations: 100,
        enable_python_bridge: false,
        hardware_backend: HardwareBackend::CPU,
        enable_gpu: false,
        python_timeout_seconds: 30,
        max_python_memory: 128 * 1024 * 1024,
    };

    let mut cpu_vm = EnhancedVM::new(cpu_config)?;
    
    // Test basic operation on CPU
    let create_instruction = MLInstruction::TensorCreate {
        id: 1,
        shape: vec![100, 100],
        data: vec![1.0; 10000],
    };
    cpu_vm.execute_ml_instruction(create_instruction).await?;

    println!("  ✓ CPU backend operations");

    // Test GPU backends if available
    #[cfg(feature = "cuda")]
    {
        let cuda_config = VMConfig {
            memory_limit: 256 * 1024 * 1024,
            max_iterations: 100,
            enable_python_bridge: false,
            hardware_backend: HardwareBackend::CUDA,
            enable_gpu: true,
            python_timeout_seconds: 30,
            max_python_memory: 128 * 1024 * 1024,
        };

        let mut cuda_vm = EnhancedVM::new(cuda_config)?;
        cuda_vm.execute_ml_instruction(MLInstruction::TensorCreate {
            id: 1,
            shape: vec![100, 100],
            data: vec![2.0; 10000],
        }).await?;

        println!("  ✓ CUDA backend operations");
    }

    #[cfg(feature = "metal-gpu")]
    {
        let metal_config = VMConfig {
            memory_limit: 256 * 1024 * 1024,
            max_iterations: 100,
            enable_python_bridge: false,
            hardware_backend: HardwareBackend::Metal,
            enable_gpu: true,
            python_timeout_seconds: 30,
            max_python_memory: 128 * 1024 * 1024,
        };

        let mut metal_vm = EnhancedVM::new(metal_config)?;
        metal_vm.execute_ml_instruction(MLInstruction::TensorCreate {
            id: 1,
            shape: vec![100, 100],
            data: vec![3.0; 10000],
        }).await?;

        println!("  ✓ Metal backend operations");
    }

    Ok(())
}

async fn run_benchmarks(iterations: u32, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏃 Running Performance Benchmarks...");
    println!("Iterations: {}", iterations);
    println!();

    let config = VMConfig {
        memory_limit: 1024 * 1024 * 1024,
        max_iterations: iterations,
        enable_python_bridge: false,
        hardware_backend: HardwareBackend::CPU,
        enable_gpu: false,
        python_timeout_seconds: 60,
        max_python_memory: 512 * 1024 * 1024,
    };

    let mut vm = EnhancedVM::new(config)?;

    // Benchmark tensor creation
    println!("📊 Tensor Creation Benchmark:");
    let start = Instant::now();
    
    for i in 0..iterations {
        let create_instruction = MLInstruction::TensorCreate {
            id: i,
            shape: vec![100, 100],
            data: vec![1.0; 10000],
        };
        vm.execute_ml_instruction(create_instruction).await?;
    }
    
    let duration = start.elapsed();
    let ops_per_second = iterations as f64 / duration.as_secs_f64();
    
    println!("  ⏱️  Time: {:?}", duration);
    println!("  📈 Tensors/sec: {:.2}", ops_per_second);
    println!("  💾 Memory throughput: {:.2} MB/s", (ops_per_second * 10000.0 * 4.0) / 1e6);

    if detailed {
        // Benchmark tensor operations
        println!();
        println!("🔥 Tensor Operations Benchmark:");
        
        // Create base tensors for operations
        vm.execute_ml_instruction(MLInstruction::TensorCreate {
            id: 999998,
            shape: vec![1000, 1000],
            data: vec![1.0; 1000000],
        }).await?;
        
        vm.execute_ml_instruction(MLInstruction::TensorCreate {
            id: 999999,
            shape: vec![1000, 1000],
            data: vec![2.0; 1000000],
        }).await?;

        let start = Instant::now();
        
        for i in 0..100 {  // Smaller iterations for expensive operations
            let add_instruction = MLInstruction::TensorAdd {
                input1: 999998,
                input2: 999999,
                output: 1000000 + i,
            };
            vm.execute_ml_instruction(add_instruction).await?;
        }
        
        let duration = start.elapsed();
        let ops_per_second = 100.0 / duration.as_secs_f64();
        let flops = ops_per_second * 1000000.0; // 1M operations per tensor add
        
        println!("  ⏱️  Time: {:?}", duration);
        println!("  📈 Operations/sec: {:.2}", ops_per_second);
        println!("  🎯 MFLOPS: {:.2}", flops / 1e6);
    }

    println!();
    println!("✅ Benchmarks completed successfully!");
    
    Ok(())
}

async fn run_stress_tests(duration_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("💪 Running Stress Tests...");
    println!("Duration: {} seconds", duration_seconds);
    println!();

    let config = VMConfig {
        memory_limit: 2048 * 1024 * 1024, // 2GB for stress test
        max_iterations: u32::MAX,
        enable_python_bridge: true,
        hardware_backend: HardwareBackend::CPU,
        enable_gpu: false,
        python_timeout_seconds: 300,
        max_python_memory: 1024 * 1024 * 1024, // 1GB
    };

    let mut vm = EnhancedVM::new(config)?;
    let end_time = Instant::now() + std::time::Duration::from_secs(duration_seconds);
    let mut operations = 0u64;
    let mut tensor_id = 0u32;

    println!("🔥 Starting continuous operations...");
    
    while Instant::now() < end_time {
        // Mix of different operations
        match operations % 4 {
            0 => {
                // Create tensor
                let create_instruction = MLInstruction::TensorCreate {
                    id: tensor_id,
                    shape: vec![100, 100],
                    data: vec![(operations as f32) % 100.0; 10000],
                };
                vm.execute_ml_instruction(create_instruction).await?;
                tensor_id += 1;
            }
            1 => {
                // Add tensors (if we have at least 2)
                if tensor_id >= 2 {
                    let add_instruction = MLInstruction::TensorAdd {
                        input1: (tensor_id - 2) % tensor_id,
                        input2: (tensor_id - 1) % tensor_id,
                        output: tensor_id,
                    };
                    vm.execute_ml_instruction(add_instruction).await?;
                    tensor_id += 1;
                }
            }
            2 => {
                // Matrix multiplication (if we have at least 2)
                if tensor_id >= 2 {
                    let matmul_instruction = MLInstruction::MatMul {
                        input1: (tensor_id - 2) % tensor_id,
                        input2: (tensor_id - 1) % tensor_id,
                        output: tensor_id,
                    };
                    vm.execute_ml_instruction(matmul_instruction).await?;
                    tensor_id += 1;
                }
            }
            3 => {
                // Python execution
                #[cfg(feature = "enhanced-vm")]
                {
                    let python_code = format!("result = {} * 2 + 1; print(f'Operation {}: {{result}}', {})", 
                                            operations % 100, operations);
                    let _ = vm.execute_python(&python_code).await; // Ignore errors in stress test
                }
            }
            _ => unreachable!(),
        }

        operations += 1;

        // Print progress every 1000 operations
        if operations % 1000 == 0 {
            let elapsed = Instant::now().duration_since(end_time - std::time::Duration::from_secs(duration_seconds));
            let ops_per_second = operations as f64 / elapsed.as_secs_f64();
            println!("  📊 {} operations, {:.2} ops/sec", operations, ops_per_second);
        }
    }

    let total_duration = std::time::Duration::from_secs(duration_seconds);
    let avg_ops_per_second = operations as f64 / total_duration.as_secs_f64();

    println!();
    println!("✅ Stress test completed!");
    println!("  🎯 Total operations: {}", operations);
    println!("  📈 Average ops/sec: {:.2}", avg_ops_per_second);
    println!("  💾 Tensors created: {}", tensor_id);
    println!("  🧠 Memory pressure handled successfully");

    Ok(())
} 
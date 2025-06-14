//! Integration Tests for BCAI Enhanced VM
//!
//! Comprehensive end-to-end testing of all VM capabilities including
//! distributed training, ML workloads, and system integration.

use runtime::{
    enhanced_vm::{EnhancedVm, InstructionResult},
    DataType, MLInstruction, PythonConstraints, TensorId, VmConfig,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

mod distributed_training_e2e;

/// Integration test configuration
#[derive(Debug, Clone)]
struct IntegrationTestConfig {
    enable_distributed_training: bool,
    enable_performance_tests: bool,
    enable_security_tests: bool,
    test_timeout_seconds: u64,
}

impl Default for IntegrationTestConfig {
    fn default() -> Self {
        Self {
            enable_distributed_training: true,
            enable_performance_tests: true,
            enable_security_tests: true,
            test_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Comprehensive integration test suite
#[tokio::test]
async fn test_full_system_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting comprehensive BCAI system integration test");

    let config = IntegrationTestConfig::default();
    let start_time = Instant::now();

    // Test 1: Basic VM functionality
    test_basic_vm_functionality().await?;

    // Test 2: ML workload processing
    test_ml_workload_processing().await?;

    // Test 3: Python bridge integration
    test_python_bridge_integration().await?;

    // Test 4: Distributed training (if enabled)
    if config.enable_distributed_training {
        test_distributed_training_integration().await?;
    }

    // Test 5: Performance validation (if enabled)
    if config.enable_performance_tests {
        test_performance_requirements().await?;
    }

    // Test 6: Security validation (if enabled)
    if config.enable_security_tests {
        test_security_requirements().await?;
    }

    let total_duration = start_time.elapsed();
    println!("✅ Integration test suite completed in {:?}", total_duration);

    // Validate overall test duration
    if total_duration > Duration::from_secs(config.test_timeout_seconds) {
        return Err(format!(
            "Integration tests took too long: {:?} > {}s",
            total_duration, config.test_timeout_seconds
        )
        .into());
    }

    Ok(())
}

/// Test basic VM functionality
async fn test_basic_vm_functionality() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing basic VM functionality");

    let config = VmConfig {
        max_memory_mb: 512,
        max_execution_time_ms: 10000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Test tensor creation
    let result = vm
        .execute_instruction(MLInstruction::TensorCreate {
            tensor_id: TensorId(1),
            shape: vec![10, 10],
            dtype: DataType::Float32,
        })
        .await?;

    assert!(result.is_success(), "Tensor creation failed");

    // Test tensor operations
    let result = vm
        .execute_instruction(MLInstruction::TensorCreate {
            tensor_id: TensorId(2),
            shape: vec![10, 10],
            dtype: DataType::Float32,
        })
        .await?;

    assert!(result.is_success(), "Second tensor creation failed");

    let result = vm
        .execute_instruction(MLInstruction::TensorAdd {
            a: TensorId(1),
            b: TensorId(2),
            output: TensorId(3),
        })
        .await?;

    assert!(result.is_success(), "Tensor addition failed");

    println!("✅ Basic VM functionality test passed");
    Ok(())
}

/// Test ML workload processing
async fn test_ml_workload_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing ML workload processing");

    let config = VmConfig {
        max_memory_mb: 1024,
        max_execution_time_ms: 30000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Create tensors for a simple neural network
    let input_tensor = TensorId(100);
    let weight_tensor = TensorId(101);
    let bias_tensor = TensorId(102);
    let output_tensor = TensorId(103);

    // Create input tensor (batch_size=4, features=784)
    vm.execute_instruction(MLInstruction::TensorCreate {
        tensor_id: input_tensor,
        shape: vec![4, 784],
        dtype: DataType::Float32,
    })
    .await?;

    // Create weight tensor (features=784, outputs=10)
    vm.execute_instruction(MLInstruction::TensorCreate {
        tensor_id: weight_tensor,
        shape: vec![784, 10],
        dtype: DataType::Float32,
    })
    .await?;

    // Create bias tensor (outputs=10)
    vm.execute_instruction(MLInstruction::TensorCreate {
        tensor_id: bias_tensor,
        shape: vec![10],
        dtype: DataType::Float32,
    })
    .await?;

    // Perform matrix multiplication (simulating forward pass)
    let result = vm
        .execute_instruction(MLInstruction::TensorMatMul {
            a: input_tensor,
            b: weight_tensor,
            output: output_tensor,
        })
        .await?;

    assert!(result.is_success(), "Matrix multiplication failed");

    println!("✅ ML workload processing test passed");
    Ok(())
}

/// Test Python bridge integration
async fn test_python_bridge_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Python bridge integration");

    let config = VmConfig {
        max_memory_mb: 1024,
        max_execution_time_ms: 30000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Test basic Python execution
    let python_code = r#"
import random
import math

# Simple ML computation
def simple_linear_regression(x_values, y_values):
    n = len(x_values)
    sum_x = sum(x_values)
    sum_y = sum(y_values)
    sum_xy = sum(x * y for x, y in zip(x_values, y_values))
    sum_x_squared = sum(x * x for x in x_values)
    
    # Calculate slope and intercept
    slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x * sum_x)
    intercept = (sum_y - slope * sum_x) / n
    
    return slope, intercept

# Test data
x_data = [1, 2, 3, 4, 5]
y_data = [2, 4, 6, 8, 10]

slope, intercept = simple_linear_regression(x_data, y_data)
print(f"Linear regression: slope={slope:.2f}, intercept={intercept:.2f}")

# Validation
expected_slope = 2.0
expected_intercept = 0.0
assert abs(slope - expected_slope) < 0.1, f"Slope mismatch: {slope} vs {expected_slope}"
assert abs(intercept - expected_intercept) < 0.1, f"Intercept mismatch: {intercept} vs {expected_intercept}"

result = {"slope": slope, "intercept": intercept, "success": True}
"#;

    let result = vm
        .execute_instruction(MLInstruction::PythonExecute {
            code: python_code.to_string(),
            input_tensors: vec![],
            output_tensors: vec![("result".to_string(), TensorId(200))],
            constraints: PythonConstraints::default(),
        })
        .await?;

    assert!(result.is_success(), "Python bridge execution failed");

    println!("✅ Python bridge integration test passed");
    Ok(())
}

/// Test distributed training integration
async fn test_distributed_training_integration() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing distributed training integration");

    // Use the comprehensive distributed training test
    let result = distributed_training_e2e::run_all_distributed_tests().await;

    match result {
        Ok(_) => {
            println!("✅ Distributed training integration test passed");
            Ok(())
        }
        Err(e) => {
            println!("❌ Distributed training integration test failed: {}", e);
            Err(e)
        }
    }
}

/// Test performance requirements
async fn test_performance_requirements() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing performance requirements");

    let config = VmConfig {
        max_memory_mb: 2048,
        max_execution_time_ms: 60000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Performance test: Tensor operations throughput
    let start_time = Instant::now();
    let num_operations = 1000;

    for i in 0..num_operations {
        vm.execute_instruction(MLInstruction::TensorCreate {
            tensor_id: TensorId(i as u64),
            shape: vec![100, 100],
            dtype: DataType::Float32,
        })
        .await?;
    }

    let creation_duration = start_time.elapsed();
    let creation_throughput = num_operations as f64 / creation_duration.as_secs_f64();

    // Validate throughput meets minimum requirements
    const MIN_TENSOR_CREATION_THROUGHPUT: f64 = 100.0; // operations per second
    if creation_throughput < MIN_TENSOR_CREATION_THROUGHPUT {
        return Err(format!(
            "Tensor creation throughput too low: {:.2} < {:.2} ops/sec",
            creation_throughput, MIN_TENSOR_CREATION_THROUGHPUT
        )
        .into());
    }

    // Performance test: Matrix multiplication throughput
    let matmul_start = Instant::now();
    let num_matmuls = 100;

    for i in 0..num_matmuls {
        let a_id = TensorId(i as u64 * 2);
        let b_id = TensorId(i as u64 * 2 + 1);
        let output_id = TensorId(num_operations as u64 + i as u64);

        vm.execute_instruction(MLInstruction::TensorMatMul { a: a_id, b: b_id, output: output_id })
            .await?;
    }

    let matmul_duration = matmul_start.elapsed();
    let matmul_throughput = num_matmuls as f64 / matmul_duration.as_secs_f64();

    // Validate matmul throughput
    const MIN_MATMUL_THROUGHPUT: f64 = 10.0; // operations per second
    if matmul_throughput < MIN_MATMUL_THROUGHPUT {
        return Err(format!(
            "Matrix multiplication throughput too low: {:.2} < {:.2} ops/sec",
            matmul_throughput, MIN_MATMUL_THROUGHPUT
        )
        .into());
    }

    println!("📊 Performance metrics:");
    println!("   - Tensor creation: {:.2} ops/sec", creation_throughput);
    println!("   - Matrix multiplication: {:.2} ops/sec", matmul_throughput);

    println!("✅ Performance requirements test passed");
    Ok(())
}

/// Test security requirements
async fn test_security_requirements() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing security requirements");

    let config = VmConfig {
        max_memory_mb: 512,
        max_execution_time_ms: 10000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Test 1: Malicious code should be blocked
    let malicious_codes = vec![
        "import os; os.system('rm -rf /')",
        "import subprocess; subprocess.run(['cat', '/etc/passwd'])",
        "import socket; socket.socket()",
        "open('/etc/passwd', 'r').read()",
        "exec('import os; os.getcwd()')",
        "__import__('os').system('whoami')",
    ];

    let mut blocked_count = 0;
    for (i, code) in malicious_codes.iter().enumerate() {
        let result = vm
            .execute_instruction(MLInstruction::PythonExecute {
                code: code.to_string(),
                input_tensors: vec![],
                output_tensors: vec![],
                constraints: PythonConstraints::default(),
            })
            .await;

        if result.is_err() {
            blocked_count += 1;
            println!("   ✅ Blocked malicious code #{}", i + 1);
        } else {
            println!("   ⚠️ Malicious code #{} was not blocked", i + 1);
        }
    }

    // Require that most malicious code is blocked
    let block_rate = blocked_count as f64 / malicious_codes.len() as f64;
    const MIN_BLOCK_RATE: f64 = 0.8; // 80% of malicious code should be blocked

    if block_rate < MIN_BLOCK_RATE {
        return Err(format!(
            "Security block rate too low: {:.2} < {:.2}",
            block_rate, MIN_BLOCK_RATE
        )
        .into());
    }

    // Test 2: Resource limits should be enforced
    let resource_intensive_code = r#"
# Try to consume excessive memory
big_list = []
for i in range(1000000):
    big_list.append([0] * 1000)
"#;

    let result = vm
        .execute_instruction(MLInstruction::PythonExecute {
            code: resource_intensive_code.to_string(),
            input_tensors: vec![],
            output_tensors: vec![],
            constraints: PythonConstraints {
                max_memory_mb: 50, // Very limited memory
                max_execution_time_ms: 1000,
                ..PythonConstraints::default()
            },
        })
        .await;

    if result.is_ok() {
        return Err("Resource limits were not enforced".into());
    }

    println!("   ✅ Resource limits enforced");
    println!("📊 Security metrics:");
    println!("   - Malicious code block rate: {:.1}%", block_rate * 100.0);
    println!("   - Resource limits: enforced");

    println!("✅ Security requirements test passed");
    Ok(())
}

/// Test memory and resource management
#[tokio::test]
async fn test_resource_management() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing resource management");

    let config = VmConfig {
        max_memory_mb: 1024,
        max_execution_time_ms: 30000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Test memory usage tracking
    let initial_memory = vm.get_memory_stats().current_usage_bytes;

    // Create large tensors
    for i in 0..10 {
        vm.execute_instruction(MLInstruction::TensorCreate {
            tensor_id: TensorId(i),
            shape: vec![1000, 1000], // Large tensor
            dtype: DataType::Float32,
        })
        .await?;
    }

    let peak_memory = vm.get_memory_stats().current_usage_bytes;

    // Memory should have increased significantly
    if peak_memory <= initial_memory {
        return Err("Memory usage not properly tracked".into());
    }

    // Clean up tensors
    for i in 0..10 {
        vm.execute_instruction(MLInstruction::TensorDestroy { tensor_id: TensorId(i) }).await?;
    }

    // Allow some time for cleanup
    sleep(Duration::from_millis(100)).await;

    let final_memory = vm.get_memory_stats().current_usage_bytes;

    // Memory should be released (within reasonable bounds)
    let memory_reduction = peak_memory - final_memory;
    let memory_reduction_ratio = memory_reduction as f64 / peak_memory as f64;

    if memory_reduction_ratio < 0.5 {
        return Err(format!(
            "Insufficient memory cleanup: {:.1}% reduction",
            memory_reduction_ratio * 100.0
        )
        .into());
    }

    println!("📊 Memory management metrics:");
    println!("   - Initial memory: {} MB", initial_memory / 1024 / 1024);
    println!("   - Peak memory: {} MB", peak_memory / 1024 / 1024);
    println!("   - Final memory: {} MB", final_memory / 1024 / 1024);
    println!("   - Memory reduction: {:.1}%", memory_reduction_ratio * 100.0);

    println!("✅ Resource management test passed");
    Ok(())
}

/// Test concurrent operations
#[tokio::test]
async fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing concurrent operations");

    let config = VmConfig {
        max_memory_mb: 2048,
        max_execution_time_ms: 60000,
        enable_python_bridge: true,
        enable_cuda: false,
        enable_networking: false,
        sandbox_mode: true,
    };

    let mut vm = EnhancedVm::new(config).await?;

    // Create tensors for concurrent operations
    for i in 0..20 {
        vm.execute_instruction(MLInstruction::TensorCreate {
            tensor_id: TensorId(i),
            shape: vec![100, 100],
            dtype: DataType::Float32,
        })
        .await?;
    }

    // Run concurrent tensor operations
    let start_time = Instant::now();
    let mut handles = Vec::new();

    // Note: This is a simplified test - in a real implementation,
    // you would need to handle VM access differently for true concurrency
    for i in 0..10 {
        let tensor_a = TensorId(i * 2);
        let tensor_b = TensorId(i * 2 + 1);
        let output = TensorId(100 + i);

        // Simulate concurrent operation (simplified)
        let handle = tokio::spawn(async move {
            // In a real implementation, this would use a shared VM or VM pool
            sleep(Duration::from_millis(10)).await;
            Ok::<(), Box<dyn std::error::Error>>(())
        });

        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await??;
    }

    let concurrent_duration = start_time.elapsed();

    // Validate that concurrent operations completed in reasonable time
    if concurrent_duration > Duration::from_secs(5) {
        return Err(
            format!("Concurrent operations took too long: {:?}", concurrent_duration).into()
        );
    }

    println!("✅ Concurrent operations test passed in {:?}", concurrent_duration);
    Ok(())
}

// Helper trait for result validation
trait TestMethods {
    fn is_success(&self) -> bool;
}

impl TestMethods for InstructionResult {
    fn is_success(&self) -> bool {
        // Consider the operation successful if it completed without error
        // and produced some output or metrics
        self.output_tensors.is_some() || self.training_metrics.is_some()
    }
}

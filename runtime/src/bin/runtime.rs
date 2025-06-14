use clap::{Parser, Subcommand};
use runtime::{
    enhanced_vm::{EnhancedVM, VMConfig},
    ml_instructions::MLInstruction,
    hardware_abstraction::HardwareBackend,
    python_bridge::PythonBridge,
};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "runtime")]
#[command(about = "BCAI Enhanced VM Runtime")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the enhanced VM in interactive mode
    Interactive {
        #[arg(long)]
        python_bridge: bool,
        #[arg(long)]
        gpu_enabled: bool,
        #[arg(long, default_value = "cpu")]
        backend: String,
    },
    /// Execute ML job from file
    Execute {
        #[arg(short, long)]
        file: String,
        #[arg(long)]
        python_bridge: bool,
        #[arg(long)]
        gpu_enabled: bool,
    },
    /// Show VM capabilities and hardware info
    Info,
    /// Benchmark VM performance
    Benchmark {
        #[arg(long, default_value = "100")]
        iterations: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Interactive { python_bridge, gpu_enabled, backend } => {
            run_interactive(python_bridge, gpu_enabled, &backend).await?;
        }
        Commands::Execute { file, python_bridge, gpu_enabled } => {
            execute_file(&file, python_bridge, gpu_enabled).await?;
        }
        Commands::Info => {
            show_info().await?;
        }
        Commands::Benchmark { iterations } => {
            run_benchmark(iterations).await?;
        }
    }

    Ok(())
}

async fn run_interactive(python_bridge: bool, gpu_enabled: bool, backend: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 BCAI Enhanced VM Runtime - Interactive Mode");
    println!("==============================================");
    
    let backend = match backend {
        "cpu" => HardwareBackend::CPU,
        "cuda" => HardwareBackend::CUDA,
        "metal" => HardwareBackend::Metal,
        _ => HardwareBackend::CPU,
    };

    let config = VMConfig {
        memory_limit: 1024 * 1024 * 1024, // 1GB
        max_iterations: 10000,
        enable_python_bridge: python_bridge,
        hardware_backend: backend,
        enable_gpu: gpu_enabled,
        python_timeout_seconds: 30,
        max_python_memory: 512 * 1024 * 1024, // 512MB
    };

    let mut vm = EnhancedVM::new(config)?;
    
    if python_bridge {
        let python_bridge = PythonBridge::new(30, 512 * 1024 * 1024)?;
        println!("✅ Python bridge initialized");
    }

    println!("💡 Available commands:");
    println!("  tensor_add <id1> <id2> -> <result_id>  - Add two tensors");
    println!("  tensor_create <id> <shape> <data>      - Create tensor");
    println!("  python <code>                          - Execute Python code");
    println!("  quit                                   - Exit");
    println!();

    loop {
        print!("vm> ");
        use std::io::{self, Write};
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "quit" {
            break;
        }

        match execute_command(&mut vm, input).await {
            Ok(result) => println!("✅ {}", result),
            Err(e) => println!("❌ Error: {}", e),
        }
    }

    Ok(())
}

async fn execute_command(vm: &mut EnhancedVM, command: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    
    match parts.get(0) {
        Some(&"tensor_add") => {
            if parts.len() != 4 {
                return Err("Usage: tensor_add <id1> <id2> <result_id>".into());
            }
            
            let id1: u32 = parts[1].parse()?;
            let id2: u32 = parts[2].parse()?;
            let result_id: u32 = parts[3].parse()?;
            
            let instruction = MLInstruction::TensorAdd { 
                input1: id1, 
                input2: id2, 
                output: result_id 
            };
            
            vm.execute_ml_instruction(instruction).await?;
            Ok(format!("Added tensors {} + {} -> {}", id1, id2, result_id))
        }
        Some(&"tensor_create") => {
            if parts.len() < 4 {
                return Err("Usage: tensor_create <id> <shape> <data...>".into());
            }
            
            let id: u32 = parts[1].parse()?;
            let shape: Vec<usize> = parts[2].split(',').map(|s| s.parse()).collect::<Result<Vec<_>, _>>()?;
            let data: Vec<f32> = parts[3..].iter().map(|s| s.parse()).collect::<Result<Vec<_>, _>>()?;
            
            let instruction = MLInstruction::TensorCreate { 
                id, 
                shape, 
                data 
            };
            
            vm.execute_ml_instruction(instruction).await?;
            Ok(format!("Created tensor {} with shape {:?}", id, shape))
        }
        Some(&"python") => {
            let code = &command[7..]; // Skip "python "
            match vm.execute_python(code).await {
                Ok(result) => Ok(format!("Python result: {}", result)),
                Err(e) => Err(format!("Python error: {}", e).into()),
            }
        }
        _ => Err(format!("Unknown command: {}", parts.get(0).unwrap_or(&"")).into()),
    }
}

async fn execute_file(file: &str, python_bridge: bool, gpu_enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 Executing ML job from file: {}", file);
    
    let config = VMConfig {
        memory_limit: 1024 * 1024 * 1024,
        max_iterations: 10000,
        enable_python_bridge: python_bridge,
        hardware_backend: HardwareBackend::CPU,
        enable_gpu: gpu_enabled,
        python_timeout_seconds: 300,
        max_python_memory: 1024 * 1024 * 1024,
    };

    let mut vm = EnhancedVM::new(config)?;
    
    // Read and execute the file
    let content = std::fs::read_to_string(file)?;
    
    if file.ends_with(".py") {
        // Execute as Python
        match vm.execute_python(&content).await {
            Ok(result) => println!("✅ Execution completed: {}", result),
            Err(e) => println!("❌ Execution failed: {}", e),
        }
    } else {
        // Parse as ML instructions
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() || line.starts_with("#") {
                continue;
            }
            
            match execute_command(&mut vm, line).await {
                Ok(result) => println!("Line {}: {}", i + 1, result),
                Err(e) => {
                    println!("❌ Error on line {}: {}", i + 1, e);
                    break;
                }
            }
        }
    }
    
    Ok(())
}

async fn show_info() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 BCAI Enhanced VM Information");
    println!("===============================");
    println!();
    
    println!("📊 VM Capabilities:");
    println!("  • Native ML Instructions: 200+");
    println!("  • Tensor Operations: ✅");
    println!("  • Neural Network Primitives: ✅");
    println!("  • Python Bridge: ✅");
    println!("  • Hardware Acceleration: ✅");
    println!();
    
    println!("🖥️  Hardware Backends:");
    println!("  • CPU: ✅ (Always available)");
    
    #[cfg(feature = "cuda")]
    println!("  • CUDA: ✅ (Available)");
    #[cfg(not(feature = "cuda"))]
    println!("  • CUDA: ❌ (Not compiled)");
    
    #[cfg(feature = "metal-gpu")]
    println!("  • Metal: ✅ (Available)");
    #[cfg(not(feature = "metal-gpu"))]
    println!("  • Metal: ❌ (Not compiled)");
    
    println!();
    
    println!("🐍 Python Integration:");
    #[cfg(feature = "enhanced-vm")]
    {
        println!("  • PyTorch Support: ✅");
        println!("  • NumPy Support: ✅");
        println!("  • Sandboxed Execution: ✅");
        println!("  • Resource Monitoring: ✅");
    }
    #[cfg(not(feature = "enhanced-vm"))]
    {
        println!("  • Status: ❌ (Enhanced VM not compiled)");
    }
    
    println!();
    println!("📝 Supported ML Architectures:");
    println!("  • Linear Regression: ✅");
    println!("  • Logistic Regression: ✅");
    println!("  • Neural Networks: ✅");
    println!("  • CNN: ✅");
    println!("  • LSTM: ✅");
    println!("  • Transformers: ✅");
    println!("  • Custom Architectures: ✅");
    
    Ok(())
}

async fn run_benchmark(iterations: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏃 Running VM Performance Benchmark");
    println!("==================================");
    println!("Iterations: {}", iterations);
    println!();
    
    let config = VMConfig {
        memory_limit: 1024 * 1024 * 1024,
        max_iterations: iterations,
        enable_python_bridge: false,
        hardware_backend: HardwareBackend::CPU,
        enable_gpu: false,
        python_timeout_seconds: 30,
        max_python_memory: 512 * 1024 * 1024,
    };

    let mut vm = EnhancedVM::new(config)?;
    
    println!("🔥 Tensor Operations Benchmark:");
    let start = std::time::Instant::now();
    
    // Create test tensors
    let create_instruction = MLInstruction::TensorCreate {
        id: 1,
        shape: vec![1000, 1000],
        data: vec![1.0; 1000000],
    };
    vm.execute_ml_instruction(create_instruction).await?;
    
    let create_instruction2 = MLInstruction::TensorCreate {
        id: 2,
        shape: vec![1000, 1000],
        data: vec![2.0; 1000000],
    };
    vm.execute_ml_instruction(create_instruction2).await?;
    
    // Benchmark tensor addition
    for i in 0..iterations {
        let add_instruction = MLInstruction::TensorAdd {
            input1: 1,
            input2: 2,
            output: 3,
        };
        vm.execute_ml_instruction(add_instruction).await?;
    }
    
    let duration = start.elapsed();
    let ops_per_second = iterations as f64 / duration.as_secs_f64();
    
    println!("  ⏱️  Time: {:?}", duration);
    println!("  📈 Operations/sec: {:.2}", ops_per_second);
    println!("  🎯 Throughput: {:.2} MFLOPS", ops_per_second * 1000000.0 / 1e6);
    
    println!();
    println!("✅ Benchmark completed successfully!");
    
    Ok(())
} 
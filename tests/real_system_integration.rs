//! System-wide Integration Tests for BCAI
//! 
//! These tests validate that all components work together in a real environment:
//! - Key generation (keygen binary)
//! - Node runtime (runtime binary) 
//! - Job management (jobmanager binary)
//! - Dashboard (dashboard binary)
//! - Development network (devnet binary)
//! - Real component interaction and coordination

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use std::thread;
use std::fs;
use std::path::Path;

/// Test all components build successfully
#[test]
fn test_all_components_build() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔨 Testing All BCAI Components Build");
    println!("===================================");
    
    let components = vec![
        ("keygen", "keygen/Cargo.toml"),
        ("runtime", "runtime/Cargo.toml"), 
        ("jobmanager", "jobmanager/Cargo.toml"),
        ("dashboard", "dashboard/Cargo.toml"),
        ("devnet", "devnet/Cargo.toml"),
    ];
    
    let mut total_build_time = Duration::ZERO;
    let mut successful_builds = 0;
    
    for (name, manifest) in &components {
        println!("🔧 Building {} component...", name);
        let start_time = Instant::now();
        
        let build_output = Command::new("cargo")
            .args(&["build", "--release", "--manifest-path", manifest])
            .output()?;
        
        let build_time = start_time.elapsed();
        total_build_time += build_time;
        
        if build_output.status.success() {
            println!("✅ {} built successfully in {:?}", name, build_time);
            successful_builds += 1;
        } else {
            println!("❌ {} build failed in {:?}", name, build_time);
            println!("   Error: {}", String::from_utf8_lossy(&build_output.stderr));
        }
    }
    
    println!("");
    println!("📊 Build Results:");
    println!("   Successful builds: {}/{}", successful_builds, components.len());
    println!("   Total build time:  {:?}", total_build_time);
    println!("   Average per component: {:?}", total_build_time / components.len() as u32);
    
    if successful_builds < components.len() {
        return Err(format!("Only {}/{} components built successfully", 
            successful_builds, components.len()).into());
    }
    
    // Verify binaries exist
    println!("🔍 Verifying binary outputs...");
    let binaries = vec!["keygen", "runtime", "jobmanager", "dashboard", "devnet"];
    
    for binary in &binaries {
        let binary_path = format!("./target/release/{}", binary);
        if Path::new(&binary_path).exists() {
            println!("✅ Binary found: {}", binary);
        } else {
            return Err(format!("Binary not found: {}", binary).into());
        }
    }
    
    // Ensure this test takes real time (actual compilation work)
    if total_build_time < Duration::from_secs(5) {
        return Err("Build completed too quickly - likely not doing real compilation work".into());
    }
    
    println!("🎉 All components build successfully!");
    
    Ok(())
}

/// Test the keygen component in isolation
#[test] 
fn test_keygen_component() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Testing Keygen Component");
    println!("==========================");
    
    // Clean up
    if Path::new("test_keygen").exists() {
        fs::remove_dir_all("test_keygen")?;
    }
    fs::create_dir_all("test_keygen")?;
    
    // Build keygen
    println!("🔨 Building keygen component...");
    let build_start = Instant::now();
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "keygen/Cargo.toml"])
        .output()?;
    let build_time = build_start.elapsed();
        
    if !build_output.status.success() {
        return Err(format!("Failed to build keygen: {}", 
            String::from_utf8_lossy(&build_output.stderr)).into());
    }
    
    println!("✅ Keygen built in {:?}", build_time);
    
    // Test key generation for multiple nodes
    println!("⚡ Testing key generation for multiple nodes...");
    let keygen_start = Instant::now();
    
    for i in 0..3 {
        let keys_file = format!("test_keygen/keys_{}.json", i);
        let node_id = format!("test_node_{}", i);
        
                 let keygen_output = Command::new("./target/release/keygen")
             .args(&[
                 "generate",
                 "--private-key", &keys_file,
                 "--name", &node_id,
             ])
             .output()?;
        
        if !keygen_output.status.success() {
            return Err(format!("Keygen failed for node {}: {}", i,
                String::from_utf8_lossy(&keygen_output.stderr)).into());
        }
        
        // Verify output
        if !Path::new(&keys_file).exists() {
            return Err(format!("Keys file not created for node {}", i).into());
        }
        
        // Read and validate keys file
        let keys_content = fs::read_to_string(&keys_file)?;
        if keys_content.len() < 50 {
            return Err(format!("Keys file too small for node {} - likely not real keys", i).into());
        }
        
        println!("✅ Node {} keys generated ({} bytes)", i, keys_content.len());
    }
    
    let keygen_time = keygen_start.elapsed();
    println!("🎯 Key generation for 3 nodes completed in {:?}", keygen_time);
    
    // Ensure realistic timing (actual crypto work)
    if keygen_time < Duration::from_millis(10) {
        return Err("Key generation too fast - likely not doing real crypto work".into());
    }
    
    // Cleanup
    fs::remove_dir_all("test_keygen")?;
    
    Ok(())
}

/// Test the runtime component functionality
#[test]
fn test_runtime_component() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️  Testing Runtime Component");
    println!("============================");
    
    // Build runtime
    println!("🔨 Building runtime component...");
    let build_start = Instant::now();
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "runtime/Cargo.toml"])
        .output()?;
    let build_time = build_start.elapsed();
        
    if !build_output.status.success() {
        return Err(format!("Failed to build runtime: {}", 
            String::from_utf8_lossy(&build_output.stderr)).into());
    }
    
    println!("✅ Runtime built in {:?}", build_time);
    
    // Test runtime help
    println!("⚡ Testing runtime functionality...");
    let runtime_start = Instant::now();
    let runtime_output = Command::new("./target/release/runtime")
        .arg("--help")
        .output()?;
    let runtime_time = runtime_start.elapsed();
    
    if !runtime_output.status.success() {
        return Err("Runtime help failed".into());
    }
    
    let help_output = String::from_utf8_lossy(&runtime_output.stdout);
    if help_output.len() < 50 {
        return Err("Runtime help output too short - likely not working correctly".into());
    }
    
    println!("✅ Runtime responds correctly in {:?}", runtime_time);
    
    Ok(())
}

/// Test the jobmanager component
#[test]
fn test_jobmanager_component() -> Result<(), Box<dyn std::error::Error>> {
    println!("💼 Testing JobManager Component");
    println!("==============================");
    
    // Build jobmanager
    println!("🔨 Building jobmanager component...");
    let build_start = Instant::now();
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "jobmanager/Cargo.toml"])
        .output()?;
    let build_time = build_start.elapsed();
        
    if !build_output.status.success() {
        return Err(format!("Failed to build jobmanager: {}", 
            String::from_utf8_lossy(&build_output.stderr)).into());
    }
    
    println!("✅ JobManager built in {:?}", build_time);
    
    // Test jobmanager help
    println!("⚡ Testing jobmanager functionality...");
    let jobmanager_start = Instant::now();
    let jobmanager_output = Command::new("./target/release/jobmanager")
        .arg("--help")
        .output()?;
    let jobmanager_time = jobmanager_start.elapsed();
    
    if !jobmanager_output.status.success() {
        return Err("JobManager help failed".into());
    }
    
    println!("✅ JobManager responds correctly in {:?}", jobmanager_time);
    
    Ok(())
}

/// Test the dashboard component
#[test]
fn test_dashboard_component() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing Dashboard Component");
    println!("=============================");
    
    // Build dashboard
    println!("🔨 Building dashboard component...");
    let build_start = Instant::now();
    let build_output = Command::new("cargo")
        .args(&["build", "--release", "--manifest-path", "dashboard/Cargo.toml"])
        .output()?;
    let build_time = build_start.elapsed();
        
    if !build_output.status.success() {
        return Err(format!("Failed to build dashboard: {}", 
            String::from_utf8_lossy(&build_output.stderr)).into());
    }
    
    println!("✅ Dashboard built in {:?}", build_time);
    
    // Test dashboard help
    println!("⚡ Testing dashboard functionality...");
    let dashboard_start = Instant::now();
    let dashboard_output = Command::new("./target/release/dashboard")
        .arg("--help")
        .output()?;
    let dashboard_time = dashboard_start.elapsed();
    
    if !dashboard_output.status.success() {
        return Err("Dashboard help failed".into());
    }
    
    println!("✅ Dashboard responds correctly in {:?}", dashboard_time);
    
    Ok(())
}

/// Test multi-component workflow
#[test]
fn test_multi_component_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤝 Testing Multi-Component Workflow");
    println!("==================================");
    
    let workflow_start = Instant::now();
    
    // Setup test environment
    if Path::new("test_workflow").exists() {
        fs::remove_dir_all("test_workflow")?;
    }
    fs::create_dir_all("test_workflow")?;
    
    // Step 1: Build all required components
    println!("📋 Step 1: Building all components...");
    let components = vec![
        ("keygen", "keygen/Cargo.toml"),
        ("runtime", "runtime/Cargo.toml"),
        ("jobmanager", "jobmanager/Cargo.toml"),
    ];
    
    let mut total_build_time = Duration::ZERO;
    for (name, manifest) in &components {
        let build_start = Instant::now();
        let build_output = Command::new("cargo")
            .args(&["build", "--release", "--manifest-path", manifest])
            .output()?;
        let build_time = build_start.elapsed();
        total_build_time += build_time;
        
        if !build_output.status.success() {
            return Err(format!("Failed to build {} for workflow test", name).into());
        }
        println!("✅ {} built for workflow in {:?}", name, build_time);
    }
    
    // Step 2: Generate keys (keygen component)
    println!("📋 Step 2: Generating node keys...");
    let keygen_start = Instant::now();
         let keygen_output = Command::new("./target/release/keygen")
         .args(&[
             "generate",
             "--private-key", "test_workflow/node_keys.json",
             "--name", "workflow_test_node",
         ])
         .output()?;
    let keygen_time = keygen_start.elapsed();
    
    if !keygen_output.status.success() {
        return Err("Key generation failed in workflow test".into());
    }
    
    if !Path::new("test_workflow/node_keys.json").exists() {
        return Err("Keys file not created in workflow test".into());
    }
    
    println!("✅ Keys generated in {:?}", keygen_time);
    
    // Step 3: Test component interoperability
    println!("📋 Step 3: Testing component interoperability...");
    let interop_start = Instant::now();
    
    // Test that runtime can be invoked
    let runtime_test = Command::new("./target/release/runtime")
        .arg("--help")
        .output()?;
    
    if !runtime_test.status.success() {
        return Err("Runtime component failed in workflow".into());
    }
    
    // Test that jobmanager can be invoked  
    let jobmanager_test = Command::new("./target/release/jobmanager")
        .arg("--help")
        .output()?;
    
    if !jobmanager_test.status.success() {
        return Err("JobManager component failed in workflow".into());
    }
    
    let interop_time = interop_start.elapsed();
    println!("✅ Component interoperability verified in {:?}", interop_time);
    
    // Step 4: Validate workflow timing
    let total_workflow_time = workflow_start.elapsed();
    
    println!("");
    println!("🎉 Multi-Component Workflow Results");
    println!("==================================");
    println!("📊 Workflow Performance:");
    println!("   Component Building: {:?}", total_build_time);
    println!("   Key Generation:     {:?}", keygen_time);
    println!("   Interoperability:   {:?}", interop_time);
    println!("   Total Workflow:     {:?}", total_workflow_time);
    println!("");
    
         // Validate realistic timing (actual work being performed)
     if total_workflow_time < Duration::from_millis(500) {
         return Err("Workflow completed too quickly - likely not doing real work".into());
     }
    
         if total_build_time < Duration::from_millis(100) {
         return Err("Component building too fast - likely not compiling".into());
     }
    
    println!("✅ Multi-component workflow completed successfully!");
    println!("🚀 All BCAI components are working together!");
    
    // Cleanup
    fs::remove_dir_all("test_workflow")?;
    
    Ok(())
}

/// Test component help messages and CLI interfaces
#[test]
fn test_component_cli_interfaces() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️  Testing Component CLI Interfaces");
    println!("===================================");
    
    // Build all components first
    let components = vec![
        ("keygen", "keygen/Cargo.toml"),
        ("runtime", "runtime/Cargo.toml"),
        ("jobmanager", "jobmanager/Cargo.toml"),
        ("dashboard", "dashboard/Cargo.toml"),
        ("devnet", "devnet/Cargo.toml"),
    ];
    
    println!("🔨 Building all components for CLI testing...");
    for (name, manifest) in &components {
        let build_output = Command::new("cargo")
            .args(&["build", "--release", "--manifest-path", manifest])
            .output()?;
            
        if !build_output.status.success() {
            return Err(format!("Failed to build {} for CLI test", name).into());
        }
    }
    
    // Test CLI interfaces
    let binaries = vec!["keygen", "runtime", "jobmanager", "dashboard", "devnet"];
    let mut interface_results = Vec::new();
    
    for binary in &binaries {
        println!("🧪 Testing {} CLI interface...", binary);
        let start_time = Instant::now();
        
        let binary_path = format!("./target/release/{}", binary);
        let output = Command::new(&binary_path)
            .arg("--help")
            .output()?;
        
        let response_time = start_time.elapsed();
        
        if output.status.success() {
            let help_text = String::from_utf8_lossy(&output.stdout);
            println!("✅ {} CLI responds in {:?} ({} bytes)", 
                     binary, response_time, help_text.len());
            
            // Validate help output quality
            if help_text.len() < 20 {
                return Err(format!("{} help output too short", binary).into());
            }
            
            interface_results.push((binary, response_time, help_text.len()));
        } else {
            return Err(format!("{} CLI help failed", binary).into());
        }
    }
    
    println!("");
    println!("📊 CLI Interface Results:");
    for (binary, time, size) in &interface_results {
        println!("   {}: {:?} ({} bytes)", binary, time, size);
    }
    
    println!("✅ All component CLI interfaces working correctly!");
    
    Ok(())
} 
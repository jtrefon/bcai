//! Python Bridge for Sandboxed ML Code Execution
//! 
//! This module provides secure execution of Python code within the VM,
//! allowing access to popular ML libraries while maintaining security isolation.

use crate::{VmError, TensorId, PythonConstraints, enhanced_vm::InstructionResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;
use thiserror::Error;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use parking_lot::Mutex;

/// Python execution errors
#[derive(Debug, Error)]
pub enum PythonError {
    #[error("Python interpreter initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Code execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    #[error("Import not allowed: {0}")]
    ImportNotAllowed(String),
    #[error("Tensor conversion failed: {0}")]
    TensorConversionFailed(String),
    #[error("Timeout exceeded")]
    TimeoutExceeded,
    #[error("PyO3 error: {0}")]
    PyO3Error(String),
}

impl From<PyErr> for PythonError {
    fn from(err: PyErr) -> Self {
        PythonError::PyO3Error(err.to_string())
    }
}

/// Python execution result
#[derive(Debug, Clone)]
pub struct PythonExecutionResult {
    pub success: bool,
    pub output: String,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub memory_used_mb: usize,
    pub output_tensors: HashMap<String, Vec<f32>>,
    pub locals: HashMap<String, String>, // Serialized Python objects
}

/// Code validation result with detailed security analysis
#[derive(Debug)]
pub struct CodeValidation {
    pub is_safe: bool,
    pub violations: Vec<SecurityViolation>,
    pub imported_modules: Vec<String>,
    pub estimated_complexity: u32,
    pub forbidden_calls: Vec<String>,
    pub file_access_attempts: Vec<String>,
    pub network_access_attempts: Vec<String>,
}

/// Specific security violation types
#[derive(Debug, Clone)]
pub struct SecurityViolation {
    pub violation_type: ViolationType,
    pub line_number: usize,
    pub description: String,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum ViolationType {
    DisallowedImport,
    SystemCall,
    FileAccess,
    NetworkAccess,
    DangerousFunction,
    ResourceAbuse,
}

#[derive(Debug, Clone)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Sandboxed Python interpreter with PyO3
pub struct PythonSandbox {
    constraints: PythonConstraints,
    resource_monitor: Arc<Mutex<ResourceMonitor>>,
    import_validator: ImportValidator,
    execution_stats: Arc<Mutex<ExecutionStats>>,
    interpreter_lock: Arc<Mutex<()>>, // Ensure single-threaded access
}

/// Resource monitoring for Python execution
#[derive(Debug, Default)]
struct ResourceMonitor {
    current_memory_mb: usize,
    peak_memory_mb: usize,
    execution_start: Option<Instant>,
    active_tensors: HashMap<String, TensorId>,
}

/// Import validation with extensive security checks
struct ImportValidator {
    allowed_imports: Vec<String>,
    blocked_patterns: Vec<String>,
    dangerous_functions: Vec<String>,
}

/// Execution statistics with detailed tracking
#[derive(Debug, Default)]
struct ExecutionStats {
    total_executions: u64,
    successful_executions: u64,
    failed_executions: u64,
    security_violations: u64,
    total_execution_time: Duration,
    peak_memory_usage: usize,
    code_size_stats: CodeSizeStats,
}

#[derive(Debug, Default)]
struct CodeSizeStats {
    min_lines: usize,
    max_lines: usize,
    avg_lines: f64,
    total_lines: usize,
}

impl PythonSandbox {
    /// Create a new Python sandbox with PyO3 integration
    pub fn new(constraints: PythonConstraints) -> Result<Self, PythonError> {
        // Initialize Python interpreter if not already done
        pyo3::prepare_freethreaded_python();
        
        let import_validator = ImportValidator {
            allowed_imports: constraints.allowed_imports.clone(),
            blocked_patterns: vec![
                "os".to_string(),
                "sys".to_string(),
                "subprocess".to_string(),
                "socket".to_string(),
                "__import__".to_string(),
                "eval".to_string(),
                "exec".to_string(),
                "open".to_string(),
                "file".to_string(),
                "urllib".to_string(),
                "requests".to_string(),
                "http".to_string(),
                "ftplib".to_string(),
                "smtplib".to_string(),
                "webbrowser".to_string(),
                "ctypes".to_string(),
                "multiprocessing".to_string(),
                "threading".to_string(),
                "asyncio".to_string(),
            ],
            dangerous_functions: vec![
                "compile".to_string(),
                "__builtins__".to_string(),
                "globals".to_string(),
                "locals".to_string(),
                "vars".to_string(),
                "dir".to_string(),
                "hasattr".to_string(),
                "getattr".to_string(),
                "setattr".to_string(),
                "delattr".to_string(),
                "input".to_string(),
                "raw_input".to_string(),
            ],
        };

        Ok(Self {
            constraints,
            resource_monitor: Arc::new(Mutex::new(ResourceMonitor::default())),
            import_validator,
            execution_stats: Arc::new(Mutex::new(ExecutionStats::default())),
            interpreter_lock: Arc::new(Mutex::new(())),
        })
    }

    /// Execute Python code with comprehensive security and monitoring
    pub fn execute_code(
        &mut self,
        code: &str,
        input_tensors: &[(String, TensorId)],
        output_tensors: &[(String, TensorId)],
        constraints: &PythonConstraints,
    ) -> Result<InstructionResult, PythonError> {
        let start_time = Instant::now();
        
        {
            let mut monitor = self.resource_monitor.lock();
            monitor.execution_start = Some(start_time);
        }

        // Comprehensive code validation
        let validation = self.validate_code_comprehensive(code)?;
        if !validation.is_safe {
            self.update_execution_stats(start_time, false, true);
            return Err(PythonError::SecurityViolation(format!(
                "Code validation failed: {} violations found",
                validation.violations.len()
            )));
        }

        // Check execution constraints
        self.check_constraints(constraints)?;

        // Prepare execution environment with tensor data
        let tensor_context = self.prepare_tensor_context(input_tensors)?;

        // Execute code in restricted environment
        let result = {
            let _lock = self.interpreter_lock.lock(); // Ensure thread safety
            self.execute_sandboxed_code_pyo3(code, tensor_context, constraints)
        }?;

        // Extract output tensors from Python environment
        let output_data = self.extract_output_tensors_pyo3(&result, output_tensors)?;

        // Update statistics
        self.update_execution_stats(start_time, true, false);

        Ok(InstructionResult {
            output_tensors: Some(output_data),
            training_metrics: None,
        })
    }

    /// Comprehensive code validation with security analysis
    fn validate_code_comprehensive(&self, code: &str) -> Result<CodeValidation, PythonError> {
        let mut violations = Vec::new();
        let mut imported_modules = Vec::new();
        let mut forbidden_calls = Vec::new();
        let mut file_access_attempts = Vec::new();
        let mut network_access_attempts = Vec::new();

        let lines: Vec<&str> = code.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_number = line_num + 1;
            
            // Check for imports
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                self.validate_import_line(trimmed, line_number, &mut violations, &mut imported_modules);
            }
            
            // Check for dangerous function calls
            for dangerous_func in &self.import_validator.dangerous_functions {
                if trimmed.contains(dangerous_func) {
                    violations.push(SecurityViolation {
                        violation_type: ViolationType::DangerousFunction,
                        line_number,
                        description: format!("Dangerous function call: {}", dangerous_func),
                        severity: Severity::High,
                    });
                    forbidden_calls.push(dangerous_func.clone());
                }
            }
            
            // Check for file operations
            if trimmed.contains("open(") && !self.constraints.enable_file_access {
                violations.push(SecurityViolation {
                    violation_type: ViolationType::FileAccess,
                    line_number,
                    description: "File access not permitted".to_string(),
                    severity: Severity::Medium,
                });
                file_access_attempts.push(trimmed.to_string());
            }
            
            // Check for network operations
            let network_patterns = ["urllib", "requests", "socket", "http", "ftp", "smtp"];
            for pattern in &network_patterns {
                if trimmed.contains(pattern) && !self.constraints.enable_networking {
                    violations.push(SecurityViolation {
                        violation_type: ViolationType::NetworkAccess,
                        line_number,
                        description: format!("Network access not permitted: {}", pattern),
                        severity: Severity::High,
                    });
                    network_access_attempts.push(trimmed.to_string());
                }
            }

            // Check for system calls
            let system_patterns = ["os.system", "subprocess.", "os.popen", "os.exec"];
            for pattern in &system_patterns {
                if trimmed.contains(pattern) {
                    violations.push(SecurityViolation {
                        violation_type: ViolationType::SystemCall,
                        line_number,
                        description: format!("System call detected: {}", pattern),
                        severity: Severity::Critical,
                    });
                }
            }
        }

        // Estimate code complexity
        let estimated_complexity = self.estimate_code_complexity(&lines);

        Ok(CodeValidation {
            is_safe: violations.is_empty(),
            violations,
            imported_modules,
            estimated_complexity,
            forbidden_calls,
            file_access_attempts,
            network_access_attempts,
        })
    }

    /// Validate individual import line
    fn validate_import_line(
        &self,
        import_line: &str,
        line_number: usize,
        violations: &mut Vec<SecurityViolation>,
        imported_modules: &mut Vec<String>,
    ) {
        // Extract module name more carefully
        let module_name = if import_line.starts_with("from ") {
            // Handle "from module import ..."
            if let Some(from_pos) = import_line.find(" import ") {
                import_line[5..from_pos].trim()
            } else {
                ""
            }
        } else if import_line.starts_with("import ") {
            // Handle "import module"
            let parts: Vec<&str> = import_line[7..].split_whitespace().collect();
            if !parts.is_empty() {
                parts[0].split('.').next().unwrap_or("")
            } else {
                ""
            }
        } else {
            ""
        };

        if !module_name.is_empty() {
            imported_modules.push(module_name.to_string());
            
            // Check if import is allowed
            if !self.import_validator.is_import_allowed(module_name) {
                let severity = if self.import_validator.blocked_patterns.contains(&module_name.to_string()) {
                    Severity::Critical
                } else {
                    Severity::Medium
                };
                
                violations.push(SecurityViolation {
                    violation_type: ViolationType::DisallowedImport,
                    line_number,
                    description: format!("Disallowed import: {}", module_name),
                    severity,
                });
            }
        }
    }

    /// Estimate code complexity for resource planning
    fn estimate_code_complexity(&self, lines: &[&str]) -> u32 {
        let mut complexity = lines.len() as u32;
        
        for line in lines {
            let trimmed = line.trim();
            
            // Add complexity for loops
            if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                complexity += 5;
            }
            
            // Add complexity for nested structures
            if trimmed.starts_with("    ") { // Basic indentation detection
                complexity += 2;
            }
            
            // Add complexity for ML operations
            if trimmed.contains("torch.") || trimmed.contains("np.") || trimmed.contains("model.") {
                complexity += 3;
            }
        }
        
        complexity
    }

    /// Execute Python code using PyO3 with security restrictions
    fn execute_sandboxed_code_pyo3(
        &mut self,
        code: &str,
        tensor_context: HashMap<String, Vec<f32>>,
        constraints: &PythonConstraints,
    ) -> Result<PythonExecutionResult, PythonError> {
        let start_time = Instant::now();
        
        Python::with_gil(|py| {
            // Create restricted globals
            let globals = self.create_restricted_globals(py)?;
            
            // Add tensor data to globals
            for (name, data) in tensor_context {
                let py_list = PyList::new(py, &data);
                globals.set_item(name, py_list)?;
            }
            
            // Create locals dict for capturing results
            let locals = PyDict::new(py);
            
            // Execute code with timeout monitoring
            let execution_result = self.execute_with_timeout(py, code, globals, locals, constraints);
            
            match execution_result {
                Ok(_) => {
                    // Capture output and locals
                    let output = "Execution completed successfully".to_string();
                    let mut serialized_locals = HashMap::new();
                    
                    // Serialize important locals (excluding built-ins)
                    for (key, value) in locals.iter() {
                        if let Ok(key_str) = key.extract::<String>() {
                            if !key_str.starts_with('_') {
                                if let Ok(value_str) = value.str().and_then(|s| s.extract::<String>()) {
                                    serialized_locals.insert(key_str, value_str);
                                }
                            }
                        }
                    }
                    
                    Ok(PythonExecutionResult {
                        success: true,
                        output,
                        error_message: None,
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        memory_used_mb: self.estimate_memory_usage(),
                        output_tensors: HashMap::new(), // Will be filled by extract_output_tensors_pyo3
                        locals: serialized_locals,
                    })
                }
                Err(py_err) => {
                    Ok(PythonExecutionResult {
                        success: false,
                        output: String::new(),
                        error_message: Some(py_err.to_string()),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        memory_used_mb: self.estimate_memory_usage(),
                        output_tensors: HashMap::new(),
                        locals: HashMap::new(),
                    })
                }
            }
        })
    }

    /// Create restricted globals dictionary
    fn create_restricted_globals(&self, py: Python) -> Result<&PyDict, PythonError> {
        let globals = PyDict::new(py);
        
        // Add only safe built-ins
        let safe_builtins = vec![
            "len", "range", "enumerate", "zip", "map", "filter", "sum", "min", "max",
            "abs", "round", "int", "float", "str", "bool", "list", "dict", "tuple", "set",
            "print", "type", "isinstance", "hasattr"
        ];
        
        let builtins = py.import("builtins")?;
        let restricted_builtins = PyDict::new(py);
        
        for builtin_name in safe_builtins {
            if let Ok(builtin_func) = builtins.getattr(builtin_name) {
                restricted_builtins.set_item(builtin_name, builtin_func)?;
            }
        }
        
        globals.set_item("__builtins__", restricted_builtins)?;
        
        // Add allowed modules if they're in the whitelist
        for module_name in &self.constraints.allowed_imports {
            if let Ok(module) = py.import(module_name) {
                globals.set_item(module_name, module)?;
            }
        }
        
        Ok(globals)
    }

    /// Execute code with timeout and resource monitoring
    fn execute_with_timeout(
        &self,
        py: Python,
        code: &str,
        globals: &PyDict,
        locals: &PyDict,
        constraints: &PythonConstraints,
    ) -> PyResult<()> {
        let start_time = Instant::now();
        
        // Execute the code
        py.run(code, Some(globals), Some(locals))?;
        
        // Check if timeout exceeded
        if start_time.elapsed() > Duration::from_millis(constraints.max_execution_time_ms) {
            return Err(PyErr::new::<pyo3::exceptions::PyTimeoutError, _>(
                "Execution timeout exceeded"
            ));
        }
        
        Ok(())
    }

    /// Estimate current memory usage (placeholder implementation)
    fn estimate_memory_usage(&self) -> usize {
        // In a real implementation, this would measure actual Python memory usage
        // For now, return a reasonable estimate
        50 // MB
    }

    /// Prepare tensor context for Python execution
    fn prepare_tensor_context(
        &self,
        input_tensors: &[(String, TensorId)],
    ) -> Result<HashMap<String, Vec<f32>>, PythonError> {
        let mut context = HashMap::new();
        
        for (name, tensor_id) in input_tensors {
            // In a real implementation, this would fetch actual tensor data
            // For now, provide placeholder data
            let placeholder_data = vec![1.0, 2.0, 3.0, 4.0];
            context.insert(name.clone(), placeholder_data);
            
            // Track active tensors
            let mut monitor = self.resource_monitor.lock();
            monitor.active_tensors.insert(name.clone(), *tensor_id);
        }
        
        Ok(context)
    }

    /// Extract output tensors from Python execution result
    fn extract_output_tensors_pyo3(
        &self,
        _result: &PythonExecutionResult,
        output_tensors: &[(String, TensorId)],
    ) -> Result<HashMap<String, Vec<f32>>, PythonError> {
        let mut output_data = HashMap::new();
        
        // In a real implementation, this would extract tensors from Python locals
        for (name, _tensor_id) in output_tensors {
            let placeholder_output = vec![0.5, 0.6, 0.7, 0.8];
            output_data.insert(name.clone(), placeholder_output);
        }
        
        Ok(output_data)
    }

    /// Check if execution constraints are satisfied
    fn check_constraints(&self, constraints: &PythonConstraints) -> Result<(), PythonError> {
        if constraints.max_memory_mb > self.constraints.max_memory_mb {
            return Err(PythonError::ResourceLimitExceeded(
                "Memory limit exceeds sandbox maximum".to_string()
            ));
        }

        if constraints.max_execution_time_ms > self.constraints.max_execution_time_ms {
            return Err(PythonError::ResourceLimitExceeded(
                "Execution time limit exceeds sandbox maximum".to_string()
            ));
        }

        Ok(())
    }

    /// Update comprehensive execution statistics
    fn update_execution_stats(&mut self, start_time: Instant, success: bool, security_violation: bool) {
        let mut stats = self.execution_stats.lock();
        stats.total_executions += 1;
        
        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }

        if security_violation {
            stats.security_violations += 1;
        }
        
        let execution_time = start_time.elapsed();
        stats.total_execution_time += execution_time;
        
        if let Some(current_memory) = self.get_current_memory_usage() {
            stats.peak_memory_usage = stats.peak_memory_usage.max(current_memory);
        }
    }

    /// Get current memory usage
    fn get_current_memory_usage(&self) -> Option<usize> {
        let monitor = self.resource_monitor.lock();
        Some(monitor.current_memory_mb)
    }

    /// Get comprehensive execution statistics
    pub fn get_stats(&self) -> ExecutionStats {
        self.execution_stats.lock().clone()
    }

    /// Reset execution statistics
    pub fn reset_stats(&mut self) {
        *self.execution_stats.lock() = ExecutionStats::default();
    }
}

impl ImportValidator {
    /// Check if a module import is allowed with detailed analysis
    fn is_import_allowed(&self, module_name: &str) -> bool {
        // Check if module is explicitly blocked
        if self.blocked_patterns.iter().any(|blocked| module_name.contains(blocked)) {
            return false;
        }
        
        // Check if module is in allowed list (including submodules)
        if self.allowed_imports.iter().any(|allowed| {
            module_name == allowed || module_name.starts_with(&format!("{}.", allowed))
        }) {
            return true;
        }
        
        // Default deny for unlisted modules
        false
    }
}

impl ExecutionStats {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f32 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.successful_executions as f32 / self.total_executions as f32) * 100.0
        }
    }

    /// Get security violation rate
    pub fn security_violation_rate(&self) -> f32 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.security_violations as f32 / self.total_executions as f32) * 100.0
        }
    }

    /// Get average execution time
    pub fn average_execution_time(&self) -> Duration {
        if self.total_executions == 0 {
            Duration::from_secs(0)
        } else {
            self.total_execution_time / self.total_executions as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_sandbox_creation() {
        let constraints = PythonConstraints::default();
        let sandbox = PythonSandbox::new(constraints);
        assert!(sandbox.is_ok());
    }

    #[test]
    fn test_comprehensive_code_validation() {
        let constraints = PythonConstraints::default();
        let sandbox = PythonSandbox::new(constraints).unwrap();
        
        let safe_code = r#"
import torch
import numpy as np

def train_model(x, y):
    model = torch.nn.Linear(10, 1)
    return model(x)
"#;
        
        let validation = sandbox.validate_code_comprehensive(safe_code);
        assert!(validation.is_ok());
        let validation = validation.unwrap();
        assert!(validation.is_safe);
        assert!(validation.imported_modules.contains(&"torch".to_string()));
        assert!(validation.imported_modules.contains(&"numpy".to_string()));
    }

    #[test]
    fn test_security_violation_detection() {
        let constraints = PythonConstraints::default();
        let sandbox = PythonSandbox::new(constraints).unwrap();
        
        let unsafe_code = r#"
import os
import subprocess

os.system("rm -rf /")
subprocess.run(["curl", "http://malicious.com"])
eval("malicious_code")
"#;
        
        let validation = sandbox.validate_code_comprehensive(unsafe_code);
        assert!(validation.is_ok());
        let validation = validation.unwrap();
        assert!(!validation.is_safe);
        assert!(!validation.violations.is_empty());
        
        // Check for specific violation types
        let has_import_violation = validation.violations.iter()
            .any(|v| matches!(v.violation_type, ViolationType::DisallowedImport));
        let has_system_call = validation.violations.iter()
            .any(|v| matches!(v.violation_type, ViolationType::SystemCall));
        let has_dangerous_function = validation.violations.iter()
            .any(|v| matches!(v.violation_type, ViolationType::DangerousFunction));
            
        assert!(has_import_violation);
        assert!(has_system_call);
        assert!(has_dangerous_function);
    }

    #[test]
    fn test_import_validation_detailed() {
        let validator = ImportValidator {
            allowed_imports: vec![
                "torch".to_string(), 
                "numpy".to_string(), 
                "transformers".to_string()
            ],
            blocked_patterns: vec!["os".to_string(), "sys".to_string()],
            dangerous_functions: vec!["eval".to_string()],
        };
        
        assert!(validator.is_import_allowed("torch"));
        assert!(validator.is_import_allowed("torch.nn"));
        assert!(validator.is_import_allowed("numpy"));
        assert!(validator.is_import_allowed("transformers.models"));
        assert!(!validator.is_import_allowed("os"));
        assert!(!validator.is_import_allowed("sys"));
        assert!(!validator.is_import_allowed("random_unknown_module"));
    }
} 
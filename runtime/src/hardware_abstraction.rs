//! Hardware Abstraction Layer
//! 
//! This module provides abstraction over different hardware backends
//! including CPU, CUDA, Metal, and WGPU for cross-platform ML acceleration.

use crate::{VmError, TensorId, enhanced_vm::HardwareBackendType};
use std::collections::HashMap;
use thiserror::Error;

/// Hardware-specific errors
#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),
    #[error("Kernel execution failed: {0}")]
    KernelExecutionFailed(String),
    #[error("Device synchronization failed: {0}")]
    SynchronizationFailed(String),
    #[error("Tensor transfer failed: {0}")]
    TensorTransferFailed(String),
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// Memory location for tensors
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryLocation {
    CPU,
    GPU(u32),
}

/// Hardware capabilities
#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub device_name: String,
    pub compute_capability: String,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub multiprocessor_count: u32,
    pub max_threads_per_block: u32,
    pub supports_fp16: bool,
    pub supports_tf32: bool,
}

/// Compute kernel for hardware execution
#[derive(Debug, Clone)]
pub struct ComputeKernel {
    pub name: String,
    pub source_code: String,
    pub entry_point: String,
    pub thread_group_size: (u32, u32, u32),
}

/// Hardware backend trait
pub trait HardwareBackend: Send + Sync {
    /// Get backend type
    fn backend_type(&self) -> HardwareBackendType;
    
    /// Get hardware capabilities
    fn capabilities(&self) -> Result<HardwareCapabilities, HardwareError>;
    
    /// Allocate memory for tensor
    fn allocate_memory(&mut self, size_bytes: usize) -> Result<*mut u8, HardwareError>;
    
    /// Free allocated memory
    fn free_memory(&mut self, ptr: *mut u8) -> Result<(), HardwareError>;
    
    /// Copy data to device
    fn copy_to_device(&mut self, src: &[u8], dst: *mut u8) -> Result<(), HardwareError>;
    
    /// Copy data from device
    fn copy_from_device(&mut self, src: *const u8, dst: &mut [u8]) -> Result<(), HardwareError>;
    
    /// Execute compute kernel
    fn execute_kernel(
        &mut self, 
        kernel: &ComputeKernel,
        buffers: &[*mut u8],
        buffer_sizes: &[usize]
    ) -> Result<(), HardwareError>;
    
    /// Synchronize device execution
    fn synchronize(&mut self) -> Result<(), HardwareError>;
    
    /// Move tensor to GPU
    fn move_to_gpu(&mut self, tensor_id: TensorId) -> Result<(), HardwareError>;
    
    /// Move tensor to CPU
    fn move_to_cpu(&mut self, tensor_id: TensorId) -> Result<(), HardwareError>;
}

/// CPU-based hardware backend
pub struct CPUBackend {
    thread_count: usize,
    memory_allocations: HashMap<*mut u8, usize>,
}

impl CPUBackend {
    pub fn new() -> Result<Self, HardwareError> {
        let thread_count = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(1);
            
        Ok(Self {
            thread_count,
            memory_allocations: HashMap::new(),
        })
    }
}

impl HardwareBackend for CPUBackend {
    fn backend_type(&self) -> HardwareBackendType {
        HardwareBackendType::CPU
    }
    
    fn capabilities(&self) -> Result<HardwareCapabilities, HardwareError> {
        Ok(HardwareCapabilities {
            device_name: format!("CPU ({} threads)", self.thread_count),
            compute_capability: "CPU".to_string(),
            total_memory_mb: 8192, // Placeholder
            available_memory_mb: 4096, // Placeholder
            multiprocessor_count: self.thread_count as u32,
            max_threads_per_block: 1,
            supports_fp16: false,
            supports_tf32: false,
        })
    }
    
    fn allocate_memory(&mut self, size_bytes: usize) -> Result<*mut u8, HardwareError> {
        let layout = std::alloc::Layout::from_size_align(size_bytes, 8)
            .map_err(|e| HardwareError::MemoryAllocationFailed(e.to_string()))?;
            
        let ptr = unsafe { std::alloc::alloc(layout) };
        
        if ptr.is_null() {
            return Err(HardwareError::MemoryAllocationFailed(
                "Failed to allocate CPU memory".to_string()
            ));
        }
        
        self.memory_allocations.insert(ptr, size_bytes);
        Ok(ptr)
    }
    
    fn free_memory(&mut self, ptr: *mut u8) -> Result<(), HardwareError> {
        if let Some(size) = self.memory_allocations.remove(&ptr) {
            let layout = std::alloc::Layout::from_size_align(size, 8)
                .map_err(|e| HardwareError::MemoryAllocationFailed(e.to_string()))?;
            unsafe { std::alloc::dealloc(ptr, layout) };
            Ok(())
        } else {
            Err(HardwareError::MemoryAllocationFailed(
                "Invalid pointer for deallocation".to_string()
            ))
        }
    }
    
    fn copy_to_device(&mut self, src: &[u8], dst: *mut u8) -> Result<(), HardwareError> {
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
        }
        Ok(())
    }
    
    fn copy_from_device(&mut self, src: *const u8, dst: &mut [u8]) -> Result<(), HardwareError> {
        unsafe {
            std::ptr::copy_nonoverlapping(src, dst.as_mut_ptr(), dst.len());
        }
        Ok(())
    }
    
    fn execute_kernel(
        &mut self, 
        kernel: &ComputeKernel,
        _buffers: &[*mut u8],
        _buffer_sizes: &[usize]
    ) -> Result<(), HardwareError> {
        // CPU "kernel" execution - would implement actual computation here
        println!("Executing CPU kernel: {}", kernel.name);
        Ok(())
    }
    
    fn synchronize(&mut self) -> Result<(), HardwareError> {
        // CPU execution is synchronous by default
        Ok(())
    }
    
    fn move_to_gpu(&mut self, _tensor_id: TensorId) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Cannot move tensor to GPU from CPU backend".to_string()
        ))
    }
    
    fn move_to_cpu(&mut self, _tensor_id: TensorId) -> Result<(), HardwareError> {
        // Tensor is already on CPU
        Ok(())
    }
}

/// CUDA-based hardware backend (placeholder)
pub struct CUDABackend {
    device_id: u32,
    context_initialized: bool,
}

impl CUDABackend {
    pub fn new(device_id: u32) -> Result<Self, HardwareError> {
        // In a real implementation, this would initialize CUDA context
        Ok(Self {
            device_id,
            context_initialized: false,
        })
    }
}

impl HardwareBackend for CUDABackend {
    fn backend_type(&self) -> HardwareBackendType {
        HardwareBackendType::CUDA
    }
    
    fn capabilities(&self) -> Result<HardwareCapabilities, HardwareError> {
        // Placeholder implementation
        Ok(HardwareCapabilities {
            device_name: format!("CUDA Device {}", self.device_id),
            compute_capability: "8.0".to_string(),
            total_memory_mb: 16384,
            available_memory_mb: 12288,
            multiprocessor_count: 108,
            max_threads_per_block: 1024,
            supports_fp16: true,
            supports_tf32: true,
        })
    }
    
    fn allocate_memory(&mut self, _size_bytes: usize) -> Result<*mut u8, HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn free_memory(&mut self, _ptr: *mut u8) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn copy_to_device(&mut self, _src: &[u8], _dst: *mut u8) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn copy_from_device(&mut self, _src: *const u8, _dst: &mut [u8]) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn execute_kernel(
        &mut self, 
        _kernel: &ComputeKernel,
        _buffers: &[*mut u8],
        _buffer_sizes: &[usize]
    ) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn synchronize(&mut self) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn move_to_gpu(&mut self, _tensor_id: TensorId) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
    
    fn move_to_cpu(&mut self, _tensor_id: TensorId) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "CUDA backend not fully implemented".to_string()
        ))
    }
}

/// Metal backend for Apple Silicon (placeholder)
pub struct MetalBackend {
    device_name: String,
}

impl MetalBackend {
    pub fn new() -> Result<Self, HardwareError> {
        Ok(Self {
            device_name: "Apple GPU".to_string(),
        })
    }
}

impl HardwareBackend for MetalBackend {
    fn backend_type(&self) -> HardwareBackendType {
        HardwareBackendType::Metal
    }
    
    fn capabilities(&self) -> Result<HardwareCapabilities, HardwareError> {
        Ok(HardwareCapabilities {
            device_name: self.device_name.clone(),
            compute_capability: "Metal 3.0".to_string(),
            total_memory_mb: 32768, // Unified memory
            available_memory_mb: 24576,
            multiprocessor_count: 32,
            max_threads_per_block: 1024,
            supports_fp16: true,
            supports_tf32: false,
        })
    }
    
    fn allocate_memory(&mut self, _size_bytes: usize) -> Result<*mut u8, HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn free_memory(&mut self, _ptr: *mut u8) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn copy_to_device(&mut self, _src: &[u8], _dst: *mut u8) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn copy_from_device(&mut self, _src: *const u8, _dst: &mut [u8]) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn execute_kernel(
        &mut self, 
        _kernel: &ComputeKernel,
        _buffers: &[*mut u8],
        _buffer_sizes: &[usize]
    ) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn synchronize(&mut self) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn move_to_gpu(&mut self, _tensor_id: TensorId) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
    
    fn move_to_cpu(&mut self, _tensor_id: TensorId) -> Result<(), HardwareError> {
        Err(HardwareError::UnsupportedOperation(
            "Metal backend not fully implemented".to_string()
        ))
    }
}

/// Create hardware backend based on type
pub fn create_backend(backend_type: &HardwareBackendType) -> Result<Box<dyn HardwareBackend>, HardwareError> {
    match backend_type {
        HardwareBackendType::Auto => {
            // Auto-select best available backend
            if is_cuda_available() {
                Ok(Box::new(CUDABackend::new(0)?))
            } else if is_metal_available() {
                Ok(Box::new(MetalBackend::new()?))
            } else {
                Ok(Box::new(CPUBackend::new()?))
            }
        }
        HardwareBackendType::CPU => Ok(Box::new(CPUBackend::new()?)),
        HardwareBackendType::CUDA => {
            if is_cuda_available() {
                Ok(Box::new(CUDABackend::new(0)?))
            } else {
                Err(HardwareError::BackendNotAvailable("CUDA not available".to_string()))
            }
        }
        HardwareBackendType::Metal => {
            if is_metal_available() {
                Ok(Box::new(MetalBackend::new()?))
            } else {
                Err(HardwareError::BackendNotAvailable("Metal not available".to_string()))
            }
        }
        HardwareBackendType::WGPU => {
            // WGPU backend would be implemented here
            Err(HardwareError::BackendNotAvailable("WGPU backend not implemented".to_string()))
        }
    }
}

/// Check if CUDA is available
fn is_cuda_available() -> bool {
    // In real implementation, this would check for CUDA runtime
    false
}

/// Check if Metal is available
fn is_metal_available() -> bool {
    // In real implementation, this would check for Metal framework
    cfg!(target_os = "macos")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_backend_creation() {
        let backend = CPUBackend::new();
        assert!(backend.is_ok());
    }

    #[test]
    fn test_backend_auto_selection() {
        let backend = create_backend(&HardwareBackendType::Auto);
        assert!(backend.is_ok());
        assert_eq!(backend.unwrap().backend_type(), HardwareBackendType::CPU);
    }

    #[test]
    fn test_cpu_capabilities() {
        let backend = CPUBackend::new().unwrap();
        let caps = backend.capabilities();
        assert!(caps.is_ok());
        let caps = caps.unwrap();
        assert!(caps.device_name.contains("CPU"));
    }

    #[test]
    fn test_memory_allocation() {
        let mut backend = CPUBackend::new().unwrap();
        let ptr = backend.allocate_memory(1024);
        assert!(ptr.is_ok());
        
        let ptr = ptr.unwrap();
        let result = backend.free_memory(ptr);
        assert!(result.is_ok());
    }
} 
//! Tensor Operations and Management
//!
//! This module provides efficient tensor storage, manipulation, and operations
//! for the enhanced VM's ML workloads using Candle for high-performance computation.

use crate::{DataType, TensorId, VmError};
use candle_core::{DType, Device, Shape, Tensor as CandleTensor};
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Unused import
use std::sync::Arc;

/// A tensor stored in the VM with Candle backend
#[derive(Debug, Clone)]
pub struct Tensor {
    pub id: TensorId,
    pub shape: Vec<usize>,
    pub dtype: DataType,
    pub candle_tensor: CandleTensor,
    pub device: Device,
    pub requires_grad: bool,
}

/// Device abstraction for tensor storage
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeviceType {
    CPU,
    CUDA(u32),
    Metal,
}

impl Tensor {
    /// Create a new tensor with given shape and data type
    pub fn new(
        id: TensorId,
        shape: Vec<usize>,
        dtype: DataType,
        device: &Device,
    ) -> Result<Self, VmError> {
        let candle_dtype = dtype.to_candle_dtype();
        let candle_shape = Shape::from(shape.as_slice());

        let candle_tensor = CandleTensor::zeros(candle_shape, candle_dtype, device)
            .map_err(|e| VmError::TensorError(format!("Failed to create tensor: {}", e)))?;

        Ok(Self { id, shape, dtype, candle_tensor, device: device.clone(), requires_grad: false })
    }

    /// Create tensor from raw data
    pub fn from_data(
        id: TensorId,
        shape: Vec<usize>,
        data: Vec<f32>,
        device: &Device,
    ) -> Result<Self, VmError> {
        let total_elements = shape.iter().product::<usize>();

        if data.len() != total_elements {
            return Err(VmError::TensorError(format!(
                "Data length {} doesn't match shape {:?} (expected {})",
                data.len(),
                shape,
                total_elements
            )));
        }

        let candle_shape = Shape::from(shape.as_slice());
        let candle_tensor = CandleTensor::from_vec(data, candle_shape, device).map_err(|e| {
            VmError::TensorError(format!("Failed to create tensor from data: {}", e))
        })?;

        Ok(Self {
            id,
            shape,
            dtype: DataType::Float32,
            candle_tensor,
            device: device.clone(),
            requires_grad: false,
        })
    }

    /// Get total number of elements
    pub fn numel(&self) -> usize {
        self.candle_tensor.elem_count()
    }

    /// Get tensor memory usage in bytes
    pub fn memory_size(&self) -> usize {
        self.numel() * self.dtype.size_bytes()
    }

    /// Convert to CPU
    pub fn to_cpu(&self) -> Result<Self, VmError> {
        if matches!(self.device, Device::Cpu) {
            return Ok(self.clone());
        }

        let cpu_tensor = self
            .candle_tensor
            .to_device(&Device::Cpu)
            .map_err(|e| VmError::TensorError(format!("Failed to move tensor to CPU: {}", e)))?;

        Ok(Self {
            id: self.id,
            shape: self.shape.clone(),
            dtype: self.dtype.clone(),
            candle_tensor: cpu_tensor,
            device: Device::Cpu,
            requires_grad: self.requires_grad,
        })
    }

    /// Convert to GPU (if available)
    pub fn to_gpu(&self, _device_id: u32) -> Result<Self, VmError> {
        #[cfg(feature = "cuda")]
        {
            let cuda_device = Device::new_cuda(device_id).map_err(|e| {
                VmError::TensorError(format!("Failed to create CUDA device: {}", e))
            })?;

            let gpu_tensor = self.candle_tensor.to_device(&cuda_device).map_err(|e| {
                VmError::TensorError(format!("Failed to move tensor to GPU: {}", e))
            })?;

            Ok(Self {
                id: self.id,
                shape: self.shape.clone(),
                dtype: self.dtype.clone(),
                candle_tensor: gpu_tensor,
                device: cuda_device,
                requires_grad: self.requires_grad,
            })
        }
        #[cfg(not(feature = "cuda"))]
        {
            Err(VmError::TensorError("CUDA support not compiled in".to_string()))
        }
    }

    /// Reshape tensor (must preserve total elements)
    pub fn reshape(&self, new_shape: Vec<usize>) -> Result<Self, VmError> {
        let old_numel = self.numel();
        let new_numel = new_shape.iter().product::<usize>();

        if old_numel != new_numel {
            return Err(VmError::TensorError(format!(
                "Cannot reshape tensor from {} to {} elements",
                old_numel, new_numel
            )));
        }

        let candle_shape = Shape::from(new_shape.as_slice());
        let reshaped_tensor = self
            .candle_tensor
            .reshape(candle_shape)
            .map_err(|e| VmError::TensorError(format!("Failed to reshape tensor: {}", e)))?;

        Ok(Self {
            id: self.id,
            shape: new_shape,
            dtype: self.dtype.clone(),
            candle_tensor: reshaped_tensor,
            device: self.device.clone(),
            requires_grad: self.requires_grad,
        })
    }

    /// Get tensor data as Vec<f32>
    pub fn to_vec(&self) -> Result<Vec<f32>, VmError> {
        let cpu_tensor = self.to_cpu()?;
        cpu_tensor
            .candle_tensor
            .to_vec1()
            .map_err(|e| VmError::TensorError(format!("Failed to convert tensor to vec: {}", e)))
    }
}

impl DataType {
    /// Convert to Candle DType
    pub fn to_candle_dtype(&self) -> DType {
        match self {
            DataType::Float32 => DType::F32,
            DataType::Float64 => DType::F64,
            DataType::Int32 => DType::I64, // Candle doesn't have I32, use I64
            DataType::Int64 => DType::I64,
            DataType::Bool => DType::U8, // Use U8 for bool
            _ => DType::F32,             // Default fallback
        }
    }
}

/// High-performance tensor manager using concurrent data structures
pub struct TensorManager {
    tensors: DashMap<TensorId, Arc<RwLock<Tensor>>>,
    max_tensors: usize,
    max_memory_bytes: usize,
    current_memory_usage: Arc<parking_lot::Mutex<usize>>,
    peak_memory_usage: Arc<parking_lot::Mutex<usize>>,
    default_device: Device,
}

impl TensorManager {
    /// Create a new tensor manager
    pub fn new(max_tensors: usize, max_memory_mb: usize) -> Self {
        let default_device = Self::get_best_device();

        Self {
            tensors: DashMap::new(),
            max_tensors,
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_usage: Arc::new(parking_lot::Mutex::new(0)),
            peak_memory_usage: Arc::new(parking_lot::Mutex::new(0)),
            default_device,
        }
    }

    /// Get the best available device
    fn get_best_device() -> Device {
        #[cfg(feature = "cuda")]
        {
            if let Ok(device) = Device::new_cuda(0) {
                return device;
            }
        }

        #[cfg(feature = "metal-gpu")]
        {
            if let Ok(device) = Device::new_metal(0) {
                return device;
            }
        }

        Device::Cpu
    }

    /// Create a new tensor
    pub fn create_tensor(
        &self,
        id: TensorId,
        shape: Vec<usize>,
        dtype: DataType,
    ) -> Result<(), VmError> {
        if self.tensors.len() >= self.max_tensors {
            return Err(VmError::ResourceLimitExceeded(format!(
                "Maximum number of tensors ({}) exceeded",
                self.max_tensors
            )));
        }

        let tensor = Tensor::new(id, shape, dtype, &self.default_device)?;
        let memory_size = tensor.memory_size();

        // Check memory limits
        {
            let mut current_usage = self.current_memory_usage.lock();
            if *current_usage + memory_size > self.max_memory_bytes {
                return Err(VmError::ResourceLimitExceeded(format!(
                    "Memory limit ({} MB) would be exceeded",
                    self.max_memory_bytes / 1024 / 1024
                )));
            }
            *current_usage += memory_size;

            let mut peak_usage = self.peak_memory_usage.lock();
            *peak_usage = (*peak_usage).max(*current_usage);
        }

        self.tensors.insert(id, Arc::new(RwLock::new(tensor)));
        Ok(())
    }

    /// Store tensor data
    pub fn store_tensor(&self, tensor: Tensor) -> Result<(), VmError> {
        if self.tensors.len() >= self.max_tensors {
            return Err(VmError::ResourceLimitExceeded(format!(
                "Maximum number of tensors ({}) exceeded",
                self.max_tensors
            )));
        }

        let memory_size = tensor.memory_size();

        // Check memory limits
        {
            let mut current_usage = self.current_memory_usage.lock();
            if *current_usage + memory_size > self.max_memory_bytes {
                return Err(VmError::ResourceLimitExceeded(format!(
                    "Memory limit ({} MB) would be exceeded",
                    self.max_memory_bytes / 1024 / 1024
                )));
            }
            *current_usage += memory_size;

            let mut peak_usage = self.peak_memory_usage.lock();
            *peak_usage = (*peak_usage).max(*current_usage);
        }

        self.tensors.insert(tensor.id, Arc::new(RwLock::new(tensor)));
        Ok(())
    }

    /// Get tensor by ID (read-only access)
    pub fn get_tensor(&self, id: TensorId) -> Result<Arc<RwLock<Tensor>>, VmError> {
        self.tensors
            .get(&id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| VmError::TensorError(format!("Tensor with ID {:?} not found", id)))
    }

    /// Destroy tensor and free memory
    pub fn destroy_tensor(&self, id: TensorId) -> Result<(), VmError> {
        if let Some((_, tensor_ref)) = self.tensors.remove(&id) {
            let tensor = tensor_ref.read();
            let memory_size = tensor.memory_size();
            drop(tensor); // Release read lock

            let mut current_usage = self.current_memory_usage.lock();
            *current_usage = current_usage.saturating_sub(memory_size);

            Ok(())
        } else {
            Err(VmError::TensorError(format!("Tensor with ID {:?} not found", id)))
        }
    }

    /// Check if tensor exists
    pub fn has_tensor(&self, id: TensorId) -> bool {
        self.tensors.contains_key(&id)
    }

    /// Get current memory usage in bytes
    pub fn current_memory_usage(&self) -> usize {
        *self.current_memory_usage.lock()
    }

    /// Get peak memory usage in bytes
    pub fn peak_memory_usage(&self) -> usize {
        *self.peak_memory_usage.lock()
    }

    /// Get number of stored tensors
    pub fn tensor_count(&self) -> usize {
        self.tensors.len()
    }

    /// Clear all tensors
    pub fn clear(&self) {
        self.tensors.clear();
        *self.current_memory_usage.lock() = 0;
    }

    /// Perform tensor addition
    pub fn tensor_add(
        &self,
        a_id: TensorId,
        b_id: TensorId,
        output_id: TensorId,
    ) -> Result<(), VmError> {
        let a_tensor = self.get_tensor(a_id)?;
        let b_tensor = self.get_tensor(b_id)?;

        let a = a_tensor.read();
        let b = b_tensor.read();

        let result = (&a.candle_tensor + &b.candle_tensor)
            .map_err(|e| VmError::TensorError(format!("Tensor addition failed: {}", e)))?;

        let output_tensor = Tensor {
            id: output_id,
            shape: a.shape.clone(),
            dtype: a.dtype.clone(),
            candle_tensor: result,
            device: a.device.clone(),
            requires_grad: a.requires_grad || b.requires_grad,
        };

        self.store_tensor(output_tensor)
    }

    /// Perform matrix multiplication
    pub fn tensor_matmul(
        &self,
        a_id: TensorId,
        b_id: TensorId,
        output_id: TensorId,
    ) -> Result<(), VmError> {
        let a_tensor = self.get_tensor(a_id)?;
        let b_tensor = self.get_tensor(b_id)?;

        let a = a_tensor.read();
        let b = b_tensor.read();

        let result = a
            .candle_tensor
            .matmul(&b.candle_tensor)
            .map_err(|e| VmError::TensorError(format!("Matrix multiplication failed: {}", e)))?;

        // Infer output shape
        let output_shape = if a.shape.len() == 2 && b.shape.len() == 2 {
            vec![a.shape[0], b.shape[1]]
        } else {
            result.shape().dims().to_vec()
        };

        let output_tensor = Tensor {
            id: output_id,
            shape: output_shape,
            dtype: a.dtype.clone(),
            candle_tensor: result,
            device: a.device.clone(),
            requires_grad: a.requires_grad || b.requires_grad,
        };

        self.store_tensor(output_tensor)
    }

    /// Apply ReLU activation
    pub fn tensor_relu(&self, input_id: TensorId, output_id: TensorId) -> Result<(), VmError> {
        let input_tensor = self.get_tensor(input_id)?;
        let input = input_tensor.read();

        let result = input
            .candle_tensor
            .relu()
            .map_err(|e| VmError::TensorError(format!("ReLU activation failed: {}", e)))?;

        let output_tensor = Tensor {
            id: output_id,
            shape: input.shape.clone(),
            dtype: input.dtype.clone(),
            candle_tensor: result,
            device: input.device.clone(),
            requires_grad: input.requires_grad,
        };

        self.store_tensor(output_tensor)
    }
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current_usage_bytes: usize,
    pub peak_usage_bytes: usize,
    pub max_memory_bytes: usize,
    pub tensor_count: usize,
    pub max_tensors: usize,
    pub device_type: String,
}

impl MemoryStats {
    /// Get current usage as percentage of limit
    pub fn usage_percentage(&self) -> f32 {
        (self.current_usage_bytes as f32 / self.max_memory_bytes as f32) * 100.0
    }

    /// Get peak usage as percentage of limit
    pub fn peak_percentage(&self) -> f32 {
        (self.peak_usage_bytes as f32 / self.max_memory_bytes as f32) * 100.0
    }
}

impl TensorManager {
    /// Get comprehensive memory statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage_bytes: self.current_memory_usage(),
            peak_usage_bytes: self.peak_memory_usage(),
            max_memory_bytes: self.max_memory_bytes,
            tensor_count: self.tensor_count(),
            max_tensors: self.max_tensors,
            device_type: match &self.default_device {
                Device::Cpu => "CPU".to_string(),
                Device::Cuda(_) => "CUDA".to_string(),
                Device::Metal(_) => "Metal".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_creation() {
        let device = Device::Cpu;
        let tensor = Tensor::new(TensorId(1), vec![2, 3], DataType::Float32, &device);
        assert!(tensor.is_ok());
        let tensor = tensor.unwrap();
        assert_eq!(tensor.numel(), 6);
        assert_eq!(tensor.shape, vec![2, 3]);
    }

    #[test]
    fn test_tensor_manager() {
        let manager = TensorManager::new(100, 1024);

        // Create tensor
        let result = manager.create_tensor(TensorId(1), vec![10, 10], DataType::Float32);
        assert!(result.is_ok());

        // Check tensor exists
        assert!(manager.has_tensor(TensorId(1)));
        assert_eq!(manager.tensor_count(), 1);

        // Destroy tensor
        let result = manager.destroy_tensor(TensorId(1));
        assert!(result.is_ok());
        assert!(!manager.has_tensor(TensorId(1)));
        assert_eq!(manager.tensor_count(), 0);
    }

    #[test]
    fn test_tensor_operations() {
        let manager = TensorManager::new(100, 1024);

        // Create test tensors
        manager.create_tensor(TensorId(1), vec![2, 2], DataType::Float32).unwrap();
        manager.create_tensor(TensorId(2), vec![2, 2], DataType::Float32).unwrap();

        // Test addition
        let result = manager.tensor_add(TensorId(1), TensorId(2), TensorId(3));
        assert!(result.is_ok());
        assert!(manager.has_tensor(TensorId(3)));
    }

    #[test]
    fn test_memory_limits() {
        let manager = TensorManager::new(1, 1); // Very small limits

        // This should fail due to memory limit
        let result = manager.create_tensor(TensorId(1), vec![1000, 1000], DataType::Float32);
        assert!(result.is_err());
    }
}

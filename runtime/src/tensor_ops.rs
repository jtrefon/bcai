//! Tensor Operations and Management
//! 
//! This module provides efficient tensor storage, manipulation, and operations
//! for the enhanced VM's ML workloads.

use crate::{VmError, TensorId, DataType};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A tensor stored in the VM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    pub id: TensorId,
    pub shape: Vec<usize>,
    pub dtype: DataType,
    pub data: TensorData,
    pub device: Device,
    pub requires_grad: bool,
}

/// Device where tensor is stored
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Device {
    CPU,
    GPU(u32), // GPU device ID
}

/// Tensor data storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TensorData {
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    Bool(Vec<bool>),
    Complex64(Vec<num_complex::Complex<f32>>),
    Complex128(Vec<num_complex::Complex<f64>>),
}

impl TensorData {
    /// Get the number of elements
    pub fn len(&self) -> usize {
        match self {
            TensorData::Float32(data) => data.len(),
            TensorData::Float64(data) => data.len(),
            TensorData::Int32(data) => data.len(),
            TensorData::Int64(data) => data.len(),
            TensorData::Bool(data) => data.len(),
            TensorData::Complex64(data) => data.len(),
            TensorData::Complex128(data) => data.len(),
        }
    }

    /// Check if tensor is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get memory size in bytes
    pub fn memory_size(&self) -> usize {
        match self {
            TensorData::Float32(data) => data.len() * 4,
            TensorData::Float64(data) => data.len() * 8,
            TensorData::Int32(data) => data.len() * 4,
            TensorData::Int64(data) => data.len() * 8,
            TensorData::Bool(data) => data.len(),
            TensorData::Complex64(data) => data.len() * 8,
            TensorData::Complex128(data) => data.len() * 16,
        }
    }
}

impl Tensor {
    /// Create a new tensor with given shape and data type
    pub fn new(id: TensorId, shape: Vec<usize>, dtype: DataType) -> Result<Self, VmError> {
        let total_elements = shape.iter().product::<usize>();
        
        let data = match dtype {
            DataType::Float32 => TensorData::Float32(vec![0.0; total_elements]),
            DataType::Float64 => TensorData::Float64(vec![0.0; total_elements]),
            DataType::Int32 => TensorData::Int32(vec![0; total_elements]),
            DataType::Int64 => TensorData::Int64(vec![0; total_elements]),
            DataType::Bool => TensorData::Bool(vec![false; total_elements]),
            DataType::Complex64 => TensorData::Complex64(vec![num_complex::Complex::new(0.0, 0.0); total_elements]),
            DataType::Complex128 => TensorData::Complex128(vec![num_complex::Complex::new(0.0, 0.0); total_elements]),
        };

        Ok(Self {
            id,
            shape,
            dtype,
            data,
            device: Device::CPU,
            requires_grad: false,
        })
    }

    /// Create tensor from raw data
    pub fn from_data(
        id: TensorId, 
        shape: Vec<usize>, 
        data: TensorData,
        device: Device
    ) -> Result<Self, VmError> {
        let total_elements = shape.iter().product::<usize>();
        
        if data.len() != total_elements {
            return Err(VmError::TensorError(format!(
                "Data length {} doesn't match shape {:?} (expected {})",
                data.len(), shape, total_elements
            )));
        }

        let dtype = match &data {
            TensorData::Float32(_) => DataType::Float32,
            TensorData::Float64(_) => DataType::Float64,
            TensorData::Int32(_) => DataType::Int32,
            TensorData::Int64(_) => DataType::Int64,
            TensorData::Bool(_) => DataType::Bool,
            TensorData::Complex64(_) => DataType::Complex64,
            TensorData::Complex128(_) => DataType::Complex128,
        };

        Ok(Self {
            id,
            shape,
            dtype,
            data,
            device,
            requires_grad: false,
        })
    }

    /// Get total number of elements
    pub fn numel(&self) -> usize {
        self.shape.iter().product()
    }

    /// Get tensor memory usage in bytes
    pub fn memory_size(&self) -> usize {
        self.data.memory_size()
    }

    /// Check if tensor shapes are compatible for broadcasting
    pub fn can_broadcast_with(&self, other: &Tensor) -> bool {
        let max_dims = self.shape.len().max(other.shape.len());
        
        for i in 0..max_dims {
            let dim1 = self.shape.get(self.shape.len().saturating_sub(i + 1)).unwrap_or(&1);
            let dim2 = other.shape.get(other.shape.len().saturating_sub(i + 1)).unwrap_or(&1);
            
            if *dim1 != *dim2 && *dim1 != 1 && *dim2 != 1 {
                return false;
            }
        }
        
        true
    }

    /// Reshape tensor (must preserve total elements)
    pub fn reshape(&mut self, new_shape: Vec<usize>) -> Result<(), VmError> {
        let old_numel = self.numel();
        let new_numel = new_shape.iter().product::<usize>();
        
        if old_numel != new_numel {
            return Err(VmError::TensorError(format!(
                "Cannot reshape tensor from {} to {} elements",
                old_numel, new_numel
            )));
        }
        
        self.shape = new_shape;
        Ok(())
    }
}

/// Tensor manager for VM execution
pub struct TensorManager {
    tensors: HashMap<TensorId, Tensor>,
    max_tensors: usize,
    max_memory_bytes: usize,
    current_memory_usage: usize,
    peak_memory_usage: usize,
}

impl TensorManager {
    /// Create a new tensor manager
    pub fn new(max_tensors: usize, max_memory_mb: usize) -> Self {
        Self {
            tensors: HashMap::new(),
            max_tensors,
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_usage: 0,
            peak_memory_usage: 0,
        }
    }

    /// Create a new tensor
    pub fn create_tensor(
        &mut self, 
        id: TensorId, 
        shape: Vec<usize>, 
        dtype: DataType
    ) -> Result<(), VmError> {
        if self.tensors.len() >= self.max_tensors {
            return Err(VmError::ResourceLimitExceeded(
                format!("Maximum number of tensors ({}) exceeded", self.max_tensors)
            ));
        }

        let tensor = Tensor::new(id, shape, dtype)?;
        let memory_size = tensor.memory_size();

        if self.current_memory_usage + memory_size > self.max_memory_bytes {
            return Err(VmError::ResourceLimitExceeded(
                format!("Memory limit ({} MB) would be exceeded", self.max_memory_bytes / 1024 / 1024)
            ));
        }

        self.current_memory_usage += memory_size;
        self.peak_memory_usage = self.peak_memory_usage.max(self.current_memory_usage);
        
        self.tensors.insert(id, tensor);
        Ok(())
    }

    /// Store tensor data
    pub fn store_tensor(&mut self, tensor: Tensor) -> Result<(), VmError> {
        if self.tensors.len() >= self.max_tensors {
            return Err(VmError::ResourceLimitExceeded(
                format!("Maximum number of tensors ({}) exceeded", self.max_tensors)
            ));
        }

        let memory_size = tensor.memory_size();
        
        if self.current_memory_usage + memory_size > self.max_memory_bytes {
            return Err(VmError::ResourceLimitExceeded(
                format!("Memory limit ({} MB) would be exceeded", self.max_memory_bytes / 1024 / 1024)
            ));
        }

        self.current_memory_usage += memory_size;
        self.peak_memory_usage = self.peak_memory_usage.max(self.current_memory_usage);
        
        self.tensors.insert(tensor.id, tensor);
        Ok(())
    }

    /// Get tensor by ID
    pub fn get_tensor(&self, id: TensorId) -> Result<&Tensor, VmError> {
        self.tensors.get(&id).ok_or_else(|| {
            VmError::TensorError(format!("Tensor with ID {:?} not found", id))
        })
    }

    /// Get mutable tensor by ID
    pub fn get_tensor_mut(&mut self, id: TensorId) -> Result<&mut Tensor, VmError> {
        self.tensors.get_mut(&id).ok_or_else(|| {
            VmError::TensorError(format!("Tensor with ID {:?} not found", id))
        })
    }

    /// Destroy tensor and free memory
    pub fn destroy_tensor(&mut self, id: TensorId) -> Result<(), VmError> {
        if let Some(tensor) = self.tensors.remove(&id) {
            self.current_memory_usage = self.current_memory_usage.saturating_sub(tensor.memory_size());
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
        self.current_memory_usage
    }

    /// Get peak memory usage in bytes
    pub fn peak_memory_usage(&self) -> usize {
        self.peak_memory_usage
    }

    /// Get number of stored tensors
    pub fn tensor_count(&self) -> usize {
        self.tensors.len()
    }

    /// Clear all tensors
    pub fn clear(&mut self) {
        self.tensors.clear();
        self.current_memory_usage = 0;
    }

    /// Get memory usage statistics
    pub fn memory_stats(&self) -> MemoryStats {
        MemoryStats {
            current_usage_bytes: self.current_memory_usage,
            peak_usage_bytes: self.peak_memory_usage,
            max_memory_bytes: self.max_memory_bytes,
            tensor_count: self.tensors.len(),
            max_tensors: self.max_tensors,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_creation() {
        let tensor = Tensor::new(TensorId(1), vec![2, 3], DataType::Float32);
        assert!(tensor.is_ok());
        let tensor = tensor.unwrap();
        assert_eq!(tensor.numel(), 6);
        assert_eq!(tensor.shape, vec![2, 3]);
    }

    #[test]
    fn test_tensor_manager() {
        let mut manager = TensorManager::new(100, 1024);
        
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
    fn test_memory_limits() {
        let mut manager = TensorManager::new(1, 1); // Very small limits
        
        // This should fail due to memory limit
        let result = manager.create_tensor(TensorId(1), vec![1000, 1000], DataType::Float32);
        assert!(result.is_err());
    }

    #[test]
    fn test_tensor_broadcast_compatibility() {
        let tensor1 = Tensor::new(TensorId(1), vec![3, 1], DataType::Float32).unwrap();
        let tensor2 = Tensor::new(TensorId(2), vec![1, 4], DataType::Float32).unwrap();
        
        assert!(tensor1.can_broadcast_with(&tensor2));
        
        let tensor3 = Tensor::new(TensorId(3), vec![3, 2], DataType::Float32).unwrap();
        assert!(!tensor1.can_broadcast_with(&tensor3));
    }

    #[test]
    fn test_tensor_reshape() {
        let mut tensor = Tensor::new(TensorId(1), vec![2, 3], DataType::Float32).unwrap();
        
        // Valid reshape
        assert!(tensor.reshape(vec![6]).is_ok());
        assert_eq!(tensor.shape, vec![6]);
        
        // Invalid reshape (different number of elements)
        assert!(tensor.reshape(vec![2, 2]).is_err());
    }
} 
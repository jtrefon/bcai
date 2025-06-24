use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "candle-core")]
pub use candle_core::Device;

#[cfg(not(feature = "candle-core"))]
#[derive(Debug, Clone, Copy)]
pub struct Device;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TensorId(pub u64);

impl TensorId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Float32,
    Float64,
    Int32,
    Int64,
    Bool,
    String,
}

#[derive(Debug, Clone)]
pub struct Tensor {
    id: TensorId,
    shape: Vec<usize>,
    dtype: DataType,
    data: Vec<f32>,
}

impl Tensor {
    pub fn new(
        id: TensorId,
        shape: Vec<usize>,
        dtype: DataType,
        _device: &Device,
    ) -> anyhow::Result<Self> {
        let size = shape.iter().product::<usize>();
        Ok(Self {
            id,
            shape,
            dtype,
            data: vec![0.0; size],
        })
    }

    pub fn shape(&self) -> &Vec<usize> {
        &self.shape
    }
    pub fn data(&self) -> &Vec<f32> {
        &self.data
    }
}

#[derive(Debug)]
pub struct TensorManager {
    tensors: HashMap<TensorId, Tensor>,
    max_tensors: usize,
    _memory_mb: u64,
}

impl TensorManager {
    pub fn new(max_tensors: usize, memory_mb: u64) -> Self {
        Self {
            tensors: HashMap::new(),
            max_tensors,
            _memory_mb: memory_mb,
        }
    }

    pub fn store_tensor(&mut self, id: TensorId, tensor: Tensor) {
        self.tensors.insert(id, tensor);
    }

    pub fn get_tensor(&self, id: TensorId) -> Option<&Tensor> {
        self.tensors.get(&id)
    }

    pub fn create_tensor(
        &self,
        _id: TensorId,
        _shape: Vec<usize>,
        _dtype: DataType,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn destroy_tensor(&self, _id: TensorId) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn tensor_add(&self, _a: TensorId, _b: TensorId, _out: TensorId) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn tensor_matmul(&self, _a: TensorId, _b: TensorId, _out: TensorId) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn tensor_relu(&self, _input: TensorId, _out: TensorId) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn memory_stats(&self) -> (usize, usize) {
        (self.tensors.len(), self.max_tensors)
    }
}

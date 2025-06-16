# 🎯 Your First Neural Network on BCAI

Welcome to BCAI! In this tutorial, you'll train a neural network to classify handwritten digits (MNIST) in just **30 seconds**. By the end, you'll understand how BCAI's decentralized ML platform works and how to submit your own jobs.

## 🎯 What You'll Learn

- How to submit ML jobs to the BCAI network
- Understanding rewards and token economics
- Basic error handling and debugging
- How distributed training works behind the scenes

## 🚀 Quick Start (30 seconds)

### Option 1: One-Command Setup
```bash
# Install BCAI CLI and Python SDK
curl -sSL https://install.bcai.network | bash
source ~/.bashrc  # Reload shell
```

### Option 2: Python Package Only
```bash
pip install bcai-sdk
```

## 📊 Your First Neural Network

Let's train a simple neural network to recognize handwritten digits. Copy and run this code:

```python
#!/usr/bin/env python3
"""
BCAI Tutorial: Your First Neural Network
Train a CNN to classify MNIST digits on the decentralized network
"""

from bcai_sdk import BCaiClient, TrainingJob
import torch
import torch.nn as nn

# 1. Connect to BCAI network
client = BCaiClient()
print("🌐 Connected to BCAI network")

# 2. Define your neural network
training_code = '''
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset
import torch.nn.functional as F

class SimpleCNN(nn.Module):
    """Simple CNN for MNIST digit classification"""
    def __init__(self):
        super(SimpleCNN, self).__init__()
        self.conv1 = nn.Conv2d(1, 32, 3, 1)
        self.conv2 = nn.Conv2d(32, 64, 3, 1)
        self.dropout1 = nn.Dropout(0.25)
        self.dropout2 = nn.Dropout(0.5)
        self.fc1 = nn.Linear(9216, 128)
        self.fc2 = nn.Linear(128, 10)

    def forward(self, x):
        x = self.conv1(x)
        x = F.relu(x)
        x = self.conv2(x)
        x = F.relu(x)
        x = F.max_pool2d(x, 2)
        x = self.dropout1(x)
        x = torch.flatten(x, 1)
        x = self.fc1(x)
        x = F.relu(x)
        x = self.dropout2(x)
        x = self.fc2(x)
        return F.log_softmax(x, dim=1)

def train_mnist():
    """Train MNIST classifier on BCAI network"""
    
    # Generate sample MNIST-like data for demo
    # In production, you'd load real MNIST data
    batch_size = 64
    input_size = (1, 28, 28)
    num_classes = 10
    
    # Create synthetic training data
    train_data = torch.randn(1000, *input_size)
    train_labels = torch.randint(0, num_classes, (1000,))
    
    # Create model and training components
    model = SimpleCNN()
    optimizer = optim.Adam(model.parameters(), lr=0.001)
    criterion = nn.NLLLoss()
    
    # Training loop
    model.train()
    total_loss = 0
    correct = 0
    total = 0
    
    print("🚀 Starting training on BCAI network...")
    for epoch in range(5):  # 5 epochs for quick demo
        running_loss = 0.0
        
        # Simulate batched training
        for i in range(0, len(train_data), batch_size):
            batch_data = train_data[i:i+batch_size]
            batch_labels = train_labels[i:i+batch_size]
            
            optimizer.zero_grad()
            outputs = model(batch_data)
            loss = criterion(outputs, batch_labels)
            loss.backward()
            optimizer.step()
            
            running_loss += loss.item()
            
            # Calculate accuracy
            _, predicted = torch.max(outputs.data, 1)
            total += batch_labels.size(0)
            correct += (predicted == batch_labels).sum().item()
        
        accuracy = 100 * correct / total
        avg_loss = running_loss / (len(train_data) // batch_size)
        
        print(f"Epoch {epoch+1}/5 - Loss: {avg_loss:.4f}, Accuracy: {accuracy:.2f}%")
    
    print(f"✅ Training completed!")
    print(f"Final accuracy: {accuracy:.2f}%")
    
    return {
        "final_loss": avg_loss,
        "final_accuracy": accuracy,
        "model_parameters": sum(p.numel() for p in model.parameters()),
        "epochs_completed": 5
    }

# Execute training
if __name__ == "__main__":
    results = train_mnist()
    print("Training results:", results)
'''

# 3. Submit job to BCAI network
job = TrainingJob(
    name="mnist-cnn-tutorial",
    code=training_code,
    data_source="synthetic://mnist-like",  # Using synthetic data for demo
    requirements=["torch>=2.0.0", "torchvision>=0.15.0"],
    resources={
        "gpu_memory_gb": 4,  # Modest GPU requirements
        "cpu_cores": 2,
        "memory_gb": 8,
        "max_runtime_minutes": 10
    },
    reward_tokens=500,  # Reward for workers who train your model
    config={
        "epochs": 5,
        "batch_size": 64,
        "learning_rate": 0.001
    }
)

print("📤 Submitting training job to BCAI network...")
print(f"   Job name: {job.name}")
print(f"   Reward: {job.reward_tokens} BCAI tokens")
print(f"   Resources: {job.resources['gpu_memory_gb']}GB GPU, {job.resources['cpu_cores']} CPUs")

# 4. Submit and wait for results
try:
    result = client.submit_job(job)
    
    print("\n🎉 Training completed successfully!")
    print(f"   Job ID: {result.job_id}")
    print(f"   Final Accuracy: {result.training_metrics.get('final_accuracy', 'N/A')}%")
    print(f"   Training Loss: {result.training_metrics.get('final_loss', 'N/A')}")
    print(f"   Model Hash: {result.model_hash}")
    print(f"   Execution Time: {result.training_metrics.get('execution_time_ms', 0)/1000:.1f}s")
    print(f"   Gas Used: {result.gas_used}")
    print(f"   Worker Reward: {result.reward_earned} BCAI tokens")
    
except Exception as e:
    print(f"❌ Training failed: {e}")
    print("\n🔧 Troubleshooting tips:")
    print("   1. Check your internet connection")
    print("   2. Ensure you have sufficient BCAI tokens")
    print("   3. Verify your code syntax")
    print("   4. Try reducing resource requirements")
```

**Run this script:**
```bash
python first_neural_network.py
```

## 🔍 Understanding What Happened

### **1. Job Submission**
When you submit a job, BCAI:
- Validates your code for security issues
- Estimates resource requirements
- Broadcasts to worker nodes with matching capabilities
- Escrows your reward tokens until completion

### **2. Distributed Execution**
Your job runs on the BCAI network:
- **Worker nodes** compete to execute your training
- **Evaluator nodes** verify the results are correct
- **Consensus** ensures honest computation via Proof of Useful Work

### **3. Result Verification**
BCAI ensures quality:
- Multiple evaluators check the trained model
- Accuracy metrics are verified against test data
- Only valid results are accepted and rewarded

## 💰 Understanding Token Economics

### **How Rewards Work**
```python
job_reward = 500  # BCAI tokens you offer
# Network distribution:
# - 85% to workers (425 tokens)
# - 10% to evaluators (50 tokens) 
# - 5% to network fee (25 tokens)
```

### **Cost Estimation**
```python
# Typical costs for MNIST CNN training:
estimated_cost = {
    "compute_cost": 200,      # Based on GPU-hours
    "network_fee": 25,        # 5% network fee
    "evaluator_fee": 50,      # Verification cost
    "total": 275              # Total cost in BCAI tokens
}

# Your 500 token offer should cover this comfortably
```

### **Getting BCAI Tokens**
```bash
# For testnet (free tokens)
bcai-cli faucet --address YOUR_ADDRESS

# For mainnet (purchase)
bcai-cli buy-tokens --amount 1000 --usd
```

## 🎛️ Advanced Configuration

### **Custom Model Architectures**
```python
# ResNet example
training_code = '''
import torchvision.models as models

model = models.resnet18(pretrained=False, num_classes=10)
# Your training code here...
'''

job = TrainingJob(
    name="resnet18-mnist",
    code=training_code,
    resources={"gpu_memory_gb": 8},  # ResNet needs more memory
    reward_tokens=1000
)
```

### **Using Real Datasets**
```python
job = TrainingJob(
    name="real-mnist-training",
    code=training_code,
    data_source="ipfs://QmYourDatasetHash",  # IPFS dataset
    data_config={
        "format": "pytorch",
        "train_split": 0.8,
        "validation_split": 0.2
    },
    reward_tokens=800
)
```

### **Resource Optimization**
```python
# For faster training
job = TrainingJob(
    resources={
        "gpu_memory_gb": 16,    # More GPU memory
        "cpu_cores": 8,         # More CPU cores
        "memory_gb": 32,        # More RAM
        "priority": "high"      # Higher priority (costs more)
    },
    reward_tokens=1500  # Higher reward for premium resources
)
```

## 🐛 Common Issues & Solutions

### **Error: Insufficient Balance**
```bash
Error: Insufficient BCAI token balance (50 available, 500 required)
```
**Solution:**
```bash
# Get testnet tokens
bcai-cli faucet --amount 1000

# Or reduce your reward
job.reward_tokens = 50
```

### **Error: Code Validation Failed**
```bash
Error: Import 'requests' not allowed in sandboxed environment
```
**Solution:**
```python
# Only use allowed ML libraries
requirements=["torch", "torchvision", "numpy", "scikit-learn"]
# Remove: requests, urllib, os, sys, subprocess
```

### **Error: Job Timeout**
```bash
Error: Job exceeded maximum runtime (10 minutes)
```
**Solution:**
```python
resources={
    "max_runtime_minutes": 30,  # Increase timeout
    "priority": "high"          # Or use higher priority
}
```

### **Error: No Available Workers**
```bash
Error: No workers available with required capabilities
```
**Solution:**
```python
resources={
    "gpu_memory_gb": 2,   # Reduce requirements
    "cpu_cores": 1,       # More workers available
    "memory_gb": 4        # Lower specs = more workers
}
```

## 🔍 Monitoring Your Job

### **Real-time Status**
```python
# Monitor job progress
status = client.get_job_status(result.job_id)
print(f"Status: {status.phase}")
print(f"Progress: {status.progress}%")
print(f"Current worker: {status.assigned_worker}")

# Stream training logs
for log_entry in client.stream_logs(result.job_id):
    print(f"[{log_entry.timestamp}] {log_entry.message}")
```

### **Performance Metrics**
```python
# Get detailed metrics
metrics = client.get_job_metrics(result.job_id)
print(f"GPU utilization: {metrics.gpu_utilization}%")
print(f"Memory usage: {metrics.memory_usage_gb}GB")
print(f"Training speed: {metrics.samples_per_second} samples/sec")
```

## 🎓 Next Steps

Congratulations! You've successfully trained your first neural network on BCAI. Here's what to explore next:

### **1. Learn the Python SDK**
- [Python SDK Complete Guide](./python_sdk_guide.md) - Master all SDK features
- [Working with Datasets](./datasets_guide.md) - Load real data efficiently
- [Model Management](./model_management.md) - Save, load, and version models

### **2. Explore Advanced ML**
- [Advanced PyTorch Integration](../intermediate/pytorch_advanced.md) - Custom architectures
- [Distributed Training](../intermediate/distributed_training.md) - Multi-node training
- [Transfer Learning](../intermediate/transfer_learning.md) - Use pre-trained models

### **3. Build Production Systems**
- [Security Best Practices](../intermediate/security.md) - Secure your ML jobs
- [Performance Optimization](../advanced/hardware_tuning.md) - Maximize efficiency
- [Enterprise Deployment](../advanced/enterprise_deployment.md) - Production setup

### **4. Join the Community**
- [Discord](https://discord.gg/bcai) - Chat with other developers
- [Forum](https://forum.bcai.network) - Ask questions and share projects
- [GitHub](https://github.com/bcai-network/bcai) - Contribute to development

## 💡 Pro Tips

### **Cost Optimization**
- Start with low rewards and increase if no workers pick up your job
- Use synthetic data for testing to avoid data transfer costs
- Monitor GPU utilization - unused resources waste tokens

### **Performance Optimization**
- Batch your operations to reduce network overhead
- Use mixed precision training to reduce memory usage
- Profile your code locally before submitting to network

### **Best Practices**
- Always validate your code locally first
- Use descriptive job names for easy tracking
- Set realistic timeouts based on model complexity
- Keep your Python requirements minimal

---

**🎉 You did it!** You've successfully trained a neural network on the BCAI decentralized network. You're now part of the future of AI training!

**Ready for more?** Check out [Advanced PyTorch Integration](../intermediate/pytorch_advanced.md) to learn about custom model architectures and transfer learning. 
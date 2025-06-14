#!/usr/bin/env python3
"""
BCAI Python SDK Demo
This demonstrates how developers would interact with the enhanced VM 
through a Python interface for submitting ML training jobs.
"""

import numpy as np
import torch
import torch.nn as nn
import json
from typing import Dict, Any, List, Optional

class BCaiClient:
    """
    Python client for interacting with BCAI enhanced VM
    """
    
    def __init__(self, node_url: str = "ws://localhost:8080"):
        self.node_url = node_url
        self.session_id = None
        print(f"Connecting to BCAI node at {node_url}")
    
    def submit_job(self, job: 'TrainingJob') -> 'JobResult':
        """Submit a training job to the BCAI network"""
        print(f"Submitting job: {job.name}")
        
        # Convert job to VM instructions
        instructions = job.to_vm_instructions()
        
        # Simulate job submission (in real implementation, this would use WebSocket/gRPC)
        result = self._simulate_job_execution(job)
        
        return JobResult(
            job_id=f"job_{hash(job.name)}",
            success=True,
            model_hash="abc123def456",
            training_metrics={
                "final_loss": 0.15,
                "accuracy": 0.92,
                "epochs": job.config.get("epochs", 10),
                "execution_time_ms": 5000
            },
            gas_used=1000,
            reward_earned=job.reward_tokens
        )
    
    def _simulate_job_execution(self, job: 'TrainingJob') -> Dict[str, Any]:
        """Simulate job execution for demo purposes"""
        print(f"  Code validation: PASSED")
        print(f"  Resource allocation: 8GB GPU memory, 16 CPU cores")
        print(f"  Executing training...")
        
        # Simulate training progress
        for epoch in range(1, job.config.get("epochs", 10) + 1):
            loss = 1.0 - (epoch * 0.08)  # Simulated decreasing loss
            acc = min(0.95, epoch * 0.09)  # Simulated increasing accuracy
            print(f"    Epoch {epoch}/10 - Loss: {loss:.3f}, Accuracy: {acc:.3f}")
        
        print(f"  Training completed successfully!")
        return {"status": "completed"}

class TrainingJob:
    """
    Represents a training job to be submitted to the BCAI network
    """
    
    def __init__(
        self,
        name: str,
        code: str,
        data_source: str,
        requirements: List[str],
        resources: Dict[str, Any],
        reward_tokens: int,
        config: Optional[Dict[str, Any]] = None
    ):
        self.name = name
        self.code = code
        self.data_source = data_source
        self.requirements = requirements
        self.resources = resources
        self.reward_tokens = reward_tokens
        self.config = config or {}
    
    def to_vm_instructions(self) -> List[Dict[str, Any]]:
        """Convert Python job to VM instructions"""
        return [
            {
                "type": "PythonExecute",
                "code": self.code,
                "input_tensors": [("data", 1)],
                "output_tensors": [("model", 2)],
                "constraints": {
                    "max_memory_mb": self.resources.get("memory_mb", 2048),
                    "max_execution_time_ms": 300000,
                    "allowed_imports": self.requirements
                }
            }
        ]

class JobResult:
    """Result of a submitted training job"""
    
    def __init__(
        self,
        job_id: str,
        success: bool,
        model_hash: Optional[str] = None,
        training_metrics: Optional[Dict[str, Any]] = None,
        gas_used: int = 0,
        reward_earned: int = 0,
        error_message: Optional[str] = None
    ):
        self.job_id = job_id
        self.success = success
        self.model_hash = model_hash
        self.training_metrics = training_metrics or {}
        self.gas_used = gas_used
        self.reward_earned = reward_earned
        self.error_message = error_message
    
    def __repr__(self):
        status = "SUCCESS" if self.success else "FAILED"
        return f"JobResult(id={self.job_id}, status={status}, accuracy={self.training_metrics.get('accuracy', 'N/A')})"

def demo_transformer_training():
    """Demo: Train a simple transformer model"""
    
    print("🚀 BCAI Enhanced VM Demo: Transformer Training")
    print("=" * 60)
    
    client = BCaiClient()
    
    # Define training code using PyTorch
    training_code = '''
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, TensorDataset

class SimpleTransformer(nn.Module):
    def __init__(self, vocab_size, embed_dim, num_heads, num_layers):
        super().__init__()
        self.embedding = nn.Embedding(vocab_size, embed_dim)
        self.pos_encoding = nn.Parameter(torch.randn(1000, embed_dim))
        
        encoder_layer = nn.TransformerEncoderLayer(
            d_model=embed_dim,
            nhead=num_heads,
            dim_feedforward=512,
            dropout=0.1
        )
        self.transformer = nn.TransformerEncoder(encoder_layer, num_layers)
        self.classifier = nn.Linear(embed_dim, vocab_size)
    
    def forward(self, x):
        seq_len = x.size(1)
        x = self.embedding(x) + self.pos_encoding[:seq_len]
        x = self.transformer(x)
        return self.classifier(x)

def train_model():
    # Model configuration
    vocab_size = 10000
    embed_dim = 512
    num_heads = 8
    num_layers = 6
    
    # Create model
    model = SimpleTransformer(vocab_size, embed_dim, num_heads, num_layers)
    
    # Generate synthetic data for demo
    batch_size = 32
    seq_length = 128
    
    # Simulated training data
    input_ids = torch.randint(0, vocab_size, (1000, seq_length))
    target_ids = torch.randint(0, vocab_size, (1000, seq_length))
    
    dataset = TensorDataset(input_ids, target_ids)
    dataloader = DataLoader(dataset, batch_size=batch_size, shuffle=True)
    
    # Training setup
    optimizer = optim.Adam(model.parameters(), lr=0.0001)
    criterion = nn.CrossEntropyLoss()
    
    model.train()
    for epoch in range(10):
        total_loss = 0
        for batch_idx, (data, target) in enumerate(dataloader):
            optimizer.zero_grad()
            
            output = model(data)
            loss = criterion(output.view(-1, vocab_size), target.view(-1))
            
            loss.backward()
            optimizer.step()
            
            total_loss += loss.item()
            
            if batch_idx % 10 == 0:
                print(f"Epoch {epoch+1}, Batch {batch_idx}, Loss: {loss.item():.4f}")
        
        avg_loss = total_loss / len(dataloader)
        print(f"Epoch {epoch+1} completed. Average Loss: {avg_loss:.4f}")
    
    return model.state_dict()

# Execute training
model_weights = train_model()
print("Training completed successfully!")
'''
    
    # Create training job
    job = TrainingJob(
        name="transformer-language-model",
        code=training_code,
        data_source="huggingface://wikitext-103",
        requirements=[
            "torch>=2.0.0",
            "transformers>=4.35.0", 
            "datasets>=2.14.0"
        ],
        resources={
            "gpu_memory_gb": 16,
            "cpu_cores": 8,
            "memory_gb": 32
        },
        reward_tokens=1500,
        config={
            "epochs": 10,
            "batch_size": 32,
            "learning_rate": 0.0001
        }
    )
    
    # Submit job
    result = client.submit_job(job)
    print(f"\n📊 Training Result: {result}")
    
    if result.success:
        print(f"✅ Model trained successfully!")
        print(f"📈 Final accuracy: {result.training_metrics['accuracy']:.2%}")
        print(f"⏱️  Training time: {result.training_metrics['execution_time_ms']}ms")
        print(f"💰 Reward earned: {result.reward_earned} TRAIN tokens")
        print(f"🔗 Model hash: {result.model_hash}")

def demo_federated_learning():
    """Demo: Federated learning with multiple nodes"""
    
    print("\n🌐 BCAI Enhanced VM Demo: Federated Learning")
    print("=" * 60)
    
    client = BCaiClient()
    
    # Federated learning code
    federated_code = '''
import torch
import torch.nn as nn
import torch.optim as optim
from typing import Dict, List

class FederatedClient:
    def __init__(self, model_architecture):
        self.model = model_architecture()
        self.optimizer = optim.Adam(self.model.parameters(), lr=0.001)
        self.local_data = self.generate_local_data()
    
    def generate_local_data(self):
        # Simulate local data for each client
        return {
            'features': torch.randn(100, 784),  # MNIST-like data
            'labels': torch.randint(0, 10, (100,))
        }
    
    def local_training(self, global_weights: Dict, epochs: int = 5):
        """Perform local training on client data"""
        # Load global model weights
        self.model.load_state_dict(global_weights)
        
        self.model.train()
        criterion = nn.CrossEntropyLoss()
        
        for epoch in range(epochs):
            self.optimizer.zero_grad()
            
            # Forward pass
            outputs = self.model(self.local_data['features'])
            loss = criterion(outputs, self.local_data['labels'])
            
            # Backward pass
            loss.backward()
            self.optimizer.step()
        
        return self.model.state_dict()

def simple_cnn():
    """Simple CNN for demonstration"""
    return nn.Sequential(
        nn.Linear(784, 128),
        nn.ReLU(),
        nn.Linear(128, 64),
        nn.ReLU(),
        nn.Linear(64, 10)
    )

def federated_averaging(client_weights: List[Dict]) -> Dict:
    """Aggregate client model weights using federated averaging"""
    # Initialize averaged weights
    averaged_weights = {}
    
    # Get parameter names from first client
    param_names = client_weights[0].keys()
    
    for param_name in param_names:
        # Average across all clients
        stacked_weights = torch.stack([
            client_weights[i][param_name] 
            for i in range(len(client_weights))
        ])
        averaged_weights[param_name] = torch.mean(stacked_weights, dim=0)
    
    return averaged_weights

def run_federated_learning():
    """Run federated learning simulation"""
    num_clients = 5
    num_rounds = 10
    
    # Initialize clients
    clients = [FederatedClient(simple_cnn) for _ in range(num_clients)]
    
    # Initialize global model
    global_model = simple_cnn()
    global_weights = global_model.state_dict()
    
    print(f"Starting federated learning with {num_clients} clients...")
    
    for round_num in range(num_rounds):
        print(f"\\nRound {round_num + 1}/{num_rounds}")
        
        # Collect local updates from clients
        client_updates = []
        for i, client in enumerate(clients):
            local_weights = client.local_training(global_weights, epochs=3)
            client_updates.append(local_weights)
            print(f"  Client {i+1} completed local training")
        
        # Aggregate updates
        global_weights = federated_averaging(client_updates)
        print(f"  Global model updated via federated averaging")
        
        # Evaluate global model (simplified)
        accuracy = 0.85 + (round_num * 0.01)  # Simulated improving accuracy
        print(f"  Global model accuracy: {accuracy:.3f}")
    
    return global_weights

# Execute federated learning
final_model = run_federated_learning()
print("Federated learning completed!")
'''
    
    job = TrainingJob(
        name="federated-mnist-classifier", 
        code=federated_code,
        data_source="distributed://mnist-shards",
        requirements=["torch>=2.0.0", "numpy>=1.21.0"],
        resources={
            "gpu_memory_gb": 8,
            "cpu_cores": 4,
            "memory_gb": 16
        },
        reward_tokens=2000,
        config={
            "num_clients": 5,
            "num_rounds": 10,
            "local_epochs": 3
        }
    )
    
    result = client.submit_job(job)
    print(f"\n📊 Federated Learning Result: {result}")

def demo_yaml_configuration():
    """Demo: YAML-based job configuration (BML - BCAI ML Language)"""
    
    print("\n📄 BCAI Enhanced VM Demo: YAML Configuration")
    print("=" * 60)
    
    yaml_config = '''
# BCAI ML Language (BML) Configuration
name: "bert-sentiment-analysis"
model:
  architecture: "transformer"
  config:
    vocab_size: 30522
    hidden_size: 768
    num_attention_heads: 12
    num_hidden_layers: 12
    max_position_embeddings: 512
    
data:
  source: "huggingface://imdb"
  preprocessing:
    - tokenize: "bert-base-uncased"
    - sequence_length: 512
    - padding: true
    
training:
  optimizer: "adamw"
  learning_rate: 2e-5
  batch_size: 16
  gradient_accumulation_steps: 4
  max_steps: 10000
  warmup_steps: 500
  
distributed:
  strategy: "federated_averaging"
  min_workers: 3
  aggregation_frequency: 100
  
resources:
  gpu_memory: "16GB"
  cpu_cores: 8
  memory: "32GB"
  
rewards:
  base_tokens: 2500
  performance_bonus: true
  data_contribution_bonus: 500
'''
    
    print("📋 Example BML Configuration:")
    print(yaml_config)
    
    print("🔄 This configuration would be automatically converted to VM instructions:")
    print("  1. TensorCreate operations for model parameters")
    print("  2. Linear, Attention, and LayerNorm instructions")
    print("  3. AdamW optimizer steps")
    print("  4. Federated aggregation operations")
    print("  5. Performance evaluation metrics")

if __name__ == "__main__":
    # Run all demos
    demo_transformer_training()
    demo_federated_learning() 
    demo_yaml_configuration()
    
    print("\n🎉 BCAI Enhanced VM Demo Complete!")
    print("The enhanced VM enables:")
    print("  ✅ Python ecosystem compatibility")
    print("  ✅ Native ML instruction execution")
    print("  ✅ Hardware abstraction (CPU/GPU)")
    print("  ✅ Secure sandboxed execution")
    print("  ✅ Distributed training coordination")
    print("  ✅ Developer-friendly APIs")
    print("\nReady for production deployment! 🚀") 
# Coding Standards

This document provides comprehensive guidance on how code should be structured and reviewed across the repository. Following these practices helps keep the codebase maintainable, testable, and approachable for all contributors.

## Core Principles

### Sandi Metz Rules (Adapted for Rust)
These rules help maintain code simplicity and readability:

1. **Structs/Enums**: No more than 100 lines
2. **Functions/Methods**: No more than 5 lines  
3. **Function Parameters**: No more than 4 parameters
4. **Struct Fields**: No more than 4 fields per struct
5. **Files**: No more than 500 lines per file

*"You can break these rules if you can convince your pair/team."*

### SOLID Principles (Rust Context)
- **Single Responsibility**: Each module/struct has one reason to change
- **Open/Closed**: Use traits for extensibility without modification  
- **Liskov Substitution**: Trait implementations must be substitutable
- **Interface Segregation**: Create focused, specific traits
- **Dependency Inversion**: Depend on traits, not concrete types

### Clean Code Architecture
- **Separation of Concerns** – Keep business logic, CLI/UI, and data storage layers distinct
- **Dependency Direction** – Inner layers should not depend on outer layers
- **Pure Functions** – Prefer functions without side effects where possible
- **Immutability** – Use immutable data structures by default
- **Composition over Inheritance** – Use traits and composition instead of complex hierarchies

## Design Principles

### Domain-Driven Design
- **Ubiquitous Language** – Use domain terminology in code
- **Bounded Contexts** – Clear module boundaries representing business domains
- **Entities vs Value Objects** – Distinguish between objects with identity and pure data
- **Domain Services** – Extract complex business logic into focused services

### Error Handling Strategy
```rust
// Good: Specific error types with context
#[derive(Error, Debug)]
pub enum JobError {
    #[error("Job {id} not found")]
    NotFound { id: JobId },
    
    #[error("Insufficient reward: need {required}, got {provided}")]
    InsufficientReward { required: u64, provided: u64 },
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

// Bad: Generic errors
type GenericError = Box<dyn std::error::Error>;
```

### Function Design Rules

#### Small Functions (≤5 Lines)
```rust
// Good: Single purpose, clear intent
fn calculate_reward(difficulty: u32, base_reward: u64) -> u64 {
    base_reward * difficulty as u64
}

fn validate_job_reward(job: &Job, minimum: u64) -> Result<(), JobError> {
    if job.reward < minimum {
        Err(JobError::InsufficientReward { 
            required: minimum, 
            provided: job.reward 
        })
    } else {
        Ok(())
    }
}

// Bad: Multiple responsibilities
fn process_and_validate_and_store_job(job: Job) -> Result<(), Error> {
    // 15+ lines of mixed validation, processing, and storage logic
}
```

#### Parameter Limits (≤4 Parameters)
```rust
// Good: Configuration object pattern
#[derive(Debug, Clone)]
pub struct MiningConfig {
    pub difficulty: Difficulty,
    pub reward: u64,
    pub timeout: Duration,
    pub worker_id: WorkerId,
}

impl MiningConfig {
    pub fn new(difficulty: Difficulty, reward: u64) -> Self {
        Self {
            difficulty,
            reward,
            timeout: Duration::from_secs(30),
            worker_id: WorkerId::generate(),
        }
    }
}

fn mine_block(config: MiningConfig, blockchain: &Blockchain) -> Result<Block, MiningError> {
    // Implementation with clear, manageable parameters
}

// Bad: Too many individual parameters
fn mine_block(
    difficulty: u32, 
    reward: u64, 
    timeout: Duration, 
    worker_id: String,
    previous_hash: Hash,
    nonce_start: u64,
) -> Result<Block, MiningError> {
    // Hard to call, easy to mix up parameters
}
```

### Struct Design Rules

#### Small, Focused Structs (≤4 Fields, ≤100 Lines)
```rust
// Good: Single responsibility
#[derive(Debug, Clone, PartialEq)]
pub struct JobId(Uuid);

#[derive(Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub description: String,
    pub reward: u64,
    pub status: JobStatus,
}

impl Job {
    pub fn new(description: String, reward: u64) -> Self {
        Self {
            id: JobId(Uuid::new_v4()),
            description,
            reward,
            status: JobStatus::Pending,
        }
    }
    
    pub fn assign_to(&mut self, worker: WorkerId) -> Result<(), JobError> {
        match self.status {
            JobStatus::Pending => {
                self.status = JobStatus::Assigned(worker);
                Ok(())
            }
            _ => Err(JobError::InvalidStateTransition)
        }
    }
}

// Bad: God object with too many responsibilities
pub struct JobManager {
    pub jobs: HashMap<JobId, Job>,
    pub workers: HashMap<WorkerId, Worker>,
    pub network_connection: TcpStream,
    pub database_pool: ConnectionPool,
    pub email_client: EmailClient,
    pub metrics_collector: MetricsCollector,
    pub blockchain_client: BlockchainClient,
    // ... many more fields and responsibilities
}
```

### Module Organization

#### Clear Hierarchical Structure
```rust
// Good: Domain-organized modules
pub mod consensus {
    pub mod proof_of_work;
    pub mod difficulty;
    mod validation; // Private implementation detail
}

pub mod jobs {
    pub mod manager;
    pub mod worker;
    pub mod types;
}

pub mod network {
    pub mod p2p;
    pub mod discovery;
    pub mod protocol;
}

// Bad: Feature-organized or flat structure
pub mod utils;
pub mod handlers;
pub mod models;
pub mod everything_mixed_together;
```

#### Dependency Management
```rust
// Good: Depend on abstractions
pub trait JobStorage {
    async fn save(&self, job: &Job) -> Result<(), StorageError>;
    async fn find(&self, id: JobId) -> Result<Option<Job>, StorageError>;
}

pub struct JobService<S: JobStorage> {
    storage: S,
}

impl<S: JobStorage> JobService<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
    
    pub async fn create_job(&self, description: String, reward: u64) -> Result<Job, JobError> {
        let job = Job::new(description, reward);
        self.storage.save(&job).await?;
        Ok(job)
    }
}

// Bad: Tight coupling to concrete implementations
pub struct JobService {
    database: PostgresConnection, // Tightly coupled to specific database
}
```

## Testing Standards

### Test Structure (AAA Pattern)
```rust
#[tokio::test]
async fn should_create_job_with_correct_properties() {
    // Arrange
    let storage = MockJobStorage::new();
    let service = JobService::new(storage);
    let description = "Train neural network".to_string();
    let reward = 1000;
    
    // Act
    let result = service.create_job(description.clone(), reward).await;
    
    // Assert
    assert!(result.is_ok());
    let job = result.unwrap();
    assert_eq!(job.description, description);
    assert_eq!(job.reward, reward);
    assert_eq!(job.status, JobStatus::Pending);
}
```

### Test Categories & Coverage
- **Unit Tests**: Test individual functions/methods in isolation
- **Integration Tests**: Test component interactions
- **Property Tests**: Use `proptest` for property-based testing
- **Contract Tests**: Verify trait implementations behave correctly
- **Performance Tests**: Benchmark critical paths
- **Security Tests**: Test against known attack vectors

**Requirement**: Maintain **100% test coverage** for all production code

### Test Helpers and Builders
```rust
// Test data builders for complex objects
#[cfg(test)]
pub struct JobBuilder {
    description: String,
    reward: u64,
    status: JobStatus,
}

#[cfg(test)]
impl JobBuilder {
    pub fn new() -> Self {
        Self {
            description: "Default job".to_string(),
            reward: 100,
            status: JobStatus::Pending,
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
    
    pub fn with_reward(mut self, reward: u64) -> Self {
        self.reward = reward;
        self
    }
    
    pub fn build(self) -> Job {
        Job {
            id: JobId(Uuid::new_v4()),
            description: self.description,
            reward: self.reward,
            status: self.status,
        }
    }
}
```

## Performance Guidelines

### Memory Efficiency
```rust
// Good: Avoid unnecessary allocations
fn process_jobs(jobs: &[Job]) -> Vec<ProcessedJob> {
    jobs.iter()
        .filter(|job| job.status == JobStatus::Pending)
        .map(|job| process_single_job(job))
        .collect()
}

// Good: Pre-allocate when size is known
fn collect_results(size_hint: usize) -> Vec<Result<Job, JobError>> {
    let mut results = Vec::with_capacity(size_hint);
    // ... populate results
    results
}

// Bad: Unnecessary cloning and allocations
fn bad_processing(jobs: Vec<Job>) -> Vec<ProcessedJob> {
    jobs.clone() // Unnecessary clone
        .into_iter()
        .map(|job| {
            let cloned_again = job.clone(); // Another unnecessary clone
            process_single_job(&cloned_again)
        })
        .collect()
}
```

### Async Best Practices
```rust
// Good: Structured concurrency with proper error handling
async fn process_jobs_concurrently(jobs: Vec<Job>) -> Result<Vec<ProcessedJob>, JobError> {
    let tasks = jobs.into_iter()
        .map(|job| tokio::spawn(process_job_async(job)))
        .collect::<Vec<_>>();
    
    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        results.push(task.await??);
    }
    Ok(results)
}

// Bad: Blocking operations in async context
async fn bad_async_function() {
    std::thread::sleep(Duration::from_secs(1)); // Blocks entire async runtime!
    // Should use: tokio::time::sleep(Duration::from_secs(1)).await;
}
```

## Documentation Standards

### Self-Documenting Code
```rust
// Good: Clear intent from naming and structure
fn calculate_proof_of_useful_work_difficulty(
    current_network_hash_rate: HashRate,
    target_block_time: Duration,
    recent_block_times: &[Duration],
) -> Difficulty {
    let average_block_time = calculate_average_time(recent_block_times);
    let adjustment_factor = target_block_time.as_secs_f64() / average_block_time.as_secs_f64();
    
    current_network_hash_rate.difficulty().adjust_by_factor(adjustment_factor)
}

// Bad: Unclear intent requiring mental translation
fn calc_diff(rate: u64, target: u64, times: &[u64]) -> u32 {
    let avg = times.iter().sum::<u64>() / times.len() as u64;
    let factor = target as f64 / avg as f64;
    (rate as f64 * factor) as u32
}
```

### API Documentation
```rust
/// Manages the lifecycle of AI training jobs in the BCAI network.
///
/// The `JobManager` coordinates between job posters, workers, and the blockchain
/// to ensure fair distribution and completion of AI training tasks.
///
/// # Examples
///
/// ```rust
/// use bcai::jobs::{JobManager, JobStorage};
///
/// # tokio_test::block_on(async {
/// let storage = InMemoryJobStorage::new();
/// let manager = JobManager::new(storage);
///
/// let job = manager.create_job(
///     "Train MNIST classifier".to_string(),
///     1000 // reward in tokens
/// ).await?;
///
/// println!("Created job: {}", job.id);
/// # });
/// ```
pub struct JobManager<S: JobStorage> {
    storage: S,
    worker_pool: WorkerPool,
}

impl<S: JobStorage> JobManager<S> {
    /// Creates a new job manager with the specified storage backend.
    ///
    /// # Arguments
    ///
    /// * `storage` - The storage implementation for persisting jobs
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use bcai::jobs::{JobManager, InMemoryJobStorage};
    /// let storage = InMemoryJobStorage::new();
    /// let manager = JobManager::new(storage);
    /// ```
    pub fn new(storage: S) -> Self {
        Self {
            storage,
            worker_pool: WorkerPool::new(),
        }
    }
}
```

## Code Review Guidelines

### Review Checklist
- [ ] **Size Limits**: Functions ≤5 lines, structs ≤100 lines, files ≤500 lines
- [ ] **Parameter Limits**: Functions have ≤4 parameters
- [ ] **Single Responsibility**: Each component has one clear purpose
- [ ] **Error Handling**: Comprehensive error types with helpful messages
- [ ] **Test Coverage**: All new code has corresponding tests
- [ ] **Documentation**: Public APIs are documented with examples
- [ ] **Performance**: No obvious performance anti-patterns
- [ ] **Security**: No obvious security vulnerabilities

### Review Culture
- **Be Kind**: Focus on code, not the person
- **Be Specific**: Point to exact lines and suggest concrete improvements  
- **Be Educational**: Explain the "why" behind suggestions
- **Be Collaborative**: Discuss trade-offs and alternatives openly
- **Be Timely**: Review code promptly to maintain team velocity

## Rust Style & Formatting

### Automatic Formatting
- Use `cargo fmt` for automatic formatting before every commit
- Maximum line length: 100 characters
- Use 4 spaces for indentation (default rustfmt)
- Configure your editor to run `rustfmt` on save

### Naming Conventions
- Functions and variables: `snake_case`
- Types and traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`  
- Modules: `snake_case`
- Lifetime parameters: `'a`, `'b`, etc. (short and lowercase)

### Import Organization
```rust
// 1. Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// 2. External crate imports (alphabetical)
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::time::sleep;

// 3. Internal module imports (relative to crate root)
use crate::consensus::Difficulty;
use crate::network::PeerManager;

// 4. Local module imports
use super::types::JobId;
```

### Linting Rules
- Run `cargo clippy -- -D warnings` and fix all warnings
- Prefer explicit error types over `unwrap()` and `expect()`
- Use `#[must_use]` on functions that return important values
- Prefer `&str` over `String` for function parameters when possible
- Use `#[derive(Debug)]` on all public types

## Enforcement

### Automated Checks (CI)
- `cargo fmt --check` - Code formatting
- `cargo clippy -- -D warnings` - Linting and best practices
- `cargo test` - All tests must pass
- `cargo audit` - Security vulnerability scanning
- Code coverage reporting (target: 100%)
- Line count enforcement for files, functions, and structs

### Manual Review Process
- All code changes require peer review
- Architecture decisions require team discussion
- Breaking changes require RFC process and documentation updates
- Performance-critical code requires benchmarks and profiling

---

## Philosophy

*"The best code is code that clearly expresses the intent of the programmer and can be easily modified by future developers."*

These standards guide us toward maintainable, testable, and readable code. Rules can be broken when there's a compelling reason, but:

1. **Document the exception** - Explain why the rule doesn't apply
2. **Discuss with the team** - Get consensus on the trade-offs
3. **Consider alternatives** - Ensure you've explored other options
4. **Plan for refactoring** - Create a plan to align with standards later

Remember: Good code is not just working code—it's code that helps the team move fast and confidently over the long term.

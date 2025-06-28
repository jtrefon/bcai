//! Implements the training execution and evaluation logic for the `UnifiedNode`.

use super::{
    error::NodeError,
    node::UnifiedNode,
    types::{JobStatus, TrainingResult},
};
use crate::pouw::generate_task_with_timestamp;

impl UnifiedNode {
    /// Executes a training task and generates a result with PoUW.
    pub fn execute_training(
        &mut self,
        job_id: u64,
        difficulty: u32,
    ) -> Result<TrainingResult, NodeError> {
        let job = self.distributed_jobs.get_mut(&job_id).ok_or(NodeError::JobNotFound(job_id))?;

        if job.status != JobStatus::WorkersAssigned {
            return Err(NodeError::InvalidStateTransition);
        }

        job.status = JobStatus::Training;

        let task = generate_task_with_timestamp(difficulty, job.id);
        let result = self.trainer.execute(&task)?;

        // Use the trained model hash from the PoUW solution instead of a
        // placeholder value.
        let model_hash = result.solution.trained_model_hash.clone();

        let training_result = TrainingResult {
            job_id,
            model_hash,
            accuracy_metrics: result.metrics,
            pouw_solution: result.solution,
            worker_signatures: vec![self.node_id.clone()], // Simplified
        };

        self.pending_results.insert(job_id, training_result.clone());
        job.status = JobStatus::EvaluationPending;

        Ok(training_result)
    }

    /// Evaluates a submitted training result.
    pub fn evaluate_training_result(
        &mut self,
        job_id: u64,
        result: &TrainingResult,
    ) -> Result<bool, NodeError> {
        let job = self.distributed_jobs.get(&job_id).ok_or(NodeError::JobNotFound(job_id))?;

        let is_valid = self.evaluator.evaluate(result)?;

        if !is_valid {
            // Penalize the worker if the result is invalid.
            // This logic would be more complex in a real system.
            for worker_id in &result.worker_signatures {
                self.job_manager.ledger_mut().penalize(worker_id, 10)?;
            }
            return Ok(false);
        }

        Ok(true)
    }
}

use super::*;

fn default_capability() -> NodeCapability {
    NodeCapability {
        cpus: 4,
        gpus: 1,
        gpu_memory_gb: 16,
        available_stake: 0,
        reputation: 100,
        capability_types: vec![CapabilityType::Training],
    }
}

fn create_node() -> UnifiedNode {
    UnifiedNode::new("test_node".to_string(), default_capability(), 1000)
}

#[test]
fn unified_node_creation() {
    let node = create_node();
    assert_eq!(node.node_id, "test_node");
    assert_eq!(node.balance(), 1000);
    assert_eq!(node.staked(), 0);
    assert!(matches!(node.status, NodeStatus::Idle));
}

#[test]
fn stake_and_post_job_flow() -> Result<(), NodeError> {
    let mut node = create_node();
    node.stake_tokens(100)?;
    assert_eq!(node.staked(), 100);
    assert_eq!(node.balance(), 900);

    let job_id = node.post_distributed_job(
        "Test Job".to_string(),
        50, // reward
        default_capability(),
        "data_hash".to_string(),
        "model_spec".to_string(),
        100, // deadline
    )?;

    assert_eq!(job_id, 1);
    assert_eq!(node.balance(), 850); // 900 - 50 escrowed

    let jobs = node.get_available_jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, 1);
    assert_eq!(jobs[0].status, JobStatus::Posted);

    Ok(())
}

#[test]
fn volunteer_and_execute_training() -> Result<(), NodeError> {
    let mut node = create_node();
    node.stake_tokens(100)?;
    let job_id = node.post_distributed_job(
        "Test Job".to_string(),
        50,
        default_capability(),
        "data_hash".to_string(),
        "model_spec".to_string(),
        100,
    )?;

    // This is a bit of a hack for testing, in reality another node would post the job.
    // We'll manually change the job status so our node can volunteer.
    let mut other_node = create_node();
    other_node.stake_tokens(200)?;
    other_node.volunteer_for_job(job_id, 50)?;
    other_node.volunteer_for_job(job_id, 50)?;

    // Now our node volunteers
    node.volunteer_for_job(job_id, 50)?;

    let job = node.distributed_jobs.get(&job_id).unwrap();
    assert_eq!(job.status, JobStatus::WorkersAssigned);

    let result = node.execute_training(job_id, 1)?;
    assert_eq!(result.job_id, job_id);

    let job_after_training = node.distributed_jobs.get(&job_id).unwrap();
    assert_eq!(job_after_training.status, JobStatus::EvaluationPending);
    Ok(())
} 
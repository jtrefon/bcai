use super::*;
use crate::large_data_transfer::descriptor::LargeDataDescriptor;
use crate::large_data_transfer::manager::ChunkManager;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn test_transfer_session_creation() {
    let session = TransferSession::new("test_hash".to_string());
    assert_eq!(session.content_hash, "test_hash");
    assert!(matches!(session.state, TransferState::Initiating));
    assert_eq!(session.progress(), 0.0);
}

#[test]
fn test_session_progress_calculation() {
    let mut session = TransferSession::new("test_hash".to_string());
    let mut descriptor = LargeDataDescriptor {
        id: "test_hash".to_string(),
        chunk_hashes: vec!["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()],
        ..Default::default()
    };
    session.descriptor = Some(descriptor);

    assert_eq!(session.progress(), 0.0);

    session.set_chunk_status(0, ChunkStatus::Complete("chunk1_id".into()));
    assert_eq!(session.progress(), 25.0);

    session.set_chunk_status(1, ChunkStatus::Complete("chunk2_id".into()));
    assert_eq!(session.progress(), 50.0);

    session.set_chunk_status(2, ChunkStatus::Downloading("peer1".to_string(), Instant::now()));
    assert_eq!(session.progress(), 50.0);

    session.set_chunk_status(3, ChunkStatus::Complete("chunk4_id".into()));
    assert_eq!(session.progress(), 75.0);
    
    session.set_chunk_status(2, ChunkStatus::Complete("chunk3_id".into()));
    assert_eq!(session.progress(), 100.0);
}

#[test]
fn test_protocol_handler() {
    let manager = ChunkManager::default();
    let mut handler = ProtocolHandler::new("node1".to_string(), Arc::new(manager));
    let descriptor = LargeDataDescriptor {
        id: "test_hash".to_string(),
        ..Default::default()
    };
    handler.start_download(descriptor).unwrap();

    let session = handler.get_session("test_hash").unwrap();
    assert!(matches!(session.state, TransferState::Active));
}

#[test]
fn test_session_timeout() {
    let mut session = TransferSession::new("test_hash".to_string());
    let timeout = Duration::from_millis(50);
    assert!(!session.is_timed_out(timeout));
    
    std::thread::sleep(Duration::from_millis(60));
    assert!(session.is_timed_out(timeout));
} 
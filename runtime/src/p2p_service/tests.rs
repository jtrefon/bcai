use super::*;
use crate::node::UnifiedNode;
use codec::WireMessage;

// Helper to create a dummy node for tests
fn create_test_node() -> UnifiedNode {
    // This is a placeholder. In a real scenario, you'd have a more robust
    // way to instantiate a node for testing.
    UnifiedNode::new(Default::default())
}

#[tokio::test]
async fn p2p_service_creation() {
    let config = P2PConfig::default();
    let (service, handle) = P2PService::new(config).await.unwrap();
    assert_eq!(service.swarm.behaviour().kademlia.kbuckets().count(), 0);
}

#[test]
fn p2p_message_serialization() {
    let msg = WireMessage::Ping;
    let serialized = serde_json::to_string(&msg).unwrap();
    let deserialized: WireMessage = serde_json::from_str(&serialized).unwrap();
    assert!(matches!(deserialized, WireMessage::Ping));
} 
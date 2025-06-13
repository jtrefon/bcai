use p2p::{Node, NodeEvent};
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn nodes_can_ping_end_to_end() {
    let mut a = Node::new(1, 0);
    let mut b = Node::new(1, 0);
    let addr = a.listen();
    b.dial(addr);

    let res = timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                e = a.next_event() => if matches!(e, NodeEvent::Ping(_)) { break },
                e = b.next_event() => if matches!(e, NodeEvent::Ping(_)) { break },
            }
        }
    })
    .await;

    assert!(res.is_ok(), "ping timeout");
}

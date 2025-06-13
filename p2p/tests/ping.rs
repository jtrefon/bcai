use p2p::Node;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn nodes_can_ping_end_to_end() {
    let mut a = Node::new();
    let mut b = Node::new();
    let addr = a.listen();
    b.dial(addr);

    let res = timeout(Duration::from_secs(5), async {
        loop {
            tokio::select! {
                e = a.next_event() => if e.result.is_ok() { break },
                e = b.next_event() => if e.result.is_ok() { break },
            }
        }
    })
    .await;

    assert!(res.is_ok(), "ping timeout");
}

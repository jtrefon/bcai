use p2p::{Node, NodeEvent};
use tokio::time::{timeout, Duration};

fn local_train(data: &[u8]) -> Vec<f32> {
    let text = std::str::from_utf8(data).expect("utf8 dataset");
    let mut floats = Vec::new();
    for line in text.lines() {
        for part in line.split(',') {
            floats.push(part.parse::<f32>().expect("parse float"));
        }
    }
    let rows = floats.len() / 6;
    let mut weights = vec![0.0f32; 5];
    for _ in 0..10 {
        let mut grads = vec![0.0f32; 5];
        for i in 0..rows {
            let start = i * 6;
            let x = &floats[start..start + 5];
            let y = floats[start + 5];
            let pred: f32 = weights.iter().zip(x).map(|(w, xi)| w * xi).sum();
            let err = pred - y;
            for j in 0..5 {
                grads[j] += err * x[j];
            }
        }
        for j in 0..5 {
            weights[j] -= 0.01 * grads[j] / rows as f32;
        }
    }
    weights
}

#[tokio::test]
async fn distributed_linear_regression() {
    let data = std::fs::read("tests/data/linear.csv").expect("read dataset");
    let mut a = Node::new(4, 1);
    let mut b = Node::new(8, 2);
    let addr = a.listen();
    b.dial(addr);

    a.send_handshake(b.peer_id);
    b.send_handshake(a.peer_id);

    let _ = timeout(Duration::from_secs(5), async {
        let mut got_a = false;
        let mut got_b = false;
        loop {
            tokio::select! {
                e = a.next_event() => match e {
                    NodeEvent::RequestResponse(ev) => if let libp2p::request_response::Event::Message { peer: _, connection_id: _, message } = ev {
                        if let libp2p::request_response::Message::Response { response, .. } = message {
                            if let p2p::JobResponse::HandshakeAck(cap) = response {
                                assert_eq!(cap, b.capability());
                                got_a = true;
                            }
                        }
                    },
                    _ => {}
                },
                e = b.next_event() => match e {
                    NodeEvent::RequestResponse(ev) => if let libp2p::request_response::Event::Message { peer: _, connection_id: _, message } = ev {
                        if let libp2p::request_response::Message::Response { response, .. } = message {
                            if let p2p::JobResponse::HandshakeAck(cap) = response {
                                assert_eq!(cap, a.capability());
                                got_b = true;
                            }
                        }
                    },
                    _ => {}
                },
            }
            if got_a && got_b { break; }
        }
    }).await;

    a.send_train(b.peer_id, data.clone());

    let weights = timeout(Duration::from_secs(10), async {
        loop {
            tokio::select! {
                e = a.next_event() => match e {
                    NodeEvent::RequestResponse(ev) => if let libp2p::request_response::Event::Message { peer:_, connection_id: _, message } = ev {
                        if let libp2p::request_response::Message::Response { response, .. } = message {
                            if let p2p::JobResponse::TrainResult(w) = response {
                                break w;
                            }
                        }
                    },
                    _ => {}
                },
                e = b.next_event() => { match e { _ => {} } },
            }
        }
    }).await.expect("train result timeout");

    let local = local_train(&data);
    for (x, y) in weights.iter().zip(local.iter()) {
        assert!((x - y).abs() < 1e-3);
    }
}

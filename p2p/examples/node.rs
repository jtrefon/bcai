use libp2p::Multiaddr;
use p2p::Node;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: node <port> [dial]");
        return;
    }
    let port: u16 = args[1].parse().expect("port");
    let mut node = Node::new_tcp(4, 1);
    let addr = node.listen_tcp(port);
    println!("listening on {addr}");
    if args.len() > 2 {
        let peer: Multiaddr = args[2].parse().expect("addr");
        node.dial(peer);
    }
    loop {
        let _ = node.next_event().await;
    }
}

use sha2::{Digest, Sha256};

pub(crate) fn calculate_content_hash(chunk_hashes: &[String]) -> String {
    let mut hasher = Sha256::new();
    for h in chunk_hashes {
        hasher.update(h.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

pub(crate) fn build_merkle_root(chunk_hashes: &[String]) -> String {
    if chunk_hashes.is_empty() {
        return String::new();
    }
    if chunk_hashes.len() == 1 {
        return chunk_hashes[0].clone();
    }
    let mut level = chunk_hashes.to_vec();
    while level.len() > 1 {
        let mut next = Vec::new();
        for pair in level.chunks(2) {
            let combined = if pair.len() == 2 {
                format!("{}{}", pair[0], pair[1])
            } else {
                format!("{}{}", pair[0], pair[0])
            };
            let hash = Sha256::digest(combined.as_bytes());
            next.push(format!("{:x}", hash));
        }
        level = next;
    }
    level[0].clone()
} 
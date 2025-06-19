use super::core::CommandHandler;
use std::error::Error;

impl CommandHandler {
    /// Retrieve basic blockchain statistics.
    pub async fn info(&self) -> Result<String, Box<dyn Error>> {
        let bc = self.blockchain.lock().await;
        if let Some(last_block) = bc.get_last_block() {
            Ok(format!(
                "Chain Info:\n  Length: {}\n  Last Block Hash: {}",
                bc.blocks.len(),
                last_block.hash
            ))
        } else {
            Ok("Chain Info:\n  The blockchain is empty.".to_string())
        }
    }
} 
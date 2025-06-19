use super::core::CommandHandler;
use std::error::Error;

impl CommandHandler {
    /// Handle account-related queries such as nonce retrieval.
    pub async fn handle_account_command(
        &self,
        account_command: crate::cli::AccountCommands,
    ) -> Result<String, Box<dyn Error>> {
        match account_command {
            crate::cli::AccountCommands::Nonce { pubkey } => {
                let bc = self.blockchain.lock().await;
                let nonce = bc.state.get_nonce(&pubkey);
                Ok(format!("{}", nonce))
            }
        }
    }
} 
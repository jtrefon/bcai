use super::core::CommandHandler;
use runtime::{
    blockchain::{validation, Transaction},
    p2p_service::WireMessage,
};
use schnorrkel::{PublicKey, SecretKey};
use std::{
    error::Error,
    fs,
    path::Path,
};
use tracing::info;
use libp2p::gossipsub::IdentTopic;

impl CommandHandler {
    /// Handle transaction-related subcommands.
    pub async fn handle_tx_command(
        &mut self,
        tx_command: crate::cli::TxCommands,
    ) -> Result<String, Box<dyn Error>> {
        match tx_command {
            crate::cli::TxCommands::Create {
                from_secret_key_file,
                to_pubkey,
                amount,
                fee,
                nonce,
            } => {
                let secret_key = self.read_secret_key(&from_secret_key_file)?;

                let to_pk_bytes = hex::decode(&to_pubkey)
                    .map_err(|_| "Invalid recipient public key hex")?;
                let to_public_key = PublicKey::from_bytes(&to_pk_bytes)
                    .map_err(|_| "Invalid recipient public key")?;

                let tx = Transaction::new_transfer(
                    &secret_key,
                    to_public_key,
                    amount,
                    fee,
                    nonce.unwrap(),
                );

                self.validate_transaction_for_mempool(&tx).await?;

                let tx_hash = tx.hash();
                self.mempool.lock().await.insert(tx.clone());

                let message = WireMessage::Transaction(tx);
                let topic = IdentTopic::new("bcai_global");
                self.p2p_handle
                    .send_message(topic, serde_json::to_vec(&message)?)
                    .await?;

                Ok(format!("Submitted transaction {} to network.", tx_hash))
            }
        }
    }

    async fn validate_transaction_for_mempool(&self, tx: &Transaction) -> Result<(), String> {
        let chain = self.blockchain.lock().await;

        validation::validate_transaction_stateless(tx)
            .and_then(|_| validation::validate_transaction_stateful(tx, &chain.state))
            .map_err(|e| e.to_string())?;

        let mempool_guard = self.mempool.lock().await;
        if mempool_guard
            .iter()
            .any(|mempool_tx| mempool_tx.hash() == tx.hash())
        {
            return Err("Transaction already in mempool".to_string());
        }

        Ok(())
    }

    fn read_secret_key(&self, path: &Path) -> Result<SecretKey, Box<dyn Error>> {
        let key_bytes = fs::read(path)?;
        SecretKey::from_bytes(&key_bytes)
            .map_err(|e| format!("Failed to create secret key from bytes: {}", e).into())
    }
} 
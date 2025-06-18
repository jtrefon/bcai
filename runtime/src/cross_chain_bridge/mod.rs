pub mod external_chain;
pub mod transactions;
pub mod error;

// Re-export commonly used types
pub use error::{BridgeError, BridgeResult};
pub use external_chain::{
    BridgeConfig, BridgeValidator, ChainId, CrossChainMessage, LiquidityPool, MessageStatus,
    MessageType,
};
pub use transactions::{
    BridgeTransaction, BridgeTransactionStatus, BridgeTransactionType, ValidatorSignature,
}; 
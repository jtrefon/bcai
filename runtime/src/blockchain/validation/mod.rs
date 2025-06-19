//! Blockchain validation utilities broken into focused sub-modules.

mod block;
mod pow;
mod transaction;

pub use block::{validate_block_structure, validate_block};
pub use pow::validate_pow_solution;
pub use transaction::{
    validate_transaction_stateless,
    validate_transaction_stateful,
    validate_transaction_with_state,
    apply_transaction_to_state,
}; 
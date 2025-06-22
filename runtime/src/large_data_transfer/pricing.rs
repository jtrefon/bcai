use super::redundancy::RedundancyPolicy;

/// Result of a price calculation for storing data on the BCAI network.
#[derive(Debug, Clone)]
pub struct PriceQuote {
    pub total_bytes: u128,
    pub redundancy: u8,
    pub price_bcai: u128,
}

/// Simple static-rate pricing model.
/// `price_per_gb_bcai` is expressed in **whole BCAI per GiB per copy**.
pub fn quote(bytes: u128, policy: RedundancyPolicy, price_per_gb_bcai: u128) -> PriceQuote {
    let copies = (policy.copies as u128).saturating_add(1); // original + copies
    let total_bytes = bytes.saturating_mul(copies);
    let gib = 1_073_741_824u128;
    let price_bcai = ((total_bytes + gib - 1) / gib) * price_per_gb_bcai;
    PriceQuote { total_bytes, redundancy: policy.copies, price_bcai }
} 
use super::descriptor::LargeDataDescriptor;

pub(crate) fn validate_descriptor(desc: &LargeDataDescriptor) -> Result<(), String> {
    if desc.chunk_hashes.len() != desc.chunk_count as usize {
        return Err("Chunk count mismatch".into());
    }
    if desc.content_hash.len() != 64 {
        return Err("Invalid content hash length".into());
    }
    Ok(())
} 
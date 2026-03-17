//! HDF5 LZ4 filter (filter ID 32004).
//!
//! The HDF5 LZ4 filter stores data as:
//! - `orig_size: u32 BE` — original uncompressed size
//! - Repeated blocks: `[block_size: u32 BE][compressed_block]`
//!
//! Each block is independently LZ4-compressed. Blocks are concatenated until
//! the total decompressed output reaches `orig_size`.

use crate::error::{Error, Result};

/// Decompress HDF5 LZ4-filtered data.
///
/// The input starts with a 4-byte big-endian original size, followed by
/// one or more `[block_size: u32 BE][compressed_block]` pairs.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 4 {
        return Err(Error::DecompressionError(
            "LZ4: input too short for header".into(),
        ));
    }

    let orig_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut output = Vec::with_capacity(orig_size);
    let mut pos = 4;

    while pos < data.len() && output.len() < orig_size {
        if pos + 4 > data.len() {
            return Err(Error::DecompressionError(
                "LZ4: truncated block header".into(),
            ));
        }
        let block_size =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;

        if pos + block_size > data.len() {
            return Err(Error::DecompressionError(
                "LZ4: truncated block data".into(),
            ));
        }

        let remaining = orig_size - output.len();
        let decompressed = lz4_flex::decompress(&data[pos..pos + block_size], remaining)
            .map_err(|e| Error::DecompressionError(format!("LZ4: {}", e)))?;
        output.extend_from_slice(&decompressed);
        pos += block_size;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lz4_round_trip() {
        let original = b"hello world hello world hello world!";

        // Build HDF5 LZ4 format: orig_size (BE) + block_size (BE) + compressed_block
        let raw_compressed = lz4_flex::compress(original);
        let mut hdf5_data = Vec::new();
        hdf5_data.extend_from_slice(&(original.len() as u32).to_be_bytes());
        hdf5_data.extend_from_slice(&(raw_compressed.len() as u32).to_be_bytes());
        hdf5_data.extend_from_slice(&raw_compressed);

        let result = decompress(&hdf5_data).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_lz4_too_short() {
        let data = &[0, 0, 0];
        assert!(decompress(data).is_err());
    }

    #[test]
    fn test_lz4_truncated_block() {
        let mut data = vec![0, 0, 0, 10]; // orig_size = 10
        data.extend_from_slice(&100u32.to_be_bytes()); // block_size = 100
        data.extend_from_slice(&[0; 5]); // only 5 bytes of "100"
        assert!(decompress(&data).is_err());
    }
}

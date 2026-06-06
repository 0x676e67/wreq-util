//! Compression algorithms for TLS 1.3 certificate compression.

use std::io::{self, Write};

use brotli::{CompressorWriter as BrotliEncoder, Decompressor as BrotliDecoder};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use wreq::tls::compress::{CertificateCompressionAlgorithm, CertificateCompressor, Codec};

#[cfg(not(any(feature = "zstd", feature = "zstd-rust")))]
compile_error!(
    "the `emulation` feature requires a zstd backend: enable `zstd` (default) or `zstd-rust`. \
     For the pure-Rust backend, use `default-features = false` with \
     `features = [\"emulation\", \"zstd-rust\"]`."
);

#[cfg(all(feature = "zstd", feature = "zstd-rust"))]
use zstd as _;

#[derive(Debug)]
pub struct BrotliCompressor;

#[derive(Debug)]
pub struct ZlibCompressor;

#[derive(Debug)]
pub struct ZstdCompressor;

impl CertificateCompressor for BrotliCompressor {
    fn compress(&self) -> Codec {
        Codec::Pointer(|input, output| {
            let mut writer = BrotliEncoder::new(output, input.len(), 11, 32);
            writer.write_all(input)?;
            writer.flush()?;
            Ok(())
        })
    }

    fn decompress(&self) -> Codec {
        Codec::Pointer(|input, output| {
            let mut reader = BrotliDecoder::new(input, 4096);
            io::copy(&mut reader, output)?;
            Ok(())
        })
    }

    fn algorithm(&self) -> CertificateCompressionAlgorithm {
        CertificateCompressionAlgorithm::BROTLI
    }
}

impl CertificateCompressor for ZlibCompressor {
    fn compress(&self) -> Codec {
        Codec::Pointer(|input, output| {
            let mut encoder = ZlibEncoder::new(output, Compression::default());
            encoder.write_all(input)?;
            encoder.finish()?;
            Ok(())
        })
    }

    fn decompress(&self) -> Codec {
        Codec::Pointer(|input, output| {
            let mut reader = ZlibDecoder::new(input);
            io::copy(&mut reader, output)?;
            Ok(())
        })
    }

    fn algorithm(&self) -> CertificateCompressionAlgorithm {
        CertificateCompressionAlgorithm::ZLIB
    }
}

impl CertificateCompressor for ZstdCompressor {
    fn compress(&self) -> Codec {
        Codec::Pointer(zstd_impl::compress)
    }

    fn decompress(&self) -> Codec {
        Codec::Pointer(zstd_impl::decompress)
    }

    fn algorithm(&self) -> CertificateCompressionAlgorithm {
        CertificateCompressionAlgorithm::ZSTD
    }
}

#[cfg(all(feature = "zstd-rust", not(feature = "zstd")))]
mod zstd_impl {
    use ruzstd::{
        decoding::StreamingDecoder,
        encoding::{self, CompressionLevel},
    };

    use super::*;

    pub(super) fn compress(input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        // `ruzstd` frame compressor panics if the drain returns an error
        let compressed = encoding::compress_to_vec(input, CompressionLevel::Fastest);
        output.write_all(&compressed)
    }

    pub(super) fn decompress(input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut decoder = StreamingDecoder::new(input).map_err(io::Error::other)?;
        io::copy(&mut decoder, output)?;
        Ok(())
    }
}

#[cfg(feature = "zstd")]
mod zstd_impl {
    use zstd::stream::{Decoder, Encoder};

    use super::*;

    pub(super) fn compress(input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut encoder = Encoder::new(output, 0)?;
        encoder.write_all(input)?;
        encoder.finish()?;
        Ok(())
    }

    pub(super) fn decompress(input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut reader = Decoder::new(input)?;
        io::copy(&mut reader, output)?;
        Ok(())
    }
}

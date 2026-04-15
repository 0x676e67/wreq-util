//! Compression algorithms for TLS 1.3 certificate compression.

use std::io::{self, Write};

use brotli::{CompressorWriter as BrotliDecoder, Decompressor as BrotliEncoder};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use wreq::tls::compress::{CertificateCompressionAlgorithm, CertificateCompressor};
use zstd::stream::{Decoder as ZstdDecoder, Encoder as ZstdEncoder};

/// Brotli compression algorithm.
#[derive(Debug)]
pub struct BrotliCompressor;

impl CertificateCompressor for BrotliCompressor {
    fn compress(&self, input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut writer = BrotliDecoder::new(output, input.len(), 11, 22);
        writer.write_all(input)?;
        writer.flush()
    }

    fn decompress(&self, input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut reader = BrotliEncoder::new(input, 4096);
        io::copy(&mut reader, output)?;
        Ok(())
    }

    #[inline]
    fn algorithm(&self) -> CertificateCompressionAlgorithm {
        CertificateCompressionAlgorithm::BROTLI
    }
}

/// Zlib compression algorithm.
#[derive(Debug)]
pub struct ZlibCompressor;

impl CertificateCompressor for ZlibCompressor {
    fn compress(&self, input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut encoder = ZlibEncoder::new(output, Compression::default());
        encoder.write_all(input)?;
        encoder.finish()?;
        Ok(())
    }

    fn decompress(&self, input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut reader = ZlibDecoder::new(input);
        io::copy(&mut reader, output)?;
        Ok(())
    }

    #[inline]
    fn algorithm(&self) -> CertificateCompressionAlgorithm {
        CertificateCompressionAlgorithm::ZLIB
    }
}

/// Zstd compression algorithm.
#[derive(Debug)]
pub struct ZstdCompressor;

impl CertificateCompressor for ZstdCompressor {
    fn compress(&self, input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut encoder = ZstdEncoder::new(output, 0)?;
        encoder.write_all(input)?;
        encoder.finish()?;
        Ok(())
    }

    fn decompress(&self, input: &[u8], output: &mut dyn io::Write) -> io::Result<()> {
        let mut reader = ZstdDecoder::new(input)?;
        io::copy(&mut reader, output)?;
        Ok(())
    }

    #[inline]
    fn algorithm(&self) -> CertificateCompressionAlgorithm {
        CertificateCompressionAlgorithm::ZSTD
    }
}

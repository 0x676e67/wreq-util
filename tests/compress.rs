#![cfg(feature = "emulation")]

use wreq::tls::compress::{CertificateCompressor, Codec};
use wreq_util::emulate::compress::{BrotliCompressor, ZlibCompressor, ZstdCompressor};

fn run(codec: Codec, input: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    match codec {
        Codec::Pointer(func) => func(input, &mut out)?,
        Codec::Dynamic(func) => func(input, &mut out)?,
    }
    Ok(out)
}

fn assert_roundtrip<C: CertificateCompressor>(compressor: &C) {
    let big = vec![b'A'; 100_000];
    let inputs: [&[u8]; 5] = [
        b"",
        b"x",
        b"hello, certificate compression",
        b"the quick brown fox jumps over the lazy dog",
        &big,
    ];
    for input in inputs {
        let compressed = run(compressor.compress(), input).expect("compress failed");
        let restored = run(compressor.decompress(), &compressed).expect("decompress failed");
        assert_eq!(
            restored,
            input,
            "roundtrip mismatch for {}-byte input",
            input.len()
        );
    }
}

#[test]
fn zstd_roundtrip() {
    assert_roundtrip(&ZstdCompressor);
}

#[test]
fn brotli_roundtrip() {
    assert_roundtrip(&BrotliCompressor);
}

#[test]
fn zlib_roundtrip() {
    assert_roundtrip(&ZlibCompressor);
}

#[test]
fn decodes_zstd_sys() {
    #[rustfmt::skip]
    const REF_ZSTD_FRAME: &[u8] = &[
        0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x58, 0x05, 0x03, 0x00, 0xe2, 0x45, 0x15,
        0x1b, 0x50, 0x4d, 0xdc, 0x63, 0xdf, 0x24, 0xb2, 0x44, 0x29, 0x56, 0xb0,
        0xc6, 0x32, 0xd6, 0xcc, 0xcc, 0x10, 0x32, 0x32, 0xb2, 0xd8, 0x35, 0xc6,
        0xcc, 0x4c, 0x08, 0x01, 0x09, 0xc4, 0xc1, 0x50, 0x20, 0x0c, 0x04, 0x01,
        0xc0, 0x1d, 0x7a, 0x3e, 0x59, 0x16, 0x69, 0xf3, 0xe9, 0x99, 0x29, 0x07,
        0xb9, 0x44, 0x1b, 0x5f, 0x8a, 0x2e, 0xe3, 0x11, 0x44, 0xae, 0x3d, 0x1f,
        0x7d, 0xc5, 0x6a, 0x63, 0xbe, 0x37, 0x67, 0x7f, 0x29, 0x62, 0xce, 0x65,
        0x4a, 0xd1, 0xf4, 0x26, 0xfe, 0x34, 0x2f, 0x4f, 0xa8, 0x9d, 0x69, 0xb9,
        0x4a, 0x02, 0x00, 0x5d, 0x34, 0x2c, 0xe2, 0x52, 0x51,
    ];
    const PLAINTEXT: &[u8] =
        b"wreq-util zstd cert-compression test vector: the quick brown fox jumps \
          over the lazy dog 0123456789 0123456789 0123456789";

    let restored =
        run(ZstdCompressor.decompress(), REF_ZSTD_FRAME).expect("decompress zstd-sys frame");
    assert_eq!(restored, PLAINTEXT);
}

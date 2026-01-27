//! Benchmarks for CAN frame operations
//!
//! Performance targets from requirements.md:
//! - Frame parse: < 500ns
//! - Frame creation: < 100ns

use busmaster_core::{CanFdFrame, CanFrame};
use busmaster_proto::{CanEncoder, CanParser};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn bench_frame_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_creation");
    group.throughput(Throughput::Elements(1));

    group.bench_function("standard_frame_8bytes", |b| {
        b.iter(|| black_box(CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap()))
    });

    group.bench_function("extended_frame_8bytes", |b| {
        b.iter(|| black_box(CanFrame::new_extended(0x12345678, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap()))
    });

    group.bench_function("canfd_frame_64bytes", |b| {
        let data = [0u8; 64];
        b.iter(|| black_box(CanFdFrame::new(0x123, false, &data).unwrap()))
    });

    group.finish();
}

fn bench_frame_access(c: &mut Criterion) {
    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    let mut group = c.benchmark_group("frame_access");
    group.throughput(Throughput::Elements(1));

    group.bench_function("get_id", |b| b.iter(|| black_box(frame.id())));

    group.bench_function("get_data", |b| b.iter(|| black_box(frame.data())));

    group.bench_function("get_dlc", |b| b.iter(|| black_box(frame.dlc())));

    group.bench_function("is_extended", |b| b.iter(|| black_box(frame.is_extended())));

    group.finish();
}

fn bench_frame_clone(c: &mut Criterion) {
    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let fd_frame = CanFdFrame::new(0x123, false, &[0u8; 64]).unwrap();

    let mut group = c.benchmark_group("frame_clone");
    group.throughput(Throughput::Elements(1));

    group.bench_function("can_frame", |b| b.iter(|| black_box(frame.clone())));

    group.bench_function("canfd_frame", |b| b.iter(|| black_box(fd_frame.clone())));

    group.finish();
}

fn bench_frame_parsing(c: &mut Criterion) {
    // Pre-encode frames for parsing benchmarks
    let frame_8 = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let bytes_8 = CanEncoder::encode(&frame_8);

    let frame_0 = CanFrame::new_standard(0x7FF, &[]).unwrap();
    let bytes_0 = CanEncoder::encode(&frame_0);

    let frame_ext = CanFrame::new_extended(0x1FFFFFFF, &[0xAA; 8]).unwrap();
    let bytes_ext = CanEncoder::encode(&frame_ext);

    let frame_fd = CanFdFrame::new(0x123, false, &[0u8; 64]).unwrap();
    let bytes_fd = CanEncoder::encode_fd(&frame_fd);

    let mut group = c.benchmark_group("frame_parsing");
    group.throughput(Throughput::Elements(1));

    group.bench_function("parse_standard_8bytes", |b| {
        b.iter(|| black_box(CanParser::parse(&bytes_8).unwrap()))
    });

    group.bench_function("parse_standard_0bytes", |b| {
        b.iter(|| black_box(CanParser::parse(&bytes_0).unwrap()))
    });

    group.bench_function("parse_extended_8bytes", |b| {
        b.iter(|| black_box(CanParser::parse(&bytes_ext).unwrap()))
    });

    group.bench_function("parse_fd_64bytes", |b| {
        b.iter(|| black_box(CanParser::parse_fd(&bytes_fd).unwrap()))
    });

    group.bench_function("validate_standard", |b| {
        b.iter(|| black_box(CanParser::validate(&bytes_8).unwrap()))
    });

    group.finish();
}

fn bench_frame_encoding(c: &mut Criterion) {
    let frame_8 = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let frame_0 = CanFrame::new_standard(0x7FF, &[]).unwrap();
    let frame_ext = CanFrame::new_extended(0x1FFFFFFF, &[0xAA; 8]).unwrap();
    let frame_fd = CanFdFrame::new(0x123, false, &[0u8; 64]).unwrap();

    let mut group = c.benchmark_group("frame_encoding");
    group.throughput(Throughput::Elements(1));

    group.bench_function("encode_standard_8bytes", |b| {
        b.iter(|| black_box(CanEncoder::encode(&frame_8)))
    });

    group.bench_function("encode_standard_0bytes", |b| {
        b.iter(|| black_box(CanEncoder::encode(&frame_0)))
    });

    group.bench_function("encode_extended_8bytes", |b| {
        b.iter(|| black_box(CanEncoder::encode(&frame_ext)))
    });

    group.bench_function("encode_fd_64bytes", |b| {
        b.iter(|| black_box(CanEncoder::encode_fd(&frame_fd)))
    });

    group.finish();
}

fn bench_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_roundtrip");
    group.throughput(Throughput::Elements(1));

    group.bench_function("standard_8bytes", |b| {
        b.iter(|| {
            let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
            let bytes = CanEncoder::encode(&frame);
            black_box(CanParser::parse(&bytes).unwrap())
        })
    });

    group.bench_function("fd_64bytes", |b| {
        b.iter(|| {
            let frame = CanFdFrame::new(0x123, false, &[0u8; 64]).unwrap();
            let bytes = CanEncoder::encode_fd(&frame);
            black_box(CanParser::parse_fd(&bytes).unwrap())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_frame_creation,
    bench_frame_access,
    bench_frame_clone,
    bench_frame_parsing,
    bench_frame_encoding,
    bench_roundtrip,
    bench_signal_extraction
);
criterion_main!(benches);

fn bench_signal_extraction(c: &mut Criterion) {
    use busmaster_core::{ByteOrder, SignalDef, ValueType};

    let data = [0x10, 0x27, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00];

    let mut group = c.benchmark_group("signal_extraction");
    group.throughput(Throughput::Elements(1));

    // 8-bit unsigned signal
    let sig_8bit = SignalDef::new("Test8", 0, 8)
        .with_byte_order(ByteOrder::LittleEndian)
        .with_value_type(ValueType::Unsigned);

    group.bench_function("extract_8bit_unsigned", |b| {
        b.iter(|| black_box(sig_8bit.extract(&data).unwrap()))
    });

    // 16-bit unsigned signal with factor/offset
    let sig_16bit = SignalDef::new("Speed", 0, 16)
        .with_byte_order(ByteOrder::LittleEndian)
        .with_value_type(ValueType::Unsigned)
        .with_factor_offset(0.1, 0.0);

    group.bench_function("extract_16bit_with_scaling", |b| {
        b.iter(|| black_box(sig_16bit.extract(&data).unwrap()))
    });

    // Signed signal
    let sig_signed = SignalDef::new("Temp", 16, 8)
        .with_byte_order(ByteOrder::LittleEndian)
        .with_value_type(ValueType::Signed)
        .with_factor_offset(1.0, -40.0);

    group.bench_function("extract_signed_with_offset", |b| {
        b.iter(|| black_box(sig_signed.extract(&data).unwrap()))
    });

    // Big-endian signal
    let sig_be = SignalDef::new("TestBE", 0, 16)
        .with_byte_order(ByteOrder::BigEndian)
        .with_value_type(ValueType::Unsigned);

    group.bench_function("extract_16bit_big_endian", |b| {
        b.iter(|| black_box(sig_be.extract(&data).unwrap()))
    });

    // Cross-byte boundary signal
    let sig_cross = SignalDef::new("CrossByte", 4, 16)
        .with_byte_order(ByteOrder::LittleEndian)
        .with_value_type(ValueType::Unsigned);

    group.bench_function("extract_cross_byte_boundary", |b| {
        b.iter(|| black_box(sig_cross.extract(&data).unwrap()))
    });

    group.finish();
}

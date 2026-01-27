use busmaster_core::{CanFrame, Direction, FilterMode, FilterRule, MessageFilter};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_id_range_filter(c: &mut Criterion) {
    let filter = MessageFilter::new().add_rule(FilterRule::IdRange {
        start: 0x100,
        end: 0x1FF,
    });

    let frame = CanFrame::new_standard(0x150, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_id_range", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

fn bench_id_mask_filter(c: &mut Criterion) {
    let filter = MessageFilter::new().add_rule(FilterRule::IdMask {
        id: 0x100,
        mask: 0x7FC,
    });

    let frame = CanFrame::new_standard(0x101, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_id_mask", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

fn bench_id_list_filter_small(c: &mut Criterion) {
    let filter = MessageFilter::new().add_rule(FilterRule::IdList {
        ids: vec![0x100, 0x200, 0x300, 0x400, 0x500],
    });

    let frame = CanFrame::new_standard(0x300, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_id_list_small", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

fn bench_id_list_filter_large(c: &mut Criterion) {
    let ids: Vec<u32> = (0..1000).collect();
    let filter = MessageFilter::new().add_rule(FilterRule::IdList { ids });

    let frame = CanFrame::new_standard(0x1F5, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_id_list_large", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

fn bench_channel_filter(c: &mut Criterion) {
    let filter = MessageFilter::new().add_rule(FilterRule::Channel { channel: 0 });

    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_channel", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

fn bench_direction_filter(c: &mut Criterion) {
    let filter = MessageFilter::new().add_rule(FilterRule::Direction {
        direction: Direction::Rx,
    });

    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_direction", |b| {
        b.iter(|| {
            black_box(filter.matches_with_direction(
                black_box(&frame),
                black_box(0),
                black_box(Direction::Rx),
            ));
        });
    });
}

fn bench_multiple_rules_any(c: &mut Criterion) {
    let filter = MessageFilter::new()
        .with_mode(FilterMode::Any)
        .add_rule(FilterRule::IdRange {
            start: 0x100,
            end: 0x1FF,
        })
        .add_rule(FilterRule::IdRange {
            start: 0x200,
            end: 0x2FF,
        })
        .add_rule(FilterRule::IdRange {
            start: 0x300,
            end: 0x3FF,
        });

    let frame = CanFrame::new_standard(0x250, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_multiple_rules_any", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

fn bench_multiple_rules_all(c: &mut Criterion) {
    let filter = MessageFilter::new()
        .with_mode(FilterMode::All)
        .add_rule(FilterRule::IdRange {
            start: 0x100,
            end: 0x1FF,
        })
        .add_rule(FilterRule::Channel { channel: 0 })
        .add_rule(FilterRule::Direction {
            direction: Direction::Rx,
        });

    let frame = CanFrame::new_standard(0x150, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_multiple_rules_all", |b| {
        b.iter(|| {
            black_box(filter.matches_with_direction(
                black_box(&frame),
                black_box(0),
                black_box(Direction::Rx),
            ));
        });
    });
}

fn bench_empty_filter(c: &mut Criterion) {
    let filter = MessageFilter::new();
    let frame = CanFrame::new_standard(0x123, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

    c.bench_function("filter_empty", |b| {
        b.iter(|| {
            black_box(filter.matches(black_box(&frame), black_box(0)));
        });
    });
}

criterion_group!(
    benches,
    bench_id_range_filter,
    bench_id_mask_filter,
    bench_id_list_filter_small,
    bench_id_list_filter_large,
    bench_channel_filter,
    bench_direction_filter,
    bench_multiple_rules_any,
    bench_multiple_rules_all,
    bench_empty_filter
);
criterion_main!(benches);

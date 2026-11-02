use criterion::{criterion_group, criterion_main, Criterion};
use fluent_primitives::{Button, TerminalGrid, TextInput};
use fluent_test_utils::RenderAudit;
use std::hint::black_box;

fn button_grid(c: &mut Criterion) {
    c.bench_function("button_grid_10x10", |b| {
        b.iter(|| {
            let audit = RenderAudit::default();
            for index in 0..100 {
                audit.element_built();
                black_box(Button::new(format!("button-{index}")));
            }
            audit.notified();
            black_box((audit.element_builds(), audit.notify_calls()));
        })
    });
}

fn text_input(c: &mut Criterion) {
    c.bench_function("text_input_interaction", |b| {
        b.iter(|| {
            let audit = RenderAudit::default();
            audit.element_built();
            black_box(TextInput::new().value("representative input"));
            audit.notified();
            black_box((audit.element_builds(), audit.notify_calls()));
        })
    });
}

fn terminal_view_model(c: &mut Criterion) {
    c.bench_function("terminal_view_200x50", |b| {
        b.iter(|| {
            let audit = RenderAudit::default();
            audit.element_built();
            black_box(TerminalGrid::new(200, 50));
            audit.notified();
            black_box((audit.element_builds(), audit.notify_calls()));
        })
    });
}

criterion_group!(render_cost, button_grid, text_input, terminal_view_model);
criterion_main!(render_cost);

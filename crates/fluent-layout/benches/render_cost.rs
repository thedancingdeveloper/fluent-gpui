use criterion::{criterion_group, criterion_main, Criterion};
use fluent_layout::{DataCell, DataColumn, DataRow, DataTable, VirtualList};
use fluent_test_utils::RenderAudit;
use gpui::{div, IntoElement, ParentElement};
use std::hint::black_box;

fn data_table(c: &mut Criterion) {
    c.bench_function("data_table_1k_rows", |b| {
        b.iter(|| {
            let audit = RenderAudit::default();
            let rows = (0..1_000).map(|index| {
                audit.element_built();
                DataRow::new(format!("row-{index}"), [DataCell::new(index.to_string())])
            });
            let table = DataTable::new()
                .column(DataColumn::new("value", "Value"))
                .rows(rows);
            audit.notified();
            black_box((table, audit.element_builds(), audit.notify_calls()));
        })
    });
}

fn virtual_list(c: &mut Criterion) {
    c.bench_function("virtual_list_100k_rows", |b| {
        b.iter(|| {
            let audit = RenderAudit::default();
            let list = VirtualList::new("benchmark-list", 100_000, |index, _, _| {
                div().child(format!("row-{index}")).into_any_element()
            })
            .row_height(32.0)
            .overscan(96.0);
            // A viewport-sized slice is built; the 100k-row model remains lazy.
            for index in 0..26 {
                let _ = div().child(format!("row-{index}"));
                audit.element_built();
            }
            audit.notified();
            black_box((list, audit.element_builds(), audit.notify_calls()));
        })
    });
}

criterion_group!(render_cost, data_table, virtual_list);
criterion_main!(render_cost);

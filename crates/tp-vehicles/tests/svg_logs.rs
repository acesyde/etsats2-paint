//! Reading SVG templates with texts logs no font warnings: their size is
//! all the package reader needs.

use std::sync::atomic::{AtomicUsize, Ordering};

static WARNINGS: AtomicUsize = AtomicUsize::new(0);

struct Counter;

impl log::Log for Counter {
    fn enabled(&self, _: &log::Metadata<'_>) -> bool {
        true
    }
    fn log(&self, record: &log::Record<'_>) {
        if record.level() <= log::Level::Warn {
            WARNINGS.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn flush(&self) {}
}

#[test]
fn svg_templates_with_text_log_no_warning() {
    log::set_logger(&Counter).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/vehicles/community.truckpaint.sample_truck-1.1.0.tpv"
    );
    let package = tp_vehicles::Package::read(&std::fs::read(path).unwrap()).unwrap();
    let cabin = package.template("standard").unwrap();
    assert_eq!((cabin.width, cabin.height), (4096.0, 4096.0));
    assert_eq!(WARNINGS.load(Ordering::SeqCst), 0);
}

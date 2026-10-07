//! The committed sample packages match their sources.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/vehicles")
}

/// Entry names and uncompressed bytes, ignoring the ZIP framing.
fn contents(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).expect("zip");
    (0..zip.len())
        .map(|i| {
            let mut entry = zip.by_index(i).expect("entry");
            let mut data = Vec::new();
            entry.read_to_end(&mut data).expect("read");
            (entry.name().to_owned(), data)
        })
        .collect()
}

/// Every `<sample>/<version>` source folder, sorted.
fn sources() -> Vec<String> {
    let mut out = Vec::new();
    for sample in std::fs::read_dir(examples()).expect("examples").flatten() {
        if !sample.path().is_dir() {
            continue;
        }
        for version in std::fs::read_dir(sample.path()).expect("sample").flatten() {
            if version.path().is_dir() {
                out.push(format!(
                    "{}/{}",
                    sample.file_name().to_string_lossy(),
                    version.file_name().to_string_lossy()
                ));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn sample_packages_are_up_to_date() {
    let sources = sources();
    assert_eq!(
        sources,
        [
            "sample-trailer/1.0.0",
            "sample-truck/1.0.0",
            "sample-truck/1.1.0"
        ]
    );
    for source in sources {
        let packed = tp_pack::pack_folder(&examples().join(&source))
            .unwrap_or_else(|e| panic!("sample {source} does not pack: {e}"));
        assert!(packed.ignored.is_empty(), "{:?}", packed.ignored);
        let name = packed.file_name();
        let committed = std::fs::read(examples().join(&name))
            .unwrap_or_else(|e| panic!("{name}: {e}; run `mise run sample-vehicles`"));
        assert!(
            contents(&committed) == contents(&packed.bytes),
            "examples/vehicles/{name} is out of date with its sources: run `mise run sample-vehicles`"
        );
    }
}

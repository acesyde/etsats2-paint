//! Metadata retained from the user-validated probes, without game geometry.

use serde_json::Value;

#[test]
fn historical_probe_coverage_is_preserved() {
    let baseline: Value = serde_json::from_str(include_str!(
        "fixtures/game-extraction/validated-baseline.json"
    ))
    .unwrap();
    for (vehicle, cabins, accessories) in [
        ("man.tgx_2020", &["gx", "gm", "gn"][..], 27),
        ("daf.2021", &["xg_plus", "xg", "xf"][..], 12),
    ] {
        let entry = baseline["vehicles"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["vehicle"] == vehicle)
            .unwrap();
        assert_eq!(entry["game_version"], "1.61.1.1");
        let parts = entry["parts"].as_array().unwrap();
        assert_eq!(parts.len(), cabins.len() + accessories);
        for cabin in cabins {
            let part = parts.iter().find(|part| part["id"] == *cabin).unwrap();
            assert_eq!(part["size"], serde_json::json!([4096, 4096]));
            assert_eq!(part["game_ids"], serde_json::json!([cabin]));
        }
        assert!(parts.iter().all(|part| part["edges"].as_u64().unwrap() > 0));
    }
}

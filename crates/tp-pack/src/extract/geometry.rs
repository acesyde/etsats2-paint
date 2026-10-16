use std::collections::{BTreeMap, BTreeSet};

use super::syntax::{Section, parse};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[usize; 3]>,
    pub shader: String,
    pub alt_uv: bool,
}

fn paint_shader(effect: &str) -> Result<Option<bool>, String> {
    // Alpha-test suffixes change opacity, not paint UV bindings.
    let family = effect
        .strip_suffix(".asafew.a")
        .or_else(|| effect.strip_suffix(".a"))
        .unwrap_or(effect);
    let Some(flavors) = family.strip_prefix("eut2.truckpaint") else {
        return Ok(None);
    };
    let mut groups = BTreeSet::new();
    if !flavors.is_empty() {
        let Some(flavors) = flavors.strip_prefix('.') else {
            return Err(format!("unsupported paint shader {effect}"));
        };
        for flavor in flavors.split('.') {
            let group = match flavor {
                "tsnmap" | "tsnmapuv" => "normal",
                "airbrush" | "flipflake" | "altuv" => flavor,
                _ => return Err(format!("unsupported paint shader {effect}")),
            };
            if !groups.insert(group) {
                return Err(format!("unsupported paint shader {effect}"));
            }
        }
    }
    Ok(Some(groups.contains("altuv")))
}

fn only<T>(items: impl IntoIterator<Item = T>, problem: &str) -> Result<T, String> {
    let mut items = items.into_iter();
    match (items.next(), items.next()) {
        (Some(item), None) => Ok(item),
        _ => Err(problem.to_owned()),
    }
}

fn selected<'a>(tree: &'a Section, kind: &str, name: &str) -> Result<&'a Section, String> {
    let matches = tree.named(kind).filter(|section| {
        section
            .field("Name")
            .is_ok_and(|n| n.eq_ignore_ascii_case(name))
    });
    only(matches, &format!("missing or ambiguous {kind}: {name}"))
}

fn integer(section: &Section, field: &str) -> Result<usize, String> {
    section
        .field(field)?
        .parse()
        .map_err(|_| format!("invalid {field}"))
}

fn row_values<const N: usize, T>(
    section: &Section,
    parse_value: impl Fn(&str) -> Result<T, String>,
) -> Result<Vec<[T; N]>, String> {
    section
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let (number, values) = row
                .split_once('(')
                .ok_or_else(|| format!("invalid row {row}"))?;
            if number.trim().parse::<usize>().ok() != Some(index) {
                return Err(format!("unexpected row index: {row}"));
            }
            let values = values.trim().strip_suffix(')').ok_or("unclosed row")?;
            let values = values
                .split_whitespace()
                .map(&parse_value)
                .collect::<Result<Vec<_>, _>>()?;
            values
                .try_into()
                .map_err(|_| format!("expected {N} values in {row}"))
        })
        .collect()
}

fn number(value: &str) -> Result<f64, String> {
    if let Some(hex) = value.strip_prefix('&') {
        let bits = u32::from_str_radix(hex, 16).map_err(|_| format!("invalid float {value}"))?;
        Ok(f32::from_bits(bits) as f64)
    } else {
        value.parse().map_err(|_| format!("invalid float {value}"))
    }
}

pub fn painted(
    pim: &str,
    pit: &str,
    look: &str,
    variant: &str,
    alt_uv: Option<bool>,
) -> Result<Vec<Mesh>, String> {
    let model = parse(pim)?;
    let traits = parse(pit)?;
    let look = selected(&traits, "Look", look)?;
    let variant = selected(&traits, "Variant", variant)?;
    let mut materials = BTreeMap::new();
    for material in look.named("Material") {
        let alias = material.field("Alias")?;
        if materials.insert(alias, material.field("Effect")?).is_some() {
            return Err(format!("duplicate material alias {alias}"));
        }
    }
    let aliases = model
        .named("Material")
        .map(|m| m.field("Alias"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut visible = BTreeSet::new();
    for part in variant.named("Part") {
        let attribute = only(
            part.named("Attribute")
                .filter(|a| a.field("Tag").is_ok_and(|tag| tag == "visible")),
            "missing or ambiguous part visibility",
        )?;
        match attribute
            .field("Value")?
            .split_whitespace()
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["(", "1", ")"] => {
                visible.insert(part.field("Name")?);
            }
            ["(", "0", ")"] => {}
            _ => return Err("invalid part visibility".into()),
        }
    }
    let mut allowed = BTreeSet::new();
    for part in model.named("Part") {
        if visible.remove(part.field("Name")?) {
            for index in part.field("Pieces")?.split_whitespace() {
                allowed.insert(index.parse::<usize>().map_err(|_| "invalid piece index")?);
            }
        }
    }
    if !visible.is_empty() {
        return Err("visible variant part is missing from the model".into());
    }
    let mut indices = BTreeSet::new();
    let mut result = Vec::new();
    for piece in model.named("Piece") {
        let index = integer(piece, "Index")?;
        if !indices.insert(index) {
            return Err(format!("duplicate piece {index}"));
        }
        if !allowed.remove(&index) {
            continue;
        }
        let alias = aliases
            .get(integer(piece, "Material")?)
            .ok_or("material index out of bounds")?;
        let effect = materials
            .get(alias)
            .ok_or("look material alias is missing")?;
        let Some(source_alt_uv) = paint_shader(effect)? else {
            continue;
        };
        let alt_uv = alt_uv.unwrap_or(source_alt_uv);
        let uv_alias = if alt_uv { "_TEXCOORD2" } else { "_TEXCOORD1" };
        let stream = only(
            piece.named("Stream").filter(|stream| {
                stream.field("Aliases").is_ok_and(|aliases| {
                    aliases
                        .split_whitespace()
                        .any(|a| a.trim_matches('"') == uv_alias)
                })
            }),
            &format!("missing or ambiguous paint UV alias {uv_alias}"),
        )?;
        let uv = row_values::<2, _>(stream, number)?;
        let triangles = only(piece.named("Triangles"), "missing or ambiguous triangles")?;
        let triangles = row_values::<3, _>(triangles, |v| {
            v.parse::<usize>()
                .map_err(|_| "invalid triangle index".into())
        })?;
        if uv.len() != integer(piece, "VertexCount")?
            || triangles.len() != integer(piece, "TriangleCount")?
        {
            return Err("geometry count mismatch".into());
        }
        // Finite overflow is valid; coordinates must still fit SVG edge quantization.
        if uv
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || v.abs() >= i64::MAX as f64 / 10_000_000.0)
        {
            return Err("invalid paint UV coordinate".into());
        }
        if triangles.iter().flatten().any(|i| *i >= uv.len()) {
            return Err("triangle index out of bounds".into());
        }
        result.push(Mesh {
            uv,
            triangles,
            shader: (*effect).to_owned(),
            alt_uv,
        });
    }
    if !allowed.is_empty() {
        return Err("visible part references a missing piece".into());
    }
    Ok(result)
}

pub fn svg(meshes: &[Mesh], size: [u32; 2]) -> Result<(Vec<u8>, usize), String> {
    let mut edges = BTreeSet::new();
    for mesh in meshes {
        for triangle in &mesh.triangles {
            let points = triangle.map(|i| mesh.uv[i].map(|v| (v * 10_000_000.0).round() as i64));
            for (a, b) in [
                (points[0], points[1]),
                (points[1], points[2]),
                (points[2], points[0]),
            ] {
                if a != b {
                    edges.insert(if a < b { (a, b) } else { (b, a) });
                }
            }
        }
    }
    if edges.is_empty() {
        return Err("empty paint template".into());
    }
    let [width, height] = size;
    let lines = edges
        .iter()
        .map(|(a, b)| {
            format!(
                "M{:.3},{:.3}L{:.3},{:.3}",
                a[0] as f64 / 10_000_000.0 * width as f64,
                a[1] as f64 / 10_000_000.0 * height as f64,
                b[0] as f64 / 10_000_000.0 * width as f64,
                b[1] as f64 / 10_000_000.0 * height as f64
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok((format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n<path fill=\"none\" stroke=\"#444\" stroke-width=\"0.6\" d=\"{lines}\"/>\n</svg>\n").into_bytes(), edges.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PIM: &str = "Material {\nAlias: \"paint\"\n}\nPiece {\nIndex: 0\nMaterial: 0\nVertexCount: 3\nTriangleCount: 1\nStream {\nTag: \"_UV7\"\nAliases: \"_TEXCOORD1\" \"_TEXCOORD2\"\n0 ( &00000000 &00000000 )\n1 ( &3f800000 &00000000 )\n2 ( &00000000 &3f800000 )\n}\nTriangles {\n0 ( 0 1 2 )\n}\n}\nPart {\nName: \"body\"\nPieces: 0\n}\n";
    const PIT: &str = "Look {\nName: \"default\"\nMaterial {\nAlias: \"paint\"\nEffect: \"eut2.truckpaint.tsnmapuv\"\n}\n}\nVariant {\nName: \"default\"\nPart {\nName: \"body\"\nAttribute {\nTag: \"visible\"\nValue: ( 1 )\n}\n}\n}\n";

    #[test]
    fn semantic_alias_visibility_and_image_coordinates() {
        let meshes = painted(PIM, PIT, "Default", "Default", None).unwrap();
        assert_eq!(meshes[0].uv, [[0., 0.], [1., 0.], [0., 1.]]);
        let (svg, edges) = svg(&meshes, [2048, 1024]).unwrap();
        assert_eq!(edges, 3);
        assert!(String::from_utf8(svg).unwrap().contains("0.000,1024.000"));
        assert!(
            painted(
                PIM,
                &PIT.replace("( 1 )", "( 0 )"),
                "default",
                "default",
                None
            )
            .unwrap()
            .is_empty()
        );
    }

    #[test]
    fn shader_families_and_export_mode_select_the_paint_uv() {
        let pim = PIM.replace(" \"_TEXCOORD2\"", "").replace(
            "Triangles {",
            "Stream {\nTag: \"_UV9\"\nAliases: \"_TEXCOORD2\"\n0 ( 0.25 0.25 )\n1 ( 0.75 0.25 )\n2 ( 0.25 0.75 )\n}\nTriangles {",
        );
        for normal in ["", ".tsnmap", ".tsnmapuv"] {
            for mode in ["", ".airbrush"] {
                for (alternate, alpha, metallic) in [false, true]
                    .into_iter()
                    .flat_map(|alt| ["", ".a", ".asafew.a"].map(|alpha| (alt, alpha)))
                    .flat_map(|(alt, alpha)| {
                        ["", ".flipflake"].map(|metallic| (alt, alpha, metallic))
                    })
                {
                    let effect = format!(
                        "eut2.truckpaint{normal}{mode}{metallic}{}{alpha}",
                        if alternate { ".altuv" } else { "" }
                    );
                    let pit = PIT.replace("eut2.truckpaint.tsnmapuv", &effect);
                    for forced in [None, Some(false), Some(true)] {
                        let meshes = painted(&pim, &pit, "default", "default", forced).unwrap();
                        let alt_uv = forced.unwrap_or(alternate);
                        assert_eq!(meshes[0].alt_uv, alt_uv);
                        assert_eq!(meshes[0].shader, effect);
                        assert_eq!(
                            meshes[0].uv[0],
                            if alt_uv { [0.25, 0.25] } else { [0., 0.] }
                        );
                    }
                }
            }
        }
        let no_alternate = pim.replace("_TEXCOORD2", "_TEXCOORD3");
        assert!(painted(&no_alternate, PIT, "default", "default", Some(true)).is_err());
        let duplicate = pim.replace("_TEXCOORD1", "_TEXCOORD2");
        assert!(painted(&duplicate, PIT, "default", "default", Some(true)).is_err());
        for effect in [
            "eut2.truckpaint.unknown",
            "eut2.truckpaint.tsnmap.tsnmapuv",
            "eut2.truckpaint.altuv.altuv",
            "eut2.truckpaint.flipflake.flipflake",
            "eut2.truckpaint_extra",
        ] {
            assert!(
                painted(
                    &pim,
                    &PIT.replace("eut2.truckpaint.tsnmapuv", effect),
                    "default",
                    "default",
                    None
                )
                .is_err()
            );
        }
    }

    #[test]
    fn finite_uv_overflow_preserves_coordinates_and_selected_channel() {
        let pim = PIM.replace("&00000000 &00000000", "-2.125 1.75");
        let mesh = painted(&pim, PIT, "default", "default", None).unwrap();
        assert_eq!(mesh[0].uv[0], [-2.125, 1.75]);
        assert!(!mesh[0].alt_uv);
        let (bytes, _) = svg(&mesh, [4096, 4096]).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("-8704.000,7168.000"));
        assert!(text.contains("viewBox=\"0 0 4096 4096\""));
        for value in ["NaN", "inf", "-inf", "1e30"] {
            assert!(
                painted(
                    &PIM.replace("&3f800000", value),
                    PIT,
                    "default",
                    "default",
                    None
                )
                .is_err()
            );
        }
    }

    #[test]
    #[ignore = "needs local game intermediates: TRUCKPAINT_BASELINE_REPORTS"]
    fn replay_validated_game_templates() {
        use serde_json::Value;
        use std::{fs, io::Read, path::Path};

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let reports = std::env::var_os("TRUCKPAINT_BASELINE_REPORTS")
            .expect("set TRUCKPAINT_BASELINE_REPORTS");
        for report_path in std::env::split_paths(&reports) {
            let report: Value = serde_json::from_slice(&fs::read(&report_path).unwrap()).unwrap();
            let exported = root
                .join(report["work_directory"].as_str().unwrap())
                .join("models");
            for result in report["results"].as_array().unwrap() {
                assert_eq!(result["status"], "success");
                let package_path = report_path
                    .parent()
                    .unwrap()
                    .join(result["package"].as_str().unwrap());
                let mut archive =
                    zip::ZipArchive::new(fs::File::open(package_path).unwrap()).unwrap();
                let manifest: Value =
                    serde_json::from_reader(archive.by_name("vehicle.json").unwrap()).unwrap();
                for texture in result["textures"].as_array().unwrap() {
                    let mut meshes = Vec::new();
                    for contributor in texture["contributors"].as_array().unwrap() {
                        for selection in contributor["selections"].as_array().unwrap() {
                            let model = exported
                                .join(selection["model"].as_str().unwrap().trim_start_matches('/'))
                                .with_extension("pim");
                            meshes.extend(
                                painted(
                                    &fs::read_to_string(&model).unwrap(),
                                    &fs::read_to_string(model.with_extension("pit")).unwrap(),
                                    selection["look"].as_str().unwrap(),
                                    selection["variant"].as_str().unwrap(),
                                    Some(manifest["game"]["alt_uv"].as_bool().unwrap()),
                                )
                                .unwrap(),
                            );
                        }
                    }
                    let size = serde_json::from_value(texture["size"].clone()).unwrap();
                    let (actual, edges) = svg(&meshes, size).unwrap();
                    let id = texture["part"].as_str().unwrap();
                    let mut expected = Vec::new();
                    archive
                        .by_name(&format!("templates/{id}.svg"))
                        .unwrap()
                        .read_to_end(&mut expected)
                        .unwrap();
                    assert!(
                        actual == expected,
                        "SVG changed: {} / {id}",
                        result["vehicle"]
                    );
                    assert_eq!(edges as u64, texture["edges"].as_u64().unwrap());
                }
            }
        }
    }

    #[test]
    fn ambiguity_unsupported_shaders_and_bad_geometry_fail() {
        for (pim, pit) in [
            (
                PIM.to_owned(),
                PIT.replace("truckpaint.tsnmapuv", "truckpaint.unknown"),
            ),
            (PIM.replace("&3f800000", "&7f800000"), PIT.to_owned()),
            (PIM.replace("0 ( 0 1 2 )", "0 ( 0 1 3 )"), PIT.to_owned()),
            (
                PIM.replace("VertexCount: 3", "VertexCount: 4"),
                PIT.to_owned(),
            ),
            (PIM.replace("_TEXCOORD1", "_TEXCOORD3"), PIT.to_owned()),
            (PIM.to_owned(), format!("{PIT}{PIT}")),
        ] {
            assert!(painted(&pim, &pit, "default", "default", None).is_err());
        }
        assert!(svg(&[], [4096, 4096]).is_err());
    }
}

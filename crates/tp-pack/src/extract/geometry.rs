use std::collections::{BTreeMap, BTreeSet};

use super::syntax::{Section, parse};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub uv: Vec<[f64; 2]>,
    pub triangles: Vec<[usize; 3]>,
}

fn selected<'a>(tree: &'a Section, kind: &str, name: &str) -> Result<&'a Section, String> {
    let matches = tree
        .named(kind)
        .filter(|section| {
            section
                .field("Name")
                .is_ok_and(|n| n.eq_ignore_ascii_case(name))
        })
        .collect::<Vec<_>>();
    let [section] = matches.as_slice() else {
        return Err(format!("missing or ambiguous {kind}: {name}"));
    };
    Ok(section)
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

pub fn painted(pim: &str, pit: &str, look: &str, variant: &str) -> Result<Vec<Mesh>, String> {
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
        let attributes = part
            .named("Attribute")
            .filter(|a| a.field("Tag").is_ok_and(|tag| tag == "visible"))
            .collect::<Vec<_>>();
        let [attribute] = attributes.as_slice() else {
            return Err("missing or ambiguous part visibility".into());
        };
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
        if !effect.starts_with("eut2.truckpaint") {
            continue;
        }
        if *effect != "eut2.truckpaint.tsnmapuv" {
            return Err(format!("unsupported paint shader {effect}"));
        }
        let streams = piece
            .named("Stream")
            .filter(|stream| {
                stream.field("Aliases").is_ok_and(|aliases| {
                    aliases
                        .split_whitespace()
                        .any(|a| a.trim_matches('"') == "_TEXCOORD1")
                })
            })
            .collect::<Vec<_>>();
        let [stream] = streams.as_slice() else {
            return Err("missing or ambiguous paint UV alias _TEXCOORD1".into());
        };
        let uv = row_values::<2, _>(stream, number)?;
        let triangle_sections = piece.named("Triangles").collect::<Vec<_>>();
        let [triangles] = triangle_sections.as_slice() else {
            return Err("missing or ambiguous triangles".into());
        };
        let triangles = row_values::<3, _>(triangles, |v| {
            v.parse::<usize>()
                .map_err(|_| "invalid triangle index".into())
        })?;
        if uv.len() != integer(piece, "VertexCount")?
            || triangles.len() != integer(piece, "TriangleCount")?
        {
            return Err("geometry count mismatch".into());
        }
        if uv
            .iter()
            .flatten()
            .any(|v| !v.is_finite() || !(-0.001..=1.001).contains(v))
        {
            return Err("invalid paint UV coordinate".into());
        }
        if triangles.iter().flatten().any(|i| *i >= uv.len()) {
            return Err("triangle index out of bounds".into());
        }
        result.push(Mesh { uv, triangles });
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
        let meshes = painted(PIM, PIT, "Default", "Default").unwrap();
        assert_eq!(meshes[0].uv, [[0., 0.], [1., 0.], [0., 1.]]);
        let (svg, edges) = svg(&meshes, [2048, 1024]).unwrap();
        assert_eq!(edges, 3);
        assert!(String::from_utf8(svg).unwrap().contains("0.000,1024.000"));
        assert!(
            painted(PIM, &PIT.replace("( 1 )", "( 0 )"), "default", "default")
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn ambiguity_unsupported_shaders_and_bad_geometry_fail() {
        for (pim, pit) in [
            (
                PIM.to_owned(),
                PIT.replace("truckpaint.tsnmapuv", "truckpaint.altuv"),
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
            assert!(painted(&pim, &pit, "default", "default").is_err());
        }
        assert!(svg(&[], [4096, 4096]).is_err());
    }
}

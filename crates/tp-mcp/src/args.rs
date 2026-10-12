//! Reading tool arguments: every value is checked here, on the server
//! thread, so the app only runs calls that make sense. Errors name the
//! argument at fault.

use std::path::Path;

use base64::Engine;
use serde_json::{Map, Value};
use tp_core::document::{GradientKind, ObjectId, Rgba, StrokeAlign};
use tp_core::kurbo::{BezPath, PathEl, Point};

use crate::call::{BoxChange, BoxSpec, ColorRef, LookSpec, PaintSpec, ShadowSpec, StrokeSpec};

/// Largest coordinate accepted, in texture pixels (textures are 8192 at
/// most; objects may hang over).
pub const MAX_COORD: f64 = 100_000.0;
/// Most segments in one path.
pub const MAX_PATH_SEGMENTS: usize = 10_000;
/// Most objects one call may name.
pub const MAX_IDS: usize = 1_000;
/// Largest image file imported.
pub const MAX_IMAGE_BYTES: u64 = 50 * 1024 * 1024;
/// Image file extensions import_image reads from a path.
pub const IMAGE_EXTENSIONS: [&str; 4] = ["png", "jpg", "jpeg", "svg"];

/// A call's arguments.
pub struct Args<'a> {
    map: &'a Map<String, Value>,
}

impl<'a> Args<'a> {
    pub fn new(value: &'a Value) -> Result<Self, String> {
        static EMPTY: std::sync::OnceLock<Map<String, Value>> = std::sync::OnceLock::new();
        match value {
            Value::Object(map) => Ok(Self { map }),
            Value::Null => Ok(Self {
                map: EMPTY.get_or_init(Map::new),
            }),
            _ => Err("arguments must be an object".to_owned()),
        }
    }

    pub fn get(&self, key: &str) -> Option<&'a Value> {
        self.map.get(key).filter(|v| !v.is_null())
    }

    pub fn has(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn string(&self, key: &str) -> Result<String, String> {
        self.opt_string(key)?
            .ok_or_else(|| format!("`{key}` is required"))
    }

    pub fn opt_string(&self, key: &str) -> Result<Option<String>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::String(s)) => Ok(Some(s.clone())),
            Some(_) => Err(format!("`{key}` must be a string")),
        }
    }

    pub fn number(&self, key: &str) -> Result<f64, String> {
        self.opt_number(key)?
            .ok_or_else(|| format!("`{key}` is required"))
    }

    /// A finite number within ±[`MAX_COORD`].
    pub fn opt_number(&self, key: &str) -> Result<Option<f64>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => number(v, key).map(Some),
        }
    }

    /// A number in `range`.
    pub fn opt_in(
        &self,
        key: &str,
        range: std::ops::RangeInclusive<f64>,
    ) -> Result<Option<f64>, String> {
        match self.opt_number(key)? {
            Some(v) if !range.contains(&v) => Err(format!(
                "`{key}` must be between {} and {}",
                range.start(),
                range.end()
            )),
            v => Ok(v),
        }
    }

    pub fn opt_bool(&self, key: &str) -> Result<Option<bool>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(Value::Bool(b)) => Ok(Some(*b)),
            Some(_) => Err(format!("`{key}` must be true or false")),
        }
    }

    pub fn id(&self, key: &str) -> Result<ObjectId, String> {
        self.get(key)
            .ok_or_else(|| format!("`{key}` is required"))
            .and_then(|v| object_id(v, key))
    }

    pub fn opt_id(&self, key: &str) -> Result<Option<ObjectId>, String> {
        self.get(key).map(|v| object_id(v, key)).transpose()
    }

    /// A non-empty list of object ids, without repeats.
    pub fn ids(&self, key: &str) -> Result<Vec<ObjectId>, String> {
        let Some(Value::Array(list)) = self.get(key) else {
            return Err(format!("`{key}` must be a list of object ids"));
        };
        if list.is_empty() {
            return Err(format!("`{key}` must name at least one object"));
        }
        if list.len() > MAX_IDS {
            return Err(format!("`{key}` names more than {MAX_IDS} objects"));
        }
        let mut ids: Vec<ObjectId> = Vec::new();
        for v in list {
            let id = object_id(v, key)?;
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    /// One of `choices` (a string), or `default` when absent.
    pub fn choice<T: Copy>(
        &self,
        key: &str,
        choices: &[(&str, T)],
        default: Option<T>,
    ) -> Result<T, String> {
        match self.get(key) {
            None => default.ok_or_else(|| format!("`{key}` is required")),
            Some(Value::String(s)) => choices
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(s))
                .map(|(_, v)| *v)
                .ok_or_else(|| {
                    let names: Vec<&str> = choices.iter().map(|(n, _)| *n).collect();
                    format!("`{key}` must be one of {}", names.join(", "))
                }),
            Some(_) => Err(format!("`{key}` must be a string")),
        }
    }

    /// `x`, `y`, `width`, `height` and an optional `rotation`.
    pub fn box_spec(&self) -> Result<BoxSpec, String> {
        let x = self.number("x")?;
        let y = self.number("y")?;
        let width = positive(self.number("width")?, "width")?;
        let height = positive(self.number("height")?, "height")?;
        let rotation = self.opt_number("rotation")?.unwrap_or(0.0);
        Ok(BoxSpec {
            x,
            y,
            width,
            height,
            rotation,
        })
    }

    /// The box when `x`, `y`, `width` or `height` is given (then all four
    /// are needed).
    pub fn opt_box_spec(&self) -> Result<Option<BoxSpec>, String> {
        let any = ["x", "y", "width", "height"].iter().any(|k| self.has(k));
        any.then(|| self.box_spec()).transpose()
    }

    /// What update_object changes of the box.
    pub fn box_change(&self) -> Result<BoxChange, String> {
        let width = self.opt_number("width")?;
        let height = self.opt_number("height")?;
        Ok(BoxChange {
            x: self.opt_number("x")?,
            y: self.opt_number("y")?,
            width: width.map(|w| positive(w, "width")).transpose()?,
            height: height.map(|h| positive(h, "height")).transpose()?,
            rotation: self.opt_number("rotation")?,
        })
    }

    /// `fill`, `stroke`, `opacity`, `shadow` and `name`.
    pub fn look(&self) -> Result<LookSpec, String> {
        Ok(LookSpec {
            fill: self.get("fill").map(|v| paint(v, "fill")).transpose()?,
            stroke: self.get_or_null("stroke").map(stroke).transpose()?,
            opacity: self.opt_in("opacity", 0.0..=1.0)?.map(|v| v as f32),
            shadow: self.get_or_null("shadow").map(shadow).transpose()?,
            name: self.opt_string("name")?.map(|n| n.trim().to_owned()),
        })
    }

    /// The value of `key`, null included (null removes a stroke or a
    /// shadow).
    fn get_or_null(&self, key: &str) -> Option<&'a Value> {
        self.map.get(key)
    }
}

fn number(v: &Value, key: &str) -> Result<f64, String> {
    let n = v
        .as_f64()
        .ok_or_else(|| format!("`{key}` must be a number"))?;
    if !n.is_finite() {
        return Err(format!("`{key}` must be a finite number"));
    }
    if n.abs() > MAX_COORD {
        return Err(format!(
            "`{key}` must be between -{MAX_COORD} and {MAX_COORD}"
        ));
    }
    Ok(n)
}

fn positive(v: f64, key: &str) -> Result<f64, String> {
    if v > 0.0 {
        Ok(v)
    } else {
        Err(format!("`{key}` must be greater than 0"))
    }
}

fn object_id(v: &Value, key: &str) -> Result<ObjectId, String> {
    v.as_u64()
        .map(ObjectId)
        .ok_or_else(|| format!("`{key}` must hold object ids (whole numbers)"))
}

/// A color: `"#rrggbb"`, `"#rrggbbaa"` or `{ "swatch": "<name or id>" }`.
pub fn color(v: &Value, key: &str) -> Result<ColorRef, String> {
    match v {
        Value::String(s) => Rgba::from_hex(s)
            .filter(|_| s.trim().starts_with('#'))
            .map(ColorRef::Hex)
            .ok_or_else(|| format!("`{key}` must be a color like \"#d4202c\" or \"#d4202c80\"")),
        Value::Object(map) => match map.get("swatch") {
            Some(Value::String(s)) if !s.trim().is_empty() => {
                Ok(ColorRef::Swatch(s.trim().to_owned()))
            }
            Some(Value::Number(n)) => Ok(ColorRef::Swatch(n.to_string())),
            _ => Err(format!(
                "`{key}` must be a hex color or {{ \"swatch\": \"<name>\" }}"
            )),
        },
        _ => Err(format!(
            "`{key}` must be a hex color or {{ \"swatch\": \"<name>\" }}"
        )),
    }
}

/// A fill: a color, `"none"`, or `{ "gradient": "linear" | "radial",
/// "angle": deg, "stops": [{ "offset": 0..1, "color": … }] }`.
pub fn paint(v: &Value, key: &str) -> Result<PaintSpec, String> {
    if v.as_str().is_some_and(|s| s.eq_ignore_ascii_case("none")) {
        return Ok(PaintSpec::None);
    }
    let Some(gradient) = v.as_object().and_then(|m| m.get("gradient")) else {
        return color(v, key).map(PaintSpec::Color);
    };
    let args = Args::new(v)?;
    let kind = match gradient.as_str() {
        Some(s) if s.eq_ignore_ascii_case("linear") => GradientKind::Linear,
        Some(s) if s.eq_ignore_ascii_case("radial") => GradientKind::Radial,
        _ => return Err(format!("`{key}.gradient` must be \"linear\" or \"radial\"")),
    };
    let angle = args.opt_number("angle")?.unwrap_or(0.0);
    let Some(Value::Array(list)) = args.get("stops") else {
        return Err(format!("`{key}.stops` must be a list of stops"));
    };
    if list.len() < tp_core::document::MIN_STOPS || list.len() > tp_core::document::MAX_STOPS {
        return Err(format!(
            "`{key}.stops` must hold {} to {} stops",
            tp_core::document::MIN_STOPS,
            tp_core::document::MAX_STOPS
        ));
    }
    let stops = list
        .iter()
        .map(|stop| {
            let s = Args::new(stop)?;
            let offset = s
                .opt_in("offset", 0.0..=1.0)?
                .ok_or_else(|| format!("each of `{key}.stops` needs an `offset`"))?;
            let c = s
                .get("color")
                .ok_or_else(|| format!("each of `{key}.stops` needs a `color`"))?;
            Ok((offset as f32, color(c, &format!("{key}.stops.color"))?))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(PaintSpec::Gradient { kind, angle, stops })
}

/// A stroke `{ "color", "width", "align" }`, or null to remove it.
fn stroke(v: &Value) -> Result<Option<StrokeSpec>, String> {
    if v.is_null() {
        return Ok(None);
    }
    let a = Args::new(v).map_err(|_| "`stroke` must be an object or null".to_owned())?;
    let color = color(
        a.get("color").ok_or("`stroke.color` is required")?,
        "stroke.color",
    )?;
    let width = a.opt_in("width", 0.0..=1000.0)?.unwrap_or(4.0);
    let align = a.choice(
        "align",
        &[
            ("center", StrokeAlign::Center),
            ("inside", StrokeAlign::Inside),
            ("outside", StrokeAlign::Outside),
        ],
        Some(StrokeAlign::Center),
    )?;
    Ok(Some(StrokeSpec {
        color,
        width,
        align,
    }))
}

/// A shadow `{ "color", "opacity", "offset_x", "offset_y", "blur" }`, or
/// null to remove it.
fn shadow(v: &Value) -> Result<Option<ShadowSpec>, String> {
    if v.is_null() {
        return Ok(None);
    }
    let a = Args::new(v).map_err(|_| "`shadow` must be an object or null".to_owned())?;
    let default = tp_core::document::Shadow::DEFAULT;
    let max = tp_core::document::MAX_SHADOW_OFFSET;
    Ok(Some(ShadowSpec {
        color: a
            .get("color")
            .map(|c| color(c, "shadow.color"))
            .transpose()?
            .unwrap_or(ColorRef::Hex(default.color)),
        opacity: a
            .opt_in("opacity", 0.0..=1.0)?
            .map_or(default.opacity, |v| v as f32),
        offset_x: a
            .opt_in("offset_x", -max..=max)?
            .unwrap_or(default.offset.x),
        offset_y: a
            .opt_in("offset_y", -max..=max)?
            .unwrap_or(default.offset.y),
        blur: a
            .opt_in("blur", 0.0..=tp_core::document::MAX_SHADOW_BLUR)?
            .unwrap_or(default.blur),
    }))
}

/// SVG path data in texture pixels: every command (absolute or relative,
/// arcs too), finite, and at most [`MAX_PATH_SEGMENTS`] segments.
pub fn path_data(d: &str) -> Result<BezPath, String> {
    let path =
        BezPath::from_svg(d).map_err(|err| format!("`d` is not valid SVG path data: {err}"))?;
    let points = path.elements().iter().flat_map(|el| match *el {
        PathEl::MoveTo(p) | PathEl::LineTo(p) => vec![p],
        PathEl::QuadTo(a, b) => vec![a, b],
        PathEl::CurveTo(a, b, c) => vec![a, b, c],
        PathEl::ClosePath => Vec::new(),
    });
    let fine = |p: Point| {
        p.x.is_finite() && p.y.is_finite() && p.x.abs() <= MAX_COORD && p.y.abs() <= MAX_COORD
    };
    if !points.into_iter().all(fine) {
        return Err(format!(
            "`d` has a coordinate that is not a number or beyond ±{MAX_COORD}"
        ));
    }
    let segments = path.segments().count();
    if segments == 0 {
        return Err("`d` draws nothing (it needs at least one segment)".to_owned());
    }
    if segments > MAX_PATH_SEGMENTS {
        return Err(format!(
            "`d` has {segments} segments; split it into paths of at most {MAX_PATH_SEGMENTS}"
        ));
    }
    Ok(path)
}

/// The image of import_image: `data` (base64) with a `name`, or a `path`
/// to a PNG, JPEG or SVG file, read once. Returns the file name and bytes.
pub fn image_source(args: &Args<'_>) -> Result<(String, Vec<u8>), String> {
    let data = args.opt_string("data")?;
    let path = args.opt_string("path")?;
    match (data, path) {
        (Some(_), Some(_)) => Err("give either `data` or `path`, not both".to_owned()),
        (None, None) => Err("give the image as `data` (base64) or as a file `path`".to_owned()),
        (Some(data), None) => {
            let name = args
                .opt_string("name")?
                .filter(|n| !n.trim().is_empty())
                .ok_or("`name` is required with `data` (for example \"logo.png\")")?;
            // A data URL's prefix is tolerated.
            let data = data.split_once("base64,").map_or(data.as_str(), |(_, d)| d);
            let cleaned: String = data.chars().filter(|c| !c.is_whitespace()).collect();
            if cleaned.len() as u64 > MAX_IMAGE_BYTES / 3 * 4 + 4 {
                return Err(format!(
                    "the image is larger than {} MB",
                    MAX_IMAGE_BYTES >> 20
                ));
            }
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(cleaned)
                .map_err(|_| "`data` is not valid base64".to_owned())?;
            Ok((name.trim().to_owned(), bytes))
        }
        (None, Some(path)) => read_image_file(Path::new(&path)),
    }
}

/// Reads an image file given by path, after checking it is an absolute
/// path to a PNG, JPEG or SVG file of at most [`MAX_IMAGE_BYTES`]. Nothing
/// else is read.
pub fn read_image_file(path: &Path) -> Result<(String, Vec<u8>), String> {
    if !path.is_absolute() {
        return Err("`path` must be an absolute path".to_owned());
    }
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if !extension.is_some_and(|e| IMAGE_EXTENSIONS.contains(&e.as_str())) {
        return Err(
            "only PNG, JPEG and SVG files can be imported (.png, .jpg, .jpeg, .svg)".to_owned(),
        );
    }
    let shown = path.display();
    let meta = std::fs::metadata(path).map_err(|err| format!("cannot read {shown}: {err}"))?;
    if !meta.is_file() {
        return Err(format!("{shown} is not a file"));
    }
    if meta.len() > MAX_IMAGE_BYTES {
        return Err(format!(
            "{shown} is larger than {} MB",
            MAX_IMAGE_BYTES >> 20
        ));
    }
    let bytes = std::fs::read(path).map_err(|err| format!("cannot read {shown}: {err}"))?;
    let name = path
        .file_name()
        .map_or_else(|| shown.to_string(), |n| n.to_string_lossy().into_owned());
    Ok((name, bytes))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn colors() {
        assert_eq!(
            color(&json!("#d4202c"), "fill"),
            Ok(ColorRef::Hex(Rgba::rgb(0xd4, 0x20, 0x2c)))
        );
        assert_eq!(
            color(&json!("#d4202c80"), "fill"),
            Ok(ColorRef::Hex(Rgba::with_alpha(0xd4, 0x20, 0x2c, 0x80)))
        );
        assert_eq!(
            color(&json!({"swatch": "Company red"}), "fill"),
            Ok(ColorRef::Swatch("Company red".into()))
        );
        assert!(color(&json!("red"), "fill").unwrap_err().contains("`fill`"));
        assert!(color(&json!("d4202c"), "fill").is_err());
    }

    #[test]
    fn numbers_are_checked() {
        let v = json!({"x": 1, "y": 2, "width": f64::NAN, "height": 0});
        let a = Args::new(&v).unwrap();
        // serde_json turns NaN into null: the width is missing.
        assert_eq!(a.box_spec().unwrap_err(), "`width` is required");
        let v = json!({"x": 1, "y": 2, "width": 10, "height": 0});
        let a = Args::new(&v).unwrap();
        assert_eq!(a.box_spec().unwrap_err(), "`height` must be greater than 0");
        let v = json!({"x": 1e9, "y": 2, "width": 10, "height": 5});
        assert!(
            Args::new(&v)
                .unwrap()
                .box_spec()
                .unwrap_err()
                .contains("`x`")
        );
        let v = json!({"x": "1", "y": 2, "width": 10, "height": 5});
        assert_eq!(
            Args::new(&v).unwrap().box_spec().unwrap_err(),
            "`x` must be a number"
        );
    }

    #[test]
    fn malformed_path_data() {
        let err = path_data("M 0 0 L foo").unwrap_err();
        assert!(err.contains("`d`"), "{err}");
        assert!(path_data("M 0 0").is_err());
        assert!(path_data("M 0 0 L 1e9 0").is_err());
    }

    #[test]
    fn relative_path_equals_absolute() {
        let abs = path_data("M 10 10 L 110 10 L 110 60 Z").unwrap();
        let rel = path_data("m 10 10 l 100 0 l 0 50 z").unwrap();
        assert_eq!(abs, rel);
    }

    #[test]
    fn arcs_become_curves() {
        let p = path_data("M 0 50 A 50 50 0 1 0 100 50 A 50 50 0 1 0 0 50 Z").unwrap();
        assert!(
            p.elements()
                .iter()
                .any(|e| matches!(e, PathEl::CurveTo(..)))
        );
        let b = tp_core::kurbo::Shape::bounding_box(&p);
        assert!((b.width() - 100.0).abs() < 0.5 && (b.height() - 100.0).abs() < 0.5);
    }

    #[test]
    fn too_many_segments() {
        let mut d = String::from("M 0 0");
        for i in 0..=MAX_PATH_SEGMENTS {
            d.push_str(&format!(" L {} {}", i % 100, i % 7));
        }
        assert!(path_data(&d).unwrap_err().contains("segments"));
    }

    #[test]
    fn gradients() {
        let v = json!({"gradient": "linear", "angle": 90, "stops": [
            {"offset": 0, "color": "#ff0000"}, {"offset": 1, "color": {"swatch": "Blue"}}]});
        let PaintSpec::Gradient { kind, angle, stops } = paint(&v, "fill").unwrap() else {
            panic!("a gradient");
        };
        assert_eq!(kind, GradientKind::Linear);
        assert_eq!(angle, 90.0);
        assert_eq!(stops.len(), 2);
        let one = json!({"gradient": "radial", "stops": [{"offset": 0, "color": "#fff"}]});
        assert!(paint(&one, "fill").is_err());
        assert_eq!(paint(&json!("none"), "fill"), Ok(PaintSpec::None));
    }

    #[test]
    fn image_from_base64_and_path() {
        let v = json!({"data": "iVBORw0K", "name": "logo.png"});
        let (name, bytes) = image_source(&Args::new(&v).unwrap()).unwrap();
        assert_eq!(name, "logo.png");
        assert!(bytes.starts_with(b"\x89PNG"));
        let both = json!({"data": "AA==", "name": "a.png", "path": "/tmp/a.png"});
        assert!(
            image_source(&Args::new(&both).unwrap())
                .unwrap_err()
                .contains("not both")
        );
        let neither = json!({});
        assert!(image_source(&Args::new(&neither).unwrap()).is_err());
        let relative = json!({"path": "logo.png"});
        assert!(
            image_source(&Args::new(&relative).unwrap())
                .unwrap_err()
                .contains("absolute")
        );
    }

    #[test]
    fn not_an_image_is_never_read() {
        let dir = std::env::temp_dir();
        let key = dir.join("tp-mcp-test-id_rsa");
        std::fs::write(&key, b"secret").unwrap();
        let err = read_image_file(&key).unwrap_err();
        assert!(err.contains("only PNG, JPEG and SVG"), "{err}");
        let missing = dir.join("tp-mcp-missing.png");
        assert!(
            read_image_file(&missing)
                .unwrap_err()
                .contains("cannot read")
        );
        let _ = std::fs::remove_file(key);
    }
}

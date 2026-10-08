//! Game versions: the versions a fleet's packages support, and the
//! versions the player lists for the mod (`compatible_versions`), as
//! intervals of versions that can be intersected and compared.

use std::cmp::Ordering;

use semver::{Comparator, Op, Version, VersionReq};
use tp_core::Project;

use crate::workspace::Workspace;

/// One end of an interval of versions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bound {
    pub version: Version,
    pub inclusive: bool,
}

/// The versions between `lower` and `upper` (`None`: unbounded).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Interval {
    pub lower: Option<Bound>,
    pub upper: Option<Bound>,
}

fn version(major: u64, minor: u64, patch: u64) -> Version {
    Version::new(major, minor, patch)
}

fn from(version: Version, inclusive: bool) -> Option<Bound> {
    Some(Bound { version, inclusive })
}

/// Orders lower bounds: an exclusive bound starts after an inclusive one.
fn cmp_lower(a: &Bound, b: &Bound) -> Ordering {
    a.version
        .cmp(&b.version)
        .then(a.inclusive.cmp(&b.inclusive).reverse())
}

/// Orders upper bounds: an exclusive bound ends before an inclusive one.
fn cmp_upper(a: &Bound, b: &Bound) -> Ordering {
    a.version
        .cmp(&b.version)
        .then(a.inclusive.cmp(&b.inclusive))
}

impl Interval {
    /// Every version.
    pub const ANY: Self = Self {
        lower: None,
        upper: None,
    };

    fn new(lower: Option<Bound>, upper: Option<Bound>) -> Self {
        Self { lower, upper }
    }

    /// The versions in both intervals.
    pub fn intersect(&self, other: &Self) -> Self {
        let lower = match (&self.lower, &other.lower) {
            (Some(a), Some(b)) => Some(if cmp_lower(a, b).is_ge() { a } else { b }.clone()),
            (a, b) => a.clone().or_else(|| b.clone()),
        };
        let upper = match (&self.upper, &other.upper) {
            (Some(a), Some(b)) => Some(if cmp_upper(a, b).is_le() { a } else { b }.clone()),
            (a, b) => a.clone().or_else(|| b.clone()),
        };
        Self { lower, upper }
    }

    /// Whether no version is in it.
    pub fn is_empty(&self) -> bool {
        match (&self.lower, &self.upper) {
            (Some(l), Some(u)) => match l.version.cmp(&u.version) {
                Ordering::Greater => true,
                Ordering::Equal => !(l.inclusive && u.inclusive),
                Ordering::Less => false,
            },
            _ => false,
        }
    }

    /// Whether `v` is in it.
    pub fn contains_version(&self, v: &Version) -> bool {
        let above = self.lower.as_ref().is_none_or(|l| match v.cmp(&l.version) {
            Ordering::Greater => true,
            Ordering::Equal => l.inclusive,
            Ordering::Less => false,
        });
        let below = self.upper.as_ref().is_none_or(|u| match v.cmp(&u.version) {
            Ordering::Less => true,
            Ordering::Equal => u.inclusive,
            Ordering::Greater => false,
        });
        above && below
    }

    /// Whether every version of `other` is in it.
    pub fn contains(&self, other: &Self) -> bool {
        let lower = match (&self.lower, &other.lower) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => cmp_lower(b, a).is_ge(),
        };
        let upper = match (&self.upper, &other.upper) {
            (None, _) => true,
            (Some(_), None) => false,
            (Some(a), Some(b)) => cmp_upper(b, a).is_le(),
        };
        lower && upper
    }

    /// The interval in the packages' notation (`>=1.56, <1.58`, `=1.56.2`);
    /// `None` for any version. Not meaningful for an empty interval.
    pub fn text(&self) -> Option<String> {
        let short = |v: &Version| {
            if v.patch == 0 {
                format!("{}.{}", v.major, v.minor)
            } else {
                format!("{}.{}.{}", v.major, v.minor, v.patch)
            }
        };
        if let (Some(l), Some(u)) = (&self.lower, &self.upper)
            && l.version == u.version
            && l.inclusive
            && u.inclusive
        {
            return Some(format!("={}", short(&l.version)));
        }
        let lower = self.lower.as_ref().map(|l| {
            let op = if l.inclusive { ">=" } else { ">" };
            format!("{op}{}", short(&l.version))
        });
        let upper = self.upper.as_ref().map(|u| {
            let op = if u.inclusive { "<=" } else { "<" };
            format!("{op}{}", short(&u.version))
        });
        let parts: Vec<String> = lower.into_iter().chain(upper).collect();
        (!parts.is_empty()).then(|| parts.join(", "))
    }
}

/// The interval of one comparator (a missing minor or patch counts as 0).
fn of_comparator(c: &Comparator) -> Interval {
    let (major, minor, patch) = (c.major, c.minor, c.patch);
    let base = version(major, minor.unwrap_or(0), patch.unwrap_or(0));
    // The first version after everything the comparator names.
    let next = match (minor, patch) {
        (None, _) => version(major + 1, 0, 0),
        (Some(m), None) => version(major, m + 1, 0),
        (Some(m), Some(p)) => version(major, m, p + 1),
    };
    match c.op {
        Op::GreaterEq => Interval::new(from(base, true), None),
        Op::Greater if patch.is_some() => Interval::new(from(base, false), None),
        Op::Greater => Interval::new(from(next, true), None),
        Op::Less => Interval::new(None, from(base, false)),
        Op::LessEq if patch.is_some() => Interval::new(None, from(base, true)),
        Op::LessEq => Interval::new(None, from(next, false)),
        Op::Exact if patch.is_some() => Interval::new(from(base.clone(), true), from(base, true)),
        Op::Exact | Op::Wildcard => Interval::new(from(base, true), from(next, false)),
        Op::Tilde => {
            let end = match minor {
                Some(m) => version(major, m + 1, 0),
                None => version(major + 1, 0, 0),
            };
            Interval::new(from(base, true), from(end, false))
        }
        Op::Caret => {
            let end = match (minor, patch) {
                _ if major > 0 => version(major + 1, 0, 0),
                (None, _) => version(1, 0, 0),
                (Some(m), _) if m > 0 => version(0, m + 1, 0),
                (Some(_), None) => version(0, 1, 0),
                (Some(_), Some(p)) => version(0, 0, p + 1),
            };
            Interval::new(from(base, true), from(end, false))
        }
        _ => Interval::ANY,
    }
}

/// The versions a range allows.
pub fn of_req(req: &VersionReq) -> Interval {
    req.comparators
        .iter()
        .map(of_comparator)
        .fold(Interval::ANY, |acc, i| acc.intersect(&i))
}

/// The versions of a range stored as text; any version when it doesn't
/// parse.
pub fn of_range_text(text: &str) -> Interval {
    VersionReq::parse(text).map_or(Interval::ANY, |r| of_req(&r))
}

/// The versions a listed game version stands for: `1.56` and `1.56.*` are
/// every 1.56.x; `1.56.2`, `1.56.2.*` and `1.56.2.3` are 1.56.2 (the
/// game's fourth number is a build). `None` when it isn't written like a
/// game version.
pub fn of_listed(text: &str) -> Option<Interval> {
    let parts: Vec<&str> = text.split('.').collect();
    let (numbers, star) = match parts.split_last() {
        Some((&"*", rest)) => (rest, true),
        _ => (&parts[..], false),
    };
    if numbers.len() < 2 || numbers.len() > 4 || (star && numbers.len() == 4) {
        return None;
    }
    let numbers: Vec<u64> = numbers
        .iter()
        .map(|n| {
            (!n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                .then(|| n.parse().ok())
                .flatten()
        })
        .collect::<Option<_>>()?;
    let (major, minor) = (numbers[0], numbers[1]);
    Some(match numbers.get(2) {
        None => Interval::new(
            from(version(major, minor, 0), true),
            from(version(major, minor + 1, 0), false),
        ),
        Some(&patch) => Interval::new(
            from(version(major, minor, patch), true),
            from(version(major, minor, patch + 1), false),
        ),
    })
}

/// A vehicle and the range it supports, as recorded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleRange {
    pub vehicle: String,
    pub range: String,
}

/// The game versions every vehicle of a fleet supports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FleetVersions {
    /// No vehicle has its game data.
    NoData,
    Common(Interval),
    /// No version in common: the vehicle that starts latest and the one
    /// that ends earliest.
    Conflict {
        first: VehicleRange,
        second: VehicleRange,
    },
}

/// The vehicles of `project` with game data, and the interval each one
/// supports.
pub fn vehicle_ranges(project: &Project) -> Vec<(VehicleRange, Interval)> {
    project
        .vehicles
        .iter()
        .filter_map(|v| {
            let data = v.game_data.as_ref()?;
            Some((
                VehicleRange {
                    vehicle: v.name.clone(),
                    range: data.versions.clone(),
                },
                of_range_text(&data.versions),
            ))
        })
        .collect()
}

/// The game versions every vehicle of `project` supports.
pub fn fleet(project: &Project) -> FleetVersions {
    let ranges = vehicle_ranges(project);
    if ranges.is_empty() {
        return FleetVersions::NoData;
    }
    let common = ranges
        .iter()
        .fold(Interval::ANY, |acc, (_, i)| acc.intersect(i));
    if !common.is_empty() {
        return FleetVersions::Common(common);
    }
    let latest_start = ranges
        .iter()
        .filter(|(_, i)| i.lower.is_some())
        .max_by(|(_, a), (_, b)| {
            cmp_lower(
                a.lower.as_ref().expect("filtered"),
                b.lower.as_ref().expect("filtered"),
            )
        });
    let earliest_end = ranges
        .iter()
        .filter(|(_, i)| i.upper.is_some())
        .min_by(|(_, a), (_, b)| {
            cmp_upper(
                a.upper.as_ref().expect("filtered"),
                b.upper.as_ref().expect("filtered"),
            )
        });
    // An empty intersection has both bounds; fall back on the first
    // vehicle for the impossible case.
    let pick = |found: Option<&(VehicleRange, Interval)>| found.unwrap_or(&ranges[0]).0.clone();
    FleetVersions::Conflict {
        first: pick(latest_start),
        second: pick(earliest_end),
    }
}

impl Workspace {
    /// Sets the game versions the mod is made for, as one undo step;
    /// records nothing when they are unchanged.
    pub fn set_game_versions(&mut self, versions: Vec<String>, now: f64) {
        self.edit("undo-edit-game-versions", now, false, |project, _| {
            project.game_versions = versions;
        });
    }
}

/// The game versions the project lists, from the field's text: split on
/// commas, trimmed, empty entries dropped.
pub fn parse_list(text: &str) -> Vec<String> {
    text.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(text: &str) -> VersionReq {
        VersionReq::parse(text).unwrap()
    }

    const SAMPLES: &[&str] = &[
        "0.0.3", "0.0.4", "0.3.0", "0.3.5", "0.4.0", "1.0.0", "1.53.0", "1.55.9", "1.56.0",
        "1.56.1", "1.56.2", "1.56.3", "1.57.0", "1.57.9", "1.58.0", "1.58.1", "1.58.2", "1.59.0",
        "2.0.0",
    ];

    #[test]
    fn intervals_match_semver() {
        for text in [
            ">=1.56",
            ">1.56",
            ">1.56.2",
            "<1.58",
            "<=1.58",
            "<=1.58.1",
            "=1.56",
            "=1.56.2",
            "~1.56",
            "~1",
            "~1.56.2",
            "^1.56",
            "^0.3",
            "^0.0.3",
            "^0.0",
            "1.56.*",
            "1.*",
            "*",
            ">=1.53, <1.58",
            ">=1.50, <1.54",
        ] {
            let interval = of_req(&req(text));
            for sample in SAMPLES {
                let v = Version::parse(sample).unwrap();
                assert_eq!(
                    interval.contains_version(&v),
                    req(text).matches(&v),
                    "{text} at {sample}"
                );
            }
        }
    }

    #[test]
    fn intersections_and_text() {
        let a = of_req(&req(">=1.53, <1.58"));
        let b = of_req(&req("^1.56"));
        assert_eq!(a.intersect(&b).text().as_deref(), Some(">=1.56, <1.58"));
        assert!(
            of_req(&req(">=1.56"))
                .intersect(&of_req(&req("<1.55")))
                .is_empty()
        );
        assert!(
            !of_req(&req(">=1.56"))
                .intersect(&of_req(&req("<=1.56.0")))
                .is_empty()
        );
        assert!(
            of_req(&req(">1.56.0"))
                .intersect(&of_req(&req("<=1.56.0")))
                .is_empty()
        );
        assert_eq!(of_req(&req("*")).text(), None);
        assert_eq!(
            of_req(&req(">=1.50, <1.54")).text().as_deref(),
            Some(">=1.50, <1.54")
        );
        assert_eq!(of_req(&req("=1.56.2")).text().as_deref(), Some("=1.56.2"));
        assert_eq!(of_range_text("not a range"), Interval::ANY);
    }

    #[test]
    fn listed_versions() {
        let every_1_56 = of_listed("1.56.*").unwrap();
        assert_eq!(of_listed("1.56"), Some(every_1_56.clone()));
        assert_eq!(every_1_56.text().as_deref(), Some(">=1.56, <1.57"));
        let patch = of_listed("1.56.2").unwrap();
        assert_eq!(of_listed("1.56.2.*"), Some(patch.clone()));
        assert_eq!(of_listed("1.56.2.3"), Some(patch.clone()));
        assert_eq!(patch.text().as_deref(), Some(">=1.56.2, <1.56.3"));
        for bad in [
            "1.56.x",
            "1",
            "*",
            "1.*",
            "1.*.2",
            "",
            "1.56.2.3.*",
            "1..2",
            " 1.56",
        ] {
            assert_eq!(of_listed(bad), None, "{bad}");
        }
        let truck = of_req(&req(">=1.56"));
        assert!(truck.contains(&every_1_56));
        assert!(!truck.contains(&of_listed("1.55.*").unwrap()));
        assert!(!of_req(&req("<1.56.2")).contains(&every_1_56));
        assert!(Interval::ANY.contains(&every_1_56));
    }

    /// A committed sample package.
    fn example(name: &str) -> tp_vehicles::Package {
        let path = format!(
            "{}/../../examples/vehicles/community.truckpaint.{name}.tpv",
            env!("CARGO_MANIFEST_DIR")
        );
        tp_vehicles::Package::read(&std::fs::read(path).unwrap()).unwrap()
    }

    fn truck() -> Project {
        let p = example("sample_truck-1.1.0");
        crate::vehicle_project::fleet_project("F", &p, &["standard".to_owned()]).unwrap()
    }

    #[test]
    fn fleet_versions() {
        let mut p = truck();
        let common = |p: &Project| match fleet(p) {
            FleetVersions::Common(i) => i.text(),
            other => panic!("{other:?}"),
        };
        assert_eq!(common(&p).as_deref(), Some(">=1.56"));
        let mut ws = crate::workspace::Workspace::new(p.clone());
        ws.add_vehicle(&example("sample_trailer-1.0.0"), &["base".to_owned()], 1.0)
            .unwrap();
        assert_eq!(common(&ws.project).as_deref(), Some(">=1.56"));
        // A vehicle that stops before 1.55.
        let mut old = p.vehicles[0].clone();
        old.name = "Old Hauler".into();
        old.package_id = "custom.old.hauler".into();
        old.game_data.as_mut().unwrap().versions = "<1.55".into();
        p.vehicles.push(old);
        assert_eq!(
            fleet(&p),
            FleetVersions::Conflict {
                first: VehicleRange {
                    vehicle: "TruckPaint Sample Truck".into(),
                    range: ">=1.56".into(),
                },
                second: VehicleRange {
                    vehicle: "Old Hauler".into(),
                    range: "<1.55".into(),
                },
            }
        );
        for v in &mut p.vehicles {
            v.game_data = None;
        }
        assert_eq!(fleet(&p), FleetVersions::NoData);
    }

    #[test]
    fn lists_from_the_field() {
        assert_eq!(parse_list(" 1.56.*,1.57.* , ,"), ["1.56.*", "1.57.*"]);
        assert!(parse_list("  ").is_empty());
    }
}

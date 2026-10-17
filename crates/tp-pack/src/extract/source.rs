use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

pub struct Sources {
    pub converter: PathBuf,
    pub archives: Vec<PathBuf>,
    pub files: BTreeMap<String, Vec<usize>>,
    pub work: PathBuf,
    pub log_index: usize,
}

pub fn files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_symlink() {
            return Err(format!("unexpected symlink {}", entry.path().display()));
        }
        if kind.is_dir() {
            result.extend(files(&entry.path())?);
        } else if kind.is_file() {
            result.push(entry.path());
        }
    }
    result.sort();
    Ok(result)
}

pub fn virtual_path(parent: &str, reference: &str) -> Result<String, String> {
    if reference.contains('\\') || reference.contains('\0') || reference.contains(':') {
        return Err(format!("unsafe game path {reference}"));
    }
    let full = if reference.starts_with('/') {
        reference.to_owned()
    } else {
        format!(
            "{}/{reference}",
            parent.rsplit_once('/').map_or("", |(p, _)| p)
        )
    };
    let mut parts = Vec::new();
    for component in Path::new(&full).components() {
        match component {
            Component::Normal(p) => parts.push(p.to_string_lossy().into_owned()),
            Component::ParentDir => {
                parts
                    .pop()
                    .ok_or_else(|| format!("path escapes game root: {reference}"))?;
            }
            Component::RootDir | Component::CurDir => {}
            Component::Prefix(_) => return Err(format!("unsafe game path {reference}")),
        }
    }
    if parts.is_empty() {
        return Err("empty game file path".into());
    }
    Ok(format!("/{}", parts.join("/")))
}

impl Sources {
    pub fn new(installation: &Path, converter: &Path, work: &Path) -> Result<Self, String> {
        let installation = installation
            .canonicalize()
            .map_err(|e| format!("installation: {e}"))?;
        let converter = converter
            .canonicalize()
            .map_err(|e| format!("converter: {e}"))?;
        let mut archives = Vec::new();
        // Matches the source pools observed in the installed game's mount log.
        for name in [
            "base.scs",
            "base_map.scs",
            "base_vehicle.scs",
            "base_share.scs",
            "def.scs",
            "core.scs",
            "effect.scs",
            "locale.scs",
            "base_cfg.scs",
            "version.scs",
        ] {
            let path = installation.join(name);
            if path.is_file() {
                archives.push(path);
            } else if matches!(name, "base.scs" | "def.scs" | "version.scs") {
                return Err(format!("missing installed archive {name}"));
            }
        }
        let mut dlc = std::fs::read_dir(&installation)
            .map_err(|e| e.to_string())?
            .map(|e| e.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .is_some_and(|n| n.starts_with("dlc_") && n.ends_with(".scs"))
            })
            .map(|e| e.path())
            .collect::<Vec<_>>();
        dlc.sort();
        archives.extend(dlc);
        std::fs::create_dir_all(work).map_err(|e| e.to_string())?;
        Ok(Self {
            converter,
            archives,
            files: BTreeMap::new(),
            work: work.to_owned(),
            log_index: 0,
        })
    }

    pub fn command(
        &mut self,
        mounts: &[usize],
        export: &Path,
        args: &[&str],
    ) -> Result<String, String> {
        let mut command = Command::new(&self.converter);
        for index in mounts {
            command.arg("-b").arg(&self.archives[*index]);
        }
        command.arg("-e").arg(export).args(args);
        let result = command.output().map_err(|e| format!("converter: {e}"))?;
        let stdout =
            String::from_utf8(result.stdout).map_err(|_| "converter output is not UTF-8")?;
        let stderr = String::from_utf8_lossy(&result.stderr);
        let diagnostic = format!("{stdout}\n{stderr}");
        let log = self
            .work
            .join(format!("converter-{:06}.log", self.log_index));
        self.log_index += 1;
        std::fs::write(&log, &diagnostic).map_err(|e| format!("{}: {e}", log.display()))?;
        let absent_directory = args.first() == Some(&"--list-directory")
            && diagnostic.contains("<error> [system] : readDir returned null!");
        let checked = if absent_directory {
            diagnostic.replace("<error> [system] : readDir returned null!", "")
        } else {
            diagnostic.clone()
        };
        if (!result.status.success() && !absent_directory)
            || ["Unable to", "[error]", "[err]", "[warn]", "<error>"]
                .iter()
                .any(|marker| checked.contains(marker))
        {
            return Err(format!(
                "converter failed for {}: see {}",
                args.join(" "),
                log.display()
            ));
        }
        Ok(stdout)
    }

    pub fn inventory(&mut self) -> Result<(), String> {
        for index in 0..self.archives.len() {
            let export = self.work.join(format!("archive-{index}"));
            std::fs::create_dir_all(&export).map_err(|e| e.to_string())?;
            // Listing absent directories is allowed; actual extraction errors are not.
            let listing = self.command(
                &[index],
                &export,
                &["--list-directory", "/def/vehicle/truck"],
            )?;
            if !listing
                .lines()
                .any(|line| line.starts_with("[D] ") || line.starts_with("[F] "))
            {
                continue;
            }
            self.command(
                &[index],
                &export,
                &["--extract-directory", "/def/vehicle/truck"],
            )?;
            let root = export.join("def/vehicle/truck");
            if !root.is_dir() {
                return Err("converter did not extract listed truck definitions".into());
            }
            for path in files(&root)? {
                let relative = path.strip_prefix(&export).map_err(|e| e.to_string())?;
                let virtual_name = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
                self.files.entry(virtual_name).or_default().push(index);
            }
        }
        Ok(())
    }

    pub fn read(&mut self, path: &str) -> Result<String, String> {
        let local = self.locate(path)?;
        std::fs::read_to_string(&local).map_err(|e| format!("{path}: {e}"))
    }

    pub fn locate(&mut self, path: &str) -> Result<PathBuf, String> {
        let path = virtual_path("/", path)?;
        if !self.files.contains_key(&path) {
            let parent = path.rsplit_once('/').unwrap().0;
            let parent = if parent.is_empty() { "/" } else { parent };
            for index in 0..self.archives.len() {
                let export = self.work.join(format!("archive-{index}"));
                let listing = self.command(&[index], &export, &["--list-directory", parent])?;
                for entry in listing.lines().filter_map(|line| line.strip_prefix("[F] ")) {
                    let entry = virtual_path("/", entry.trim())?;
                    let indices = self.files.entry(entry).or_default();
                    if !indices.contains(&index) {
                        indices.push(index);
                        indices.sort_unstable();
                    }
                }
            }
            if !self.files.contains_key(&path) {
                return Err(format!("missing game file {path}"));
            }
        }
        let index = *self.files[&path].last().ok_or("missing source")?;
        let local = self
            .work
            .join(format!("archive-{index}"))
            .join(path.trim_start_matches('/'));
        if !local.is_file() {
            let export = self.work.join(format!("archive-{index}"));
            self.command(&[index], &export, &["--extract-file", &path])?;
            if !local.is_file() {
                return Err(format!("converter did not extract {path}"));
            }
        }
        Ok(local)
    }

    pub fn resolved(
        &mut self,
        path: &str,
        active: &mut BTreeSet<String>,
        evidence: &mut BTreeSet<String>,
    ) -> Result<String, String> {
        let path = virtual_path("/", path)?;
        if active.len() >= 64 || !active.insert(path.clone()) {
            return Err(format!("cyclic or too deep include: {path}"));
        }
        evidence.insert(path.clone());
        let text = self.read(&path)?;
        let mut result = String::new();
        for line in text.lines() {
            if let Some(include) = line.trim().strip_prefix("@include") {
                let include = include.trim();
                let end = include
                    .strip_prefix('"')
                    .and_then(|v| v.find('"'))
                    .ok_or_else(|| format!("invalid include in {path}"))?;
                let tail = include[end + 2..].trim();
                if !tail.is_empty() && !tail.starts_with('#') && !tail.starts_with("//") {
                    return Err(format!("invalid include in {path}"));
                }
                let reference = virtual_path(&path, &include[1..end + 1])?;
                result.push_str(&self.resolved(&reference, active, evidence)?);
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        active.remove(&path);
        Ok(result)
    }

    pub fn provenance(&self, paths: &BTreeSet<String>) -> Value {
        json!(
            paths
                .iter()
                .map(|path| {
                    let sources = self
                        .files
                        .get(path)
                        .map(|indices| {
                            indices
                                .iter()
                                .map(|i| {
                                    self.archives[*i]
                                        .file_name()
                                        .unwrap()
                                        .to_string_lossy()
                                        .to_string()
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    json!({"path": path, "source": sources.last(), "mounted_sources": sources})
                })
                .collect::<Vec<_>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn includes_normalize_inside_game_root_only() {
        assert_eq!(
            virtual_path("/def/truck/cabin/high.sii", "../common.sui").unwrap(),
            "/def/truck/common.sui"
        );
        assert_eq!(
            virtual_path("/def/a.sii", "/vehicle/shared/model.pmd").unwrap(),
            "/vehicle/shared/model.pmd"
        );
        for reference in ["../../outside", "C:/outside", "..\\outside", ""] {
            assert!(virtual_path("/a.sii", reference).is_err(), "{reference}");
        }
    }

    #[test]
    fn overlays_inclusions_and_missing_or_cyclic_references() {
        let temp = tempfile::tempdir().unwrap();
        for index in 0..2 {
            let dir = temp.path().join(format!("archive-{index}/def"));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("main.sii"),
                format!("@include \"common.sui\"\nname: source{index}\n"),
            )
            .unwrap();
            std::fs::write(dir.join("common.sui"), "model: \"/shared/model.pmd\"\n").unwrap();
        }
        let mut sources = Sources {
            converter: "unused".into(),
            archives: vec!["def.scs".into(), "dlc_tuning.scs".into()],
            files: BTreeMap::from([
                ("/def/main.sii".into(), vec![0, 1]),
                ("/def/common.sui".into(), vec![0, 1]),
            ]),
            work: temp.path().to_owned(),
            log_index: 0,
        };
        let mut evidence = BTreeSet::new();
        let text = sources
            .resolved("/def/main.sii", &mut BTreeSet::new(), &mut evidence)
            .unwrap();
        assert!(text.contains("source1"));
        assert!(text.contains("/shared/model.pmd"));
        assert_eq!(sources.provenance(&evidence)[0]["source"], "dlc_tuning.scs");
        std::fs::write(
            temp.path().join("archive-1/def/common.sui"),
            "@include \"main.sii\"\n",
        )
        .unwrap();
        assert!(
            sources
                .resolved("/def/main.sii", &mut BTreeSet::new(), &mut BTreeSet::new())
                .unwrap_err()
                .contains("cyclic")
        );
        std::fs::remove_file(temp.path().join("archive-1/def/common.sui")).unwrap();
        assert!(
            sources
                .resolved("/def/main.sii", &mut BTreeSet::new(), &mut BTreeSet::new())
                .is_err()
        );
    }
}

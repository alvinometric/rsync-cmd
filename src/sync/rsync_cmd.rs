use crate::core::config::Config;
use std::path::PathBuf;

pub fn build_argv(config: &Config, timestamp_label: &str, dry_run: bool) -> Vec<String> {
    let mut argv: Vec<String> = vec!["rsync".into(), "-a".into()];

    if dry_run {
        argv.push("--dry-run".into());
    }

    let dest = &config.destination.path;

    if config.mirror {
        argv.push("--delete".into());
    }

    if let Some(versioning) = &config.versioning {
        let snapshots_root: PathBuf = versioning
            .snapshots_path
            .clone()
            .unwrap_or_else(|| dest.parent().unwrap().join("snapshots"));
        let backup_dir = snapshots_root.join(timestamp_label);
        argv.push("--backup".into());
        argv.push(format!("--backup-dir={}", backup_dir.display()));
    }

    for pattern in &config.excludes {
        argv.push(format!("--exclude={pattern}"));
    }

    for src in &config.sources {
        argv.push(src.display().to_string());
    }

    argv.push(dest.display().to_string());

    argv
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::*;
    use std::path::PathBuf;

    fn dest() -> Destination {
        Destination {
            drive_id: Some("drive-1".into()),
            path: PathBuf::from("/mnt/backup/live"),
        }
    }

    #[test]
    fn minimal_argv() {
        let c = Config {
            sources: vec![PathBuf::from("/home/alvin/docs")],
            destination: dest(),
            ..Config::default()
        };
        let argv = build_argv(&c, "snapshot_2026-05-06_1430", false);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "--backup",
                "--backup-dir=/mnt/backup/snapshots/snapshot_2026-05-06_1430",
                "/home/alvin/docs",
                "/mnt/backup/live",
            ]
        );
    }

    #[test]
    fn dry_run() {
        let c = Config {
            sources: vec![PathBuf::from("/home/alvin/docs")],
            destination: dest(),
            ..Config::default()
        };
        let argv = build_argv(&c, "snap", true);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "--dry-run",
                "--backup",
                "--backup-dir=/mnt/backup/snapshots/snap",
                "/home/alvin/docs",
                "/mnt/backup/live",
            ]
        );
    }

    #[test]
    fn mirror_mode() {
        let c = Config {
            sources: vec![PathBuf::from("/home/alvin/docs")],
            destination: dest(),
            mirror: true,
            ..Config::default()
        };
        let argv = build_argv(&c, "snap", false);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "--delete",
                "--backup",
                "--backup-dir=/mnt/backup/snapshots/snap",
                "/home/alvin/docs",
                "/mnt/backup/live",
            ]
        );
    }

    #[test]
    fn versioning_disabled() {
        let mut c = Config {
            sources: vec![PathBuf::from("/home/alvin/docs")],
            destination: dest(),
            ..Config::default()
        };
        c.versioning = None;
        let argv = build_argv(&c, "snap", false);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "/home/alvin/docs",
                "/mnt/backup/live",
            ]
        );
    }

    #[test]
    fn excludes() {
        let mut c = Config {
            sources: vec![PathBuf::from("/home/alvin/docs")],
            destination: dest(),
            excludes: vec!["node_modules".into(), "*.tmp".into(), ".cache/".into()],
            ..Config::default()
        };
        c.versioning = None;
        let argv = build_argv(&c, "snap", false);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "--exclude=node_modules",
                "--exclude=*.tmp",
                "--exclude=.cache/",
                "/home/alvin/docs",
                "/mnt/backup/live",
            ]
        );
    }

    #[test]
    fn multiple_sources() {
        let mut c = Config {
            sources: vec![
                PathBuf::from("/a"),
                PathBuf::from("/b/c"),
                PathBuf::from("/d e/f"),
            ],
            destination: dest(),
            ..Config::default()
        };
        c.versioning = None;
        let argv = build_argv(&c, "snap", false);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "/a",
                "/b/c",
                "/d e/f",
                "/mnt/backup/live",
            ]
        );
    }

    #[test]
    fn explicit_snapshots_path() {
        let mut c = Config {
            sources: vec![PathBuf::from("/home/alvin/docs")],
            destination: dest(),
            ..Config::default()
        };
        c.versioning.as_mut().unwrap().snapshots_path = Some(PathBuf::from("/mnt/archive/snaps"));
        let argv = build_argv(&c, "snap", false);
        assert_eq!(
            argv,
            vec![
                "rsync",
                "-a",
                "--backup",
                "--backup-dir=/mnt/archive/snaps/snap",
                "/home/alvin/docs",
                "/mnt/backup/live",
            ]
        );
    }

}

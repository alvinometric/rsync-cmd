use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrecheckError {
    SourceEqualsDestination(PathBuf),
    SourceContainsDestination { source: PathBuf, destination: PathBuf },
    DestinationContainsSource { source: PathBuf, destination: PathBuf },
}

pub fn check(sources: &[PathBuf], destination: &Path) -> Result<(), PrecheckError> {
    let dest = normalize(destination);
    for src in sources {
        let src_n = normalize(src);
        if src_n == dest {
            return Err(PrecheckError::SourceEqualsDestination(src.clone()));
        }
        if dest.starts_with(&src_n) {
            return Err(PrecheckError::SourceContainsDestination {
                source: src.clone(),
                destination: destination.to_path_buf(),
            });
        }
        if src_n.starts_with(&dest) {
            return Err(PrecheckError::DestinationContainsSource {
                source: src.clone(),
                destination: destination.to_path_buf(),
            });
        }
    }
    Ok(())
}

fn normalize(path: &Path) -> PathBuf {
    path.components().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_paths_ok() {
        check(&[PathBuf::from("/home/alvin/docs")], Path::new("/mnt/backup")).unwrap();
        check(&[PathBuf::from("/data-old")], Path::new("/data")).unwrap();
    }

    #[test]
    fn overlap_is_rejected() {
        assert!(check(&[PathBuf::from("/data")], Path::new("/data")).is_err());
        assert!(check(&[PathBuf::from("/data")], Path::new("/data/backup")).is_err());
        assert!(check(&[PathBuf::from("/data/sub")], Path::new("/data")).is_err());
    }

    #[test]
    fn trailing_slash_normalized() {
        assert!(check(&[PathBuf::from("/data/")], Path::new("/data")).is_err());
    }
}

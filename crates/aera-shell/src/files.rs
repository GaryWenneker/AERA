use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Entry {
    pub name: OsString,
    pub path: PathBuf,
    pub kind: &'static str,
    pub size: Option<u64>,
}

pub fn list(path: &Path) -> std::io::Result<Vec<Entry>> {
    let mut entries = Vec::new();
    for item in std::fs::read_dir(path)?.take(1000) {
        let item = item?;
        let meta = std::fs::symlink_metadata(item.path())?;
        let (kind, size) = if meta.is_dir() {
            ("Folder", None)
        } else if meta.is_symlink() {
            ("Link", None)
        } else if meta.is_file() {
            ("File", Some(meta.len()))
        } else {
            ("Other", None)
        };
        entries.push(Entry {
            name: item.file_name(),
            path: item.path(),
            kind,
            size,
        });
    }
    entries.sort_by(|a, b| {
        (a.kind != "Folder", &a.name).cmp(&(b.kind != "Folder", &b.name))
    });
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_path_reports_error() {
        let path = std::env::temp_dir().join(format!("aera-missing-{}", std::process::id()));
        assert!(list(&path.join("not-present")).is_err());
    }

    #[test]
    fn real_file_metadata_and_directory_order() {
        let root = std::env::temp_dir().join(format!("aera-files-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("folder")).unwrap();
        std::fs::write(root.join("example.txt"), b"abc").unwrap();
        let entries = list(&root).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].kind, "Folder");
        assert_eq!(entries[1].size, Some(3));
    }
}

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
    entries.sort_by(|a, b| (a.kind != "Folder", &a.name).cmp(&(b.kind != "Folder", &b.name)));
    Ok(entries)
}

fn destination(directory: &Path, name: &str) -> std::io::Result<PathBuf> {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (1..=9).any(|n| stem == format!("COM{n}") || stem == format!("LPT{n}"));
    if name.is_empty()
        || name.trim() != name
        || name.ends_with('.')
        || reserved
        || name.chars().any(|c| c.is_control() || r#"/\:*?"<>|"#.contains(c))
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Enter a single valid file or folder name",
        ));
    }
    Ok(directory.join(name))
}

pub fn create(directory: &Path, name: &str, folder: bool) -> std::io::Result<()> {
    let path = destination(directory, name)?;
    if folder {
        std::fs::create_dir(path)
    } else {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(|_| ())
    }
}

pub fn copy_file(source: &Path, directory: &Path, name: &str) -> std::io::Result<()> {
    if !std::fs::symlink_metadata(source)?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Only regular files can be copied",
        ));
    }
    let mut input = std::fs::File::open(source)?;
    if !input.metadata()?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Source is not a regular file",
        ));
    }
    let path = destination(directory, name)?;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let result = std::io::copy(&mut input, &mut output).and_then(|_| output.sync_all());
    drop(output);
    // Keep an incomplete file on error rather than deleting a potentially replaced path.
    // The UI reports the error; recovery/cleanup is deliberately explicit.
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_names_rejected() {
        for name in ["", "..", "a/b", "a\\b", "CON.txt", "NUL", "x.", " x"] {
            assert!(destination(Path::new("."), name).is_err());
        }
    }

    #[test]
    fn creation_and_copy_never_overwrite() {
        let root = std::env::temp_dir().join(format!("aera-actions-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        create(&root, "folder", true).unwrap();
        create(&root, "source.txt", false).unwrap();
        std::fs::write(root.join("source.txt"), b"payload").unwrap();
        copy_file(&root.join("source.txt"), &root, "copy.txt").unwrap();
        assert!(copy_file(&root.join("source.txt"), &root, "copy.txt").is_err());
        assert!(create(&root, "source.txt", false).is_err());
        assert_eq!(std::fs::read(root.join("copy.txt")).unwrap(), b"payload");
        assert_eq!(std::fs::read(root.join("source.txt")).unwrap(), b"payload");
        std::fs::remove_dir_all(&root).unwrap();
    }

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

use std::path::PathBuf;

pub struct Volume {
    pub name: String,
    pub mount: PathBuf,
    pub filesystem: String,
    pub total: u64,
    pub available: u64,
}

pub fn inventory() -> Vec<Volume> {
    sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .map(|disk| Volume {
            name: disk.name().to_string_lossy().into_owned(),
            mount: disk.mount_point().to_path_buf(),
            filesystem: disk.file_system().to_string_lossy().into_owned(),
            total: disk.total_space(),
            available: disk.available_space(),
        })
        .collect()
}

pub fn used_fraction(total: u64, available: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    total.saturating_sub(available) as f32 / total as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_is_bounded() {
        assert_eq!(used_fraction(100, 25), 0.75);
        assert_eq!(used_fraction(0, 0), 0.0);
        assert_eq!(used_fraction(100, 200), 0.0);
    }
}

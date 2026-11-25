#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target<'a> {
    pub os: &'a str,
    pub arch: &'a str,
}

impl Target<'static> {
    pub const fn current() -> Self {
        Self {
            os: std::env::consts::OS,
            arch: std::env::consts::ARCH,
        }
    }
}

impl Target<'_> {
    /// Compile-target eligibility only, not runtime hardware qualification.
    pub fn hosted_eligible(self) -> bool {
        matches!(self.os, "windows" | "macos" | "linux")
            && matches!(self.arch, "x86_64" | "aarch64")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_matrix() {
        for os in ["windows", "macos", "linux"] {
            for arch in ["x86_64", "aarch64"] {
                assert!(Target { os, arch }.hosted_eligible());
            }
        }
    }

    #[test]
    fn unsupported_matrix() {
        for (os, arch) in [
            ("freebsd", "x86_64"),
            ("linux", "riscv64"),
            ("", ""),
            ("windows", "x86"),
        ] {
            assert!(!Target { os, arch }.hosted_eligible());
        }
    }

    #[test]
    fn current_matches_compile_target() {
        assert_eq!(Target::current().os, std::env::consts::OS);
        assert_eq!(Target::current().arch, std::env::consts::ARCH);
    }
}

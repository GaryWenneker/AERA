use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_aera"))
        .args(args)
        .output()
        .expect("CLI must execute")
}

#[test]
fn info_reports_target() {
    let out = run(&["info"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("hosted prototype"));
    assert!(text.contains(&format!("os={}", std::env::consts::OS)));
    assert!(text.contains(&format!("arch={}", std::env::consts::ARCH)));
}

#[test]
fn doctor_reports_eligibility_not_readiness() {
    let out = run(&["doctor"]);
    let eligible = aera_core::Target::current().hosted_eligible();
    assert_eq!(out.status.code(), Some(if eligible { 0 } else { 1 }));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains(&format!("hosted_target_eligible={eligible}")));
    assert!(text.contains("Hardware readiness: not assessed"));
}

#[test]
fn invalid_arguments_fail() {
    for args in [&["unknown"][..], &["info", "extra"][..]] {
        let out = run(args);
        assert_eq!(out.status.code(), Some(2));
        assert!(String::from_utf8(out.stderr).unwrap().contains("Usage:"));
    }
}

#[test]
fn help_succeeds() {
    for args in [&[][..], &["help"][..], &["--help"][..]] {
        let out = run(args);
        assert!(out.status.success());
        assert!(String::from_utf8(out.stdout).unwrap().contains("Usage:"));
    }
}

use assert_cmd::Command;
use predicates::prelude::*;

fn jm() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("jm")
}

// ─── Basic CLI ─────────────────────────────────────────────────────────────

#[test]
fn version_flag() {
    jm().arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("jm "));
}

#[test]
fn help_flag() {
    jm().arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("JDK version manager"))
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("uninstall"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("use"))
        .stdout(predicate::str::contains("default"))
        .stdout(predicate::str::contains("current"))
        .stdout(predicate::str::contains("which"))
        .stdout(predicate::str::contains("env"))
        .stdout(predicate::str::contains("shell"))
        .stdout(predicate::str::contains("config"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("upgrade"));
}

#[test]
fn no_args_shows_help() {
    jm().assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

// ─── Subcommand help ───────────────────────────────────────────────────────

#[test]
fn install_help() {
    jm().args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Install a JDK version"));
}

#[test]
fn uninstall_help() {
    jm().args(["uninstall", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove an installed JDK"));
}

#[test]
fn search_help() {
    jm().args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Search available JDK"));
}

// ─── list (local, no JDKs installed) ───────────────────────────────────────

#[test]
fn list_empty_with_custom_home() {
    let tmp = tempfile::tempdir().unwrap();
    jm().arg("list")
        .env("JM_HOME", tmp.path())
        .assert()
        .success();
}

// ─── current (no JDK set) ──────────────────────────────────────────────────

#[test]
fn current_no_jdk_with_custom_home() {
    let tmp = tempfile::tempdir().unwrap();
    jm().arg("current")
        .env("JM_HOME", tmp.path())
        .assert()
        .success();
}

// ─── config subcommands ────────────────────────────────────────────────────

#[test]
fn config_path() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["config", "path"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn config_list_defaults() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["config", "list"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("preferred_distribution"))
        .stdout(predicate::str::contains("temurin"));
}

#[test]
fn config_get_known_key() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["config", "get", "global.preferred_distribution"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("temurin"));
}

#[test]
fn config_get_unknown_key() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["config", "get", "nonexistent.key"])
        .env("JM_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown config key"));
}

#[test]
fn config_set_and_get() {
    let tmp = tempfile::tempdir().unwrap();
    // Ensure dirs exist
    std::fs::create_dir_all(tmp.path()).unwrap();

    jm().args(["config", "set", "global.preferred_distribution", "corretto"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success();

    jm().args(["config", "get", "global.preferred_distribution"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("corretto"));
}

#[test]
fn config_set_invalid_color() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["config", "set", "ui.color", "purple"])
        .env("JM_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("auto, always, or never"));
}

// ─── env ───────────────────────────────────────────────────────────────────

#[test]
fn env_with_custom_home() {
    let tmp = tempfile::tempdir().unwrap();
    jm().arg("env")
        .env("JM_HOME", tmp.path())
        .assert()
        .success();
}

// ─── shell init ────────────────────────────────────────────────────────────

#[test]
fn shell_init_bash() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["shell", "init", "bash"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("JAVA_HOME"))
        .stdout(predicate::str::contains("__jm_auto_switch"));
}

#[test]
fn shell_init_zsh() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["shell", "init", "zsh"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("JAVA_HOME"))
        .stdout(predicate::str::contains("chpwd"));
}

#[test]
fn shell_init_fish() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["shell", "init", "fish"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("JAVA_HOME"))
        .stdout(predicate::str::contains("--on-variable PWD"));
}

#[test]
fn shell_init_powershell() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["shell", "init", "powershell"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("JAVA_HOME"))
        .stdout(predicate::str::contains("Invoke-Expression"));
}

#[test]
fn shell_init_unknown() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["shell", "init", "tcsh"])
        .env("JM_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown shell"));
}

// ─── which (no JDK) ───────────────────────────────────────────────────────

#[test]
fn which_no_jdk() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["which", "java"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success();
}

// ─── uninstall (no match) ──────────────────────────────────────────────────

#[test]
fn uninstall_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["uninstall", "temurin-99", "--force"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("No installed JDK"));
}

// ─── use (no match) ────────────────────────────────────────────────────────

#[test]
fn use_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["use", "99"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success();
}

// ─── aliases ───────────────────────────────────────────────────────────────

#[test]
fn alias_ls_works() {
    let tmp = tempfile::tempdir().unwrap();
    jm().arg("ls").env("JM_HOME", tmp.path()).assert().success();
}

#[test]
fn alias_rm_works() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["rm", "temurin-99", "--force"])
        .env("JM_HOME", tmp.path())
        .assert()
        .success();
}

// ─── invalid distribution validation ───────────────────────────────────────

#[test]
fn install_rejects_path_traversal_distribution() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["install", "../../etc/passwd-21"])
        .env("JM_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid distribution name"));
}

#[test]
fn list_rejects_invalid_distribution() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["list", "--distribution", "../../escape"])
        .env("JM_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid distribution name"));
}

#[test]
fn search_rejects_invalid_distribution_before_network_access() {
    let tmp = tempfile::tempdir().unwrap();
    jm().args(["search", "21", "--distribution", "../../escape"])
        .env("JM_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid distribution name"));
}

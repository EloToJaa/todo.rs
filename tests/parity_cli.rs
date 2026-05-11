use std::fs;
use std::process::Command;

use tempfile::tempdir;

fn write_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp = tempdir().expect("temp dir");
    let calendars = temp.path().join("calendars");
    let home = calendars.join("home");
    fs::create_dir_all(&home).expect("home dir");

    let vevent = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:event-1\r\nSUMMARY:Team Sync\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let vtodo = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VTODO\r\nUID:todo-1\r\nSUMMARY:Buy milk\r\nSTATUS:NEEDS-ACTION\r\nEND:VTODO\r\nEND:VCALENDAR\r\n";

    fs::write(home.join("event.ics"), vevent).expect("write event");
    fs::write(home.join("todo.ics"), vtodo).expect("write todo");

    let config = temp.path().join("config.toml");
    fs::write(
        &config,
        format!(
            "path = \"{}/*\"\ncache_path = \"{}/cache.sqlite3\"\ndefault_command = \"list\"\n",
            calendars.display(),
            temp.path().display()
        ),
    )
    .expect("write config");

    (temp, config)
}

fn write_todoman_config(temp: &tempfile::TempDir) -> std::path::PathBuf {
    let config = temp.path().join("todoman_config.py");
    let calendars = temp.path().join("calendars");
    fs::write(
        &config,
        format!(
            "path = \"{}/*\"\ncache_path = \"{}/todoman-cache.sqlite3\"\ndefault_command = \"list\"\n",
            calendars.display(),
            temp.path().display()
        ),
    )
    .expect("write todoman config");
    config
}

#[test]
fn list_only_shows_vtodo_entries() {
    let (_temp, config) = write_fixture();
    let bin = env!("CARGO_BIN_EXE_todors");

    let output = Command::new(bin)
        .arg("--config")
        .arg(&config)
        .arg("list")
        .output()
        .expect("run list");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Buy milk"));
    assert!(!stdout.contains("Team Sync"));
}

#[test]
fn todors_matches_todoman_for_basic_list_if_available() {
    let (temp, config) = write_fixture();
    let bin = env!("CARGO_BIN_EXE_todors");
    let todoman_config = write_todoman_config(&temp);

    let todoman_check = Command::new("todoman").arg("--help").output();
    let Ok(check) = todoman_check else {
        return;
    };
    if !check.status.success() {
        return;
    }

    let ours = Command::new(bin)
        .arg("--config")
        .arg(&config)
        .arg("list")
        .output()
        .expect("run todors list");
    assert!(ours.status.success());
    let ours_out = String::from_utf8_lossy(&ours.stdout);

    let theirs = Command::new("todoman")
        .arg("--config")
        .arg(&todoman_config)
        .arg("list")
        .output()
        .expect("run todoman list");
    assert!(theirs.status.success());
    let theirs_out = String::from_utf8_lossy(&theirs.stdout);

    assert_eq!(ours_out.lines().count(), theirs_out.lines().count());
}

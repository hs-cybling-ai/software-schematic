use software_schematic_cli::init_project;
use std::{
    fs,
    process::{Command, Stdio},
};
use tempfile::tempdir;

#[cfg(unix)]
#[test]
fn pinned_wrapper_binds_mcp_to_its_project_from_another_working_directory() {
    let project_parent = tempdir().unwrap();
    let project = project_parent.path().join("Bound Project");
    let layout = init_project(&project).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_ss"), layout.tool.join("bin/ss")).unwrap();
    let elsewhere = tempdir().unwrap();
    let output = Command::new(project.join("ssw"))
        .arg("mcp")
        .current_dir(elsewhere.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("project=Bound Project"), "{stderr}");
    assert!(!stderr.contains(&elsewhere.path().display().to_string()));
    let project_id = fs::read_to_string(layout.tool.join("project-id")).unwrap();
    assert!(!project_id.trim().is_empty());
}

#[test]
fn mcp_rejects_an_ambient_uninitialized_directory() {
    let directory = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ss"))
        .args(["mcp", "--project"])
        .arg(directory.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("project has no confined schematics directory")
    );
}

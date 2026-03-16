use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_add_schema_creates_file() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    fs::write(project_root.join("package.json"), "{}").unwrap();
    fs::create_dir_all(project_root.join("db/schema")).unwrap();

    let exe_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("tsx.exe");

    let result = Command::new(&exe_path)
        .args([
            "generate",
            "schema",
            "--json",
            r#"{"name":"products","fields":[{"name":"title","type":"string","required":true}]}"#,
        ])
        .current_dir(&project_root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(result.status.success(), "Command failed: {}", stderr);
    assert!(stdout.contains("success"), "Expected success in output");

    let schema_file = project_root.join("db/schema/products.ts");
    assert!(schema_file.exists(), "Schema file should be created");

    let content = fs::read_to_string(&schema_file).unwrap();
    assert!(
        content.contains("products"),
        "Schema should contain table name"
    );
}

#[test]
fn test_dry_run_no_files_created() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    fs::write(project_root.join("package.json"), "{}").unwrap();

    let exe_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("tsx.exe");

    let result = Command::new(&exe_path)
        .args([
            "generate",
            "schema",
            "--dry-run",
            "--json",
            r#"{"name":"test","fields":[{"name":"title","type":"string","required":true}]}"#,
        ])
        .current_dir(&project_root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    println!("stdout: {}", stdout);

    assert!(result.status.success(), "Command should succeed in dry-run");
    assert!(stdout.contains("success"), "Expected success in output");
    assert!(
        stdout.contains("db/schema/test.ts"),
        "Should list would-be-created file"
    );

    let schema_file = project_root.join("db/schema/test.ts");
    assert!(
        !schema_file.exists(),
        "File should NOT be created in dry-run mode"
    );
}

#[test]
fn test_missing_package_json_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    let exe_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("tsx.exe");

    let result = Command::new(&exe_path)
        .args([
            "generate",
            "schema",
            "--json",
            r#"{"name":"test","fields":[{"name":"title","type":"string","required":true}]}"#,
        ])
        .current_dir(&project_root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    println!("stdout: {}", stdout);

    assert!(stdout.contains("\"success\": false"), "Expected failure");
    assert!(
        stdout.contains("package.json"),
        "Error should mention package.json"
    );
}

#[test]
fn test_add_feature_creates_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    fs::write(project_root.join("package.json"), "{}").unwrap();

    let exe_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("tsx.exe");

    let result = Command::new(&exe_path)
        .args([
            "generate", "feature",
            "--json",
            r#"{"name":"posts","fields":[{"name":"title","type":"string"}],"operations":["list","create"],"auth":false}"#,
        ])
        .current_dir(&project_root)
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    println!("stdout: {}", stdout);
    println!("stderr: {}", stderr);

    assert!(result.status.success(), "Command failed: {}", stderr);

    assert!(
        project_root.join("db/schema/posts.ts").exists(),
        "Schema should be created"
    );
    assert!(
        project_root.join("server-functions/postslist.ts").exists(),
        "List server fn should be created"
    );
    assert!(
        project_root
            .join("server-functions/postscreate.ts")
            .exists(),
        "Create server fn should be created"
    );
    assert!(
        project_root
            .join("components/posts/posts-table.tsx")
            .exists(),
        "Table should be created"
    );
    assert!(
        project_root.join("routes/posts.tsx").exists(),
        "Index page should be created"
    );
    assert!(
        project_root.join("routes/posts-$id.tsx").exists(),
        "Detail page should be created"
    );
}

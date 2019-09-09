use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::{env, fs};

use fs_extra::dir;
use toml::Value;

/// Setting an env var with this name (e.g. 'CL_WASM_DIR=../resources'), will cause the compiled
/// Wasm files to be copied there rather than the default
/// '.../execution-engine/target/wasm32-unknown-unknown/release'.
const OUTPUT_DIR_ENV_VAR_NAME: &str = "CL_WASM_DIR";

/// List of all targets which should always be excluded, i.e. ones which don't build to a compiled
/// Wasm file.
const EXCLUDED_TARGETS: [&str; 1] = ["create-test-node-shared"];

/// Reads the given Cargo.toml and parses the contents to a `toml::Value`.
fn parse_manifest<T: AsRef<Path> + Copy>(manifest: T) -> Value {
    let cargo_contents = fs::read_to_string(manifest)
        .expect(&format!("Expected to read {}", manifest.as_ref().display()));
    cargo_contents.parse::<Value>().expect(&format!(
        "Expected to parse {} as TOML",
        manifest.as_ref().display()
    ))
}

/// Watch all Rust files in 'contracts' for changes.
fn watch_for_file_changes(contracts_dir: &Path) {
    let dir_contents =
        dir::get_dir_content(contracts_dir).expect("Expected to read 'contracts' dir");
    for file in dir_contents.files {
        if file.ends_with("Cargo.toml") || file.ends_with(".rs") {
            println!("cargo:rerun-if-changed={}", file);
        }
    }
}

/// For the given `feature`, checks that a corresponding subdirectory exists and exits the process
/// with an error message if not.
///
/// Note that feature 'default' is skipped from this check.
fn validate(feature: &str, contracts_dir: &Path) {
    if feature == "default" {
        return;
    }

    let expected_subdir = contracts_dir.join(feature);
    if !fs::metadata(&expected_subdir)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
    {
        println!(
            "Feature '{}' in {} implies a directory {} should exist.",
            feature,
            contracts_dir.join("Cargo.toml").display(),
            expected_subdir.display()
        );
        process::exit(1);
    }
}

/// For the given feature, returns `true` if it has been enabled for this build.  Ignores the
/// 'default' feature, as this doesn't correspond to any subdirectory.
fn is_enabled(feature: &str) -> bool {
    feature != "default" && env::var(format!("CARGO_FEATURE_{}", feature.to_uppercase())).is_ok()
}

/// Returns the list of enabled features for this build.  Each corresponds to a subdirectory of
/// '.../execution-engine/contracts' and will cause all contracts in that subdirectory to be built.
fn enabled_features(contracts_dir: &Path) -> Vec<String> {
    let manifest_path = contracts_dir.join("Cargo.toml");
    let parsed_contents = parse_manifest(&manifest_path);
    parsed_contents["features"]
        .as_table()
        .expect(&format!(
            "Expected 'features' section in {}",
            manifest_path.display()
        ))
        .keys()
        .filter(|feature| {
            validate(feature, contracts_dir);
            is_enabled(feature)
        })
        .cloned()
        .collect()
}

/// Returns `true` if `target` is not a member of `EXCLUDED_TARGETS`.
fn is_not_excluded(target: &String) -> bool {
    !EXCLUDED_TARGETS.contains(&target.as_str())
}

/// Returns the list of Rust targets contained in `subdir` and its children.
fn targets(subdir: &Path) -> Vec<String> {
    let dir_contents =
        dir::get_dir_content(subdir).expect(&format!("Expected to read {}", subdir.display()));
    dir_contents
        .files
        .iter()
        .filter(|file| file.ends_with("Cargo.toml"))
        .map(|manifest_path| {
            let parsed_contents = parse_manifest(manifest_path);
            parsed_contents["package"]["name"]
                .as_str()
                .expect(&format!(
                    "Expected {} to have a 'package.name'",
                    manifest_path
                ))
                .to_string()
        })
        .filter(is_not_excluded)
        .collect()
}

/// Returns the path of the output directory, i.e. where the compiled Wasm files will be copied to.
/// By default, this will be the normal top-level Wasm directory, i.e.
/// '.../execution-engine/target/wasm32-unknown-unknown/release', but this can be overridden by
/// setting the env var 'CL_WASM_DIR' to a different directory.
fn get_output_dir(contracts_dir: &Path) -> PathBuf {
    println!("cargo:rerun-if-env-changed={}", OUTPUT_DIR_ENV_VAR_NAME);
    match env::var(OUTPUT_DIR_ENV_VAR_NAME) {
        Ok(output_dir) => PathBuf::from(output_dir),
        Err(_) => contracts_dir
            .ancestors()
            .skip(1)
            .next()
            .unwrap()
            .join("target")
            .join("wasm32-unknown-unknown")
            .join("release"),
    }
}

fn copy_wasm_files(source_dir: &Path, destination_dir: &Path, targets: &[String]) {
    let _ = fs::create_dir_all(destination_dir);
    for target in targets {
        let filename = format!("{}.wasm", target.replace("-", "_"));
        let source_file = source_dir.join(&filename);
        let destination_file = destination_dir.join(&filename);
        let copy_result = fs::copy(&source_file, &destination_file);
        assert!(
            copy_result.is_ok(),
            "\n\nFailed to copy {} to {}: {:?}\n\nShould '{}' be added to 'EXCLUDED_TARGETS' in \
             'execution-engine/contracts/build.rs'?\n\n",
            source_file.display(),
            destination_file.display(),
            copy_result,
            target
        );
    }
}

fn main() {
    // Full path to the cargo binary.
    let cargo = env::var("CARGO").expect("Expected env var 'CARGO' to be set");
    // Full path to the 'execution-engine/contracts' dir.
    let contracts_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Expected env var 'CARGO_MANIFEST_DIR' to be set"),
    );

    watch_for_file_changes(&contracts_dir);

    let mut build_args = vec![
        "build".to_string(),
        "--release".to_string(),
        "--target=wasm32-unknown-unknown".to_string(),
    ];

    let enabled_features = enabled_features(&contracts_dir);
    let build_targets = enabled_features
        .iter()
        .flat_map(|feature| targets(&contracts_dir.join(feature)))
        .collect::<Vec<String>>();
    build_args.extend(
        build_targets
            .iter()
            .map(|target| format!("--package={}", target)),
    );

    // We can't build the contracts right into the normal target dir since cargo has a lock on
    // this while building e.g. engine-tests or engine-core.  Instead, we'll build to
    // '.../execution-engine/contracts/target/built-contracts' and then copy the resulting
    // 'wasm32-unknown-unknown' dir from there to '.../execution-engine/target'.

    let target_dir = contracts_dir.join("target").join("built-contracts");
    build_args.push(format!(
        "--target-dir={}",
        target_dir.to_str().expect("Expected valid unicode")
    ));

    eprintln!("{:?}", build_args);

    // Build the contracts.
    let output = Command::new(&cargo)
        .args(build_args)
        .output()
        .expect("Expected to build Wasm contracts");
    assert!(
        output.status.success(),
        "Failed to build Wasm contracts:\n{:?}",
        output
    );

    // Full path to the dir currently holding the compiled Wasm files, i.e.
    // '.../execution-engine/contracts/target/built-contracts/wasm32-unknown-unknown/release'.
    let compiled_wasm_dir = target_dir.join("wasm32-unknown-unknown").join("release");

    let output_dir = get_output_dir(&contracts_dir);
    copy_wasm_files(&compiled_wasm_dir, &output_dir, &build_targets);
}

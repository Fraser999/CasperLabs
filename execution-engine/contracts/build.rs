use fs_extra::dir::{self, CopyOptions};
use std::{env, path::PathBuf, process::Command};

fn main() {
    // Full path to the cargo binary.
    let cargo = env::var("CARGO").expect("Expected env var 'CARGO' to be set.");
    // Full path to the "execution-engine/contracts" dir.
    let contracts_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Expected env var 'CARGO_MANIFEST_DIR' to be set."),
    );

    // Gather a list of all Cargo.toml files except the one running this script, and watch all Rust
    // files in "contracts" for changes.
    let mut manifests = vec![];
    let dir_contents =
        dir::get_dir_content(&contracts_dir).expect("Expected to read \"contracts\" dir.");
    for file in dir_contents.files {
        let is_manifest = file.ends_with("Cargo.toml") && !file.ends_with("contracts/Cargo.toml");
        if is_manifest || file.ends_with(".rs") {
            println!("cargo:rerun-if-changed={}", file);
        }
        if is_manifest {
            manifests.push(file);
        }
    }

    // Full path to the current top-level target dir, e.g. ".../execution-engine/target".
    let target_dir = contracts_dir
        .ancestors()
        .skip(1)
        .next()
        .unwrap()
        .join("target");

    // We can't build the contracts right into `target_dir` since cargo has a lock on
    // this while building e.g. engine-tests.  Instead, we'll build to
    // ".../execution-engine/target/contracts" and then copy the resulting "wasm32-unknown-unknown"
    // dir from there to ".../execution-engine/target".

    // Full path to the contracts target-dir, e.g. ".../execution-engine/target/contracts".
    let contracts_target_dir = target_dir.join("contracts");

    // Build the contracts.
    for manifest in manifests {
        let output = Command::new(&cargo)
            .args(&[
                "build",
                "--release",
                "--manifest-path",
                &manifest,
                "--target-dir",
                contracts_target_dir
                    .to_str()
                    .expect("Expected valid unicode."),
            ])
            .current_dir(&contracts_dir)
            .output()
            .expect("Expected to build Wasm contract.");
        assert!(
            output.status.success(),
            "Failed to build Wasm contracts:\n{:?}",
            output
        );
    }

    // Full path to the dir currently holding the compiled Wasm files, e.g.
    // ".../execution-engine/target/contracts/wasm32-unknown-unknown".
    let compiled_wasm_dir = contracts_target_dir.join("wasm32-unknown-unknown");

    // Copy the Wasm files.
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    copy_options.skip_exist = true;
    let copy_result = dir::copy(compiled_wasm_dir, target_dir, &copy_options);
    assert!(
        copy_result.is_ok(),
        "Failed to copy output dir: {:?}",
        copy_result
    );
}

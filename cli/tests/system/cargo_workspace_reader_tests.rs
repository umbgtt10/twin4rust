// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use cargo_metadata::Metadata;
use serde_json::Value;
use serde_json::from_value;
use serde_json::json;
use std::path::PathBuf;
use twin4rust::config::Config;
use twin4rust::manifest_resolver::ManifestResolver;
use twin4rust::traits::workspace_reader::WorkspaceReader;
use twin4rust::workspace::source_path::SourcePath;
use twin4rust_cli::system::cargo_workspace_reader::CargoWorkspaceReader;

fn metadata(resolve: Value) -> Metadata {
    from_value(json!({
        "packages": [
            {
                "name": "root-pkg",
                "version": "0.1.0",
                "id": "root-pkg 0.1.0 (path+file:///project)",
                "license": null,
                "license_file": null,
                "description": null,
                "source": null,
                "dependencies": [],
                "targets": [
                    {
                        "name": "root_pkg",
                        "kind": ["lib"],
                        "crate_types": ["lib"],
                        "src_path": "/project/src/lib.rs",
                        "edition": "2021",
                        "doc": true,
                        "doctest": true,
                        "test": true
                    },
                    {
                        "name": "build-script-build",
                        "kind": ["custom-build"],
                        "crate_types": ["bin"],
                        "src_path": "/project/build.rs",
                        "edition": "2021",
                        "doc": false,
                        "doctest": false,
                        "test": false
                    }
                ],
                "features": {},
                "manifest_path": "/project/Cargo.toml",
                "metadata": null,
                "publish": null,
                "authors": [],
                "categories": [],
                "keywords": [],
                "readme": null,
                "repository": null,
                "homepage": null,
                "documentation": null,
                "edition": "2021",
                "links": null
            }
        ],
        "workspace_members": ["root-pkg 0.1.0 (path+file:///project)"],
        "workspace_default_members": ["root-pkg 0.1.0 (path+file:///project)"],
        "resolve": resolve,
        "target_directory": "/project/target",
        "version": 1,
        "workspace_root": "/elsewhere",
        "metadata": null
    }))
    .expect("valid metadata")
}

#[test]
fn describe_against_this_workspace_lists_every_member() {
    // Arrange
    let reader = CargoWorkspaceReader::new(Some(PathBuf::from("../Cargo.toml")));

    // Act
    let workspace = reader.describe().expect("describe");

    // Assert
    let mut names: Vec<&str> = workspace
        .packages
        .iter()
        .map(|package| package.name.as_str())
        .collect();
    names.sort_unstable();
    assert_eq!(names, vec!["cargo-twin4rust", "twin4rust-core", "xtask"]);
    assert_eq!(workspace.root_package, None);
}

#[test]
fn describe_with_a_manifest_that_does_not_exist_fails_to_read_cargo_metadata() {
    // Arrange
    let reader = CargoWorkspaceReader::new(Some(PathBuf::from(
        "C:\\nonexistent_path_12345\\Cargo.toml",
    )));

    // Act
    let result = reader.describe();

    // Assert
    assert_eq!(
        result.expect_err("no manifest").to_string(),
        "failed to read Cargo metadata"
    );
}

#[test]
fn resolve_against_this_package_returns_it_with_its_source_root() {
    // Arrange
    let config = Config {
        packages: vec![String::from("cargo-twin4rust")],
    };
    let reader = CargoWorkspaceReader::new(Some(PathBuf::from("Cargo.toml")));
    let resolver = ManifestResolver::new(&reader);

    // Act
    let packages = resolver.resolve(&config).expect("resolve packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "cargo-twin4rust");
    assert!(!packages[0].source_roots.is_empty());
}

#[test]
fn translate_carries_each_targets_kinds_and_source_path() {
    // Arrange
    let metadata = metadata(Value::Null);

    // Act
    let workspace = CargoWorkspaceReader::translate(&metadata);

    // Assert
    let package = &workspace.packages[0];
    assert_eq!(package.name, "root-pkg");
    assert_eq!(
        package.manifest_path,
        SourcePath::new(String::from("/project/Cargo.toml"))
    );
    assert_eq!(package.targets[0].kinds, vec![String::from("lib")]);
    assert_eq!(
        package.targets[0].src_path,
        SourcePath::new(String::from("/project/src/lib.rs"))
    );
    assert_eq!(package.targets[1].kinds, vec![String::from("custom-build")]);
}

#[test]
fn translate_names_the_root_package_cargo_resolved() {
    // Arrange
    let metadata = metadata(json!({
        "nodes": [
            {
                "id": "root-pkg 0.1.0 (path+file:///project)",
                "dependencies": [],
                "deps": []
            }
        ],
        "root": "root-pkg 0.1.0 (path+file:///project)"
    }));

    // Act
    let workspace = CargoWorkspaceReader::translate(&metadata);

    // Assert
    assert_eq!(workspace.root_package, Some(String::from("root-pkg")));
}

#[test]
fn translate_without_a_resolved_root_outside_the_workspace_root_names_none() {
    // Arrange
    let metadata = metadata(Value::Null);

    // Act
    let workspace = CargoWorkspaceReader::translate(&metadata);

    // Assert
    assert_eq!(workspace.root_package, None);
}

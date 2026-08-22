// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use cargo_metadata::Metadata;
use serde_json::from_value;
use std::path::Path;
use std::path::PathBuf;
use twin4rust::config::Config;
use twin4rust::manifest_resolver::ManifestResolver;
use twin4rust::target_root_collector::TargetRootCollector;

fn bench_target(src_path: &str) -> cargo_metadata::Target {
    from_value(serde_json::json!({
        "name": "bench_heavy",
        "kind": ["bench"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": false
    }))
    .expect("valid Target")
}

fn bin_target(src_path: &str) -> cargo_metadata::Target {
    from_value(serde_json::json!({
        "name": "my_bin",
        "kind": ["bin"],
        "crate_types": ["bin"],
        "src_path": src_path,
        "edition": "2021",
        "doc": true,
        "doctest": false,
        "test": true
    }))
    .expect("valid Target")
}

fn build_target(src_path: &str) -> cargo_metadata::Target {
    from_value(serde_json::json!({
        "name": "build_script",
        "kind": ["custom-build"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": false
    }))
    .expect("valid Target")
}

fn example_target(src_path: &str) -> cargo_metadata::Target {
    from_value(serde_json::json!({
        "name": "example_demo",
        "kind": ["example"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": false
    }))
    .expect("valid Target")
}

fn lib_target(src_path: &str) -> cargo_metadata::Target {
    from_value(serde_json::json!({
        "name": "my_crate",
        "kind": ["lib"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": true,
        "doctest": true,
        "test": true
    }))
    .expect("valid Target")
}

fn metadata_with_package(name: &str, root_id: Option<&str>) -> Metadata {
    let package_json = serde_json::json!({
        "name": name,
        "version": "0.1.0",
        "id": format!("{} 0.1.0 (path+file:///project/{})", name, name),
        "license": null,
        "license_file": null,
        "description": null,
        "source": null,
        "dependencies": [],
        "targets": [],
        "features": {},
        "manifest_path": format!("/project/{}/Cargo.toml", name),
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
    });

    let resolve = root_id.map(|id| {
        serde_json::json!({
            "nodes": [{
                "id": id,
                "dependencies": [],
                "deps": []
            }],
            "root": id
        })
    });

    let workspace_members: Vec<String> = root_id.into_iter().map(|id| id.to_string()).collect();

    from_value(serde_json::json!({
        "packages": [package_json],
        "workspace_members": workspace_members,
        "workspace_default_members": workspace_members,
        "resolve": resolve,
        "target_directory": "/project/target",
        "version": 1,
        "workspace_root": "/project",
        "metadata": null
    }))
    .expect("valid Metadata")
}

fn test_target(src_path: &str) -> cargo_metadata::Target {
    from_value(serde_json::json!({
        "name": "test_integration",
        "kind": ["test"],
        "crate_types": ["lib"],
        "src_path": src_path,
        "edition": "2021",
        "doc": false,
        "doctest": false,
        "test": true
    }))
    .expect("valid Target")
}

#[test]
fn is_production_target_bench_returns_false() {
    // Arrange
    let target = bench_target("/project/benches/bench.rs");

    // Act
    let result = TargetRootCollector::is_production_target(&target);

    // Assert
    assert!(!result);
}

#[test]
fn is_production_target_bin_returns_true() {
    // Arrange
    let target = bin_target("/project/src/main.rs");

    // Act
    let result = TargetRootCollector::is_production_target(&target);

    // Assert
    assert!(result);
}

#[test]
fn is_production_target_custom_build_returns_false() {
    // Arrange
    let target = build_target("/project/build.rs");

    // Act
    let result = TargetRootCollector::is_production_target(&target);

    // Assert
    assert!(!result);
}

#[test]
fn is_production_target_example_returns_false() {
    // Arrange
    let target = example_target("/project/examples/demo.rs");

    // Act
    let result = TargetRootCollector::is_production_target(&target);

    // Assert
    assert!(!result);
}

#[test]
fn is_production_target_lib_returns_true() {
    // Arrange
    let target = lib_target("/project/src/lib.rs");

    // Act
    let result = TargetRootCollector::is_production_target(&target);

    // Assert
    assert!(result);
}

#[test]
fn is_production_target_test_returns_false() {
    // Arrange
    let target = test_target("/project/tests/integration.rs");

    // Act
    let result = TargetRootCollector::is_production_target(&target);

    // Assert
    assert!(!result);
}

#[test]
fn relative_file_inside_base_dir_strips_prefix() {
    // Arrange
    let base_dir = Path::new("/home/user/project");
    let file_path = Path::new("/home/user/project/src/lib.rs");

    // Act
    let relative = ManifestResolver::relative_file(base_dir, file_path);

    // Assert
    assert_eq!(relative, "src/lib.rs");
}

#[test]
fn relative_file_normalizes_backslashes_to_forward_slashes() {
    // Arrange
    let base_dir = Path::new("C:\\Users\\user\\project");
    let file_path = Path::new("C:\\Users\\user\\project\\src\\lib.rs");

    // Act
    let relative = ManifestResolver::relative_file(base_dir, file_path);

    // Assert
    assert_eq!(relative, "src/lib.rs");
    assert!(!relative.contains('\\'));
}

#[test]
fn relative_file_outside_base_dir_returns_full_path() {
    // Arrange
    let base_dir = Path::new("/home/user/project");
    let file_path = Path::new("/tmp/other.rs");

    // Act
    let relative = ManifestResolver::relative_file(base_dir, file_path);

    // Assert
    assert_eq!(relative, "/tmp/other.rs");
}

#[test]
fn relative_file_same_path_returns_empty_string() {
    // Arrange
    let base_dir = Path::new("/home/user/project");
    let file_path = Path::new("/home/user/project");

    // Act
    let relative = ManifestResolver::relative_file(base_dir, file_path);

    // Assert
    assert_eq!(relative, "");
}

#[test]
fn resolve_against_this_package_returns_it_with_its_source_root() {
    // Arrange
    let config = Config {
        manifest_path: Some(PathBuf::from("Cargo.toml")),
        packages: vec![String::from("cargo-twin4rust")],
    };
    let resolver = ManifestResolver::new(config);

    // Act
    let packages = resolver.resolve().expect("resolve packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "cargo-twin4rust");
    assert!(!packages[0].source_roots.is_empty());
}

#[test]
fn resolve_with_a_package_that_is_not_in_the_manifest_returns_an_error() {
    // Arrange
    let config = Config {
        manifest_path: Some(PathBuf::from("Cargo.toml")),
        packages: vec![String::from("no-such-package")],
    };
    let resolver = ManifestResolver::new(config);

    // Act
    let result = resolver.resolve();

    // Assert
    assert!(result.is_err());
}

#[test]
fn select_packages_with_requested_name_returns_matching() {
    // Arrange
    let metadata = metadata_with_package("foo", None);

    // Act
    let packages = ManifestResolver::select_packages(&metadata, &["foo".to_string()])
        .expect("select_packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "foo");
}

#[test]
fn select_packages_with_unknown_name_returns_error() {
    // Arrange
    let metadata = metadata_with_package("foo", None);

    // Act
    let result = ManifestResolver::select_packages(&metadata, &["nonexistent".to_string()]);

    // Assert
    assert!(result.is_err());
}

#[test]
fn select_packages_without_requested_uses_root() {
    // Arrange
    let metadata = metadata_with_package(
        "root-pkg",
        Some("root-pkg 0.1.0 (path+file:///project/root-pkg)"),
    );

    // Act
    let packages = ManifestResolver::select_packages(&metadata, &[]).expect("select_packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "root-pkg");
}

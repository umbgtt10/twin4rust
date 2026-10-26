// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use crate::support::in_memory_workspace_reader::InMemoryWorkspaceReader;
use crate::support::in_memory_workspace_reader::library_package;
use twin4rust::config::Config;
use twin4rust::manifest_resolver::ManifestResolver;
use twin4rust::target_root_collector::TargetRootCollector;
use twin4rust::workspace::package_description::PackageDescription;
use twin4rust::workspace::source_path::SourcePath;
use twin4rust::workspace::target_description::TargetDescription;
use twin4rust::workspace::workspace_description::WorkspaceDescription;

fn bench_target(src_path: &str) -> TargetDescription {
    target("bench", src_path)
}

fn bin_target(src_path: &str) -> TargetDescription {
    target("bin", src_path)
}

fn build_target(src_path: &str) -> TargetDescription {
    target("custom-build", src_path)
}

fn example_target(src_path: &str) -> TargetDescription {
    target("example", src_path)
}

fn lib_target(src_path: &str) -> TargetDescription {
    target("lib", src_path)
}

fn path(text: &str) -> SourcePath {
    SourcePath::new(String::from(text))
}

fn target(kind: &str, src_path: &str) -> TargetDescription {
    TargetDescription {
        kinds: vec![String::from(kind)],
        src_path: path(src_path),
    }
}

fn test_target(src_path: &str) -> TargetDescription {
    target("test", src_path)
}

fn workspace_with_package(name: &str, root: Option<&str>) -> WorkspaceDescription {
    WorkspaceDescription {
        packages: vec![PackageDescription {
            name: String::from(name),
            manifest_path: path(&format!("/project/{name}/Cargo.toml")),
            targets: vec![],
        }],
        root_package: root.map(String::from),
    }
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
    let base_dir = path("/home/user/project");
    let file_path = path("/home/user/project/src/lib.rs");

    // Act
    let relative = ManifestResolver::relative_file(&base_dir, &file_path);

    // Assert
    assert_eq!(relative, "src/lib.rs");
}

// Built with `join` rather than written as a literal `C:\Users\...`. Windows
// reads that literal as a path and Unix reads it as one filename, so the
// prefix strips on one platform and not on the other, and the test passes for
// the wrong reason. Joining leaves a backslash inside the relative portion on
// both, which is the thing being normalized.
#[test]
fn relative_file_normalizes_backslashes_to_forward_slashes() {
    // Arrange
    let base_dir = path("project");
    let file_path = base_dir.join("src\\lib.rs");

    // Act
    let relative = ManifestResolver::relative_file(&base_dir, &file_path);

    // Assert
    assert_eq!(relative, "src/lib.rs");
    assert!(!relative.contains('\\'));
}

#[test]
fn relative_file_outside_base_dir_returns_full_path() {
    // Arrange
    let base_dir = path("/home/user/project");
    let file_path = path("/tmp/other.rs");

    // Act
    let relative = ManifestResolver::relative_file(&base_dir, &file_path);

    // Assert
    assert_eq!(relative, "/tmp/other.rs");
}

#[test]
fn relative_file_same_path_returns_empty_string() {
    // Arrange
    let base_dir = path("/home/user/project");
    let file_path = path("/home/user/project");

    // Act
    let relative = ManifestResolver::relative_file(&base_dir, &file_path);

    // Assert
    assert_eq!(relative, "");
}

#[test]
fn resolve_for_a_package_with_no_production_target_falls_back_to_its_src() {
    // Arrange
    let reader = InMemoryWorkspaceReader::new(workspace_with_package("foo", Some("foo")));
    let config = Config { packages: vec![] };

    // Act
    let packages = ManifestResolver::new(&reader)
        .resolve(&config)
        .expect("resolve packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].manifest_dir, path("/project/foo"));
    assert_eq!(packages[0].source_roots, vec![path("/project/foo/src")]);
}

#[test]
fn resolve_for_a_validation_package_drops_its_src_root() {
    // Arrange
    let reader = InMemoryWorkspaceReader::new(WorkspaceDescription {
        packages: vec![library_package("raft-validation", "/w/raft-validation")],
        root_package: None,
    });
    let config = Config {
        packages: vec![String::from("raft-validation")],
    };

    // Act
    let packages = ManifestResolver::new(&reader)
        .resolve(&config)
        .expect("resolve packages");

    // Assert
    assert_eq!(packages[0].name, "raft-validation");
    assert!(packages[0].source_roots.is_empty());
}

#[test]
fn resolve_with_a_package_that_is_not_in_the_manifest_returns_an_error() {
    // Arrange
    let reader = InMemoryWorkspaceReader::new(workspace_with_package("foo", None));
    let config = Config {
        packages: vec![String::from("no-such-package")],
    };
    let resolver = ManifestResolver::new(&reader);

    // Act
    let result = resolver.resolve(&config);

    // Assert
    assert!(result.is_err());
}

#[test]
fn resolve_with_a_requested_package_returns_it_with_its_source_root() {
    // Arrange
    let reader = InMemoryWorkspaceReader::new(WorkspaceDescription {
        packages: vec![
            library_package("alpha", "/w/alpha"),
            library_package("beta", "/w/beta"),
        ],
        root_package: None,
    });
    let config = Config {
        packages: vec![String::from("beta")],
    };

    // Act
    let packages = ManifestResolver::new(&reader)
        .resolve(&config)
        .expect("resolve packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "beta");
    assert_eq!(packages[0].source_roots, vec![path("/w/beta/src")]);
}

#[test]
fn resolve_with_a_workspace_that_cannot_be_read_returns_the_error() {
    // Arrange
    let reader = InMemoryWorkspaceReader::failing();
    let config = Config { packages: vec![] };

    // Act
    let result = ManifestResolver::new(&reader).resolve(&config);

    // Assert
    assert_eq!(
        result.err().map(|failure| failure.to_string()),
        Some(String::from("failed to read Cargo metadata"))
    );
}

#[test]
fn select_packages_with_requested_name_returns_matching() {
    // Arrange
    let workspace = workspace_with_package("foo", None);

    // Act
    let packages = ManifestResolver::select_packages(&workspace, &["foo".to_string()])
        .expect("select_packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "foo");
}

#[test]
fn select_packages_with_unknown_name_returns_error() {
    // Arrange
    let workspace = workspace_with_package("foo", None);

    // Act
    let result = ManifestResolver::select_packages(&workspace, &["nonexistent".to_string()]);

    // Assert
    assert!(result.is_err());
}

#[test]
fn select_packages_without_requested_and_without_a_root_asks_for_a_package() {
    // Arrange
    let workspace = workspace_with_package("foo", None);

    // Act
    let result = ManifestResolver::select_packages(&workspace, &[]);

    // Assert
    assert_eq!(
        result.err().map(|failure| failure.to_string()),
        Some(String::from(
            "manifest contains multiple packages; pass --package <name>"
        ))
    );
}

#[test]
fn select_packages_without_requested_uses_root() {
    // Arrange
    let workspace = workspace_with_package("root-pkg", Some("root-pkg"));

    // Act
    let packages = ManifestResolver::select_packages(&workspace, &[]).expect("select_packages");

    // Assert
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].name, "root-pkg");
}

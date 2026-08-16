// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use cargo_metadata::Target;

#[derive(Default)]
pub struct TargetRootCollector {
    source_roots: BTreeSet<PathBuf>,
}

impl TargetRootCollector {
    pub fn new() -> Self {
        Self {
            source_roots: BTreeSet::new(),
        }
    }

    pub fn collect_from_targets(&mut self, targets: &[Target]) {
        for target in targets {
            if !Self::is_production_target(target) {
                continue;
            }

            let src_path = target.src_path.clone().into_std_path_buf();
            if let Some(parent) = src_path.parent() {
                self.source_roots.insert(parent.to_path_buf());
            }
        }
    }

    pub fn ensure_fallback(&mut self, manifest_dir: &Path) {
        if self.source_roots.is_empty() {
            self.source_roots.insert(manifest_dir.join("src"));
        }
    }

    pub fn into_roots(self) -> Vec<PathBuf> {
        self.source_roots.into_iter().collect()
    }

    pub fn is_production_target(target: &Target) -> bool {
        let kinds = target
            .kind
            .iter()
            .map(|kind| kind.to_string())
            .collect::<Vec<_>>();

        if kinds
            .iter()
            .any(|kind| matches!(kind.as_str(), "test" | "bench" | "example" | "custom-build"))
        {
            return false;
        }

        kinds.iter().any(|kind| {
            matches!(
                kind.as_str(),
                "lib" | "bin" | "proc-macro" | "rlib" | "dylib" | "cdylib" | "staticlib"
            )
        })
    }
}

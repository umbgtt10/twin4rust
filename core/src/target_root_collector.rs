// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::workspace::source_path::SourcePath;
use crate::workspace::target_description::TargetDescription;

#[derive(Default)]
pub struct TargetRootCollector {
    source_roots: BTreeSet<SourcePath>,
}

impl TargetRootCollector {
    pub fn new() -> Self {
        Self {
            source_roots: BTreeSet::new(),
        }
    }

    pub fn collect_from_targets(&mut self, targets: &[TargetDescription]) {
        for target in targets {
            if !Self::is_production_target(target) {
                continue;
            }

            if let Some(parent) = target.src_path.parent() {
                self.source_roots.insert(parent);
            }
        }
    }

    pub fn ensure_fallback(&mut self, manifest_dir: &SourcePath) {
        if self.source_roots.is_empty() {
            self.source_roots.insert(manifest_dir.join("src"));
        }
    }

    pub fn into_roots(self) -> Vec<SourcePath> {
        let roots: Vec<SourcePath> = self.source_roots.into_iter().collect();
        roots
            .iter()
            .filter(|candidate| !Self::is_nested_in_any(candidate, &roots))
            .cloned()
            .collect()
    }

    pub fn is_production_target(target: &TargetDescription) -> bool {
        if target
            .kinds
            .iter()
            .any(|kind| matches!(kind.as_str(), "test" | "bench" | "example" | "custom-build"))
        {
            return false;
        }

        target.kinds.iter().any(|kind| {
            matches!(
                kind.as_str(),
                "lib" | "bin" | "proc-macro" | "rlib" | "dylib" | "cdylib" | "staticlib"
            )
        })
    }

    // A root inside another is redundant, because the walk recurses: every file
    // under it is already reached through the outer root, and left in it would
    // be read, parsed and reported twice. Compared component-wise rather than as
    // text, so `src_generated` is not mistaken for something inside `src`.
    fn is_nested_in_any(candidate: &SourcePath, roots: &[SourcePath]) -> bool {
        roots
            .iter()
            .any(|other| other != candidate && candidate.strip_prefix(other).is_some())
    }
}

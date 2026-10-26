// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use std::collections::BTreeSet;
use std::path::MAIN_SEPARATOR;

use cargo_metadata::Target;

use crate::workspace::source_path::SourcePath;

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

    pub fn collect_from_targets(&mut self, targets: &[Target]) {
        for target in targets {
            if !Self::is_production_target(target) {
                continue;
            }

            if let Some(parent) = Self::source_path(target.src_path.as_str()).parent() {
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

    // A root inside another is redundant, because the walk recurses: every file
    // under it is already reached through the outer root, and left in it would
    // be read, parsed and reported twice. Compared component-wise rather than as
    // text, so `src_generated` is not mistaken for something inside `src`.
    fn is_nested_in_any(candidate: &SourcePath, roots: &[SourcePath]) -> bool {
        roots
            .iter()
            .any(|other| other != candidate && candidate.strip_prefix(other).is_some())
    }

    // cargo's paths are UTF-8, so nothing is lost; the platform's separator
    // becomes the core's `/`.
    fn source_path(text: &str) -> SourcePath {
        SourcePath::new(text.replace(MAIN_SEPARATOR, "/"))
    }
}

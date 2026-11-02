// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

// What a source file's mirrored test means: demanded, allowed, or a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirrorExpectation {
    Optional,
    Required,
    Unearned,
}

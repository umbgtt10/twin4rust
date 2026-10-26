// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use core::error;
use core::fmt;

use syn::Error;

// A source that is not Rust syn can parse, and syn's reason.
#[derive(Debug)]
pub struct ParseFailure {
    pub cause: Error,
}

impl fmt::Display for ParseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("failed to parse Rust source")
    }
}

impl error::Error for ParseFailure {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(&self.cause)
    }
}

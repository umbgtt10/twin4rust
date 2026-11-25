// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use syn::{Expr, FnArg, ImplItemFn, Pat, Receiver, Stmt};

pub struct MethodParts<'a> {
    method: &'a ImplItemFn,
}

impl<'a> MethodParts<'a> {
    pub fn new(method: &'a ImplItemFn) -> Self {
        Self { method }
    }

    pub fn input_count(&self) -> usize {
        self.method.sig.inputs.len()
    }

    pub fn parameter_names(&self) -> Vec<String> {
        self.method
            .sig
            .inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Typed(typed) => match &*typed.pat {
                    Pat::Ident(binding) => Some(binding.ident.to_string()),
                    _ => None,
                },
                FnArg::Receiver(_) => None,
            })
            .collect()
    }

    pub fn receiver(&self) -> Option<&'a Receiver> {
        match self.method.sig.inputs.first() {
            Some(FnArg::Receiver(receiver)) => Some(receiver),
            _ => None,
        }
    }

    pub fn single_expression(&self) -> Option<&'a Expr> {
        match self.method.block.stmts.as_slice() {
            [Stmt::Expr(expression, None)] => Some(expression),
            _ => None,
        }
    }
}

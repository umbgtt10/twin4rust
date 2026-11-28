// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::{String, ToString};

use syn::{Expr, ImplItemFn, Path, PathSegment, ReceiverKind, ReturnType, Type, UnOp};

use crate::syntax::method_parts::MethodParts;

// The three shapes stern4rust's `records-and-objects` lets a record carry: a
// getter, a plain constructor and a `with_` step. A type whose methods are all
// of these holds data and nothing else, so the two tools must agree on them.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccessorClassifier;

impl AccessorClassifier {
    const FIELD_READS: [&'static str; 9] = [
        "as_deref",
        "as_path",
        "as_ref",
        "as_slice",
        "as_str",
        "clone",
        "to_owned",
        "to_string",
        "to_vec",
    ];

    pub fn new() -> Self {
        Self
    }

    pub fn is_accessor(&self, method: &ImplItemFn, type_name: &str) -> bool {
        let parts = MethodParts::new(method);
        Self::is_getter(&parts)
            || self.is_plain_constructor(method, type_name)
            || Self::is_step(&parts, type_name)
    }

    pub fn is_plain_constructor(&self, method: &ImplItemFn, type_name: &str) -> bool {
        let parts = MethodParts::new(method);
        if parts.receiver().is_some() || !Self::returns_own_type(method, type_name) {
            return false;
        }
        let parameters = parts.parameter_names();
        match parts.single_expression() {
            Some(Expr::Struct(literal)) => {
                literal.rest.is_none()
                    && Self::names_own_type(&literal.path, type_name)
                    && literal
                        .fields
                        .iter()
                        .all(|field| Self::is_bare_parameter(&field.expr, &parameters))
            }
            Some(Expr::Call(call)) => {
                matches!(&*call.func, Expr::Path(path) if Self::names_own_type(&path.path, type_name))
                    && call
                        .args
                        .iter()
                        .all(|argument| Self::is_bare_parameter(argument, &parameters))
            }
            Some(Expr::Path(path)) => {
                parameters.is_empty() && Self::names_own_type(&path.path, type_name)
            }
            _ => false,
        }
    }

    fn is_bare_parameter(expression: &Expr, parameters: &[String]) -> bool {
        matches!(expression, Expr::Path(path)
            if path.path.get_ident().is_some_and(|ident| parameters.contains(&ident.to_string())))
    }

    fn is_getter(parts: &MethodParts) -> bool {
        let shared = parts
            .receiver()
            .is_some_and(|receiver| matches!(receiver.kind, ReceiverKind::Reference(_, _, None)));
        shared
            && parts.input_count() == 1
            && parts.single_expression().is_some_and(Self::reads_one_field)
    }

    fn is_step(parts: &MethodParts, type_name: &str) -> bool {
        let by_value = parts
            .receiver()
            .is_some_and(|receiver| !matches!(receiver.kind, ReceiverKind::Reference(..)));
        let parameters = parts.parameter_names();
        let Some(Expr::Struct(literal)) = parts.single_expression() else {
            return false;
        };
        let rest_is_self = literal
            .rest
            .as_deref()
            .is_some_and(|rest| matches!(rest, Expr::Path(path) if path.path.is_ident("self")));
        by_value
            && parameters.len() == 1
            && rest_is_self
            && Self::names_own_type(&literal.path, type_name)
            && literal.fields.len() == 1
            && literal
                .fields
                .iter()
                .all(|field| Self::is_bare_parameter(&field.expr, &parameters))
    }

    fn names_own_type(path: &Path, type_name: &str) -> bool {
        let names_a_type =
            |segment: &PathSegment| segment.ident == "Self" || segment.ident == type_name;
        let mut segments = path.segments.iter().rev();
        segments.next().is_some_and(names_a_type) && !segments.any(names_a_type)
    }

    fn reads_one_field(expression: &Expr) -> bool {
        match expression {
            Expr::Paren(inner) => Self::reads_one_field(&inner.expr),
            Expr::Reference(inner) => {
                inner.mutability.is_none() && Self::reads_one_field(&inner.expr)
            }
            Expr::Unary(inner) => {
                matches!(inner.op, UnOp::Deref(_)) && Self::reads_one_field(&inner.expr)
            }
            Expr::Field(field) => {
                matches!(&*field.base, Expr::Path(path) if path.path.is_ident("self"))
            }
            Expr::MethodCall(call) => {
                call.args.is_empty()
                    && Self::FIELD_READS.contains(&call.method.to_string().as_str())
                    && Self::reads_one_field(&call.receiver)
            }
            _ => false,
        }
    }

    fn returns_own_type(method: &ImplItemFn, type_name: &str) -> bool {
        matches!(&method.sig.output, ReturnType::Type(_, output)
            if matches!(&**output, Type::Path(path) if path.qself.is_none() && Self::names_own_type(&path.path, type_name)))
    }
}

// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use syn::{Expr, ImplItem, ImplItemFn, Item, ItemImpl, ItemMod, ReturnType, Stmt, Type};

use crate::accessor_classifier::AccessorClassifier;

pub struct HumbleAdapterDetector {
    accessor_classifier: AccessorClassifier,
}

impl HumbleAdapterDetector {
    #[must_use]
    pub fn new(accessor_classifier: AccessorClassifier) -> Self {
        Self {
            accessor_classifier,
        }
    }

    // A single type whose inherent methods all either hold what they were given
    // or forward it onwards. See
    // docs/ADRs/ADR-HumbleAdaptersAtUntestableBoundaries.md.
    #[must_use]
    pub fn file_is_humble_adapter(&self, items: &[&Item]) -> bool {
        let Some(type_name) = Self::single_declared_type(items) else {
            return false;
        };

        let impls: Vec<&ItemImpl> = items
            .iter()
            .filter_map(|item| match item {
                Item::Impl(item_impl) => Some(item_impl),
                _ => None,
            })
            .collect();

        if impls.is_empty() || !items.iter().all(|item| Self::is_allowed_item(item)) {
            return false;
        }

        if !impls
            .iter()
            .all(|item_impl| self.impl_is_humble(item_impl, &type_name))
        {
            return false;
        }

        // At least one method must actually forward. Without this the rule would
        // swallow an empty `impl`, whose methods are all humble only because
        // there are none, and a file of constructors alone, which
        // `DataHolderDetector` answers for: it exempts one only when every
        // constructor is plain, where a humble `new` may compute its fields.
        Self::has_forwarding_method(&impls)
    }

    fn has_forwarding_method(impls: &[&ItemImpl]) -> bool {
        impls.iter().any(|item_impl| {
            item_impl
                .items
                .iter()
                .any(|impl_item| matches!(impl_item, ImplItem::Fn(method) if Self::is_forwarding(method)))
        })
    }

    fn impl_is_humble(&self, item_impl: &ItemImpl, type_name: &str) -> bool {
        if item_impl.trait_.is_some() {
            return false;
        }

        let Type::Path(type_path) = item_impl.self_ty.as_ref() else {
            return false;
        };

        let Some(segment) = type_path.path.segments.last() else {
            return false;
        };

        if segment.ident != type_name {
            return false;
        }

        item_impl.items.iter().all(|impl_item| match impl_item {
            ImplItem::Fn(method) => self.method_is_humble(method, type_name),
            _ => false,
        })
    }

    // `Item::Macro` and `Item::Verbatim` are absent for the same reason they are
    // absent from `DefinitionAnalyzer`: an unexpanded macro says nothing about
    // the behaviour it expands to, so one keeps the file in scope.
    fn is_allowed_item(item: &Item) -> bool {
        matches!(
            item,
            Item::Use(_)
                | Item::ExternCrate(_)
                | Item::Mod(ItemMod { content: None, .. })
                | Item::Const(_)
                | Item::Static(_)
                | Item::Type(_)
                | Item::Trait(_)
                | Item::Struct(_)
                | Item::Enum(_)
                | Item::Impl(_)
        )
    }

    // Returns nothing, and its whole body is one call. A method that returns a
    // value produces something worth asserting; one that returns nothing only
    // says where an effect lands.
    fn is_forwarding(method: &ImplItemFn) -> bool {
        if !Self::returns_nothing(&method.sig.output) {
            return false;
        }

        if method.block.stmts.len() != 1 {
            return false;
        }

        matches!(
            &method.block.stmts[0],
            Stmt::Expr(Expr::Call(_) | Expr::MethodCall(_), _)
        )
    }

    fn is_trivial_constructor(method: &ImplItemFn) -> bool {
        method.sig.ident == "new"
            && method.sig.generics.params.is_empty()
            && matches!(
                method.sig.output,
                ReturnType::Type(_, ref ty)
                    if matches!(ty.as_ref(), Type::Path(path) if path.path.is_ident("Self"))
            )
            && method.block.stmts.len() == 1
            && matches!(&method.block.stmts[0], Stmt::Expr(Expr::Struct(_), _))
    }

    // Whatever the data-holder rule takes as a record's constructor is taken
    // here too, so a file does not leave the exemption by renaming `new` or by
    // naming its type instead of `Self`. The trivial `new` stays beside it: its
    // fields may be any expression, so an adapter may convert what it is given.
    fn method_is_humble(&self, method: &ImplItemFn, type_name: &str) -> bool {
        Self::is_trivial_constructor(method)
            || self
                .accessor_classifier
                .is_plain_constructor(method, type_name)
            || Self::is_forwarding(method)
    }

    fn returns_nothing(output: &ReturnType) -> bool {
        match output {
            ReturnType::Default => true,
            ReturnType::Type(_, ty) => {
                matches!(ty.as_ref(), Type::Tuple(tuple) if tuple.elems.is_empty())
            }
        }
    }

    fn single_declared_type(items: &[&Item]) -> Option<String> {
        let mut names = items.iter().filter_map(|item| match item {
            Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
            Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
            _ => None,
        });
        let first = names.next()?;
        names.next().is_none().then_some(first)
    }
}

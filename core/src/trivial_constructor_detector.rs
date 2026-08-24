// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::{Expr, ImplItem, ImplItemFn, Item, ItemImpl, Stmt, Type};

use crate::behaviourless_impl_detector::BehaviourlessImplDetector;

#[derive(Default)]
pub struct TrivialConstructorDetector {
    behaviourless_impl_detector: BehaviourlessImplDetector,
}

impl TrivialConstructorDetector {
    pub fn new() -> Self {
        Self {
            behaviourless_impl_detector: BehaviourlessImplDetector::new(),
        }
    }

    pub fn file_has_trivial_constructor(&self, items: &[&Item]) -> bool {
        let mut type_name: Option<String> = None;
        let mut constructor_impl_count = 0;

        for item in items {
            if !self.process_item(item, &mut type_name, &mut constructor_impl_count) {
                return false;
            }
        }

        type_name.is_some() && constructor_impl_count == 1
    }

    fn process_item(
        &self,
        item: &Item,
        type_name: &mut Option<String>,
        constructor_impl_count: &mut usize,
    ) -> bool {
        if !self.try_track_type(item, type_name) {
            return false;
        }
        if self
            .try_track_constructor(item, type_name, constructor_impl_count)
            .is_some()
        {
            return true;
        }
        self.is_allowed_item(item)
    }

    pub fn impl_is_trivial_constructor_only(
        &self,
        item_impl: &ItemImpl,
        expected_name: &str,
    ) -> bool {
        if item_impl.trait_.is_some() {
            return false;
        }

        let Type::Path(type_path) = item_impl.self_ty.as_ref() else {
            return false;
        };

        let Some(target_segment) = type_path.path.segments.last() else {
            return false;
        };

        if target_segment.ident != expected_name {
            return false;
        }

        if item_impl.items.len() != 1 {
            return false;
        }

        let ImplItem::Fn(method) = &item_impl.items[0] else {
            return false;
        };

        if method.sig.ident != "new" {
            return false;
        }

        if !method.sig.generics.params.is_empty() {
            return false;
        }

        Self::returns_self(method) && Self::has_single_struct_expr_body(method)
    }

    fn returns_self(method: &ImplItemFn) -> bool {
        matches!(
            method.sig.output,
            syn::ReturnType::Type(_, ref ty)
                if matches!(ty.as_ref(), Type::Path(return_path) if return_path.path.is_ident("Self"))
        )
    }

    fn has_single_struct_expr_body(method: &ImplItemFn) -> bool {
        method.block.stmts.len() == 1
            && matches!(&method.block.stmts[0], Stmt::Expr(Expr::Struct(_), _))
    }

    fn try_track_type(&self, item: &Item, type_name: &mut Option<String>) -> bool {
        match item {
            Item::Struct(item_struct) => {
                if type_name.is_some() {
                    return false;
                }
                *type_name = Some(item_struct.ident.to_string());
            }
            Item::Enum(item_enum) => {
                if type_name.is_some() {
                    return false;
                }
                *type_name = Some(item_enum.ident.to_string());
            }
            _ => return true,
        }
        true
    }

    fn try_track_constructor(
        &self,
        item: &Item,
        type_name: &mut Option<String>,
        count: &mut usize,
    ) -> Option<()> {
        let Item::Impl(item_impl) = item else {
            return None;
        };
        let expected_name = type_name.as_deref()?;
        if !self.impl_is_trivial_constructor_only(item_impl, expected_name) {
            return None;
        }
        *count += 1;
        if *count > 1 {
            return None;
        }
        Some(())
    }

    // Every impl block other than the single trivial constructor must earn its
    // way past this, because rule 5 asks for exactly one inherent impl and no
    // other top-level behaviour. Admitting `Item::Impl(_)` outright granted the
    // exemption to any file holding a trivial `new`, whatever else its impls
    // did -- which is the shape of every adapter behind a seam.
    //
    // `Item::Macro` and `Item::Verbatim` are absent for the reason they are
    // absent from the other two detectors: an unexpanded macro says nothing
    // about the behaviour it expands to, so one keeps the file in scope.
    fn is_allowed_item(&self, item: &Item) -> bool {
        matches!(
            item,
            Item::Use(_)
                | Item::ExternCrate(_)
                | Item::Mod(syn::ItemMod { content: None, .. })
                | Item::Const(_)
                | Item::Static(_)
                | Item::Type(_)
                | Item::Trait(_)
                | Item::Struct(_)
                | Item::Enum(_)
        ) || self.behaviourless_impl_detector.is_behaviourless_impl(item)
    }
}

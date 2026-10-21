// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::{Expr, ImplItem, ImplItemFn, Item, ItemImpl, Stmt, Type};

use crate::behaviourless_impl_detector::BehaviourlessImplDetector;
use crate::constructor_scan_state::ConstructorScanState;

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
        let mut state = ConstructorScanState {
            type_name: None,
            constructor_impl_count: 0,
        };

        for item in items {
            match self.process_item(item, state) {
                Some(updated) => state = updated,
                None => return false,
            }
        }

        state.type_name.is_some() && state.constructor_impl_count == 1
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

    fn has_single_struct_expr_body(method: &ImplItemFn) -> bool {
        method.block.stmts.len() == 1
            && matches!(&method.block.stmts[0], Stmt::Expr(Expr::Struct(_), _))
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

    fn process_item(
        &self,
        item: &Item,
        state: ConstructorScanState,
    ) -> Option<ConstructorScanState> {
        let state = self.try_track_type(item, state)?;
        let (state, constructor_matched) = self.try_track_constructor(item, state);
        if constructor_matched.is_some() {
            return Some(state);
        }
        if self.is_allowed_item(item) {
            Some(state)
        } else {
            None
        }
    }

    fn returns_self(method: &ImplItemFn) -> bool {
        matches!(
            method.sig.output,
            syn::ReturnType::Type(_, ref ty)
                if matches!(ty.as_ref(), Type::Path(return_path) if return_path.path.is_ident("Self"))
        )
    }

    fn try_track_constructor(
        &self,
        item: &Item,
        state: ConstructorScanState,
    ) -> (ConstructorScanState, Option<()>) {
        let Item::Impl(item_impl) = item else {
            return (state, None);
        };
        let matched = match state.type_name.as_deref() {
            Some(expected_name) => self.impl_is_trivial_constructor_only(item_impl, expected_name),
            None => false,
        };
        if !matched {
            return (state, None);
        }
        let constructor_impl_count = state.constructor_impl_count + 1;
        let verdict = if constructor_impl_count > 1 {
            None
        } else {
            Some(())
        };
        (
            ConstructorScanState {
                type_name: state.type_name,
                constructor_impl_count,
            },
            verdict,
        )
    }

    fn try_track_type(
        &self,
        item: &Item,
        state: ConstructorScanState,
    ) -> Option<ConstructorScanState> {
        match item {
            Item::Struct(item_struct) => {
                if state.type_name.is_some() {
                    return None;
                }
                Some(ConstructorScanState {
                    type_name: Some(item_struct.ident.to_string()),
                    constructor_impl_count: state.constructor_impl_count,
                })
            }
            Item::Enum(item_enum) => {
                if state.type_name.is_some() {
                    return None;
                }
                Some(ConstructorScanState {
                    type_name: Some(item_enum.ident.to_string()),
                    constructor_impl_count: state.constructor_impl_count,
                })
            }
            _ => Some(state),
        }
    }
}

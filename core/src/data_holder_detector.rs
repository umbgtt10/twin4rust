// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::{String, ToString};

use syn::{ImplItem, Item, ItemImpl, Type};

use crate::accessor_classifier::AccessorClassifier;
use crate::behaviourless_impl_detector::BehaviourlessImplDetector;
use syn::ItemMod;

#[derive(Default)]
pub struct DataHolderDetector {
    accessor_classifier: AccessorClassifier,
    behaviourless_impl_detector: BehaviourlessImplDetector,
}

impl DataHolderDetector {
    pub fn new() -> Self {
        Self {
            accessor_classifier: AccessorClassifier::new(),
            behaviourless_impl_detector: BehaviourlessImplDetector::new(),
        }
    }

    pub fn file_is_data_holder(&self, items: &[&Item]) -> bool {
        let mut state = HolderScanState {
            type_name: None,
            accessor_count: 0,
        };

        for item in items {
            match self.process_item(item, state) {
                Some(updated) => state = updated,
                None => return false,
            }
        }

        state.type_name.is_some() && state.accessor_count > 0
    }

    pub fn impl_holds_only_accessors(&self, item_impl: &ItemImpl, expected_name: &str) -> bool {
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

        item_impl.items.iter().all(|member| {
            matches!(member, ImplItem::Fn(method)
                if self.accessor_classifier.is_accessor(method, expected_name))
        })
    }

    // Every impl block other than the type's own accessor impls must earn its
    // way past this: a trait impl carrying a method is behaviour, whatever the
    // accessors beside it do, because a trivial `new` beside a behaviour-bearing
    // trait impl is the shape of every adapter behind a seam.
    //
    // `Item::Macro` and `Item::Verbatim` are absent for the reason they are
    // absent from the other two detectors: an unexpanded macro says nothing
    // about the behaviour it expands to, so one keeps the file in scope.
    fn is_allowed_item(&self, item: &Item) -> bool {
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
        ) || self.behaviourless_impl_detector.is_behaviourless_impl(item)
    }

    fn process_item(&self, item: &Item, state: HolderScanState) -> Option<HolderScanState> {
        let state = self.try_track_type(item, state)?;
        let (state, accessors_matched) = self.try_track_accessor_impl(item, state);
        if accessors_matched || self.is_allowed_item(item) {
            Some(state)
        } else {
            None
        }
    }

    fn try_track_accessor_impl(
        &self,
        item: &Item,
        state: HolderScanState,
    ) -> (HolderScanState, bool) {
        let Item::Impl(item_impl) = item else {
            return (state, false);
        };
        let matched = match state.type_name.as_deref() {
            Some(expected_name) => self.impl_holds_only_accessors(item_impl, expected_name),
            None => false,
        };
        if !matched {
            return (state, false);
        }
        (
            HolderScanState {
                accessor_count: state.accessor_count + item_impl.items.len(),
                ..state
            },
            true,
        )
    }

    fn try_track_type(&self, item: &Item, state: HolderScanState) -> Option<HolderScanState> {
        let name = match item {
            Item::Struct(item_struct) => item_struct.ident.to_string(),
            Item::Enum(item_enum) => item_enum.ident.to_string(),
            _ => return Some(state),
        };
        if state.type_name.is_some() {
            return None;
        }
        Some(HolderScanState {
            type_name: Some(name),
            ..state
        })
    }
}

struct HolderScanState {
    pub type_name: Option<String>,
    pub accessor_count: usize,
}

// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use alloc::string::ToString;
use alloc::vec::Vec;

use quote::ToTokens;
use syn::{Attribute, File, Item, ItemImpl, ItemMod, TraitItem, parse_file};

use crate::behaviourless_impl_detector::BehaviourlessImplDetector;
use crate::data_holder_detector::DataHolderDetector;
use crate::humble_adapter_detector::HumbleAdapterDetector;
use crate::mirror_expectation::MirrorExpectation;
use crate::parse_failure::ParseFailure;

#[derive(Default)]
pub struct DefinitionAnalyzer {
    behaviourless_impl_detector: BehaviourlessImplDetector,
}

impl DefinitionAnalyzer {
    pub fn new() -> Self {
        Self {
            behaviourless_impl_detector: BehaviourlessImplDetector::new(),
        }
    }

    pub fn has_test_attrs(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            let path = attr.path();
            path.is_ident("cfg") && attr.meta.to_token_stream().to_string().contains("test")
        })
    }

    pub fn is_definition_only_source(&self, source: &str) -> Result<bool, ParseFailure> {
        Ok(self.mirror_expectation(source)? != MirrorExpectation::Required)
    }

    pub fn mirror_expectation(&self, source: &str) -> Result<MirrorExpectation, ParseFailure> {
        let syntax = parse_file(source).map_err(|cause| ParseFailure { cause })?;
        Ok(self.expectation_of(&syntax))
    }

    pub fn mod_file_is_import_only(&self, source: &str) -> Result<bool, ParseFailure> {
        let syntax = parse_file(source).map_err(|cause| ParseFailure { cause })?;
        let items = self.top_level_non_test_items(&syntax.items);
        Ok(items.iter().all(|item| {
            matches!(
                item,
                Item::Use(_) | Item::ExternCrate(_) | Item::Mod(ItemMod { content: None, .. })
            )
        }))
    }

    // `Item::Macro` and `Item::Verbatim` are deliberately absent: a top-level
    // macro expands to code this analyzer never sees, so its unexpanded form
    // says nothing about whether the file carries behaviour. One opaque item
    // keeps the whole file in scope.
    fn all_definition_only(&self, items: &[&Item]) -> bool {
        items.iter().all(|item| {
            matches!(
                item,
                Item::Use(_)
                    | Item::ExternCrate(_)
                    | Item::Mod(ItemMod { content: None, .. })
                    | Item::Const(_)
                    | Item::Static(_)
                    | Item::Struct(_)
                    | Item::Enum(_)
                    | Item::Type(_)
                    | Item::Trait(_)
            ) || self.behaviourless_impl_detector.is_behaviourless_impl(item)
        })
    }

    // A file must declare at least one inert thing to count as definition-only.
    // A type is one; so is a `const` or `static`, which is what makes a module
    // of lookup tables exempt. A file of nothing but `use` statements declares
    // nothing and stays in scope.
    fn count_declaring_items(&self, items: &[&Item]) -> usize {
        items
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    Item::Struct(_)
                        | Item::Enum(_)
                        | Item::Type(_)
                        | Item::Trait(_)
                        | Item::Const(_)
                        | Item::Static(_)
                )
            })
            .count()
    }

    // A file of definitions or a data holder has nothing a mirror could test,
    // so a mirror for one is a finding. A humble adapter has nothing worth one
    // either, but its forwarding is real code, and a trait's default method is
    // behaviour: a mirror of either is allowed rather than demanded.
    fn expectation_of(&self, syntax: &File) -> MirrorExpectation {
        let items = self.top_level_non_test_items(&syntax.items);

        if items.is_empty() {
            return MirrorExpectation::Required;
        }

        let inert = (self.all_definition_only(&items) && self.count_declaring_items(&items) > 0)
            || DataHolderDetector::new().file_is_data_holder(&items);
        if inert {
            return if Self::has_default_method(&items) {
                MirrorExpectation::Optional
            } else {
                MirrorExpectation::Unearned
            };
        }

        if HumbleAdapterDetector::new().file_is_humble_adapter(&items) {
            return MirrorExpectation::Optional;
        }

        MirrorExpectation::Required
    }

    fn has_default_method(items: &[&Item]) -> bool {
        items.iter().any(|item| {
            matches!(item, Item::Trait(item_trait) if item_trait.items.iter().any(|member| {
                matches!(member, TraitItem::Fn(method) if method.default.is_some())
            }))
        })
    }

    fn top_level_non_test_items<'a>(&self, items: &'a [Item]) -> Vec<&'a Item> {
        items
            .iter()
            .filter(|item| !Self::has_test_attrs(item.attrs()))
            .collect()
    }
}

trait ItemAttrs {
    fn attrs(&self) -> &[Attribute];
}

impl ItemAttrs for Item {
    fn attrs(&self) -> &[Attribute] {
        match self {
            Item::Const(item) => &item.attrs,
            Item::Enum(item) => &item.attrs,
            Item::ExternCrate(item) => &item.attrs,
            Item::Fn(item) => &item.attrs,
            Item::ForeignMod(item) => &item.attrs,
            Item::Impl(ItemImpl { attrs, .. }) => attrs,
            Item::Macro(item) => &item.attrs,
            Item::Mod(item) => &item.attrs,
            Item::Static(item) => &item.attrs,
            Item::Struct(item) => &item.attrs,
            Item::Trait(item) => &item.attrs,
            Item::TraitAlias(item) => &item.attrs,
            Item::Type(item) => &item.attrs,
            Item::Union(item) => &item.attrs,
            Item::Use(item) => &item.attrs,
            Item::Verbatim(_) => &[],
            _ => &[],
        }
    }
}

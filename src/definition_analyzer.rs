// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use quote::ToTokens;
use syn::{Attribute, File, Item, ItemImpl, ItemMod, parse_file};

use crate::behaviourless_impl_detector::BehaviourlessImplDetector;
use crate::humble_adapter_detector::HumbleAdapterDetector;
use crate::trivial_constructor_detector::TrivialConstructorDetector;

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

    pub fn is_definition_only_source(&self, source: &str) -> Result<bool> {
        let syntax = parse_file(source).context("failed to parse Rust source")?;
        Ok(self.file_is_definition_only(&syntax))
    }

    pub fn mod_file_is_import_only(&self, source: &str) -> Result<bool> {
        let syntax = parse_file(source).context("failed to parse Rust source")?;
        let items = self.top_level_non_test_items(&syntax.items);
        Ok(items.iter().all(|item| {
            matches!(
                item,
                Item::Use(_) | Item::ExternCrate(_) | Item::Mod(ItemMod { content: None, .. })
            )
        }))
    }

    fn file_is_definition_only(&self, syntax: &File) -> bool {
        let items = self.top_level_non_test_items(&syntax.items);

        if items.is_empty() {
            return false;
        }

        if self.all_definition_only(&items) {
            return self.count_declaring_items(&items) > 0;
        }

        if TrivialConstructorDetector::new().file_has_trivial_constructor(&items) {
            return true;
        }

        HumbleAdapterDetector::new().file_is_humble_adapter(&items)
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

    fn top_level_non_test_items<'a>(&self, items: &'a [Item]) -> Vec<&'a Item> {
        items
            .iter()
            .filter(|item| !Self::has_test_attrs(item.attrs()))
            .collect()
    }

    pub fn has_test_attrs(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            let path = attr.path();
            path.is_ident("cfg") && attr.meta.to_token_stream().to_string().contains("test")
        })
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

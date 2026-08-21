// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use syn::parse_str;
use syn::{File, Item};
use twin4rust::humble_adapter_detector::HumbleAdapterDetector;

fn is_humble(source: &str) -> bool {
    let items = items_of(source);
    let borrowed: Vec<&Item> = items.iter().collect();
    HumbleAdapterDetector::new().file_is_humble_adapter(&borrowed)
}

fn items_of(source: &str) -> Vec<Item> {
    let parsed: File = parse_str(source).expect("source should parse");
    parsed.items
}

#[test]
fn file_is_humble_adapter_for_a_bare_function_alongside_the_type_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

impl Halter {
    pub fn halt(&self) {
        driver::halt(self.id);
    }
}

pub fn reset_everything(serial: &str) {
    driver::reset(serial);
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_branching_body_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

impl Halter {
    pub fn halt(&self) {
        if self.id > 0 {
            driver::halt(self.id);
        }
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_constructor_alone_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Holder {
    id: u8,
}

impl Holder {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_constructor_plus_a_forwarding_method_returns_true() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct BoardHalter {
    board_id: BoardId,
    probe_serial: String,
}

impl BoardHalter {
    pub fn new(board_id: BoardId, probe_serial: &str) -> Self {
        Self { board_id, probe_serial: probe_serial.to_string() }
    }

    pub fn halt(self) {
        BoardEraser::new(self.board_id, &self.probe_serial).erase();
    }
}
"#,
    );

    // Assert
    assert!(humble);
}

#[test]
fn file_is_humble_adapter_for_a_delegating_getter_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Wrapper {
    inner: Inner,
}

impl Wrapper {
    pub fn name(&self) -> &str {
        self.inner.name()
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

// The companion to the limit above: a helper that computes returns a value, so
// its presence takes the whole file back out of the exemption.
#[test]
fn file_is_humble_adapter_for_a_forwarding_method_beside_a_computing_one_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Counter {
    left: u64,
    right: u64,
}

impl Counter {
    pub fn total(&self) -> u64 {
        self.left + self.right
    }

    pub fn record(&self) {
        sink::send(self.total());
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_forwarding_method_taking_arguments_returns_true() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Sink {
    inner: Writer,
}

impl Sink {
    pub fn new(inner: Writer) -> Self {
        Self { inner }
    }

    pub fn write(&self, payload: &[u8]) {
        self.inner.write_all(payload);
    }
}
"#,
    );

    // Assert
    assert!(humble);
}

#[test]
fn file_is_humble_adapter_for_a_macro_body_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Logger {
    prefix: String,
}

impl Logger {
    pub fn log(&self) {
        println!("{}", self.prefix);
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

// The discriminator: a method returning a value produces something, and what it
// produces is worth asserting. `to_label` has a one-line body and must stay in
// scope.
#[test]
fn file_is_humble_adapter_for_a_method_returning_a_value_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct RttChannelConfig {
    buffer_size: RttBufferSize,
    level: FirmwareLogLevel,
}

impl RttChannelConfig {
    pub fn to_label(&self) -> String {
        format!("{}@{}", self.buffer_size.label(), self.level.label())
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_trait_impl_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

impl Halting for Halter {
    fn halt(&self) {
        driver::halt(self.id);
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_two_statement_body_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

impl Halter {
    pub fn halt(&self) {
        driver::prepare(self.id);
        driver::halt(self.id);
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_a_type_with_no_impl_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}
"#,
    );

    // Assert
    assert!(!humble);
}

// Every method is humble only because there are none. Vacuous truth must not
// buy an exemption.
#[test]
fn file_is_humble_adapter_for_an_empty_impl_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

impl Halter {}
"#,
    );

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_an_explicit_unit_return_returns_true() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

impl Halter {
    pub fn halt(&self) -> () {
        driver::halt(self.id);
    }
}
"#,
    );

    // Assert
    assert!(humble);
}

// A deliberate limit, pinned rather than fixed: the rule looks at the statement,
// not into the arguments, so arithmetic passed to a forwarded call is still
// exempt. Bounded by the single-statement requirement, and by the fact that any
// helper computing it would itself return a value and disqualify the file.
#[test]
fn file_is_humble_adapter_for_computation_inside_an_argument_returns_true() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Counter {
    left: u64,
    right: u64,
}

impl Counter {
    pub fn record(&self) {
        sink::send(self.left + self.right);
    }
}
"#,
    );

    // Assert
    assert!(humble);
}

#[test]
fn file_is_humble_adapter_for_no_items_returns_false() {
    // Arrange & Act
    let humble = is_humble("");

    // Assert
    assert!(!humble);
}

#[test]
fn file_is_humble_adapter_for_several_forwarding_methods_returns_true() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Probe {
    serial: String,
}

impl Probe {
    pub fn halt(&self) {
        driver::halt(&self.serial);
    }

    pub fn resume(&self) {
        driver::resume(&self.serial);
    }
}
"#,
    );

    // Assert
    assert!(humble);
}

#[test]
fn file_is_humble_adapter_for_two_declared_types_returns_false() {
    // Arrange & Act
    let humble = is_humble(
        r#"
pub struct Halter {
    id: u8,
}

pub struct Resumer {
    id: u8,
}

impl Halter {
    pub fn halt(&self) {
        driver::halt(self.id);
    }
}
"#,
    );

    // Assert
    assert!(!humble);
}

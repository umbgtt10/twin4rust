// Copyright 2026 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the MIT License
// SPDX-License-Identifier: MIT

use serge4rust::process::command_outcome::CommandOutcome;
use serge4rust::process::command_runner::CommandRunner;
use serge4rust::process::launch_failure::LaunchFailure;
use std::cell::RefCell;

// The seam the self-analysis gate is constructed against: every program
// available, every call recorded and answered with one exit code.
pub struct FakeCommandRunner {
    streaming_code: Option<i32>,
    calls: RefCell<Vec<Vec<String>>>,
}

impl FakeCommandRunner {
    pub fn new() -> Self {
        Self {
            streaming_code: Some(0),
            calls: RefCell::new(Vec::new()),
        }
    }

    pub fn with_streaming_code(mut self, streaming_code: Option<i32>) -> Self {
        self.streaming_code = streaming_code;
        self
    }

    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls.borrow().clone()
    }

    fn record(&self, args: &[String]) {
        self.calls.borrow_mut().push(args.to_vec());
    }
}

impl Default for FakeCommandRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandRunner for FakeCommandRunner {
    fn is_available(&self, _binary_name: &str) -> bool {
        true
    }

    fn run_streaming(&self, _program: &str, args: &[String]) -> Result<Option<i32>, LaunchFailure> {
        self.record(args);
        Ok(self.streaming_code)
    }

    fn run_capturing(
        &self,
        _program: &str,
        args: &[String],
    ) -> Result<CommandOutcome, LaunchFailure> {
        self.record(args);
        Ok(CommandOutcome::new(
            self.streaming_code,
            String::new(),
            String::new(),
        ))
    }
}

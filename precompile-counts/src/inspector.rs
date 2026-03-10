use std::collections::HashMap;
use alloy::primitives::Address;
use revm_inspector::Inspector;
use revm::interpreter::{CallInputs, CallOutcome, InterpreterTypes};

#[derive(Default, Debug)]
pub struct PrecompileCounter {
    pub counts: HashMap<&'static str, usize>,
}

impl<CTX, INTR: InterpreterTypes> Inspector<CTX, INTR> for PrecompileCounter {
    fn call(&mut self, _context: &mut CTX, inputs: &mut CallInputs) -> Option<CallOutcome> {
        let target = inputs.target_address;
        if target == Address::with_last_byte(2) {
            *self.counts.entry("sha256").or_default() += 1;
        } else if target == Address::with_last_byte(3) {
            *self.counts.entry("ripemd160").or_default() += 1;
        } else if target == Address::with_last_byte(4) {
            *self.counts.entry("identity").or_default() += 1;
        } else if target == Address::with_last_byte(5) {
            *self.counts.entry("modexp").or_default() += 1;
        } else if target == Address::with_last_byte(9) {
            *self.counts.entry("blake2f").or_default() += 1;
        } else if target == Address::with_last_byte(10) {
            *self.counts.entry("point_eval").or_default() += 1;
        }
        None
    }
}

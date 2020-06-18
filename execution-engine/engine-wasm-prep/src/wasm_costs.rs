use std::collections::BTreeMap;

use pwasm_utils::rules::{InstructionType, Metering, Set};
use serde::{Deserialize, Serialize};

// Taken (partially) from parity-ethereum
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmCosts {
    /// Default opcode cost
    pub regular: u32,
    /// Div operations multiplier.
    pub div: u32,
    /// Mul operations multiplier.
    pub mul: u32,
    /// Memory (load/store) operations multiplier.
    pub mem: u32,
    /// Memory stipend. Amount of free memory (in 64kb pages) each contract can
    /// use for stack.
    pub initial_mem: u32,
    /// Grow memory cost, per page (64kb)
    pub grow_mem: u32,
    /// Memory copy cost, per byte
    pub memcpy: u32,
    /// Max stack height (native WebAssembly stack limiter)
    pub max_stack_height: u32,
    /// Cost of wasm opcode is calculated as TABLE_ENTRY_COST * `opcodes_mul` /
    /// `opcodes_div`
    pub opcodes_mul: u32,
    /// Cost of wasm opcode is calculated as TABLE_ENTRY_COST * `opcodes_mul` /
    /// `opcodes_div`
    pub opcodes_div: u32,
}

impl WasmCosts {
    pub(crate) fn to_set(&self) -> Set {
        let meterings = {
            let mut tmp = BTreeMap::new();
            tmp.insert(InstructionType::Load, Metering::Fixed(self.mem));
            tmp.insert(InstructionType::Store, Metering::Fixed(self.mem));
            tmp.insert(InstructionType::Div, Metering::Fixed(self.div));
            tmp.insert(InstructionType::Mul, Metering::Fixed(self.mul));
            tmp
        };
        Set::new(self.regular, meterings)
            .with_grow_cost(self.grow_mem)
            .with_forbidden_floats()
    }
}

pub mod gens {
    use proptest::{num, prop_compose};

    use crate::wasm_costs::WasmCosts;

    prop_compose! {
        pub fn wasm_costs_arb()(
            regular in num::u32::ANY,
            div in num::u32::ANY,
            mul in num::u32::ANY,
            mem in num::u32::ANY,
            initial_mem in num::u32::ANY,
            grow_mem in num::u32::ANY,
            memcpy in num::u32::ANY,
            max_stack_height in num::u32::ANY,
            opcodes_mul in num::u32::ANY,
            opcodes_div in num::u32::ANY,
        ) -> WasmCosts {
            WasmCosts {
                regular,
                div,
                mul,
                mem,
                initial_mem,
                grow_mem,
                memcpy,
                max_stack_height,
                opcodes_mul,
                opcodes_div,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::proptest;

    use types::encoding;

    use super::gens;
    use crate::wasm_costs::WasmCosts;

    fn wasm_costs_mock() -> WasmCosts {
        WasmCosts {
            regular: 1,
            div: 16,
            mul: 4,
            mem: 2,
            initial_mem: 4096,
            grow_mem: 8192,
            memcpy: 1,
            max_stack_height: 64 * 1024,
            opcodes_mul: 3,
            opcodes_div: 8,
        }
    }

    fn wasm_costs_free() -> WasmCosts {
        WasmCosts {
            regular: 0,
            div: 0,
            mul: 0,
            mem: 0,
            initial_mem: 4096,
            grow_mem: 8192,
            memcpy: 0,
            max_stack_height: 64 * 1024,
            opcodes_mul: 1,
            opcodes_div: 1,
        }
    }

    #[test]
    fn should_serialize_and_deserialize() {
        let mock = wasm_costs_mock();
        let free = wasm_costs_free();
        encoding::test_serialization_roundtrip(&mock);
        encoding::test_serialization_roundtrip(&free);
    }

    proptest! {
        #[test]
        fn should_serialize_and_deserialize_with_arbitrary_values(
            wasm_costs in gens::wasm_costs_arb()
        ) {
            encoding::test_serialization_roundtrip(&wasm_costs);
        }
    }
}

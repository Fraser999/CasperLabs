use std::cmp;

use engine_shared::gas::Gas;
use types::U512;

#[derive(Debug, Copy, Clone)]
pub(super) struct GasLimitError {}

#[derive(Debug, Copy, Clone)]
pub(super) enum GasCounter {
    Small(SmallGasCounter),
    Large(LargeGasCounter),
}

impl GasCounter {
    pub fn new(limit: Gas, initial_count: Gas) -> Self {
        assert!(limit >= initial_count);
        if limit.value() <= U512::from(u64::max_value()) {
            GasCounter::Small(SmallGasCounter::new(limit, initial_count))
        } else {
            GasCounter::Large(LargeGasCounter::new(limit, initial_count))
        }
    }

    pub fn add(&mut self, additional_gas: u64) -> Result<(), GasLimitError> {
        match self {
            GasCounter::Small(small_counter) => small_counter.add(additional_gas),
            GasCounter::Large(large_counter) => large_counter.add(additional_gas),
        }
    }

    pub fn set_gas_used(&mut self, gas_used: Gas) {
        match self {
            GasCounter::Small(small_counter) => small_counter.set_gas_used(gas_used),
            GasCounter::Large(large_counter) => large_counter.set_gas_used(gas_used),
        }
    }

    pub fn limit(&self) -> Gas {
        match self {
            GasCounter::Small(small_counter) => small_counter.limit(),
            GasCounter::Large(large_counter) => large_counter.limit(),
        }
    }

    pub fn used(&self) -> Gas {
        match self {
            GasCounter::Small(small_counter) => small_counter.used(),
            GasCounter::Large(large_counter) => large_counter.used(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct SmallGasCounter {
    limit: u64,
    used: u64,
}

impl SmallGasCounter {
    fn new(limit: Gas, initial_count: Gas) -> Self {
        SmallGasCounter {
            limit: limit.value().as_u64(),
            used: initial_count.value().as_u64(),
        }
    }

    fn add(&mut self, additional_gas: u64) -> Result<(), GasLimitError> {
        if self.limit - self.used < additional_gas {
            Err(GasLimitError {})
        } else {
            // Can't overflow as we already checked this sum is <= `self.limit`
            self.used += additional_gas;
            Ok(())
        }
    }

    fn set_gas_used(&mut self, gas_used: Gas) {
        self.used = gas_used.value().as_u64()
    }

    fn limit(&self) -> Gas {
        Gas::new(U512::from(self.limit))
    }

    fn used(&self) -> Gas {
        Gas::new(U512::from(self.used))
    }
}

#[derive(Debug, Copy, Clone)]
pub(super) struct LargeGasCounter {
    total_limit: U512,
    limit_for_buffer: u64,
    used_buffer: u64,
    rest_of_used: U512,
}

impl LargeGasCounter {
    fn new(limit: Gas, initial_count: Gas) -> Self {
        let mut counter = LargeGasCounter {
            total_limit: limit.value(),
            limit_for_buffer: 0,
            used_buffer: 0,
            rest_of_used: initial_count.value(),
        };
        counter.flush_buffer();
        counter
    }

    fn flush_buffer(&mut self) {
        self.rest_of_used += U512::from(self.used_buffer);
        self.used_buffer = 0;
        self.limit_for_buffer = cmp::min(
            U512::from(u64::max_value()),
            self.total_limit - self.rest_of_used,
        )
        .as_u64();
    }

    fn add(&mut self, additional_gas: u64) -> Result<(), GasLimitError> {
        if self.limit_for_buffer - self.used_buffer < additional_gas {
            // We'd overflow, so flush buffer if required and try again, or fail
            if self.used_buffer != 0 {
                self.flush_buffer();
                self.add(additional_gas)
            } else {
                Err(GasLimitError {})
            }
        } else {
            // Can't overflow as we already checked this sum is <= `self.limit_for_buffer`
            self.used_buffer += additional_gas;
            Ok(())
        }
    }

    fn set_gas_used(&mut self, gas_used: Gas) {
        self.used_buffer = 0;
        self.rest_of_used = gas_used.value();
    }

    fn limit(&self) -> Gas {
        Gas::new(self.total_limit)
    }

    fn used(&self) -> Gas {
        Gas::new(U512::from(self.used_buffer) + self.rest_of_used)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tester() {
        let mut u512 = U512::default();
        u512.0[0] = 1;
        u512.0[1] = 1;

        let mut gc = GasCounter::new(Gas::new(u512), Gas::default());
        gc.add(u64::max_value() - 1).unwrap();
        gc.add(3).unwrap();
        gc.add(1).unwrap();
    }
}

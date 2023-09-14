//! configuration for network inflation

use {lazy_static::lazy_static, solana_sdk::pubkey::Pubkey, std::collections::HashMap};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Copy, AbiExample)]
#[serde(rename_all = "camelCase")]
pub struct Inflation {
    /// Initial inflation percentage, from time=0
    pub initial: f64,

    /// Terminal inflation percentage, to time=INF
    pub terminal: f64,

    /// Rate per year, at which inflation is lowered until reaching terminal
    ///  i.e. inflation(year) == MAX(terminal, initial*((1-taper)^year))
    pub taper: f64,

    /// Percentage of total inflation allocated to the foundation
    pub foundation: f64,
    /// Duration of foundation pool inflation, in years
    pub foundation_term: f64,

    /// DEPRECATED, this field is currently unused
    __unused: f64,
}

const DEFAULT_INITIAL: f64 = 0.08;
const DEFAULT_TERMINAL: f64 = 0.015;
const DEFAULT_TAPER: f64 = 0.15;
const DEFAULT_FOUNDATION: f64 = 0.05;
const DEFAULT_FOUNDATION_TERM: f64 = 7.0;

pub mod vault_addresses {
    pub mod foo {
        solana_sdk::declare_id!("dummy11111111111111111111111111111111111111");
    }

    pub mod bar {
        solana_sdk::declare_id!("dummy22222222222222222222222222222222222222");
    }
}

lazy_static! {
    pub static ref VAULT_ADDRESSES: HashMap<Pubkey, f64> = [
        (vault_addresses::foo::id(), 0.01),
        (vault_addresses::bar::id(), 0.02),
    ]
    .iter()
    .cloned()
    .collect();
}

impl Default for Inflation {
    fn default() -> Self {
        Self {
            initial: DEFAULT_INITIAL,
            terminal: DEFAULT_TERMINAL,
            taper: DEFAULT_TAPER,
            foundation: DEFAULT_FOUNDATION,
            foundation_term: DEFAULT_FOUNDATION_TERM,
            __unused: 0.0,
        }
    }
}

impl Inflation {
    pub fn new_disabled() -> Self {
        Self {
            initial: 0.0,
            terminal: 0.0,
            taper: 0.0,
            foundation: 0.0,
            foundation_term: 0.0,
            __unused: 0.0,
        }
    }

    // fixed inflation rate at `validator` percentage for staking rewards, and none for foundation
    pub fn new_fixed(validator: f64) -> Self {
        Self {
            initial: validator,
            terminal: validator,
            taper: 1.0,
            foundation: 0.0,
            foundation_term: 0.0,
            __unused: 0.0,
        }
    }

    pub fn pico() -> Self {
        Self::new_fixed(0.0001) // 0.01% inflation
    }

    pub fn full() -> Self {
        Self {
            initial: DEFAULT_INITIAL,
            terminal: DEFAULT_TERMINAL,
            taper: DEFAULT_TAPER,
            foundation: 0.0,
            foundation_term: 0.0,
            __unused: 0.0,
        }
    }

    /// inflation rate at year
    pub fn total(&self, year: f64) -> f64 {
        assert!(year >= 0.0);
        let tapered = self.initial * ((1.0 - self.taper).powf(year));

        if tapered > self.terminal {
            tapered
        } else {
            self.terminal
        }
    }

    /// portion of total that goes to validators
    pub fn validator(&self, year: f64) -> f64 {
        self.total(year) - self.foundation(year) - self.vault()
    }

    /// portion of total that goes to foundation
    pub fn foundation(&self, year: f64) -> f64 {
        if year < self.foundation_term {
            self.total(year) * self.foundation
        } else {
            0.0
        }
    }

    /// portion of total that goes to the listed vaults
    pub fn vault(&self) -> f64 {
        VAULT_ADDRESSES.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_inflation_basic() {
        let inflation = Inflation::default();

        let mut last = inflation.total(0.0);

        for year in &[0.1, 0.5, 1.0, DEFAULT_FOUNDATION_TERM, 100.0] {
            let total = inflation.total(*year);
            assert_eq!(
                total,
                inflation.validator(*year) + inflation.foundation(*year)
            );
            assert!(total < last);
            assert!(total >= inflation.terminal);
            last = total;
        }
        assert_eq!(last, inflation.terminal);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_inflation_fixed() {
        let inflation = Inflation::new_fixed(0.001);
        for year in &[0.1, 0.5, 1.0, DEFAULT_FOUNDATION_TERM, 100.0] {
            assert_eq!(inflation.total(*year), 0.001);
        }
    }
}

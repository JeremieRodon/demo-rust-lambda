use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Represent a weight. Internally stored as a [u64] representing micrograms.
/// This struct provide a [Display] impl that always print the [Weight] with
/// the correct unit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Weight(u64);
impl Weight {
    pub const ZERO: Weight = Weight(0);
    pub const MIN: Weight = Weight(80_000_000_000);
    pub const MAX: Weight = Weight(160_000_000_000);

    /// Instantiate a [Weight] from a [f64] in kilograms
    pub fn from_kg(weight: f64) -> Self {
        Self((weight * 1_000_000_000f64) as u64)
    }

    /// Instantiate a [Weight] from a [f64] in grams
    pub fn from_g(weight: f64) -> Self {
        Self((weight * 1_000_000f64) as u64)
    }

    /// Instantiate a [Weight] from a [f64] in milligrams
    pub fn from_mg(weight: f64) -> Self {
        Self((weight * 1_000f64) as u64)
    }

    /// Instantiate a [Weight] from a [u64] in micrograms.
    pub fn from_ug(weight: u64) -> Self {
        Self(weight)
    }

    pub fn as_kg(&self) -> f64 {
        self.0 as f64 / 1_000_000_000f64
    }

    pub fn as_g(&self) -> f64 {
        self.0 as f64 / 1_000_000f64
    }

    pub fn as_mg(&self) -> f64 {
        self.0 as f64 / 1_000f64
    }

    pub fn as_ug(&self) -> u64 {
        self.0
    }
}

impl Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 > 1_000_000_000 {
            write!(f, "{:.3}kg", self.as_kg())
        } else if self.0 > 1_000_000 {
            write!(f, "{:.3}g", self.as_g())
        } else if self.0 > 1_000 {
            write!(f, "{:.3}mg", self.as_mg())
        } else {
            write!(f, "{}ug", self.0)
        }
    }
}
impl std::ops::Add for Weight {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Weight(self.0 + rhs.0)
    }
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct Tatoo(pub u64);
impl Display for Tatoo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheep {
    pub tatoo: Tatoo,
    pub weight: Weight,
}
impl Display for Sheep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sheep({}) weighting {}", self.tatoo, self.weight)
    }
}
/// Sheeps equality is only driven by their [Tatoo]
impl PartialEq for Sheep {
    fn eq(&self, other: &Self) -> bool {
        self.tatoo == other.tatoo
    }
}
impl Eq for Sheep {}

/// [std::hash::Hash] implementation should be equivalent to [Eq]
impl std::hash::Hash for Sheep {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tatoo.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_display() {
        assert_eq!(Weight::from_kg(1.430983).to_string(), "1.431kg");
        assert_eq!(Weight::from_g(3489.43982).to_string(), "3.489kg");
        assert_eq!(Weight::from_g(489.43982).to_string(), "489.440g");
        assert_eq!(Weight::from_mg(432.87).to_string(), "432.870mg");
        assert_eq!(Weight::from_ug(976).to_string(), "976ug");
        assert_eq!(Weight::from_mg(0.457).to_string(), "457ug");
    }

    #[test]
    fn sheeps_equal_equiv_tatoos_equal() {
        let t1 = Tatoo(1);
        let t2 = Tatoo(2);
        let w1 = Weight::from_kg(100.0);
        let w2 = Weight::from_kg(150.0);

        let sheep1 = Sheep {
            tatoo: t1.clone(),
            weight: w1,
        };
        let sheep2 = Sheep {
            tatoo: t2.clone(),
            weight: w1,
        };
        let sheep3 = Sheep {
            tatoo: t1,
            weight: w2,
        };
        let sheep4 = Sheep {
            tatoo: t2,
            weight: w2,
        };
        assert_eq!(sheep1, sheep3);
        assert_eq!(sheep2, sheep4);
        assert_ne!(sheep1, sheep2);
        assert_ne!(sheep3, sheep4);
    }
}

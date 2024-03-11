use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// The various units of [Weight] that we can display
#[derive(Debug, Clone, Copy)]
pub enum WeightUnit {
    Micrograms,
    Milligrams,
    Grams,
    Kilograms,
}
impl WeightUnit {
    fn best_unit(weight: &Weight) -> Self {
        match weight.0 {
            w if w > 1_000_000_000 => WeightUnit::Kilograms,
            w if w > 1_000_000 => WeightUnit::Grams,
            w if w > 1_000 => WeightUnit::Milligrams,
            _ => WeightUnit::Micrograms,
        }
    }

    fn unit_ratio(&self) -> f64 {
        match self {
            WeightUnit::Micrograms => 1.0,
            WeightUnit::Milligrams => 1_000.0,
            WeightUnit::Grams => 1_000_000.0,
            WeightUnit::Kilograms => 1_000_000_000.0,
        }
    }
}
impl Display for WeightUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeightUnit::Micrograms => write!(f, "ug"),
            WeightUnit::Milligrams => write!(f, "mg"),
            WeightUnit::Grams => write!(f, "g"),
            WeightUnit::Kilograms => write!(f, "kg"),
        }
    }
}

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

    /// Instantiate a [Weight] from a [f64] in the specified [WeightUnit].
    pub fn from_unit(weight: f64, wu: WeightUnit) -> Self {
        Self((weight * wu.unit_ratio()) as u64)
    }

    /// Instantiate a [Weight] from a [u64] in micrograms.
    pub fn from_ug(weight: u64) -> Self {
        Self(weight)
    }

    /// Return the [Weight] as an [f64] in the specified [WeightUnit].
    pub fn as_unit(&self, wu: WeightUnit) -> f64 {
        self.0 as f64 / wu.unit_ratio()
    }

    /// Return the [Weight] as an [u64] in micrograms.
    pub fn as_ug(&self) -> u64 {
        self.0
    }
}

impl Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let best_unit = WeightUnit::best_unit(self);
        write!(f, "{:.3}{best_unit}", self.as_unit(best_unit))
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
pub struct Tattoo(pub u64);
impl Display for Tattoo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheep {
    pub tattoo: Tattoo,
    pub weight: Weight,
}
impl Display for Sheep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sheep({}) weighting {}", self.tattoo, self.weight)
    }
}
/// Sheeps equality is only driven by their [Tattoo] equality
impl PartialEq for Sheep {
    fn eq(&self, other: &Self) -> bool {
        self.tattoo == other.tattoo
    }
}
impl Eq for Sheep {}

/// [std::hash::Hash] implementation must be equivalent to [Eq]
/// so we ignore the weight field of the [Sheep]
impl std::hash::Hash for Sheep {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tattoo.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weight_display() {
        assert_eq!(
            Weight::from_unit(1.430983, WeightUnit::Kilograms).to_string(),
            "1.431kg"
        );
        assert_eq!(
            Weight::from_unit(3489.43982, WeightUnit::Grams).to_string(),
            "3.489kg"
        );
        assert_eq!(
            Weight::from_unit(489.43982, WeightUnit::Grams).to_string(),
            "489.440g"
        );
        assert_eq!(
            Weight::from_unit(432.87, WeightUnit::Milligrams).to_string(),
            "432.870mg"
        );
        assert_eq!(Weight::from_ug(976).to_string(), "976.000ug");
        assert_eq!(
            Weight::from_unit(0.457, WeightUnit::Milligrams).to_string(),
            "457.000ug"
        );
    }

    #[test]
    fn sheeps_equal_equiv_tattoos_equal() {
        let t1 = Tattoo(1);
        let t2 = Tattoo(2);
        let w1 = Weight::from_unit(100.0, WeightUnit::Kilograms);
        let w2 = Weight::from_unit(150.0, WeightUnit::Kilograms);

        let sheep1 = Sheep {
            tattoo: t1.clone(),
            weight: w1,
        };
        let sheep2 = Sheep {
            tattoo: t2.clone(),
            weight: w1,
        };
        let sheep3 = Sheep {
            tattoo: t1,
            weight: w2,
        };
        let sheep4 = Sheep {
            tattoo: t2,
            weight: w2,
        };
        assert_eq!(sheep1, sheep3);
        assert_eq!(sheep2, sheep4);
        assert_ne!(sheep1, sheep2);
        assert_ne!(sheep3, sheep4);
    }
}

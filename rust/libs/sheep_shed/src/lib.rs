pub mod errors;
mod sheep;

pub use sheep::{Sheep, Tattoo, Weight, WeightUnit};
use std::collections::HashMap;

/// The trait for [SheepShed] that can hold [Sheep]s and provides basic methods
/// for interacting with it.
pub trait SheepShed {
    /// Add a new [Sheep] in the [SheepShed]
    /// # Errors
    /// It is not allowed to add a duplicated [Sheep], will return an
    /// [errors::Error::SheepDuplicationError] if the user tries to add
    /// a [Sheep] with an already known [Tattoo]
    fn add_sheep(&mut self, sheep: Sheep) -> Result<(), errors::Error>;
    /// Return the number of [Sheep] in the [SheepShed]
    fn sheep_count(&self) -> Result<usize, errors::Error>;
    /// Return an [Iterator] over all the [Sheep]s in the [SheepShed]
    fn sheep_iter(&self) -> Result<impl Iterator<Item = Sheep>, errors::Error>;
    /// Kill an unlucky Sheep.
    /// Remove it from the [SheepShed] and return it's body.
    /// # Errors
    /// It is not allowed to kill an inexistant [Sheep], will return an
    /// [errors::Error::SheepNotPresent] if the user tries to kill
    /// a [Sheep] that is not in the [SheepShed]
    fn kill_sheep(&mut self, tattoo: &Tattoo) -> Result<Sheep, errors::Error>;
}

#[derive(Debug, Clone, Default)]
pub struct MemorySheepShed(HashMap<Tattoo, Sheep>);

impl SheepShed for MemorySheepShed {
    fn add_sheep(&mut self, sheep: Sheep) -> Result<(), errors::Error> {
        if self.0.contains_key(&sheep.tattoo) {
            Err(errors::Error::SheepDuplicationError(sheep.tattoo))
        } else {
            self.0.insert(sheep.tattoo.clone(), sheep);
            Ok(())
        }
    }

    /// Return the number of [Sheep] in the [SheepShed]
    /// Never returns an [Err] variant.
    fn sheep_count(&self) -> Result<usize, errors::Error> {
        Ok(self.0.len())
    }

    fn sheep_iter(&self) -> Result<impl Iterator<Item = Sheep>, errors::Error> {
        Ok(self.0.values().cloned())
    }

    fn kill_sheep(&mut self, tattoo: &Tattoo) -> Result<Sheep, errors::Error> {
        if self.0.contains_key(tattoo) {
            Ok(self.0.remove(tattoo).unwrap())
        } else {
            Err(errors::Error::SheepNotPresent(tattoo.to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    macro_rules! impl_test_template {
        ($tn: tt) => {
            #[test]
            fn $tn() {
                let sheep_shed = MemorySheepShed::default();
                crate::test_templates::$tn(sheep_shed)
            }
        };
    }

    impl_test_template!(cannot_duplicate_sheep);
    impl_test_template!(sheep_shed_sheep_count);
    impl_test_template!(sheep_shed_iterator);
    impl_test_template!(cannot_kill_inexistent_sheep);
}

#[cfg(any(feature = "sheepshed_tests", test))]
pub mod test_templates {
    use crate::{errors::Error, sheep::WeightUnit, Sheep, SheepShed, Tattoo, Weight};

    fn prep_base_sheep_shed<T: SheepShed>(mut sheep_shed: T) -> T {
        let sheep1 = Sheep {
            tattoo: Tattoo(1),
            weight: Weight::from_unit(100.0, WeightUnit::Kilograms),
        };
        let sheep2 = Sheep {
            tattoo: Tattoo(2),
            weight: Weight::from_unit(120.0, WeightUnit::Kilograms),
        };
        sheep_shed.add_sheep(sheep1).unwrap();
        sheep_shed.add_sheep(sheep2).unwrap();
        sheep_shed
    }

    pub fn cannot_duplicate_sheep<T: SheepShed>(sheep_shed: T) {
        let mut sheep_shed = prep_base_sheep_shed(sheep_shed);
        let sheep3 = Sheep {
            tattoo: Tattoo(1),
            weight: Weight::from_unit(120.0, WeightUnit::Kilograms),
        };
        // Sheep3 has the same Tattoo as Sheep1 so it should fail
        assert!(sheep_shed.add_sheep(sheep3).is_err_and(|e| match e {
            Error::SheepDuplicationError(_) => true,
            _ => false,
        }));
    }

    pub fn sheep_shed_sheep_count<T: SheepShed>(sheep_shed: T) {
        let mut sheep_shed = prep_base_sheep_shed(sheep_shed);
        let sheep3 = Sheep {
            tattoo: Tattoo(4),
            weight: Weight::from_unit(120.0, WeightUnit::Kilograms),
        };
        assert_eq!(sheep_shed.sheep_count().unwrap(), 2);
        sheep_shed.add_sheep(sheep3).unwrap();
        assert_eq!(sheep_shed.sheep_count().unwrap(), 3);
    }

    pub fn sheep_shed_iterator<T: SheepShed>(sheep_shed: T) {
        let sheep_shed = prep_base_sheep_shed(sheep_shed);
        let weight = sheep_shed
            .sheep_iter()
            .unwrap()
            .fold(Weight::ZERO, |acc, sheep| acc + sheep.weight);
        assert_eq!(weight, Weight::from_unit(220.0, WeightUnit::Kilograms));
    }

    pub fn cannot_kill_inexistent_sheep<T: SheepShed>(sheep_shed: T) {
        let mut sheep_shed = prep_base_sheep_shed(sheep_shed);
        // Inexistant tattoo
        assert!(sheep_shed.kill_sheep(&Tattoo(4)).is_err_and(|e| match e {
            Error::SheepNotPresent(_) => true,
            _ => false,
        }));
        // Existing tattoo
        assert!(sheep_shed.kill_sheep(&Tattoo(2)).is_ok());
        // Not anymore
        assert!(sheep_shed.kill_sheep(&Tattoo(2)).is_err_and(|e| match e {
            Error::SheepNotPresent(_) => true,
            _ => false,
        }));
    }
}

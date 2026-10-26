//! Raw-to-cooked weight conversion through a cooking yield, and the calorie
//! density arithmetic that goes with it.
//!
//! A cooking yield is `cooked weight / raw weight`. Meat loses water when it
//! cooks (yield below 1.0); pasta and rice absorb it (yield above 1.0). The
//! calories in the piece do not change, so the calories per gram do.
//!
//! The constants are mean yields from the USDA *Table of Cooking Yields for
//! Meat and Poultry*, Release 2 (2012). An interactive calculator carrying the
//! full table is at <https://recipetool.net/tools/raw-to-cooked-weight-calculator>.

#![forbid(unsafe_code)]

use std::fmt;

/// Error returned for impossible inputs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum YieldError {
    /// A weight was zero, negative or not finite.
    BadWeight,
    /// A yield fraction was zero, negative or not finite.
    BadYield,
}

impl fmt::Display for YieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YieldError::BadWeight => write!(f, "weight must be a positive finite number"),
            YieldError::BadYield => write!(f, "yield must be a positive finite fraction"),
        }
    }
}

impl std::error::Error for YieldError {}

/// A cooking yield expressed as a fraction (`0.72` = 72 %).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Yield(f64);

impl Yield {
    /// Build a yield from a fraction such as `0.72`.
    pub fn new(fraction: f64) -> Result<Self, YieldError> {
        if fraction.is_finite() && fraction > 0.0 {
            Ok(Yield(fraction))
        } else {
            Err(YieldError::BadYield)
        }
    }

    /// Build a yield from a percentage such as `72.0`.
    pub fn from_percent(percent: f64) -> Result<Self, YieldError> {
        Self::new(percent / 100.0)
    }

    /// Yield you measured yourself: weigh raw, cook, weigh again hot.
    pub fn measured(raw_g: f64, cooked_g: f64) -> Result<Self, YieldError> {
        if !(raw_g.is_finite() && raw_g > 0.0 && cooked_g.is_finite() && cooked_g > 0.0) {
            return Err(YieldError::BadWeight);
        }
        Self::new(cooked_g / raw_g)
    }

    /// The yield as a fraction.
    pub fn fraction(self) -> f64 {
        self.0
    }

    /// Cooked weight produced by `raw_g` grams of raw food.
    pub fn cooked_from_raw(self, raw_g: f64) -> f64 {
        raw_g * self.0
    }

    /// Raw weight needed for (or equivalent to) `cooked_g` grams cooked.
    pub fn raw_from_cooked(self, cooked_g: f64) -> f64 {
        cooked_g / self.0
    }

    /// Calories per 100 g after cooking, given the raw entry per 100 g.
    pub fn cooked_density(self, raw_kcal_per_100g: f64) -> f64 {
        raw_kcal_per_100g / self.0
    }

    /// Relative logging error from weighing cooked food and applying the raw
    /// entry. Negative means under-counted: for a 72 % yield it is about -0.28.
    pub fn cooked_weighed_raw_logged_error(self) -> f64 {
        self.0 - 1.0
    }
}

/// Calories in a weighed portion, given its state and the state of the entry.
///
/// `entry_is_raw` says which state the nutrition entry describes;
/// `portion_is_raw` says which state the scale reading describes.
pub fn portion_kcal(
    portion_g: f64,
    portion_is_raw: bool,
    entry_kcal_per_100g: f64,
    entry_is_raw: bool,
    y: Yield,
) -> f64 {
    let grams_in_entry_state = match (portion_is_raw, entry_is_raw) {
        (true, true) | (false, false) => portion_g,
        (false, true) => y.raw_from_cooked(portion_g),
        (true, false) => y.cooked_from_raw(portion_g),
    };
    grams_in_entry_state * entry_kcal_per_100g / 100.0
}

/// Chicken, broiler-fryer, breast, baked or roasted: mean yield 72 %.
pub fn chicken_breast_roasted() -> Yield {
    Yield(0.72)
}

/// Chicken, broiler-fryer, thigh, baked or roasted: mean yield 69 %.
pub fn chicken_thigh_roasted() -> Yield {
    Yield(0.69)
}

/// Beef, ground, low fat (<12 %), patty, pan-broiled: mean yield 77 %.
pub fn ground_beef_lean_pan_broiled() -> Yield {
    Yield(0.77)
}

/// Pork, cured, bacon, baked or roasted: mean yield 32 %.
pub fn bacon_baked() -> Yield {
    Yield(0.32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roast_breast_round_trip() {
        let y = chicken_breast_roasted();
        assert!((y.cooked_from_raw(500.0) - 360.0).abs() < 1e-9);
        assert!((y.raw_from_cooked(360.0) - 500.0).abs() < 1e-9);
    }

    #[test]
    fn density_matches_usda_cooked_entry_closely() {
        // raw 120 kcal/100 g through a 72 % yield ~ 167, USDA roasted ~ 165
        let d = chicken_breast_roasted().cooked_density(120.0);
        assert!((d - 166.67).abs() < 0.01);
    }

    #[test]
    fn mixed_state_error() {
        let y = chicken_breast_roasted();
        let right = portion_kcal(150.0, false, 120.0, true, y);
        let wrong = 150.0 * 120.0 / 100.0;
        assert_eq!(right.round(), 250.0);
        assert_eq!(wrong, 180.0);
        assert!((y.cooked_weighed_raw_logged_error() + 0.28).abs() < 1e-9);
    }

    #[test]
    fn pasta_absorbs_water() {
        // dry spaghetti ~371 kcal/100 g, cooked ~158 kcal/100 g
        let y = Yield::new(371.0 / 158.0).unwrap();
        assert!(y.fraction() > 2.3 && y.fraction() < 2.4);
        assert_eq!(portion_kcal(235.0, false, 371.0, true, y).round(), 371.0);
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(Yield::new(0.0), Err(YieldError::BadYield));
        assert_eq!(Yield::new(f64::NAN), Err(YieldError::BadYield));
        assert_eq!(Yield::measured(-1.0, 5.0), Err(YieldError::BadWeight));
        assert!(Yield::from_percent(77.0).is_ok());
    }
}

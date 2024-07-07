
use super::base::UnitParser;
use super::table::TableBasedParser;
use super::prefix::PrefixParser;
use crate::units::dimension::{Dimension, BaseDimension};
use crate::units::unit::Unit;

use num::One;
use num::pow::Pow;

use std::ops::Div;
use std::f64::consts::PI;

fn fraction<T>(n: i64, d: i64) -> T
where T: Div<Output = T> + From<i64> {
  T::from(n) / T::from(d)
}

pub fn default_parser<S>() -> PrefixParser<TableBasedParser<S>>
where S: Div<Output = S> + Pow<i64, Output = S> + From<i64> + From<f64> {
  PrefixParser::new_si(default_units_table())
}

pub fn default_units_table<S>() -> TableBasedParser<S>
where S: Div<Output = S> + Pow<i64, Output = S> + From<i64> + From<f64> {
  use BaseDimension::*;
  let units = vec![
    // Length units
    Unit::new("m", Length, fraction(1, 1)),
    Unit::new("in", Length, fraction(254, 10_000)),
    Unit::new("ft", Length, fraction(3_048, 10_000)),
    Unit::new("yd", Length, fraction(9_144, 10_000)),
    Unit::new("mi", Length, fraction(1_609_344, 1_000)),
    Unit::new("au", Length, fraction(149_597_870_700, 1)),
    Unit::new("lyr", Length, fraction(9_460_730_472_580_800, 1)),
    Unit::new("pc", Length, fraction(30_856_804_799_935_500, 1)), // Parsec
    Unit::new("nmi", Length, fraction(1_852, 1)), // Nautical mile
    Unit::new("fath", Length, fraction(18_288, 10_000)),
    Unit::new("fur", Length, fraction(201_168, 1_000)),
    Unit::new("mu", Length, fraction(1, 1_000_000)), // Micron (eqv to micrometer)
    Unit::new("mil", Length, fraction(254, 10_000_000)), // Mil (eqv to milli-inch)
    Unit::new("point", Length, fraction(254, 720_000)), // Point (Postscript convention)
    Unit::new("Ang", Length, fraction(1, 10_000_000_000)), // Angstrom
    // Time units
    Unit::new("s", Time, fraction(1, 1)),
    Unit::new("sec", Time, fraction(1, 1)),
    Unit::new("min", Time, fraction(60, 1)),
    Unit::new("hr", Time, fraction(3600, 1)),
    Unit::new("day", Time, fraction(86400, 1)),
    Unit::new("wk", Time, fraction(604800, 1)),
    Unit::new("yr", Time, fraction(31557600, 1)),
    // Mass units
    Unit::new("g", Mass, fraction(1, 1)),
    Unit::new("lb", Mass, fraction(45_359_237, 100_000)),
    Unit::new("oz", Mass, fraction(45_359_237, 1_600_000)),
    Unit::new("ton", Mass, fraction(45_359_237, 50)),
    Unit::new("t", Mass, fraction(1_000_000, 1)), // Metric ton (eqv. megagram)
    // Temperature units (relative)
    Unit::new("K", Temperature, fraction(1, 1)),
    Unit::new("degK", Temperature, fraction(1, 1)),
    Unit::new("dK", Temperature, fraction(1, 1)),
    Unit::new("dC", Temperature, fraction(1, 1)),
    Unit::new("degC", Temperature, fraction(1, 1)),
    Unit::new("dF", Temperature, fraction(5, 9)),
    Unit::new("degF", Temperature, fraction(5, 9)),
    // Electrical current units
    Unit::new("A", Current, fraction(1, 1)),
    // Luminous intensity units
    Unit::new("cd", LuminousIntensity, fraction(1, 1)),
    // Amount of substance units
    Unit::new("mol", AmountOfSubstance, fraction(1, 1)),
    // Angular units
    Unit::new("rad", Dimension::one(), fraction(1, 1)),
    Unit::new("deg", Dimension::one(), S::from(180.0 / PI)),
    // Units with nontrivial dimension
    Unit::new("c", Length / Time, fraction(299_792_458, 1)),
    Unit::new("hect", Length.pow(2), fraction(10_000, 1)), // Hectare
    Unit::new("a", Length.pow(2), fraction(100, 1)), // Are
    Unit::new("acre", Length.pow(2), fraction(316_160_658, 78_125)),
    Unit::new("b", Length.pow(2), S::from(10).pow(-28)), // Barn
    Unit::new("L", Length.pow(3), fraction(1, 1_000)), // Liter
    Unit::new("l", Length.pow(3), fraction(1, 1_000)), // Liter (synonym)
    Unit::new("gal", Length.pow(3), fraction(473_176_473, 125_000_000_000)), // US Gallon
    Unit::new("qt", Length.pow(3), fraction(473_176_473, 500_000_000_000)), // Quart
    Unit::new("pt", Length.pow(3), fraction(473_176_473, 1_000_000_000_000)), // Pint
    Unit::new("cup", Length.pow(3), fraction(473_176_473, 2_000_000_000_000)),
    Unit::new("floz", Length.pow(3), fraction(473_176_473, 16_000_000_000_000)),
    Unit::new("ozfl", Length.pow(3), fraction(473_176_473, 16_000_000_000_000)),
    Unit::new("tbsp", Length.pow(3), fraction(473_176_473, 32_000_000_000_000)),
    Unit::new("tsp", Length.pow(3), fraction(157_725_491, 32_000_000_000_000)),
    Unit::new("Hz", Time.pow(-1), fraction(1, 1)), // Hertz
    Unit::new("mph", Length / Time, fraction(1397, 3125)), // Miles per hour
    Unit::new("kph", Length / Time, fraction(5, 18)), // Kilometers per hour
    Unit::new("knot", Length / Time, fraction(463, 900)),
    Unit::new("c", Length / Time, fraction(299792458, 1)), // Speed of light
    Unit::new("ga", Length / Time.pow(2), fraction(980_665, 100_000)), // "g" acceleration
    Unit::new("N", Mass * Length / Time.pow(2), fraction(1, 1)), // Newton
    Unit::new("dyn", Mass * Length / Time.pow(2), fraction(1, 100_000)), // Dyne
    Unit::new("J", Mass * Length.pow(2) / Time.pow(2), fraction(1, 1)), // Joule
    Unit::new("cal", Mass * Length.pow(2) / Time.pow(2), fraction(41_868, 10_000)), // Calorie
    Unit::new("calth", Mass * Length.pow(2) / Time.pow(2), fraction(4_184, 1_000)), // Thermochemical Calorie
    Unit::new("Cal", Mass * Length.pow(2) / Time.pow(2), fraction(41_868, 10)), // Large Calorie
  ];
  units.into_iter().collect()
}

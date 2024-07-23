
use super::table::TableBasedParser;
use super::prefix::PrefixParser;
use crate::units::dimension::{Dimension, BaseDimension};
use crate::units::unit::Unit;

use num::One;
use num::pow::Pow;

use std::ops::Div;
use std::f64::consts::PI;
use std::collections::HashMap;

/// Parent trait for the functionality we require of a scalar type in
/// the default SI units table. No type should ever need to implement
/// this trait by hand, since a blanket impl takes types from the
/// parent traits to this one.
pub trait ScalarLike: One + Div<Output = Self> + Pow<i64, Output = Self> + From<i64> + From<f64> {}

impl<S> ScalarLike for S
where S: One + Div<Output = S> + Pow<i64, Output = S> + From<i64> + From<f64> {}

fn fraction<T>(n: i64, d: i64) -> T
where T: Div<Output = T> + From<i64> {
  T::from(n) / T::from(d)
}

pub fn default_parser<S>() -> PrefixParser<TableBasedParser<S>>
where S: ScalarLike + 'static {
  PrefixParser::new_si(default_units_table())
}

pub fn default_units_table<S>() -> TableBasedParser<S>
where S: ScalarLike + 'static {
  use BaseDimension::*;
  let units = vec![
    // Length units
    meters(),
    inches(),
    Unit::new("ft", Length, fraction(3_048, 10_000)),
    Unit::new("yd", Length, fraction(9_144, 10_000)),
    miles(),
    Unit::new("au", Length, fraction(149_597_870_700, 1)),
    Unit::new("lyr", Length, fraction(9_460_730_472_580_800, 1)),
    Unit::new("pc", Length, fraction(30_856_804_799_935_500, 1)), // Parsec
    nautical_miles(),
    Unit::new("fath", Length, fraction(18_288, 10_000)),
    furlongs(),
    chains(),
    Unit::new("mu", Length, fraction(1, 1_000_000)), // Micron (eqv to micrometer)
    Unit::new("mil", Length, fraction(254, 10_000_000)), // Mil (eqv to milli-inch)
    Unit::new("point", Length, fraction(254, 720_000)), // Point (Postscript convention)
    Unit::new("Ang", Length, fraction(1, 10_000_000_000)), // Angstrom
    // Time units
    seconds(),
    Unit::new("sec", Time, fraction(1, 1)),
    Unit::new("min", Time, fraction(60, 1)),
    hours(),
    Unit::new("day", Time, fraction(86400, 1)),
    Unit::new("wk", Time, fraction(604800, 1)),
    Unit::new("yr", Time, fraction(31557600, 1)),
    // Mass units
    grams(),
    Unit::new("lb", Mass, fraction(45_359_237, 100_000)),
    Unit::new("oz", Mass, fraction(45_359_237, 1_600_000)),
    Unit::new("ton", Mass, fraction(45_359_237, 50)),
    Unit::new("t", Mass, fraction(1_000_000, 1)), // Metric ton (eqv. megagram)
    // Temperature units (relative)
    kelvins(),
    Unit::new("degK", Temperature, fraction(1, 1))
      .with_temperature_offset(fraction(0, 1)),
    Unit::new("dK", Temperature, fraction(1, 1))
      .with_temperature_offset(fraction(0, 1)),
    Unit::new("dC", Temperature, fraction(1, 1))
      .with_temperature_offset(fraction(27_315, 100)),
    Unit::new("degC", Temperature, fraction(1, 1))
      .with_temperature_offset(fraction(27_315, 100)),
    Unit::new("dF", Temperature, fraction(5, 9))
      .with_temperature_offset(fraction(2_554, 10)),
    Unit::new("degF", Temperature, fraction(5, 9))
      .with_temperature_offset(fraction(2_554, 10)),
    // Electrical current units
    amperes(),
    // Luminous intensity units
    candela(),
    // Amount of substance units
    moles(),
    // Angular units
    Unit::new("rad", Dimension::one(), fraction(1, 1)),
    Unit::new("deg", Dimension::one(), S::from(180.0 / PI)),
    // Units with nontrivial dimension
    Unit::new("c", Length / Time, fraction(299_792_458, 1)) // Speed of light
      .with_composed(meters() / seconds()),
    Unit::new("hect", Length.pow(2), fraction(10_000, 1)) // Hectare
      .with_composed(hectameters().pow(2)),
    Unit::new("a", Length.pow(2), fraction(100, 1)) // Are
      .with_composed(dekameters().pow(2)),
    Unit::new("acre", Length.pow(2), fraction(316_160_658, 78_125))
      .with_composed(furlongs() * chains()),
    Unit::new("b", Length.pow(2), S::from(10).pow(-28)) // Barn
      .with_composed(femtometers().pow(2)),
    Unit::new("L", Length.pow(3), fraction(1, 1_000)) // Liter
      .with_composed(decimeters().pow(3)),
    Unit::new("l", Length.pow(3), fraction(1, 1_000)) // Liter (synonym)
      .with_composed(decimeters().pow(3)),
    Unit::new("gal", Length.pow(3), fraction(473_176_473, 125_000_000_000)) // US Gallon
      .with_composed(inches().pow(3)),
    Unit::new("qt", Length.pow(3), fraction(473_176_473, 500_000_000_000)) // Quart
      .with_composed(inches().pow(3)),
    Unit::new("pt", Length.pow(3), fraction(473_176_473, 1_000_000_000_000)) // Pint
      .with_composed(inches().pow(3)),
    Unit::new("cup", Length.pow(3), fraction(473_176_473, 2_000_000_000_000))
      .with_composed(inches().pow(3)),
    Unit::new("floz", Length.pow(3), fraction(473_176_473, 16_000_000_000_000))
      .with_composed(inches().pow(3)),
    Unit::new("ozfl", Length.pow(3), fraction(473_176_473, 16_000_000_000_000))
      .with_composed(inches().pow(3)),
    Unit::new("tbsp", Length.pow(3), fraction(473_176_473, 32_000_000_000_000))
      .with_composed(inches().pow(3)),
    Unit::new("tsp", Length.pow(3), fraction(157_725_491, 32_000_000_000_000))
      .with_composed(inches().pow(3)),
    Unit::new("Hz", Time.pow(-1), fraction(1, 1)) // Hertz
      .with_composed(seconds().pow(-1)),
    Unit::new("mph", Length / Time, fraction(1397, 3125)) // Miles per hour
      .with_composed(miles() / hours()),
    Unit::new("kph", Length / Time, fraction(5, 18)) // Kilometers per hour
      .with_composed(kilometers() / hours()),
    Unit::new("knot", Length / Time, fraction(463, 900))
      .with_composed(nautical_miles() / hours()),
    Unit::new("ga", Length / Time.pow(2), fraction(980_665, 100_000)) // "g" acceleration
      .with_composed(meters() / seconds().pow(2)),
    Unit::new("N", Mass * Length / Time.pow(2), fraction(1, 1)) // Newton
      .with_composed(grams() * meters() / seconds().pow(2)),
    Unit::new("dyn", Mass * Length / Time.pow(2), fraction(1, 100_000)) // Dyne
      .with_composed(grams() * meters() / seconds().pow(2)),
    Unit::new("J", Mass * Length.pow(2) / Time.pow(2), fraction(1, 1)) // Joule
      .with_composed(grams() * meters().pow(2) / seconds().pow(2)),
    Unit::new("cal", Mass * Length.pow(2) / Time.pow(2), fraction(41_868, 10_000)) // Calorie
      .with_composed(grams() * meters().pow(2) / seconds().pow(2)),
    Unit::new("calth", Mass * Length.pow(2) / Time.pow(2), fraction(4_184, 1_000)) // Thermochemical Calorie
      .with_composed(grams() * meters().pow(2) / seconds().pow(2)),
    Unit::new("Cal", Mass * Length.pow(2) / Time.pow(2), fraction(41_868, 10)) // Large Calorie
      .with_composed(grams() * meters().pow(2) / seconds().pow(2)),
  ];
  let units_table: HashMap<_, _> = units.into_iter().map(|unit| (unit.name().to_string(), unit)).collect();
  TableBasedParser::new(units_table, si_base_unit)
}

pub fn si_base_unit<S: One + From<i64>>(dimension: BaseDimension) -> Unit<S> {
  use BaseDimension::*;
  match dimension {
    Length => meters(),
    Time => seconds(),
    Mass => grams(),
    Temperature => kelvins(),
    Current => amperes(),
    LuminousIntensity => candela(),
    AmountOfSubstance => moles(),
  }
}

fn meters<S: One>() -> Unit<S> {
  Unit::new("m", BaseDimension::Length, S::one())
}

fn femtometers<S: ScalarLike>() -> Unit<S> {
  Unit::new("fm", BaseDimension::Length, S::from(10).pow(-15))
}

fn kilometers<S: ScalarLike>() -> Unit<S> {
  Unit::new("km", BaseDimension::Length, fraction(1_000, 1))
}

fn decimeters<S: ScalarLike>() -> Unit<S> {
  Unit::new("dm", BaseDimension::Length, fraction(1, 10))
}

fn hectameters<S: ScalarLike>() -> Unit<S> {
  Unit::new("hm", BaseDimension::Length, fraction(100, 1))
}

fn dekameters<S: ScalarLike>() -> Unit<S> {
  Unit::new("Dm", BaseDimension::Length, fraction(10, 1))
}

fn furlongs<S: ScalarLike>() -> Unit<S> {
  Unit::new("fur", BaseDimension::Length, fraction(201_168, 1_000))
}

fn chains<S: ScalarLike>() -> Unit<S> {
  Unit::new("ch", BaseDimension::Length, fraction(12_573, 625))
}

fn inches<S: ScalarLike>() -> Unit<S> {
  Unit::new("in", BaseDimension::Length, fraction(254, 10_000))
}

fn miles<S: ScalarLike>() -> Unit<S> {
  Unit::new("mi", BaseDimension::Length, fraction(1_609_344, 1_000))
}

fn nautical_miles<S: ScalarLike>() -> Unit<S> {
  Unit::new("nmi", BaseDimension::Length, fraction(1_852, 1))
}

fn seconds<S: One>() -> Unit<S> {
  Unit::new("s", BaseDimension::Time, S::one())
}

fn hours<S: ScalarLike>() -> Unit<S> {
  Unit::new("hr", BaseDimension::Time, fraction(3600, 1))
}

fn grams<S: One>() -> Unit<S> {
  Unit::new("g", BaseDimension::Mass, S::one())
}

fn kelvins<S: One + From<i64>>() -> Unit<S> {
  Unit::new("K", BaseDimension::Temperature, S::one())
    .with_temperature_offset(S::from(0i64))
}

fn amperes<S: One>() -> Unit<S> {
  Unit::new("A", BaseDimension::Current, S::one())
}

fn candela<S: One>() -> Unit<S> {
  Unit::new("cd", BaseDimension::LuminousIntensity, S::one())
}

fn moles<S: One>() -> Unit<S> {
  Unit::new("mol", BaseDimension::AmountOfSubstance, S::one())
}

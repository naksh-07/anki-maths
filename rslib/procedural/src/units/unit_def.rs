// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

use super::dimension::Dimension;

/// Canonical Physics and Chemistry units supported by the StudyLab Procedural Engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    // Dimensionless
    Dimensionless,
    Percent,

    // Mass [M]
    Kilogram,
    Gram,
    Milligram,
    Microgram,
    Tonne,

    // Length [L]
    Meter,
    Kilometer,
    Centimeter,
    Millimeter,
    Micrometer,
    Nanometer,
    Decimeter,
    Angstrom,

    // Time [T]
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
    Minute,
    Hour,
    Day,

    // Amount of Substance [N]
    Mole,
    Millimole,
    Micromole,
    Kilomole,

    // Temperature [K]
    Kelvin,
    Celsius,

    // Velocity / Speed [L T^-1]
    MeterPerSecond,
    KilometerPerHour,
    KilometerPerSecond,
    CentimeterPerSecond,
    MilesPerHour,

    // Acceleration [L T^-2]
    MeterPerSecondSquared,
    CentimeterPerSecondSquared,

    // Force [M L T^-2]
    Newton,
    Kilonewton,
    Millinewton,
    Dyne,

    // Energy / Work / Heat [M L^2 T^-2]
    Joule,
    Kilojoule,
    Millijoule,
    Calorie,
    Kilocalorie,
    ElectronVolt,
    KiloelectronVolt,
    MegaelectronVolt,

    // Power [M L^2 T^-3]
    Watt,
    Kilowatt,
    Megawatt,
    Milliwatt,

    // Pressure [M L^-1 T^-2]
    Pascal,
    Kilopascal,
    Megapascal,
    Bar,
    Millibar,
    Atmosphere,
    Torr,
    MmHg,

    // Volume [L^3]
    CubicMeter,
    Liter,
    Milliliter,
    Microliter,
    CubicCentimeter,
    CubicDecimeter,

    // Concentration / Molarity [L^-3 N]
    Molar,          // mol/L
    Millimolar,     // mmol/L
    Micromolar,     // μmol/L
    MolePerCubicMeter,

    // Molar Mass [M N^-1]
    GramPerMole,
    KilogramPerMole,

    // Molar Energy [M L^2 T^-2 N^-1]
    JoulePerMole,
    KilojoulePerMole,
    CaloriePerMole,
    KilocaloriePerMole,

    // Density [M L^-3]
    KilogramPerCubicMeter,
    GramPerCubicCentimeter,
    GramPerMilliliter,
    GramPerLiter,

    // Frequency [T^-1]
    Hertz,
    Kilohertz,
    Megahertz,
    Gigahertz,
}

impl Unit {
    pub fn dimension(&self) -> Dimension {
        match self {
            Unit::Dimensionless | Unit::Percent => Dimension::DIMENSIONLESS,
            Unit::Kilogram | Unit::Gram | Unit::Milligram | Unit::Microgram | Unit::Tonne => Dimension::MASS,
            Unit::Meter | Unit::Kilometer | Unit::Centimeter | Unit::Millimeter | Unit::Micrometer | Unit::Nanometer | Unit::Decimeter | Unit::Angstrom => Dimension::LENGTH,
            Unit::Second | Unit::Millisecond | Unit::Microsecond | Unit::Nanosecond | Unit::Minute | Unit::Hour | Unit::Day => Dimension::TIME,
            Unit::Mole | Unit::Millimole | Unit::Micromole | Unit::Kilomole => Dimension::AMOUNT,
            Unit::Kelvin | Unit::Celsius => Dimension::TEMPERATURE,
            Unit::MeterPerSecond | Unit::KilometerPerHour | Unit::KilometerPerSecond | Unit::CentimeterPerSecond | Unit::MilesPerHour => Dimension::VELOCITY,
            Unit::MeterPerSecondSquared | Unit::CentimeterPerSecondSquared => Dimension::ACCELERATION,
            Unit::Newton | Unit::Kilonewton | Unit::Millinewton | Unit::Dyne => Dimension::FORCE,
            Unit::Joule | Unit::Kilojoule | Unit::Millijoule | Unit::Calorie | Unit::Kilocalorie | Unit::ElectronVolt | Unit::KiloelectronVolt | Unit::MegaelectronVolt => Dimension::ENERGY,
            Unit::Watt | Unit::Kilowatt | Unit::Megawatt | Unit::Milliwatt => Dimension::POWER,
            Unit::Pascal | Unit::Kilopascal | Unit::Megapascal | Unit::Bar | Unit::Millibar | Unit::Atmosphere | Unit::Torr | Unit::MmHg => Dimension::PRESSURE,
            Unit::CubicMeter | Unit::Liter | Unit::Milliliter | Unit::Microliter | Unit::CubicCentimeter | Unit::CubicDecimeter => Dimension::VOLUME,
            Unit::Molar | Unit::Millimolar | Unit::Micromolar | Unit::MolePerCubicMeter => Dimension::CONCENTRATION,
            Unit::GramPerMole | Unit::KilogramPerMole => Dimension::MOLAR_MASS,
            Unit::JoulePerMole | Unit::KilojoulePerMole | Unit::CaloriePerMole | Unit::KilocaloriePerMole => Dimension::MOLAR_ENERGY,
            Unit::KilogramPerCubicMeter | Unit::GramPerCubicCentimeter | Unit::GramPerMilliliter | Unit::GramPerLiter => Dimension::DENSITY,
            Unit::Hertz | Unit::Kilohertz | Unit::Megahertz | Unit::Gigahertz => Dimension::FREQUENCY,
        }
    }

    pub fn to_si_multiplier(&self) -> f64 {
        match self {
            Unit::Dimensionless => 1.0,
            Unit::Percent => 0.01,
            Unit::Kilogram => 1.0,
            Unit::Gram => 1e-3,
            Unit::Milligram => 1e-6,
            Unit::Microgram => 1e-9,
            Unit::Tonne => 1e3,
            Unit::Meter => 1.0,
            Unit::Kilometer => 1e3,
            Unit::Centimeter => 1e-2,
            Unit::Millimeter => 1e-3,
            Unit::Micrometer => 1e-6,
            Unit::Nanometer => 1e-9,
            Unit::Decimeter => 0.1,
            Unit::Angstrom => 1e-10,
            Unit::Second => 1.0,
            Unit::Millisecond => 1e-3,
            Unit::Microsecond => 1e-6,
            Unit::Nanosecond => 1e-9,
            Unit::Minute => 60.0,
            Unit::Hour => 3600.0,
            Unit::Day => 86400.0,
            Unit::Mole => 1.0,
            Unit::Millimole => 1e-3,
            Unit::Micromole => 1e-6,
            Unit::Kilomole => 1e3,
            Unit::Kelvin | Unit::Celsius => 1.0,
            Unit::MeterPerSecond => 1.0,
            Unit::KilometerPerHour => 5.0 / 18.0,
            Unit::KilometerPerSecond => 1e3,
            Unit::CentimeterPerSecond => 0.01,
            Unit::MilesPerHour => 0.44704,
            Unit::MeterPerSecondSquared => 1.0,
            Unit::CentimeterPerSecondSquared => 0.01,
            Unit::Newton => 1.0,
            Unit::Kilonewton => 1e3,
            Unit::Millinewton => 1e-3,
            Unit::Dyne => 1e-5,
            Unit::Joule => 1.0,
            Unit::Kilojoule => 1e3,
            Unit::Millijoule => 1e-3,
            Unit::Calorie => 4.184,
            Unit::Kilocalorie => 4184.0,
            Unit::ElectronVolt => 1.602176634e-19,
            Unit::KiloelectronVolt => 1.602176634e-16,
            Unit::MegaelectronVolt => 1.602176634e-13,
            Unit::Watt => 1.0,
            Unit::Kilowatt => 1e3,
            Unit::Megawatt => 1e6,
            Unit::Milliwatt => 1e-3,
            Unit::Pascal => 1.0,
            Unit::Kilopascal => 1e3,
            Unit::Megapascal => 1e6,
            Unit::Bar => 1e5,
            Unit::Millibar => 100.0,
            Unit::Atmosphere => 101325.0,
            Unit::Torr | Unit::MmHg => 101325.0 / 760.0,
            Unit::CubicMeter => 1.0,
            Unit::Liter | Unit::CubicDecimeter => 1e-3,
            Unit::Milliliter | Unit::CubicCentimeter => 1e-6,
            Unit::Microliter => 1e-9,
            Unit::Molar => 1000.0,
            Unit::Millimolar => 1.0,
            Unit::Micromolar => 1e-3,
            Unit::MolePerCubicMeter => 1.0,
            Unit::KilogramPerMole => 1.0,
            Unit::GramPerMole => 1e-3,
            Unit::JoulePerMole => 1.0,
            Unit::KilojoulePerMole => 1e3,
            Unit::CaloriePerMole => 4.184,
            Unit::KilocaloriePerMole => 4184.0,
            Unit::KilogramPerCubicMeter | Unit::GramPerLiter => 1.0,
            Unit::GramPerCubicCentimeter | Unit::GramPerMilliliter => 1000.0,
            Unit::Hertz => 1.0,
            Unit::Kilohertz => 1e3,
            Unit::Megahertz => 1e6,
            Unit::Gigahertz => 1e9,
        }
    }

    pub fn offset_to_si(&self) -> f64 {
        match self {
            Unit::Celsius => 273.15,
            _ => 0.0,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Unit::Dimensionless => "",
            Unit::Percent => "%",
            Unit::Kilogram => "kg",
            Unit::Gram => "g",
            Unit::Milligram => "mg",
            Unit::Microgram => "μg",
            Unit::Tonne => "t",
            Unit::Meter => "m",
            Unit::Kilometer => "km",
            Unit::Centimeter => "cm",
            Unit::Millimeter => "mm",
            Unit::Micrometer => "μm",
            Unit::Nanometer => "nm",
            Unit::Decimeter => "dm",
            Unit::Angstrom => "Å",
            Unit::Second => "s",
            Unit::Millisecond => "ms",
            Unit::Microsecond => "μs",
            Unit::Nanosecond => "ns",
            Unit::Minute => "min",
            Unit::Hour => "h",
            Unit::Day => "d",
            Unit::Mole => "mol",
            Unit::Millimole => "mmol",
            Unit::Micromole => "μmol",
            Unit::Kilomole => "kmol",
            Unit::Kelvin => "K",
            Unit::Celsius => "°C",
            Unit::MeterPerSecond => "m/s",
            Unit::KilometerPerHour => "km/h",
            Unit::KilometerPerSecond => "km/s",
            Unit::CentimeterPerSecond => "cm/s",
            Unit::MilesPerHour => "mph",
            Unit::MeterPerSecondSquared => "m/s²",
            Unit::CentimeterPerSecondSquared => "cm/s²",
            Unit::Newton => "N",
            Unit::Kilonewton => "kN",
            Unit::Millinewton => "mN",
            Unit::Dyne => "dyn",
            Unit::Joule => "J",
            Unit::Kilojoule => "kJ",
            Unit::Millijoule => "mJ",
            Unit::Calorie => "cal",
            Unit::Kilocalorie => "kcal",
            Unit::ElectronVolt => "eV",
            Unit::KiloelectronVolt => "keV",
            Unit::MegaelectronVolt => "MeV",
            Unit::Watt => "W",
            Unit::Kilowatt => "kW",
            Unit::Megawatt => "MW",
            Unit::Milliwatt => "mW",
            Unit::Pascal => "Pa",
            Unit::Kilopascal => "kPa",
            Unit::Megapascal => "MPa",
            Unit::Bar => "bar",
            Unit::Millibar => "mbar",
            Unit::Atmosphere => "atm",
            Unit::Torr => "torr",
            Unit::MmHg => "mmHg",
            Unit::CubicMeter => "m³",
            Unit::Liter => "L",
            Unit::Milliliter => "mL",
            Unit::Microliter => "μL",
            Unit::CubicCentimeter => "cm³",
            Unit::CubicDecimeter => "dm³",
            Unit::Molar => "M",
            Unit::Millimolar => "mM",
            Unit::Micromolar => "μM",
            Unit::MolePerCubicMeter => "mol/m³",
            Unit::GramPerMole => "g/mol",
            Unit::KilogramPerMole => "kg/mol",
            Unit::JoulePerMole => "J/mol",
            Unit::KilojoulePerMole => "kJ/mol",
            Unit::CaloriePerMole => "cal/mol",
            Unit::KilocaloriePerMole => "kcal/mol",
            Unit::KilogramPerCubicMeter => "kg/m³",
            Unit::GramPerCubicCentimeter => "g/cm³",
            Unit::GramPerMilliliter => "g/mL",
            Unit::GramPerLiter => "g/L",
            Unit::Hertz => "Hz",
            Unit::Kilohertz => "kHz",
            Unit::Megahertz => "MHz",
            Unit::Gigahertz => "GHz",
        }
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.dimension().is_compatible_with(&other.dimension())
    }

    pub fn convert_to(&self, value: f64, target_unit: &Self) -> Option<f64> {
        if !self.is_compatible_with(target_unit) {
            return None;
        }
        if self.dimension() == Dimension::TEMPERATURE {
            let si_k = value * self.to_si_multiplier() + self.offset_to_si();
            let target_val = (si_k - target_unit.offset_to_si()) / target_unit.to_si_multiplier();
            return Some(target_val);
        }
        let si_val = value * self.to_si_multiplier();
        let target_mult = target_unit.to_si_multiplier();
        if target_mult == 0.0 {
            Some(si_val)
        } else {
            Some(si_val / target_mult)
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl FromStr for Unit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw == "M" {
            return Ok(Unit::Molar);
        }
        if raw == "mM" {
            return Ok(Unit::Millimolar);
        }
        if raw == "uM" || raw == "μM" {
            return Ok(Unit::Micromolar);
        }

        let norm = raw.to_lowercase()
            .replace('²', "^2")
            .replace('³', "^3")
            .replace('·', "*")
            .replace('μ', "u")
            .replace("°c", "degc")
            .replace('å', "angstrom");

        match norm.as_str() {
            "" | "dimensionless" | "none" | "1" | "scalar" | "ratio" => Ok(Unit::Dimensionless),
            "%" | "percent" | "percentage" | "pct" => Ok(Unit::Percent),
            "kg" | "kilogram" | "kilograms" | "kilo" => Ok(Unit::Kilogram),
            "g" | "gram" | "grams" | "gm" => Ok(Unit::Gram),
            "mg" | "milligram" | "milligrams" => Ok(Unit::Milligram),
            "ug" | "microgram" | "micrograms" => Ok(Unit::Microgram),
            "t" | "tonne" | "tonnes" | "ton" | "tons" => Ok(Unit::Tonne),
            "m" | "meter" | "meters" | "metre" | "metres" => Ok(Unit::Meter),
            "km" | "kilometer" | "kilometers" | "kilometre" | "kilometres" => Ok(Unit::Kilometer),
            "cm" | "centimeter" | "centimeters" | "centimetre" | "centimetres" => Ok(Unit::Centimeter),
            "mm" | "millimeter" | "millimeters" | "millimetre" | "millimetres" => Ok(Unit::Millimeter),
            "um" | "micrometer" | "micrometers" | "micron" | "microns" => Ok(Unit::Micrometer),
            "nm" | "nanometer" | "nanometers" => Ok(Unit::Nanometer),
            "dm" | "decimeter" | "decimeters" => Ok(Unit::Decimeter),
            "angstrom" | "angstroms" | "a" => Ok(Unit::Angstrom),
            "s" | "sec" | "secs" | "second" | "seconds" => Ok(Unit::Second),
            "ms" | "msec" | "millisecond" | "milliseconds" => Ok(Unit::Millisecond),
            "us" | "usec" | "microsecond" | "microseconds" => Ok(Unit::Microsecond),
            "ns" | "nanosecond" | "nanoseconds" => Ok(Unit::Nanosecond),
            "min" | "mins" | "minute" | "minutes" => Ok(Unit::Minute),
            "h" | "hr" | "hrs" | "hour" | "hours" => Ok(Unit::Hour),
            "d" | "day" | "days" => Ok(Unit::Day),
            "mol" | "mole" | "moles" => Ok(Unit::Mole),
            "mmol" | "millimole" | "millimoles" => Ok(Unit::Millimole),
            "umol" | "micromole" | "micromoles" => Ok(Unit::Micromole),
            "kmol" | "kilomole" | "kilomoles" => Ok(Unit::Kilomole),
            "k" | "kelvin" | "kelvins" => Ok(Unit::Kelvin),
            "degc" | "c" | "celsius" | "centigrade" | "deg c" | "degree celsius" => Ok(Unit::Celsius),
            "m/s" | "mps" | "m*s^-1" | "m s^-1" | "meter/second" | "meters/second" => Ok(Unit::MeterPerSecond),
            "km/h" | "kmh" | "kph" | "kmph" | "km/hr" | "kilometer/hour" => Ok(Unit::KilometerPerHour),
            "km/s" | "kmps" => Ok(Unit::KilometerPerSecond),
            "cm/s" | "cmps" => Ok(Unit::CentimeterPerSecond),
            "mph" | "miles/hour" | "mi/h" => Ok(Unit::MilesPerHour),
            "m/s^2" | "m/s2" | "mps2" | "mps^2" | "m*s^-2" | "m s^-2" => Ok(Unit::MeterPerSecondSquared),
            "cm/s^2" | "cm/s2" => Ok(Unit::CentimeterPerSecondSquared),
            "n" | "newton" | "newtons" => Ok(Unit::Newton),
            "kn" | "kilonewton" | "kilonewtons" => Ok(Unit::Kilonewton),
            "mn" | "millinewton" => Ok(Unit::Millinewton),
            "dyn" | "dyne" | "dynes" => Ok(Unit::Dyne),
            "j" | "joule" | "joules" => Ok(Unit::Joule),
            "kj" | "kilojoule" | "kilojoules" => Ok(Unit::Kilojoule),
            "mj" | "millijoule" => Ok(Unit::Millijoule),
            "cal" | "calorie" | "calories" => Ok(Unit::Calorie),
            "kcal" | "kilocalorie" | "kilocalories" => Ok(Unit::Kilocalorie),
            "ev" | "electronvolt" | "electronvolts" => Ok(Unit::ElectronVolt),
            "kev" => Ok(Unit::KiloelectronVolt),
            "mev" => Ok(Unit::MegaelectronVolt),
            "w" | "watt" | "watts" => Ok(Unit::Watt),
            "kw" | "kilowatt" | "kilowatts" => Ok(Unit::Kilowatt),
            "mw" | "megawatt" | "megawatts" => Ok(Unit::Megawatt),
            "milliwatt" => Ok(Unit::Milliwatt),
            "pa" | "pascal" | "pascals" | "n/m^2" | "n/m2" => Ok(Unit::Pascal),
            "kpa" | "kilopascal" | "kilopascals" => Ok(Unit::Kilopascal),
            "mpa" | "megapascal" => Ok(Unit::Megapascal),
            "bar" | "bars" => Ok(Unit::Bar),
            "mbar" | "millibar" => Ok(Unit::Millibar),
            "atm" | "atmosphere" | "atmospheres" => Ok(Unit::Atmosphere),
            "torr" => Ok(Unit::Torr),
            "mmhg" => Ok(Unit::MmHg),
            "m^3" | "m3" | "cubic meter" | "cubic meters" => Ok(Unit::CubicMeter),
            "l" | "liter" | "liters" | "litre" | "litres" | "dm^3" | "dm3" => Ok(Unit::Liter),
            "ml" | "milliliter" | "milliliters" | "millilitre" | "cc" | "cm^3" | "cm3" => Ok(Unit::Milliliter),
            "ul" | "microliter" | "microliters" => Ok(Unit::Microliter),
            "molar" | "mol/l" | "mol/liter" | "mol/dm^3" | "mol/dm3" | "mol*l^-1" | "mol l^-1" => Ok(Unit::Molar),
            "millimolar" | "mmol/l" | "mmol/liter" | "mmol/dm^3" | "mmol/dm3" => Ok(Unit::Millimolar),
            "umolar" | "micromolar" | "umol/l" | "umol/liter" => Ok(Unit::Micromolar),
            "mol/m^3" | "mol/m3" => Ok(Unit::MolePerCubicMeter),
            "g/mol" | "g*mol^-1" | "g mol^-1" | "grams/mole" => Ok(Unit::GramPerMole),
            "kg/mol" | "kg*mol^-1" => Ok(Unit::KilogramPerMole),
            "j/mol" | "j*mol^-1" => Ok(Unit::JoulePerMole),
            "kj/mol" | "kj*mol^-1" | "kilojoules/mole" => Ok(Unit::KilojoulePerMole),
            "cal/mol" => Ok(Unit::CaloriePerMole),
            "kcal/mol" => Ok(Unit::KilocaloriePerMole),
            "kg/m^3" | "kg/m3" | "g/l" | "g/liter" => Ok(Unit::KilogramPerCubicMeter),
            "g/cm^3" | "g/cm3" | "g/ml" | "g/milliliter" => Ok(Unit::GramPerCubicCentimeter),
            "hz" | "hertz" | "s^-1" | "1/s" => Ok(Unit::Hertz),
            "khz" | "kilohertz" => Ok(Unit::Kilohertz),
            "mhz" | "megahertz" => Ok(Unit::Megahertz),
            "ghz" | "gigahertz" => Ok(Unit::Gigahertz),
            _ => Err(format!("Unrecognized unit string: '{}'", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversions() {
        let mps = Unit::KilometerPerHour.convert_to(72.0, &Unit::MeterPerSecond).unwrap();
        assert!((mps - 20.0).abs() < 1e-6);
        let kmh = Unit::MeterPerSecond.convert_to(20.0, &Unit::KilometerPerHour).unwrap();
        assert!((kmh - 72.0).abs() < 1e-6);
        let kg = Unit::Gram.convert_to(500.0, &Unit::Kilogram).unwrap();
        assert!((kg - 0.5).abs() < 1e-6);
        let g = Unit::Kilogram.convert_to(2.5, &Unit::Gram).unwrap();
        assert!((g - 2500.0).abs() < 1e-6);
        let mmol = Unit::Mole.convert_to(0.045, &Unit::Millimole).unwrap();
        assert!((mmol - 45.0).abs() < 1e-6);
        let mm = Unit::Molar.convert_to(1.2e-3, &Unit::Millimolar).unwrap();
        assert!((mm - 1.2).abs() < 1e-6);
        let kpa = Unit::Atmosphere.convert_to(1.0, &Unit::Kilopascal).unwrap();
        assert!((kpa - 101.325).abs() < 1e-4);
        let k = Unit::Celsius.convert_to(25.0, &Unit::Kelvin).unwrap();
        assert!((k - 298.15).abs() < 1e-6);
        assert!(Unit::Second.convert_to(10.0, &Unit::Meter).is_none());
    }
}

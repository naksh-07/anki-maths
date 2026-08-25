// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

/**
 * Fundamental 5-dimensional vector for physical and chemical dimensional analysis:
 * [Mass]^m * [Length]^l * [Time]^t * [AmountOfSubstance]^n * [Temperature]^k
 */
export class PhysicalDimension {
    public readonly mass: number;        // [M] (kg)
    public readonly length: number;      // [L] (m)
    public readonly time: number;        // [T] (s)
    public readonly amount: number;      // [N] (mol)
    public readonly temperature: number; // [K] (K)

    constructor(mass = 0, length = 0, time = 0, amount = 0, temperature = 0) {
        this.mass = mass;
        this.length = length;
        this.time = time;
        this.amount = amount;
        this.temperature = temperature;
    }

    public static readonly DIMENSIONLESS = new PhysicalDimension(0, 0, 0, 0, 0);
    public static readonly MASS = new PhysicalDimension(1, 0, 0, 0, 0);
    public static readonly LENGTH = new PhysicalDimension(0, 1, 0, 0, 0);
    public static readonly TIME = new PhysicalDimension(0, 0, 1, 0, 0);
    public static readonly AMOUNT = new PhysicalDimension(0, 0, 0, 1, 0);
    public static readonly TEMPERATURE = new PhysicalDimension(0, 0, 0, 0, 1);

    // Derived kinematics & mechanics
    public static readonly AREA = new PhysicalDimension(0, 2, 0, 0, 0);
    public static readonly VOLUME = new PhysicalDimension(0, 3, 0, 0, 0);
    public static readonly VELOCITY = new PhysicalDimension(0, 1, -1, 0, 0);
    public static readonly ACCELERATION = new PhysicalDimension(0, 1, -2, 0, 0);
    public static readonly FORCE = new PhysicalDimension(1, 1, -2, 0, 0);
    public static readonly ENERGY = new PhysicalDimension(1, 2, -2, 0, 0);
    public static readonly POWER = new PhysicalDimension(1, 2, -3, 0, 0);
    public static readonly PRESSURE = new PhysicalDimension(1, -1, -2, 0, 0);
    public static readonly DENSITY = new PhysicalDimension(1, -3, 0, 0, 0);
    public static readonly FREQUENCY = new PhysicalDimension(0, 0, -1, 0, 0);

    // Derived chemistry & thermodynamics
    public static readonly CONCENTRATION = new PhysicalDimension(0, -3, 0, 1, 0);
    public static readonly MOLAR_MASS = new PhysicalDimension(1, 0, 0, -1, 0);
    public static readonly MOLAR_ENERGY = new PhysicalDimension(1, 2, -2, -1, 0);
    public static readonly MOLAR_VOLUME = new PhysicalDimension(0, 3, 0, -1, 0);
    public static readonly MOLAR_HEAT_CAPACITY = new PhysicalDimension(1, 2, -2, -1, -1);
    public static readonly SPECIFIC_HEAT_CAPACITY = new PhysicalDimension(0, 2, -2, 0, -1);

    public isDimensionless(): boolean {
        return (
            this.mass === 0 &&
            this.length === 0 &&
            this.time === 0 &&
            this.amount === 0 &&
            this.temperature === 0
        );
    }

    public isCompatibleWith(other: PhysicalDimension): boolean {
        return (
            this.mass === other.mass &&
            this.length === other.length &&
            this.time === other.time &&
            this.amount === other.amount &&
            this.temperature === other.temperature
        );
    }

    public multiply(other: PhysicalDimension): PhysicalDimension {
        return new PhysicalDimension(
            this.mass + other.mass,
            this.length + other.length,
            this.time + other.time,
            this.amount + other.amount,
            this.temperature + other.temperature,
        );
    }

    public divide(other: PhysicalDimension): PhysicalDimension {
        return new PhysicalDimension(
            this.mass - other.mass,
            this.length - other.length,
            this.time - other.time,
            this.amount - other.amount,
            this.temperature - other.temperature,
        );
    }

    public pow(exp: number): PhysicalDimension {
        return new PhysicalDimension(
            this.mass * exp,
            this.length * exp,
            this.time * exp,
            this.amount * exp,
            this.temperature * exp,
        );
    }

    public toString(): string {
        if (this.isDimensionless()) {
            return "1 (dimensionless)";
        }
        const parts: string[] = [];
        if (this.mass !== 0) {parts.push(`[M]^${this.mass}`);}
        if (this.length !== 0) {parts.push(`[L]^${this.length}`);}
        if (this.time !== 0) {parts.push(`[T]^${this.time}`);}
        if (this.amount !== 0) {parts.push(`[N]^${this.amount}`);}
        if (this.temperature !== 0) {parts.push(`[K]^${this.temperature}`);}
        return parts.join(" * ");
    }
}

/**
 * Physical/Chemical unit definition with dimensional mapping and conversion scaling.
 */
export interface PhysicalUnit {
    id: string;
    symbol: string;
    dimension: PhysicalDimension;
    toSiMultiplier: number;
    offsetToSi: number;
    aliases: string[];
}

/**
 * Global registry and conversion engine for all canonical Physics and Chemistry units.
 */
export class UnitRegistry {
    private static units: Map<string, PhysicalUnit> = new Map();
    private static aliasMap: Map<string, PhysicalUnit> = new Map();

    private static register(
        id: string,
        symbol: string,
        dimension: PhysicalDimension,
        toSiMultiplier = 1.0,
        offsetToSi = 0.0,
        aliases: string[] = [],
    ): PhysicalUnit {
        const u: PhysicalUnit = {
            id,
            symbol,
            dimension,
            toSiMultiplier,
            offsetToSi,
            aliases,
        };
        this.units.set(id, u);
        this.aliasMap.set(symbol.toLowerCase(), u);
        this.aliasMap.set(id.toLowerCase(), u);
        for (const a of aliases) {
            this.aliasMap.set(a.toLowerCase(), u);
        }
        return u;
    }

    // Initialize units table
    public static readonly DIMENSIONLESS = UnitRegistry.register(
        "dimensionless", "", PhysicalDimension.DIMENSIONLESS, 1.0, 0.0,
        ["none", "1", "scalar", "ratio"],
    );
    public static readonly PERCENT = UnitRegistry.register(
        "percent", "%", PhysicalDimension.DIMENSIONLESS, 0.01, 0.0,
        ["pct", "percentage"],
    );

    // Mass [M]
    public static readonly KILOGRAM = UnitRegistry.register(
        "kilogram", "kg", PhysicalDimension.MASS, 1.0, 0.0,
        ["kilograms", "kilo", "kilos"],
    );
    public static readonly GRAM = UnitRegistry.register(
        "gram", "g", PhysicalDimension.MASS, 1e-3, 0.0,
        ["grams", "gm", "gms"],
    );
    public static readonly MILLIGRAM = UnitRegistry.register(
        "milligram", "mg", PhysicalDimension.MASS, 1e-6, 0.0,
        ["milligrams"],
    );
    public static readonly MICROGRAM = UnitRegistry.register(
        "microgram", "μg", PhysicalDimension.MASS, 1e-9, 0.0,
        ["ug", "micrograms", "mcg"],
    );
    public static readonly TONNE = UnitRegistry.register(
        "tonne", "t", PhysicalDimension.MASS, 1e3, 0.0,
        ["ton", "tonnes", "tons", "metric ton"],
    );

    // Length [L]
    public static readonly METER = UnitRegistry.register(
        "meter", "m", PhysicalDimension.LENGTH, 1.0, 0.0,
        ["meters", "metre", "metres"],
    );
    public static readonly KILOMETER = UnitRegistry.register(
        "kilometer", "km", PhysicalDimension.LENGTH, 1e3, 0.0,
        ["kilometers", "kilometre", "kilometres"],
    );
    public static readonly CENTIMETER = UnitRegistry.register(
        "centimeter", "cm", PhysicalDimension.LENGTH, 1e-2, 0.0,
        ["centimeters", "centimetre", "centimetres"],
    );
    public static readonly MILLIMETER = UnitRegistry.register(
        "millimeter", "mm", PhysicalDimension.LENGTH, 1e-3, 0.0,
        ["millimeters", "millimetre", "millimetres"],
    );
    public static readonly MICROMETER = UnitRegistry.register(
        "micrometer", "μm", PhysicalDimension.LENGTH, 1e-6, 0.0,
        ["um", "micrometers", "micron", "microns"],
    );
    public static readonly NANOMETER = UnitRegistry.register(
        "nanometer", "nm", PhysicalDimension.LENGTH, 1e-9, 0.0,
        ["nanometers"],
    );
    public static readonly DECIMETER = UnitRegistry.register(
        "decimeter", "dm", PhysicalDimension.LENGTH, 0.1, 0.0,
        ["decimeters", "decimetre"],
    );
    public static readonly ANGSTROM = UnitRegistry.register(
        "angstrom", "Å", PhysicalDimension.LENGTH, 1e-10, 0.0,
        ["angstroms", "a", "å"],
    );

    // Time [T]
    public static readonly SECOND = UnitRegistry.register(
        "second", "s", PhysicalDimension.TIME, 1.0, 0.0,
        ["sec", "secs", "seconds"],
    );
    public static readonly MILLISECOND = UnitRegistry.register(
        "millisecond", "ms", PhysicalDimension.TIME, 1e-3, 0.0,
        ["msec", "milliseconds"],
    );
    public static readonly MICROSECOND = UnitRegistry.register(
        "microsecond", "μs", PhysicalDimension.TIME, 1e-6, 0.0,
        ["us", "usec", "microseconds"],
    );
    public static readonly NANOSECOND = UnitRegistry.register(
        "nanosecond", "ns", PhysicalDimension.TIME, 1e-9, 0.0,
        ["nanoseconds"],
    );
    public static readonly MINUTE = UnitRegistry.register(
        "minute", "min", PhysicalDimension.TIME, 60.0, 0.0,
        ["mins", "minutes"],
    );
    public static readonly HOUR = UnitRegistry.register(
        "hour", "h", PhysicalDimension.TIME, 3600.0, 0.0,
        ["hr", "hrs", "hours"],
    );
    public static readonly DAY = UnitRegistry.register(
        "day", "d", PhysicalDimension.TIME, 86400.0, 0.0,
        ["days"],
    );

    // Amount of Substance [N]
    public static readonly MOLE = UnitRegistry.register(
        "mole", "mol", PhysicalDimension.AMOUNT, 1.0, 0.0,
        ["moles"],
    );
    public static readonly MILLIMOLE = UnitRegistry.register(
        "millimole", "mmol", PhysicalDimension.AMOUNT, 1e-3, 0.0,
        ["millimoles"],
    );
    public static readonly MICROMOLE = UnitRegistry.register(
        "micromole", "μmol", PhysicalDimension.AMOUNT, 1e-6, 0.0,
        ["umol", "micromoles"],
    );
    public static readonly KILOMOLE = UnitRegistry.register(
        "kilomole", "kmol", PhysicalDimension.AMOUNT, 1e3, 0.0,
        ["kilomoles"],
    );

    // Temperature [K]
    public static readonly KELVIN = UnitRegistry.register(
        "kelvin", "K", PhysicalDimension.TEMPERATURE, 1.0, 0.0,
        ["k", "kelvins"],
    );
    public static readonly CELSIUS = UnitRegistry.register(
        "celsius", "°C", PhysicalDimension.TEMPERATURE, 1.0, 273.15,
        ["degc", "c", "centigrade", "deg c", "degree celsius", "degrees celsius"],
    );

    // Velocity / Speed [L T^-1]
    public static readonly METER_PER_SECOND = UnitRegistry.register(
        "meter_per_second", "m/s", PhysicalDimension.VELOCITY, 1.0, 0.0,
        ["mps", "m*s^-1", "m s^-1", "meter/second", "meters/second"],
    );
    public static readonly KILOMETER_PER_HOUR = UnitRegistry.register(
        "kilometer_per_hour", "km/h", PhysicalDimension.VELOCITY, 5.0 / 18.0, 0.0,
        ["kmh", "kph", "kmph", "km/hr", "kilometer/hour", "kilometers/hour"],
    );
    public static readonly KILOMETER_PER_SECOND = UnitRegistry.register(
        "kilometer_per_second", "km/s", PhysicalDimension.VELOCITY, 1e3, 0.0,
        ["kmps"],
    );
    public static readonly CENTIMETER_PER_SECOND = UnitRegistry.register(
        "centimeter_per_second", "cm/s", PhysicalDimension.VELOCITY, 0.01, 0.0,
        ["cmps"],
    );
    public static readonly MILES_PER_HOUR = UnitRegistry.register(
        "miles_per_hour", "mph", PhysicalDimension.VELOCITY, 0.44704, 0.0,
        ["miles/hour", "mi/h"],
    );

    // Acceleration [L T^-2]
    public static readonly METER_PER_SECOND_SQUARED = UnitRegistry.register(
        "meter_per_second_squared", "m/s²", PhysicalDimension.ACCELERATION, 1.0, 0.0,
        ["m/s^2", "m/s2", "mps2", "mps^2", "m*s^-2", "m s^-2", "m/s2"],
    );
    public static readonly CENTIMETER_PER_SECOND_SQUARED = UnitRegistry.register(
        "centimeter_per_second_squared", "cm/s²", PhysicalDimension.ACCELERATION, 0.01, 0.0,
        ["cm/s^2", "cm/s2"],
    );

    // Force [M L T^-2]
    public static readonly NEWTON = UnitRegistry.register(
        "newton", "N", PhysicalDimension.FORCE, 1.0, 0.0,
        ["newtons"],
    );
    public static readonly KILONEWTON = UnitRegistry.register(
        "kilonewton", "kN", PhysicalDimension.FORCE, 1e3, 0.0,
        ["kilonewtons"],
    );
    public static readonly MILLINEWTON = UnitRegistry.register(
        "millinewton", "mN", PhysicalDimension.FORCE, 1e-3, 0.0,
        ["millinewtons"],
    );
    public static readonly DYNE = UnitRegistry.register(
        "dyne", "dyn", PhysicalDimension.FORCE, 1e-5, 0.0,
        ["dynes"],
    );

    // Energy / Work / Heat [M L^2 T^-2]
    public static readonly JOULE = UnitRegistry.register(
        "joule", "J", PhysicalDimension.ENERGY, 1.0, 0.0,
        ["joules"],
    );
    public static readonly KILOJOULE = UnitRegistry.register(
        "kilojoule", "kJ", PhysicalDimension.ENERGY, 1e3, 0.0,
        ["kilojoules"],
    );
    public static readonly MILLIJOULE = UnitRegistry.register(
        "millijoule", "mJ", PhysicalDimension.ENERGY, 1e-3, 0.0,
        ["millijoules"],
    );
    public static readonly CALORIE = UnitRegistry.register(
        "calorie", "cal", PhysicalDimension.ENERGY, 4.184, 0.0,
        ["calories"],
    );
    public static readonly KILOCALORIE = UnitRegistry.register(
        "kilocalorie", "kcal", PhysicalDimension.ENERGY, 4184.0, 0.0,
        ["kilocalories"],
    );
    public static readonly ELECTRON_VOLT = UnitRegistry.register(
        "electron_volt", "eV", PhysicalDimension.ENERGY, 1.602176634e-19, 0.0,
        ["electronvolt", "electronvolts", "ev"],
    );
    public static readonly KILOELECTRON_VOLT = UnitRegistry.register(
        "kiloelectron_volt", "keV", PhysicalDimension.ENERGY, 1.602176634e-16, 0.0,
        ["kev"],
    );
    public static readonly MEGAELECTRON_VOLT = UnitRegistry.register(
        "megaelectron_volt", "MeV", PhysicalDimension.ENERGY, 1.602176634e-13, 0.0,
        ["mev"],
    );

    // Power [M L^2 T^-3]
    public static readonly WATT = UnitRegistry.register(
        "watt", "W", PhysicalDimension.POWER, 1.0, 0.0,
        ["watts"],
    );
    public static readonly KILOWATT = UnitRegistry.register(
        "kilowatt", "kW", PhysicalDimension.POWER, 1e3, 0.0,
        ["kilowatts"],
    );
    public static readonly MEGAWATT = UnitRegistry.register(
        "megawatt", "MW", PhysicalDimension.POWER, 1e6, 0.0,
        ["megawatts"],
    );
    public static readonly MILLIWATT = UnitRegistry.register(
        "milliwatt", "mW", PhysicalDimension.POWER, 1e-3, 0.0,
        ["milliwatts"],
    );

    // Pressure [M L^-1 T^-2]
    public static readonly PASCAL = UnitRegistry.register(
        "pascal", "Pa", PhysicalDimension.PRESSURE, 1.0, 0.0,
        ["pascals", "n/m^2", "n/m2", "n*m^-2"],
    );
    public static readonly KILOPASCAL = UnitRegistry.register(
        "kilopascal", "kPa", PhysicalDimension.PRESSURE, 1e3, 0.0,
        ["kilopascals"],
    );
    public static readonly MEGAPASCAL = UnitRegistry.register(
        "megapascal", "MPa", PhysicalDimension.PRESSURE, 1e6, 0.0,
        ["megapascals"],
    );
    public static readonly BAR = UnitRegistry.register(
        "bar", "bar", PhysicalDimension.PRESSURE, 1e5, 0.0,
        ["bars"],
    );
    public static readonly MILLIBAR = UnitRegistry.register(
        "millibar", "mbar", PhysicalDimension.PRESSURE, 100.0, 0.0,
        ["millibars"],
    );
    public static readonly ATMOSPHERE = UnitRegistry.register(
        "atmosphere", "atm", PhysicalDimension.PRESSURE, 101325.0, 0.0,
        ["atmospheres"],
    );
    public static readonly TORR = UnitRegistry.register(
        "torr", "torr", PhysicalDimension.PRESSURE, 101325.0 / 760.0, 0.0,
        ["mmhg"],
    );

    // Volume [L^3]
    public static readonly CUBIC_METER = UnitRegistry.register(
        "cubic_meter", "m³", PhysicalDimension.VOLUME, 1.0, 0.0,
        ["m^3", "m3", "cubic meter", "cubic meters"],
    );
    public static readonly LITER = UnitRegistry.register(
        "liter", "L", PhysicalDimension.VOLUME, 1e-3, 0.0,
        ["l", "liters", "litre", "litres", "dm^3", "dm3"],
    );
    public static readonly MILLILITER = UnitRegistry.register(
        "milliliter", "mL", PhysicalDimension.VOLUME, 1e-6, 0.0,
        ["ml", "milliliters", "millilitre", "cc", "cm^3", "cm3"],
    );
    public static readonly MICROLITER = UnitRegistry.register(
        "microliter", "μL", PhysicalDimension.VOLUME, 1e-9, 0.0,
        ["ul", "microliters"],
    );

    // Concentration / Molarity [L^-3 N]
    public static readonly MOLAR = UnitRegistry.register(
        "molar", "M", PhysicalDimension.CONCENTRATION, 1000.0, 0.0,
        ["mol/l", "mol/liter", "mol/dm^3", "mol/dm3", "mol*l^-1", "mol l^-1"],
    );
    public static readonly MILLIMOLAR = UnitRegistry.register(
        "millimolar", "mM", PhysicalDimension.CONCENTRATION, 1.0, 0.0,
        ["mmol/l", "mmol/liter", "mmol/dm^3", "mmol/dm3"],
    );
    public static readonly MICROMOLAR = UnitRegistry.register(
        "micromolar", "μM", PhysicalDimension.CONCENTRATION, 1e-3, 0.0,
        ["um", "umolar", "umol/l", "umol/liter"],
    );
    public static readonly MOLE_PER_CUBIC_METER = UnitRegistry.register(
        "mole_per_cubic_meter", "mol/m³", PhysicalDimension.CONCENTRATION, 1.0, 0.0,
        ["mol/m^3", "mol/m3"],
    );

    // Molar Mass [M N^-1]
    public static readonly GRAM_PER_MOLE = UnitRegistry.register(
        "gram_per_mole", "g/mol", PhysicalDimension.MOLAR_MASS, 1e-3, 0.0,
        ["g*mol^-1", "g mol^-1", "grams/mole"],
    );
    public static readonly KILOGRAM_PER_MOLE = UnitRegistry.register(
        "kilogram_per_mole", "kg/mol", PhysicalDimension.MOLAR_MASS, 1.0, 0.0,
        ["kg*mol^-1", "kg mol^-1"],
    );

    // Molar Energy [M L^2 T^-2 N^-1]
    public static readonly JOULE_PER_MOLE = UnitRegistry.register(
        "joule_per_mole", "J/mol", PhysicalDimension.MOLAR_ENERGY, 1.0, 0.0,
        ["j*mol^-1", "j mol^-1"],
    );
    public static readonly KILOJOULE_PER_MOLE = UnitRegistry.register(
        "kilojoule_per_mole", "kJ/mol", PhysicalDimension.MOLAR_ENERGY, 1e3, 0.0,
        ["kj*mol^-1", "kj mol^-1", "kilojoules/mole"],
    );
    public static readonly CALORIE_PER_MOLE = UnitRegistry.register(
        "calorie_per_mole", "cal/mol", PhysicalDimension.MOLAR_ENERGY, 4.184, 0.0,
        ["cal*mol^-1"],
    );
    public static readonly KILOCALORIE_PER_MOLE = UnitRegistry.register(
        "kilocalorie_per_mole", "kcal/mol", PhysicalDimension.MOLAR_ENERGY, 4184.0, 0.0,
        ["kcal*mol^-1"],
    );

    // Density [M L^-3]
    public static readonly KILOGRAM_PER_CUBIC_METER = UnitRegistry.register(
        "kilogram_per_cubic_meter", "kg/m³", PhysicalDimension.DENSITY, 1.0, 0.0,
        ["kg/m^3", "kg/m3", "g/l", "g/liter"],
    );
    public static readonly GRAM_PER_CUBIC_CENTIMETER = UnitRegistry.register(
        "gram_per_cubic_centimeter", "g/cm³", PhysicalDimension.DENSITY, 1000.0, 0.0,
        ["g/cm^3", "g/cm3", "g/ml", "g/milliliter"],
    );

    // Frequency [T^-1]
    public static readonly HERTZ = UnitRegistry.register(
        "hertz", "Hz", PhysicalDimension.FREQUENCY, 1.0, 0.0,
        ["s^-1", "1/s"],
    );
    public static readonly KILOHERTZ = UnitRegistry.register(
        "kilohertz", "kHz", PhysicalDimension.FREQUENCY, 1e3, 0.0,
        ["khz"],
    );
    public static readonly MEGAHERTZ = UnitRegistry.register(
        "megahertz", "MHz", PhysicalDimension.FREQUENCY, 1e6, 0.0,
        ["mhz"],
    );
    public static readonly GIGAHERTZ = UnitRegistry.register(
        "gigahertz", "GHz", PhysicalDimension.FREQUENCY, 1e9, 0.0,
        ["ghz"],
    );

    /**
     * Find a unit by exact symbol, name, or registered alias.
     */
    public static findUnit(unitStr: string): PhysicalUnit | null {
        const raw = unitStr.trim();
        if (!raw) {return null;}
        if (raw === "M") {return UnitRegistry.MOLAR;}
        if (raw === "mM") {return UnitRegistry.MILLIMOLAR;}
        if (raw === "uM" || raw === "μM") {return UnitRegistry.MICROMOLAR;}

        const norm = raw
            .toLowerCase()
            .replace(/²/g, "^2")
            .replace(/³/g, "^3")
            .replace(/⁻¹/g, "^-1")
            .replace(/⁻²/g, "^-2")
            .replace(/⁻³/g, "^-3")
            .replace(/·/g, "*")
            .replace(/•/g, "*")
            .replace(/μ/g, "u")
            .replace(/µ/g, "u")
            .replace(/°c/g, "degc")
            .replace(/℃/g, "degc")
            .replace(/å/g, "angstrom")
            .replace(/Å/g, "angstrom");

        return this.aliasMap.get(norm) || null;
    }

    /**
     * Convert value from source unit to target unit.
     * Returns null if dimensions are incompatible.
     */
    public static convert(value: number, fromUnit: PhysicalUnit, toUnit: PhysicalUnit): number | null {
        if (!fromUnit.dimension.isCompatibleWith(toUnit.dimension)) {
            return null;
        }
        if (fromUnit.dimension.isCompatibleWith(PhysicalDimension.TEMPERATURE)) {
            const siK = value * fromUnit.toSiMultiplier + fromUnit.offsetToSi;
            return (siK - toUnit.offsetToSi) / toUnit.toSiMultiplier;
        }
        const siVal = value * fromUnit.toSiMultiplier;
        const targetMult = toUnit.toSiMultiplier;
        return targetMult === 0 ? siVal : siVal / targetMult;
    }
}

/**
 * Standard Physical & Chemical Constants
 */
export const PHYSICAL_CONSTANTS = {
    AVOGADRO: 6.02214076e23,      // mol^-1
    GAS_CONSTANT: 8.314462618,    // J / (mol K)
    SPEED_OF_LIGHT: 299792458,    // m / s
    GRAVITY: 9.80665,             // m / s^2 (standard)
    GRAVITY_ACCEL: 9.8,           // m / s^2 (common textbook)
    PLANCK: 6.62607015e-34,       // J s
    FARADAY: 96485.33212,         // C / mol
    BOLTZMANN: 1.380649e-23,      // J / K
    ELEMENTARY_CHARGE: 1.602176634e-19, // C
    PI: Math.PI,
} as const;

/**
 * Result of parsing a numerical input with optional unit.
 */
export interface NumericalParseResult {
    value: number;
    unit: PhysicalUnit | null;
    rawUnitStr: string | null;
    hasExplicitUnit: boolean;
    isValid: boolean;
}

/**
 * Numerical tolerance specification.
 */
export interface NumericalTolerance {
    type: "absolute" | "relative" | "combined";
    absolute?: number;
    relative?: number;
}

/**
 * Robust parser for physical & chemical numerical responses in TypeScript.
 */
export class NumericalParser {
    /**
     * Parse any student numeric input (e.g. `12 m/s`, `5 kg`, `1.2e-3 mol/L`, `6.022 x 10^23`, `3/4`).
     */
    public static parse(input: string | null | undefined): NumericalParseResult {
        if (!input) {
            return { value: NaN, unit: null, rawUnitStr: null, hasExplicitUnit: false, isValid: false };
        }

        let s = String(input).trim();
        if (!s) {
            return { value: NaN, unit: null, rawUnitStr: null, hasExplicitUnit: false, isValid: false };
        }

        // 1. Strip equation prefixes (e.g. "v = ", "x = ", "m = ", "[H+] = ", "ans = ", "answer: ")
        const eqIdx = s.search(/[:=]/);
        if (eqIdx !== -1) {
            const prefix = s.slice(0, eqIdx).trim();
            if (prefix.length > 0 && /^[a-zA-Z_\s[\]+-]+$/.test(prefix)) {
                s = s.slice(eqIdx + 1).trim();
            }
        }

        // 2. Remove currency symbols and comma separators
        const cleaned = s.replace(/[$€£₹,]/g, "").trim();
        if (!cleaned) {
            return { value: NaN, unit: null, rawUnitStr: null, hasExplicitUnit: false, isValid: false };
        }

        // 3. Percent handling (e.g. "75%", "+32%")
        if (cleaned.endsWith("%")) {
            const numStr = cleaned.slice(0, -1).trim();
            const n = parseFloat(numStr);
            if (!isNaN(n) && isFinite(n)) {
                return {
                    value: n,
                    unit: UnitRegistry.PERCENT,
                    rawUnitStr: "%",
                    hasExplicitUnit: true,
                    isValid: true,
                };
            }
        }

        // 4. Unicode Exponent / Math normalization
        const normalized = cleaned
            .replace(/⁰/g, "0")
            .replace(/¹/g, "1")
            .replace(/²/g, "2")
            .replace(/³/g, "3")
            .replace(/⁴/g, "4")
            .replace(/⁵/g, "5")
            .replace(/⁶/g, "6")
            .replace(/⁷/g, "7")
            .replace(/⁸/g, "8")
            .replace(/⁹/g, "9")
            .replace(/⁻/g, "-")
            .replace(/⁺/g, "+")
            .replace(/[·•]/g, "*")
            .replace(/×/g, "x");

        // 5. Scientific notation with x 10^ or * 10^ or e
        const sciMatch = normalized.match(
            /^([+-]?(?:\d+\.?\d*|\.\d+))\s*(?:[x*]\s*10\^?([+-]?\d+)|e([+-]?\d+))\s*(.*)$/i,
        );
        if (sciMatch) {
            const mantissa = parseFloat(sciMatch[1]);
            const expStr = sciMatch[2] || sciMatch[3];
            const exponent = parseInt(expStr, 10);
            if (!isNaN(mantissa) && !isNaN(exponent)) {
                const totalVal = mantissa * Math.pow(10, exponent);
                if (!isNaN(totalVal) && isFinite(totalVal)) {
                    const unitStr = sciMatch[4]?.trim() || "";
                    if (!unitStr) {
                        return {
                            value: totalVal,
                            unit: UnitRegistry.DIMENSIONLESS,
                            rawUnitStr: null,
                            hasExplicitUnit: false,
                            isValid: true,
                        };
                    }
                    const parsedUnit = UnitRegistry.findUnit(unitStr);
                    return {
                        value: totalVal,
                        unit: parsedUnit,
                        rawUnitStr: unitStr,
                        hasExplicitUnit: true,
                        isValid: true,
                    };
                }
            }
        }

        // 6. Arithmetic fraction handling (e.g. "3/4", "3/4 m/s", "7/8 kg")
        const slashIdx = normalized.indexOf("/");
        if (slashIdx !== -1) {
            const leftPart = normalized.slice(0, slashIdx).trim();
            const rightPart = normalized.slice(slashIdx + 1).trim();
            // Check if slash is for units (e.g., m/s or mol/L) or numerical fraction
            const isUnitSlash = /[a-zA-Z]/.test(leftPart) && !/\d/.test(leftPart);
            if (!isUnitSlash) {
                const num = parseFloat(leftPart);
                const denMatch = rightPart.match(/^([+-]?(?:\d+\.?\d*|\.\d+))\s*(.*)$/);
                if (!isNaN(num) && denMatch) {
                    const den = parseFloat(denMatch[1]);
                    if (!isNaN(den) && Math.abs(den) > Number.EPSILON) {
                        const fracVal = num / den;
                        if (!isNaN(fracVal) && isFinite(fracVal)) {
                            const unitStr = denMatch[2]?.trim() || "";
                            if (!unitStr) {
                                return {
                                    value: fracVal,
                                    unit: UnitRegistry.DIMENSIONLESS,
                                    rawUnitStr: null,
                                    hasExplicitUnit: false,
                                    isValid: true,
                                };
                            }
                            const parsedUnit = UnitRegistry.findUnit(unitStr);
                            return {
                                value: fracVal,
                                unit: parsedUnit,
                                rawUnitStr: unitStr,
                                hasExplicitUnit: true,
                                isValid: true,
                            };
                        }
                    }
                }
            }
        }

        // 7. Leading float extraction with unit remainder (e.g. "12 m/s", "5kg", "1.2e-3 mol/L", "-9.8 m/s^2")
        const floatMatch = normalized.match(/^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)\s*(.*)$/);
        if (floatMatch) {
            const numVal = parseFloat(floatMatch[1]);
            if (!isNaN(numVal) && isFinite(numVal)) {
                const rem = floatMatch[2]?.trim() || "";
                if (!rem) {
                    return {
                        value: numVal,
                        unit: UnitRegistry.DIMENSIONLESS,
                        rawUnitStr: null,
                        hasExplicitUnit: false,
                        isValid: true,
                    };
                }
                const parsedUnit = UnitRegistry.findUnit(rem);
                return {
                    value: numVal,
                    unit: parsedUnit,
                    rawUnitStr: rem,
                    hasExplicitUnit: true,
                    isValid: true,
                };
            }
        }

        return { value: NaN, unit: null, rawUnitStr: null, hasExplicitUnit: false, isValid: false };
    }

    /**
     * Backward-compatible scalar extractor for simple numerical inputs.
     */
    public static parseScalar(input: string | null | undefined): number | null {
        const res = this.parse(input);
        return res.isValid ? res.value : null;
    }
}

/**
 * Result of comprehensive Physics & Chemistry numerical evaluation.
 */
export interface NumericalEvaluationResult {
    isCorrect: boolean;
    score: number;
    studentValue: number;
    studentUnit: string | null;
    convertedValue: number;
    expectedValue: number;
    expectedUnit: string | null;
    errorCategory?: "calculation" | "unit" | "concept" | "sign" | "unknown";
    reason?: string;
    diagnosticMessage?: string;
}

/**
 * Configuration options for NumericalContainer.
 */
export interface NumericalContainerOptions {
    container?: HTMLElement;
    inputElement?: HTMLInputElement | null;
    submitButton?: HTMLElement | null;
    correctAnswer?: Record<string, any>;
    expectedValue?: number;
    expectedUnit?: string | PhysicalUnit;
    tolerance?: number | NumericalTolerance;
    requireUnit?: boolean;
    enforceNonNegative?: boolean;
    targetTimeMs?: number;
    onEvaluation?: (result: NumericalEvaluationResult) => void;
    onInputChange?: (parseResult: NumericalParseResult) => void;
}

/**
 * Production-grade Numerical Modality Container & Unit Parser.
 *
 * Implements:
 * - Dedicated numeric input handling with scientific notation, fractions, units.
 * - 5D physical & chemical dimensional analysis and cross-unit conversion.
 * - Absolute and relative tolerance checking.
 * - Prevention of NaN / crash errors on complex or malformed inputs.
 * - Clean UI lifecycle and keyboard handling (Enter / Escape / input preview).
 */
export class NumericalContainer {
    private container: HTMLElement;
    private inputEl: HTMLInputElement | null = null;
    private submitBtn: HTMLElement | null = null;
    private unitBadgeEl: HTMLElement | null = null;
    private previewPillEl: HTMLElement | null = null;
    private options: NumericalContainerOptions;
    private disposables: Array<() => void> = [];
    private isEvaluated = false;

    constructor(container: HTMLElement, options: NumericalContainerOptions = {}) {
        this.container = container;
        this.options = options;
        this.init();
    }

    private addListener(
        element: EventTarget | null,
        type: string,
        listener: EventListenerOrEventListenerObject,
        opts?: boolean | AddEventListenerOptions,
    ): void {
        if (!element) {return;}
        element.addEventListener(type, listener, opts);
        this.disposables.push(() => {
            element.removeEventListener(type, listener, opts);
        });
    }

    private init(): void {
        this.discoverOrRenderElements();
        this.attachEventListeners();
        this.updatePreview();
    }

    private discoverOrRenderElements(): void {
        this.inputEl =
            this.options.inputElement ||
            this.container.querySelector<HTMLInputElement>("#proc-answer-input") ||
            this.container.querySelector<HTMLInputElement>(".proc-answer-input");

        this.submitBtn =
            this.options.submitButton ||
            this.container.querySelector<HTMLElement>("#proc-quick-submit") ||
            this.container.querySelector<HTMLElement>(".proc-quick-submit") ||
            this.container.querySelector<HTMLElement>("#proc-submit-btn");

        // Make sure container is visible
        const quickContainer = this.container.querySelector<HTMLElement>("#proc-quick-container");
        if (quickContainer) {
            quickContainer.classList.remove("hidden");
            quickContainer.style.display = "";
        }

        // Bind existing preview pill if present in DOM
        this.previewPillEl =
            this.container.querySelector<HTMLElement>("#proc-num-preview") ||
            this.container.querySelector<HTMLElement>(".proc-num-preview-pill");

        // Setup unit hint badge if explicitly present in DOM
        this.unitBadgeEl = this.container.querySelector<HTMLElement>(".proc-unit-hint");

        // Setup accessibility attributes
        if (this.inputEl) {
            this.inputEl.setAttribute("aria-label", "Numeric Answer with optional unit");
            this.inputEl.setAttribute("autocomplete", "off");
            this.inputEl.setAttribute("spellcheck", "false");
        }
    }

    private attachEventListeners(): void {
        if (this.inputEl) {
            this.addListener(this.inputEl, "input", () => {
                this.updatePreview();
                if (this.options.onInputChange && this.inputEl) {
                    const parsed = NumericalParser.parse(this.inputEl.value);
                    this.options.onInputChange(parsed);
                }
            });

            this.addListener(this.inputEl, "keydown", (e: Event) => {
                const kb = e as KeyboardEvent;
                if (kb.key === "Enter") {
                    kb.preventDefault();
                    this.submit();
                } else if (kb.key === "Escape") {
                    if (this.inputEl) {
                        this.inputEl.value = "";
                        this.updatePreview();
                    }
                }
            });
        }

        if (this.submitBtn) {
            this.addListener(this.submitBtn, "click", (e: Event) => {
                e.preventDefault();
                this.submit();
            });
        }
    }

    private updatePreview(): void {
        if (!this.inputEl) {return;}
        const val = this.inputEl.value.trim();
        if (!val) {
            if (this.previewPillEl) {
                this.previewPillEl.textContent = "";
                this.previewPillEl.classList.add("hidden");
                this.previewPillEl.style.display = "none";
            }
            return;
        }

        const parsed = NumericalParser.parse(val);
        if (!this.previewPillEl && this.inputEl.parentElement) {
            this.previewPillEl = document.createElement("div");
            this.previewPillEl.className = "proc-num-preview-pill";
            this.previewPillEl.id = "proc-num-preview";
            this.previewPillEl.style.cssText =
                "font-size: 0.78em; color: var(--text-muted, #666); margin-top: 4px; transition: opacity 0.2s;";
            this.inputEl.parentElement.appendChild(this.previewPillEl);
        }

        if (this.previewPillEl) {
            if (parsed.isValid) {
                const unitStr = parsed.unit?.symbol || parsed.rawUnitStr || "";
                this.previewPillEl.textContent = `Parsed: ${parsed.value} ${unitStr}`.trim();
                this.previewPillEl.classList.remove("hidden");
                this.previewPillEl.style.display = "block";
                this.previewPillEl.style.color = "var(--text-muted, #666)";
            } else {
                this.previewPillEl.textContent = "Enter a valid number, fraction, or scientific notation";
                this.previewPillEl.classList.remove("hidden");
                this.previewPillEl.style.display = "block";
                this.previewPillEl.style.color = "var(--danger, #dc3545)";
            }
        }
    }

    public getExpectedUnit(): PhysicalUnit | null {
        if (this.options.expectedUnit) {
            if (typeof this.options.expectedUnit === "string") {
                return UnitRegistry.findUnit(this.options.expectedUnit) || UnitRegistry.DIMENSIONLESS;
            }
            return this.options.expectedUnit;
        }
        if (this.options.correctAnswer?.unit) {
            return UnitRegistry.findUnit(String(this.options.correctAnswer.unit)) || UnitRegistry.DIMENSIONLESS;
        }
        return UnitRegistry.DIMENSIONLESS;
    }

    public getExpectedValue(): number | null {
        if (this.options.expectedValue !== undefined) {
            return this.options.expectedValue;
        }
        if (this.options.correctAnswer?.value !== undefined) {
            return Number(this.options.correctAnswer.value);
        }
        if (typeof this.options.correctAnswer?.answer === "number") {
            return this.options.correctAnswer.answer;
        }
        if (typeof this.options.correctAnswer?.answer === "string") {
            return NumericalParser.parseScalar(this.options.correctAnswer.answer);
        }
        return null;
    }

    public getTolerance(): NumericalTolerance {
        if (typeof this.options.tolerance === "number") {
            return { type: "absolute", absolute: this.options.tolerance };
        }
        if (this.options.tolerance && typeof this.options.tolerance === "object") {
            return this.options.tolerance;
        }
        if (this.options.correctAnswer?.tolerance !== undefined) {
            const tolVal = this.options.correctAnswer.tolerance;
            if (typeof tolVal === "number") {
                return { type: "absolute", absolute: tolVal };
            }
            if (typeof tolVal === "object") {
                return tolVal as NumericalTolerance;
            }
        }
        const expVal = this.getExpectedValue() || 0;
        return {
            type: "combined",
            absolute: 0.01,
            relative: 0.005, // 0.5% default relative tolerance
        };
    }

    public isWithinTolerance(actual: number, expected: number, tol: NumericalTolerance): boolean {
        if (isNaN(actual) || isNaN(expected) || !isFinite(actual) || !isFinite(expected)) {
            return false;
        }
        const diff = Math.abs(actual - expected);
        if (tol.type === "absolute") {
            const absTol = tol.absolute !== undefined ? Math.abs(tol.absolute) : 0.01;
            return diff <= absTol;
        }
        if (tol.type === "relative") {
            const relTol = tol.relative !== undefined ? Math.abs(tol.relative) : 0.01;
            return diff <= Math.abs(expected) * relTol;
        }
        const absTol = tol.absolute !== undefined ? Math.abs(tol.absolute) : 0.01;
        const relTol = tol.relative !== undefined ? Math.abs(tol.relative) : 0.005;
        return diff <= Math.max(absTol, Math.abs(expected) * relTol);
    }

    /**
     * Evaluate a student submission against the expected value, unit, and tolerance.
     */
    public evaluate(userText?: string): NumericalEvaluationResult {
        const text = userText !== undefined ? userText : (this.inputEl?.value || "");
        const parsed = NumericalParser.parse(text);
        const expectedVal = this.getExpectedValue();
        const expectedUnit = this.getExpectedUnit() || UnitRegistry.DIMENSIONLESS;
        const tol = this.getTolerance();

        if (expectedVal === null || isNaN(expectedVal)) {
            // Fallback string matching if problem has no numerical answer
            const canonical = String(
                this.options.correctAnswer?.formatted ||
                this.options.correctAnswer?.correct_option ||
                this.options.correctAnswer?.answer ||
                "",
            ).trim().toLowerCase();
            const isMatch = text.trim().toLowerCase() === canonical && canonical.length > 0;
            return {
                isCorrect: isMatch,
                score: isMatch ? 1.0 : 0.0,
                studentValue: parsed.value,
                studentUnit: parsed.unit?.symbol || parsed.rawUnitStr || null,
                convertedValue: parsed.value,
                expectedValue: 0,
                expectedUnit: null,
                reason: isMatch ? undefined : "Answer does not match canonical solution.",
            };
        }

        if (!parsed.isValid || isNaN(parsed.value)) {
            return {
                isCorrect: false,
                score: 0.0,
                studentValue: NaN,
                studentUnit: null,
                convertedValue: NaN,
                expectedValue: expectedVal,
                expectedUnit: expectedUnit.symbol,
                errorCategory: "calculation",
                reason: "Please enter a valid numeric value, fraction, or physical quantity.",
                diagnosticMessage: "Invalid numerical format.",
            };
        }

        // Non-negative physical sanity check
        const isNaturallyNonNegative =
            expectedUnit.dimension.mass > 0 ||
            expectedUnit.dimension.length > 0 ||
            expectedUnit.dimension.amount > 0 ||
            (expectedUnit.dimension.time > 0 && expectedVal >= 0);

        if (
            (this.options.enforceNonNegative ?? true) &&
            isNaturallyNonNegative &&
            parsed.value < -1e-6
        ) {
            return {
                isCorrect: false,
                score: 0.0,
                studentValue: parsed.value,
                studentUnit: parsed.unit?.symbol || parsed.rawUnitStr || null,
                convertedValue: parsed.value,
                expectedValue: expectedVal,
                expectedUnit: expectedUnit.symbol,
                errorCategory: "concept",
                reason: `Physical Sanity Violation: Quantity cannot be negative (received ${parsed.value}).`,
                diagnosticMessage: "Physical quantities such as mass, amount, and distance must be non-negative.",
            };
        }

        // Required unit check
        if (
            this.options.requireUnit &&
            expectedUnit !== UnitRegistry.DIMENSIONLESS &&
            !parsed.hasExplicitUnit
        ) {
            return {
                isCorrect: false,
                score: 0.0,
                studentValue: parsed.value,
                studentUnit: null,
                convertedValue: parsed.value,
                expectedValue: expectedVal,
                expectedUnit: expectedUnit.symbol,
                errorCategory: "unit",
                reason: `Missing Unit: An explicit unit is required for this answer (e.g. ${expectedUnit.symbol}).`,
                diagnosticMessage: `Please include the correct unit (${expectedUnit.symbol}).`,
            };
        }

        // Unrecognized unit check
        if (parsed.rawUnitStr && !parsed.unit) {
            return {
                isCorrect: false,
                score: 0.0,
                studentValue: parsed.value,
                studentUnit: parsed.rawUnitStr,
                convertedValue: parsed.value,
                expectedValue: expectedVal,
                expectedUnit: expectedUnit.symbol,
                errorCategory: "unit",
                reason: `Unrecognized unit '${parsed.rawUnitStr}'. Expected physical dimension: ${expectedUnit.dimension} (${expectedUnit.symbol}).`,
                diagnosticMessage: `Unknown unit '${parsed.rawUnitStr}'.`,
            };
        }

        // Unit conversion & dimensional compatibility
        let convertedVal = parsed.value;
        if (
            parsed.unit &&
            parsed.unit !== UnitRegistry.DIMENSIONLESS &&
            expectedUnit !== UnitRegistry.DIMENSIONLESS
        ) {
            if (!parsed.unit.dimension.isCompatibleWith(expectedUnit.dimension)) {
                return {
                    isCorrect: false,
                    score: 0.0,
                    studentValue: parsed.value,
                    studentUnit: parsed.unit.symbol,
                    convertedValue: parsed.value,
                    expectedValue: expectedVal,
                    expectedUnit: expectedUnit.symbol,
                    errorCategory: "unit",
                    reason: `Dimensional Incompatibility: Received unit '${parsed.unit.symbol}' (${parsed.unit.dimension}), but expected dimension ${expectedUnit.dimension} (${expectedUnit.symbol}).`,
                    diagnosticMessage: `Incompatible physical dimension.`,
                };
            }

            const conv = UnitRegistry.convert(parsed.value, parsed.unit, expectedUnit);
            if (conv !== null) {
                convertedVal = conv;
            }
        }

        const isCorrect = this.isWithinTolerance(convertedVal, expectedVal, tol);
        if (isCorrect) {
            const hasUnitConversion =
                parsed.unit &&
                parsed.unit !== expectedUnit &&
                parsed.unit !== UnitRegistry.DIMENSIONLESS;

            return {
                isCorrect: true,
                score: 1.0,
                studentValue: parsed.value,
                studentUnit: parsed.unit?.symbol || null,
                convertedValue: convertedVal,
                expectedValue: expectedVal,
                expectedUnit: expectedUnit.symbol || null,
                diagnosticMessage: hasUnitConversion
                    ? `✓ Correct (${parsed.value} ${parsed.unit?.symbol} converts to ${convertedVal.toFixed(4)} ${expectedUnit.symbol})`
                    : "✓ Correct answer.",
            };
        }

        // Heuristics for common unit mistakes
        if (expectedUnit === UnitRegistry.METER_PER_SECOND) {
            const kmhVal = UnitRegistry.convert(parsed.value, UnitRegistry.KILOMETER_PER_HOUR, UnitRegistry.METER_PER_SECOND);
            if (kmhVal !== null && this.isWithinTolerance(kmhVal, expectedVal, tol)) {
                return {
                    isCorrect: false,
                    score: 0.0,
                    studentValue: parsed.value,
                    studentUnit: parsed.unit?.symbol || null,
                    convertedValue: convertedVal,
                    expectedValue: expectedVal,
                    expectedUnit: expectedUnit.symbol,
                    errorCategory: "unit",
                    reason: `Missing Unit Conversion: You answered ${parsed.value} (km/h) without converting to SI unit m/s (expected ${expectedVal} m/s = ${parsed.value} * 5/18).`,
                    diagnosticMessage: `Convert km/h to m/s by multiplying by 5/18.`,
                };
            }
        }

        if (expectedUnit === UnitRegistry.KILOGRAM) {
            const gVal = UnitRegistry.convert(parsed.value, UnitRegistry.GRAM, UnitRegistry.KILOGRAM);
            if (gVal !== null && this.isWithinTolerance(gVal, expectedVal, tol)) {
                return {
                    isCorrect: false,
                    score: 0.0,
                    studentValue: parsed.value,
                    studentUnit: parsed.unit?.symbol || null,
                    convertedValue: convertedVal,
                    expectedValue: expectedVal,
                    expectedUnit: expectedUnit.symbol,
                    errorCategory: "unit",
                    reason: `Missing Unit Conversion: You answered ${parsed.value} (g) without converting to SI unit kg (expected ${expectedVal} kg = ${parsed.value} / 1000).`,
                    diagnosticMessage: `Convert grams to kilograms by dividing by 1000.`,
                };
            }
        }

        if (expectedUnit === UnitRegistry.MOLAR) {
            const mmVal = UnitRegistry.convert(parsed.value, UnitRegistry.MILLIMOLAR, UnitRegistry.MOLAR);
            if (mmVal !== null && this.isWithinTolerance(mmVal, expectedVal, tol)) {
                return {
                    isCorrect: false,
                    score: 0.0,
                    studentValue: parsed.value,
                    studentUnit: parsed.unit?.symbol || null,
                    convertedValue: convertedVal,
                    expectedValue: expectedVal,
                    expectedUnit: expectedUnit.symbol,
                    errorCategory: "unit",
                    reason: `Missing Unit Conversion: You answered ${parsed.value} (mM) without converting to Molar (expected ${expectedVal} M = ${parsed.value} / 1000).`,
                    diagnosticMessage: `Convert millimolar to molar by dividing by 1000.`,
                };
            }
        }

        return {
            isCorrect: false,
            score: 0.0,
            studentValue: parsed.value,
            studentUnit: parsed.unit?.symbol || null,
            convertedValue: convertedVal,
            expectedValue: expectedVal,
            expectedUnit: expectedUnit.symbol || null,
            errorCategory: "calculation",
            reason: `Calculation Error: Expected ${expectedVal} ${expectedUnit.symbol || ""}, but received ${text}.`.trim(),
            diagnosticMessage: `Expected ${expectedVal} ${expectedUnit.symbol || ""}`.trim(),
        };
    }

    public submit(): void {
        if (this.isEvaluated) {return;}
        const result = this.evaluate();
        this.isEvaluated = true;
        if (this.options.onEvaluation) {
            this.options.onEvaluation(result);
        }
    }

    public destroy(): void {
        for (const dispose of this.disposables) {
            try {
                dispose();
            } catch {
                // Ignore cleanup errors
            }
        }
        this.disposables = [];
        if (this.previewPillEl && this.previewPillEl.parentElement) {
            this.previewPillEl.parentElement.removeChild(this.previewPillEl);
            this.previewPillEl = null;
        }
    }
}

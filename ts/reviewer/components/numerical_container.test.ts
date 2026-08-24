// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { beforeEach, describe, expect, test } from "vitest";

import {
    NumericalContainer,
    NumericalParser,
    PHYSICAL_CONSTANTS,
    PhysicalDimension,
    UnitRegistry,
} from "./numerical_container";

describe("PhysicalDimension & 5D Dimensional Vector", () => {
    test("dimensionless vector identity", () => {
        const dim = PhysicalDimension.DIMENSIONLESS;
        expect(dim.isDimensionless()).toBe(true);
        expect(dim.mass).toBe(0);
        expect(dim.length).toBe(0);
        expect(dim.time).toBe(0);
        expect(dim.amount).toBe(0);
        expect(dim.temperature).toBe(0);
        expect(dim.toString()).toBe("1 (dimensionless)");
    });

    test("kinematics and mechanics dimensional algebra", () => {
        // [L] / [T] = Velocity
        const vel = PhysicalDimension.LENGTH.divide(PhysicalDimension.TIME);
        expect(vel.isCompatibleWith(PhysicalDimension.VELOCITY)).toBe(true);

        // Velocity / [T] = Acceleration
        const accel = vel.divide(PhysicalDimension.TIME);
        expect(accel.isCompatibleWith(PhysicalDimension.ACCELERATION)).toBe(true);

        // [M] * Acceleration = Force
        const force = PhysicalDimension.MASS.multiply(accel);
        expect(force.isCompatibleWith(PhysicalDimension.FORCE)).toBe(true);

        // Force * [L] = Energy
        const energy = force.multiply(PhysicalDimension.LENGTH);
        expect(energy.isCompatibleWith(PhysicalDimension.ENERGY)).toBe(true);

        // Energy / [T] = Power
        const power = energy.divide(PhysicalDimension.TIME);
        expect(power.isCompatibleWith(PhysicalDimension.POWER)).toBe(true);

        // Force / Area = Pressure
        const pressure = force.divide(PhysicalDimension.AREA);
        expect(pressure.isCompatibleWith(PhysicalDimension.PRESSURE)).toBe(true);
    });

    test("chemistry and thermodynamics dimensional algebra", () => {
        // [N] / Volume = Concentration
        const conc = PhysicalDimension.AMOUNT.divide(PhysicalDimension.VOLUME);
        expect(conc.isCompatibleWith(PhysicalDimension.CONCENTRATION)).toBe(true);

        // [M] / [N] = Molar Mass
        const molarMass = PhysicalDimension.MASS.divide(PhysicalDimension.AMOUNT);
        expect(molarMass.isCompatibleWith(PhysicalDimension.MOLAR_MASS)).toBe(true);

        // Energy / [N] = Molar Energy
        const molarEnergy = PhysicalDimension.ENERGY.divide(PhysicalDimension.AMOUNT);
        expect(molarEnergy.isCompatibleWith(PhysicalDimension.MOLAR_ENERGY)).toBe(true);
    });
});

describe("UnitRegistry & Cross-Unit Conversions", () => {
    test("velocity unit conversions", () => {
        // 72 km/h == 20 m/s
        const mps = UnitRegistry.convert(72.0, UnitRegistry.KILOMETER_PER_HOUR, UnitRegistry.METER_PER_SECOND);
        expect(mps).toBeCloseTo(20.0, 5);

        // 20 m/s == 72 km/h
        const kmh = UnitRegistry.convert(20.0, UnitRegistry.METER_PER_SECOND, UnitRegistry.KILOMETER_PER_HOUR);
        expect(kmh).toBeCloseTo(72.0, 5);

        // 100 cm/s == 1 m/s
        const mpsFromCm = UnitRegistry.convert(100.0, UnitRegistry.CENTIMETER_PER_SECOND, UnitRegistry.METER_PER_SECOND);
        expect(mpsFromCm).toBeCloseTo(1.0, 5);
    });

    test("mass unit conversions", () => {
        // 2500 g == 2.5 kg
        const kg = UnitRegistry.convert(2500.0, UnitRegistry.GRAM, UnitRegistry.KILOGRAM);
        expect(kg).toBeCloseTo(2.5, 5);

        // 1500 mg == 1.5 g
        const g = UnitRegistry.convert(1500.0, UnitRegistry.MILLIGRAM, UnitRegistry.GRAM);
        expect(g).toBeCloseTo(1.5, 5);

        // 2 tonne == 2000 kg
        const kgFromTonne = UnitRegistry.convert(2.0, UnitRegistry.TONNE, UnitRegistry.KILOGRAM);
        expect(kgFromTonne).toBeCloseTo(2000.0, 5);
    });

    test("chemistry concentration & molar mass conversions", () => {
        // 1.2 mM == 0.0012 M
        const m = UnitRegistry.convert(1.2, UnitRegistry.MILLIMOLAR, UnitRegistry.MOLAR);
        expect(m).toBeCloseTo(0.0012, 6);

        // 0.045 M == 45 mM
        const mm = UnitRegistry.convert(0.045, UnitRegistry.MOLAR, UnitRegistry.MILLIMOLAR);
        expect(mm).toBeCloseTo(45.0, 5);

        // 18.015 g/mol == 0.018015 kg/mol
        const kgMol = UnitRegistry.convert(18.015, UnitRegistry.GRAM_PER_MOLE, UnitRegistry.KILOGRAM_PER_MOLE);
        expect(kgMol).toBeCloseTo(0.018015, 6);
    });

    test("temperature conversions with offset", () => {
        // 25 °C == 298.15 K
        const k = UnitRegistry.convert(25.0, UnitRegistry.CELSIUS, UnitRegistry.KELVIN);
        expect(k).toBeCloseTo(298.15, 4);

        // 373.15 K == 100 °C
        const c = UnitRegistry.convert(373.15, UnitRegistry.KELVIN, UnitRegistry.CELSIUS);
        expect(c).toBeCloseTo(100.0, 4);

        // 0 °C == 273.15 K
        const kZero = UnitRegistry.convert(0.0, UnitRegistry.CELSIUS, UnitRegistry.KELVIN);
        expect(kZero).toBeCloseTo(273.15, 4);
    });

    test("pressure and energy conversions", () => {
        // 1 atm == 101.325 kPa
        const kpa = UnitRegistry.convert(1.0, UnitRegistry.ATMOSPHERE, UnitRegistry.KILOPASCAL);
        expect(kpa).toBeCloseTo(101.325, 3);

        // 1 bar == 100 kPa
        const kpaBar = UnitRegistry.convert(1.0, UnitRegistry.BAR, UnitRegistry.KILOPASCAL);
        expect(kpaBar).toBeCloseTo(100.0, 3);

        // 500 cal == 2092 J
        const j = UnitRegistry.convert(500.0, UnitRegistry.CALORIE, UnitRegistry.JOULE);
        expect(j).toBeCloseTo(2092.0, 2);

        // 1 eV == 1.602176634e-19 J
        const jEv = UnitRegistry.convert(1.0, UnitRegistry.ELECTRON_VOLT, UnitRegistry.JOULE);
        expect(jEv).toBeCloseTo(1.602176634e-19, 25);
    });

    test("incompatible dimensions return null", () => {
        const invalid = UnitRegistry.convert(10.0, UnitRegistry.SECOND, UnitRegistry.METER);
        expect(invalid).toBeNull();
        const invalid2 = UnitRegistry.convert(5.0, UnitRegistry.KILOGRAM, UnitRegistry.JOULE);
        expect(invalid2).toBeNull();
    });

    test("physical constants lookup and sanity", () => {
        expect(PHYSICAL_CONSTANTS.AVOGADRO).toBeCloseTo(6.02214076e23, 10);
        expect(PHYSICAL_CONSTANTS.GAS_CONSTANT).toBeCloseTo(8.31446, 4);
        expect(PHYSICAL_CONSTANTS.GRAVITY).toBeCloseTo(9.80665, 4);
        expect(PHYSICAL_CONSTANTS.SPEED_OF_LIGHT).toBe(299792458);
    });
});

describe("NumericalParser Robust Parsing", () => {
    test("parses standard decimal and signed floats", () => {
        const p1 = NumericalParser.parse("12.5");
        expect(p1.isValid).toBe(true);
        expect(p1.value).toBe(12.5);
        expect(p1.unit).toBe(UnitRegistry.DIMENSIONLESS);

        const p2 = NumericalParser.parse("-9.8");
        expect(p2.isValid).toBe(true);
        expect(p2.value).toBe(-9.8);

        const p3 = NumericalParser.parse("+42.00");
        expect(p3.isValid).toBe(true);
        expect(p3.value).toBe(42.0);
    });

    test("parses equations, prefixes, and variables", () => {
        const p1 = NumericalParser.parse("v = 15.5 m/s");
        expect(p1.isValid).toBe(true);
        expect(p1.value).toBe(15.5);
        expect(p1.unit).toBe(UnitRegistry.METER_PER_SECOND);

        const p2 = NumericalParser.parse("x: 3.2 km");
        expect(p2.isValid).toBe(true);
        expect(p2.value).toBe(3.2);
        expect(p2.unit).toBe(UnitRegistry.KILOMETER);

        const p3 = NumericalParser.parse("[H+] = 1.0e-7 M");
        expect(p3.isValid).toBe(true);
        expect(p3.value).toBeCloseTo(1e-7, 10);
        expect(p3.unit).toBe(UnitRegistry.MOLAR);

        const p4 = NumericalParser.parse("ans = 100");
        expect(p4.isValid).toBe(true);
        expect(p4.value).toBe(100);
    });

    test("parses fractions with and without units", () => {
        const p1 = NumericalParser.parse("3/4");
        expect(p1.isValid).toBe(true);
        expect(p1.value).toBe(0.75);

        const p2 = NumericalParser.parse("3/4 m/s");
        expect(p2.isValid).toBe(true);
        expect(p2.value).toBe(0.75);
        expect(p2.unit).toBe(UnitRegistry.METER_PER_SECOND);

        const p3 = NumericalParser.parse("-1/2 kg");
        expect(p3.isValid).toBe(true);
        expect(p3.value).toBe(-0.5);
        expect(p3.unit).toBe(UnitRegistry.KILOGRAM);
    });

    test("parses scientific notation in multiple formats", () => {
        // Standard exponential
        const p1 = NumericalParser.parse("1.2e-3 mol/L");
        expect(p1.isValid).toBe(true);
        expect(p1.value).toBeCloseTo(0.0012, 6);
        expect(p1.unit).toBe(UnitRegistry.MOLAR);

        const p2 = NumericalParser.parse("6.022E23");
        expect(p2.isValid).toBe(true);
        expect(Math.abs(p2.value - 6.022e23) / 6.022e23).toBeLessThan(1e-6);

        // Multiplication exponential: x 10^ or * 10^
        const p3 = NumericalParser.parse("1.2 x 10^-3 mol/L");
        expect(p3.isValid).toBe(true);
        expect(p3.value).toBeCloseTo(0.0012, 6);
        expect(p3.unit).toBe(UnitRegistry.MOLAR);

        const p4 = NumericalParser.parse("3x10^4 J");
        expect(p4.isValid).toBe(true);
        expect(p4.value).toBe(30000);
        expect(p4.unit).toBe(UnitRegistry.JOULE);

        // Unicode exponent superscripts
        const p5 = NumericalParser.parse("6.022 x 10²³ mol^-1");
        expect(p5.isValid).toBe(true);
        expect(Math.abs(p5.value - 6.022e23) / 6.022e23).toBeLessThan(1e-6);

        const p6 = NumericalParser.parse("1.2 × 10⁻³ M");
        expect(p6.isValid).toBe(true);
        expect(p6.value).toBeCloseTo(0.0012, 6);
        expect(p6.unit).toBe(UnitRegistry.MOLAR);
    });

    test("parses compound and complex units", () => {
        const p1 = NumericalParser.parse("-9.8 m/s^2");
        expect(p1.isValid).toBe(true);
        expect(p1.value).toBe(-9.8);
        expect(p1.unit).toBe(UnitRegistry.METER_PER_SECOND_SQUARED);

        const p2 = NumericalParser.parse("50.5 kJ/mol");
        expect(p2.isValid).toBe(true);
        expect(p2.value).toBe(50.5);
        expect(p2.unit).toBe(UnitRegistry.KILOJOULE_PER_MOLE);

        const p3 = NumericalParser.parse("1.03 g/cm^3");
        expect(p3.isValid).toBe(true);
        expect(p3.value).toBe(1.03);
        expect(p3.unit).toBe(UnitRegistry.GRAM_PER_CUBIC_CENTIMETER);

        const p4 = NumericalParser.parse("25 °C");
        expect(p4.isValid).toBe(true);
        expect(p4.value).toBe(25);
        expect(p4.unit).toBe(UnitRegistry.CELSIUS);
    });

    test("parses percentages and currency formatting safely", () => {
        const p1 = NumericalParser.parse("75%");
        expect(p1.isValid).toBe(true);
        expect(p1.value).toBe(75);
        expect(p1.unit).toBe(UnitRegistry.PERCENT);

        const p2 = NumericalParser.parse("$1,250.50");
        expect(p2.isValid).toBe(true);
        expect(p2.value).toBe(1250.5);
    });

    test("handles malformed inputs cleanly without NaN crashes", () => {
        expect(NumericalParser.parse("").isValid).toBe(false);
        expect(NumericalParser.parse("   ").isValid).toBe(false);
        expect(NumericalParser.parse("invalid_string").isValid).toBe(false);
        expect(NumericalParser.parse("///").isValid).toBe(false);
        expect(NumericalParser.parseScalar("invalid_string")).toBeNull();
    });
});

describe("NumericalContainer Component & Evaluation", () => {
    let container: HTMLElement;

    beforeEach(() => {
        container = document.createElement("div");
        container.innerHTML = `
            <div id="proc-quick-container">
                <input id="proc-answer-input" type="text" />
                <button id="proc-quick-submit">Submit</button>
            </div>
        `;
        document.body.appendChild(container);
    });

    test("evaluates exact numeric answer with unit", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 20.0,
            expectedUnit: UnitRegistry.METER_PER_SECOND,
        });

        const res = numContainer.evaluate("20 m/s");
        expect(res.isCorrect).toBe(true);
        expect(res.score).toBe(1.0);
        expect(res.convertedValue).toBe(20.0);

        numContainer.destroy();
    });

    test("evaluates cross-unit equivalence in Physics (72 km/h == 20 m/s)", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 20.0,
            expectedUnit: UnitRegistry.METER_PER_SECOND,
        });

        const res = numContainer.evaluate("72 km/h");
        expect(res.isCorrect).toBe(true);
        expect(res.score).toBe(1.0);
        expect(res.convertedValue).toBeCloseTo(20.0, 5);
        expect(res.diagnosticMessage).toContain("72 km/h converts to 20.0000 m/s");

        numContainer.destroy();
    });

    test("evaluates cross-unit equivalence in Chemistry (1.2 mM == 0.0012 M)", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 0.0012,
            expectedUnit: UnitRegistry.MOLAR,
        });

        const res = numContainer.evaluate("1.2 mM");
        expect(res.isCorrect).toBe(true);
        expect(res.score).toBe(1.0);
        expect(res.convertedValue).toBeCloseTo(0.0012, 6);

        numContainer.destroy();
    });

    test("evaluates mass conversions (2500 g == 2.5 kg)", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 2.5,
            expectedUnit: UnitRegistry.KILOGRAM,
        });

        const res = numContainer.evaluate("2500 g");
        expect(res.isCorrect).toBe(true);
        expect(res.score).toBe(1.0);
        expect(res.convertedValue).toBeCloseTo(2.5, 5);

        numContainer.destroy();
    });

    test("evaluates scientific notation (1.2 x 10^-3 mol/L == 0.0012 M)", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 0.0012,
            expectedUnit: UnitRegistry.MOLAR,
        });

        const res = numContainer.evaluate("1.2 x 10^-3 mol/L");
        expect(res.isCorrect).toBe(true);
        expect(res.score).toBe(1.0);

        numContainer.destroy();
    });

    test("rejects dimensional incompatibilities with clear diagnostic", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 20.0,
            expectedUnit: UnitRegistry.METER_PER_SECOND, // [L T^-1]
        });

        const res = numContainer.evaluate("20 kg"); // [M]
        expect(res.isCorrect).toBe(false);
        expect(res.errorCategory).toBe("unit");
        expect(res.reason).toContain("Dimensional Incompatibility");
        expect(res.reason).toContain("kg");

        numContainer.destroy();
    });

    test("rejects unrecognized units with helpful message", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 20.0,
            expectedUnit: UnitRegistry.METER_PER_SECOND,
        });

        const res = numContainer.evaluate("20 foobars");
        expect(res.isCorrect).toBe(false);
        expect(res.errorCategory).toBe("unit");
        expect(res.reason).toContain("Unrecognized unit 'foobars'");

        numContainer.destroy();
    });

    test("identifies missing unit conversion mistake (72 km/h without converting to m/s)", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 20.0,
            expectedUnit: UnitRegistry.METER_PER_SECOND,
        });

        const res = numContainer.evaluate("72");
        expect(res.isCorrect).toBe(false);
        expect(res.errorCategory).toBe("unit");
        expect(res.reason).toContain("Missing Unit Conversion");
        expect(res.reason).toContain("5/18");

        numContainer.destroy();
    });

    test("enforces physical non-negative sanity for mass/moles", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 5.0,
            expectedUnit: UnitRegistry.KILOGRAM,
            enforceNonNegative: true,
        });

        const res = numContainer.evaluate("-5 kg");
        expect(res.isCorrect).toBe(false);
        expect(res.errorCategory).toBe("concept");
        expect(res.reason).toContain("Physical Sanity Violation");

        numContainer.destroy();
    });

    test("tolerance evaluation (absolute and relative)", () => {
        const numContainer = new NumericalContainer(container, {
            expectedValue: 100.0,
            tolerance: { type: "relative", relative: 0.02 }, // 2%
        });

        // 101.5 is within 2% of 100
        expect(numContainer.evaluate("101.5").isCorrect).toBe(true);
        // 103.0 is outside 2% of 100
        expect(numContainer.evaluate("103.0").isCorrect).toBe(false);

        numContainer.destroy();
    });

    test("UI lifecycle: live typing preview, keyboard submit, and teardown", () => {
        let submittedResult: any = null;
        const numContainer = new NumericalContainer(container, {
            expectedValue: 15.0,
            onEvaluation: (r) => {
                submittedResult = r;
            },
        });

        const inputEl = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        inputEl.value = "15";
        inputEl.dispatchEvent(new Event("input"));

        const pill = container.querySelector<HTMLElement>(".proc-num-preview-pill");
        expect(pill).not.toBeNull();
        expect(pill?.textContent).toContain("Parsed: 15");

        // Submit via enter
        inputEl.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
        expect(submittedResult).not.toBeNull();
        expect(submittedResult.isCorrect).toBe(true);

        numContainer.destroy();
    });
});

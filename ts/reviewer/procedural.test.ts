// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { proceduralAPI, ProceduralReviewer } from "./procedural";

describe("ProceduralReviewer API", () => {
    let container: HTMLDivElement;

    beforeEach(() => {
        container = document.createElement("div");
        container.id = "procedural-card";
        container.className = "procedural-card-container";
        container.innerHTML = `
            <div class="proc-header">
                <span class="proc-timer" id="proc-stopwatch">00:00</span>
            </div>
            <div class="proc-mode-switch">
                <button type="button" id="tab-quick" class="proc-tab active">Quick Solve</button>
                <button type="button" id="tab-stepwise" class="proc-tab">Step-by-Step Solve</button>
            </div>
            <div id="proc-quick-container">
                <input type="text" id="proc-answer-input" class="proc-input" />
                <button type="button" id="proc-submit-btn" class="proc-btn">Submit</button>
            </div>
            <div id="proc-stepwise-container" class="hidden">
                <div id="proc-steps-list">
                    <div class="proc-step-row" data-step-idx="0">
                        <span class="proc-step-label">Step 1</span>
                        <input type="text" class="proc-input proc-step-input" />
                    </div>
                </div>
                <div class="proc-controls">
                    <button type="button" id="proc-add-step-btn" class="proc-btn">+ Add Step</button>
                    <button type="button" id="proc-hint-btn" class="proc-btn">Hint</button>
                    <button type="button" id="proc-reset-steps-btn" class="proc-btn">Reset</button>
                    <button type="button" id="proc-check-steps-btn" class="proc-btn">Check</button>
                </div>
            </div>
            <div id="proc-hint-container" class="proc-hint-box hidden"></div>
            <div id="proc-result-panel" class="proc-result hidden">
                <div id="proc-result-title"></div>
                <div id="proc-result-feedback"></div>
                <div id="proc-actual-time"></div>
                <button type="button" id="proc-next-btn" class="proc-btn">Next</button>
            </div>
        `;
        document.body.appendChild(container);

        (window as any).bridgeCommand = vi.fn();
    });

    afterEach(() => {
        container.remove();
        vi.restoreAllMocks();
    });

    test("initializes correctly with container binding and timer", () => {
        const reviewer = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-123",
            familyId: "math.percentage.successive",
            targetTimeMs: 45000,
            correctAnswer: { value: 25.44, formatted: "25.44%" },
        });

        expect(reviewer).toBeInstanceOf(ProceduralReviewer);
        expect(reviewer.getState()).toBe("solving");
        reviewer.destroy();
        expect(reviewer.getState()).toBe("teardown");
    });

    test("parses numeric values and fractions accurately", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.ratio",
            targetTimeMs: 45000,
            correctAnswer: { value: 0.75 },
        });

        expect(reviewer.parseNumericValue("0.75")).toBe(0.75);
        expect(reviewer.parseNumericValue("3/4")).toBe(0.75);
        expect(reviewer.parseNumericValue(" 75% ")).toBe(75);
        expect(reviewer.parseNumericValue("$1,250.50")).toBe(1250.5);
        expect(reviewer.parseNumericValue("invalid")).toBeNull();

        reviewer.destroy();
    });

    test("evaluates correct numeric answer within tolerance", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
        });

        const correctRes = reviewer.evaluateLocally("12");
        expect(correctRes.isCorrect).toBe(true);
        expect(correctRes.score).toBe(1.0);

        const closeRes = reviewer.evaluateLocally("12.005");
        expect(closeRes.isCorrect).toBe(true);

        const wrongRes = reviewer.evaluateLocally("15.5");
        expect(wrongRes.isCorrect).toBe(false);
        expect(wrongRes.score).toBe(0.0);

        reviewer.destroy();
    });

    test("mode switching changes active tabs and container visibility", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.percentage.successive",
            targetTimeMs: 45000,
            correctAnswer: { value: 10.0 },
        });

        const quickTab = container.querySelector("#tab-quick")!;
        const stepwiseTab = container.querySelector("#tab-stepwise")!;
        const quickCont = container.querySelector("#proc-quick-container")!;
        const stepCont = container.querySelector("#proc-stepwise-container")!;

        reviewer.switchMode("stepwise");
        expect(stepwiseTab.classList.contains("active")).toBe(true);
        expect(quickTab.classList.contains("active")).toBe(false);
        expect(stepCont.classList.contains("hidden")).toBe(false);
        expect(quickCont.classList.contains("hidden")).toBe(true);

        reviewer.switchMode("quick");
        expect(quickTab.classList.contains("active")).toBe(true);
        expect(stepCont.classList.contains("hidden")).toBe(true);
        expect(quickCont.classList.contains("hidden")).toBe(false);

        reviewer.destroy();
    });

    test("progressive hints requests and bridge notification", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.percentage.successive",
            targetTimeMs: 45000,
            correctAnswer: { value: 10.0 },
            solutionGraph: {
                steps: [
                    { description: "Step 1: Set multiplier", hints: [{ level: 1, title: "Hint 1", content: "Consider formula" }] },
                ],
            },
        });

        reviewer.requestHint();
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_hint:"),
            undefined,
        );

        const hintBox = container.querySelector("#proc-hint-container")!;
        expect(hintBox.classList.contains("hidden")).toBe(false);
        expect(hintBox.textContent).toContain("Consider formula");

        reviewer.destroy();
    });

    test("adding and resetting step rows", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.percentage.successive",
            targetTimeMs: 45000,
            correctAnswer: { value: 10.0 },
        });

        reviewer.addStepRow();
        let rows = container.querySelectorAll(".proc-step-row");
        expect(rows.length).toBe(2);

        reviewer.resetSteps();
        rows = container.querySelectorAll(".proc-step-row");
        expect(rows.length).toBe(1);

        reviewer.destroy();
    });

    test("computes speed and accuracy quadrants accurately", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.kinematics",
            targetTimeMs: 30000,
            correctAnswer: { value: 42.0 },
        });

        // Fast & Correct -> Fluency Strength
        const q1 = reviewer.computeSpeedQuadrant(true, 15000, 30000);
        expect(q1.quadrant).toBe("fluency_strength");

        // Slow & Correct -> Speed Opportunity
        const q2 = reviewer.computeSpeedQuadrant(true, 45000, 30000);
        expect(q2.quadrant).toBe("speed_opportunity");

        // Fast & Incorrect -> Strategy/Trap
        const q3 = reviewer.computeSpeedQuadrant(false, 10000, 30000);
        expect(q3.quadrant).toBe("strategy_trap");

        // Slow & Incorrect -> Concept/Setup
        const q4 = reviewer.computeSpeedQuadrant(false, 50000, 30000);
        expect(q4.quadrant).toBe("concept_setup");

        reviewer.destroy();
    });

    test("ConceptCheck option selection and bridge notification", () => {
        container.innerHTML = `
            <div class="proc-prompt">What is the formula for successive percentage change?</div>
            <div class="proc-option-group" role="radiogroup">
                <button type="button" class="proc-option-item" data-opt-id="opt-a" role="radio" aria-checked="false">
                    <span class="proc-option-key">1</span>
                    <span class="proc-option-label">a + b + (ab/100)</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-b" role="radio" aria-checked="false">
                    <span class="proc-option-key">2</span>
                    <span class="proc-option-label">a * b / 100</span>
                </button>
            </div>
            <div id="proc-result-panel" class="proc-result hidden">
                <div id="proc-result-title"></div>
                <div id="proc-result-feedback"></div>
                <div id="proc-actual-time"></div>
                <button type="button" id="proc-next-btn" class="proc-btn">Next</button>
            </div>
        `;

        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-concept-1",
            familyId: "math.percentage.successive",
            targetTimeMs: 20000,
            objectType: "concept_check",
            conceptCheck: {
                prompt: "What is the formula for successive percentage change?",
                options: [
                    { id: "opt-a", label: "a + b + (ab/100)", is_correct: true, concept_tag: "successive_formula", feedback: "Correct formula." },
                    { id: "opt-b", label: "a * b / 100", is_correct: false, concept_tag: "product_misconception", feedback: "Incorrect." },
                ],
                expected_option_id: "opt-a",
                explanation: "The combined effect of a% followed by b% is a + b + ab/100.",
            },
        });

        const optA = container.querySelector<HTMLElement>('[data-opt-id="opt-a"]')!;
        optA.click();

        expect(optA.classList.contains("selected")).toBe(true);
        expect(optA.classList.contains("correct")).toBe(true);
        expect(reviewer.getState()).toBe("feedback");

        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_attempt:"),
            undefined,
        );

        reviewer.destroy();
    });

    test("StrategyDrill option selection and keyboard shortcuts", () => {
        container.innerHTML = `
            <div class="proc-prompt">A ball is thrown upwards with initial velocity u. Which model applies at maximum height?</div>
            <div class="proc-option-group" role="radiogroup">
                <button type="button" class="proc-option-item" data-opt-id="opt-1" role="radio" aria-checked="false">
                    <span class="proc-option-key">1</span>
                    <span class="proc-option-label">Set v = 0 and apply v^2 = u^2 - 2gh</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-2" role="radio" aria-checked="false">
                    <span class="proc-option-key">2</span>
                    <span class="proc-option-label">Set a = 0</span>
                </button>
            </div>
            <div id="proc-result-panel" class="proc-result hidden">
                <div id="proc-result-title"></div>
                <div id="proc-result-feedback"></div>
                <div id="proc-actual-time"></div>
                <button type="button" id="proc-next-btn" class="proc-btn">Next</button>
            </div>
        `;

        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-strategy-1",
            familyId: "physics.kinematics.1d",
            targetTimeMs: 15000,
            objectType: "strategy_drill",
            strategyDrill: {
                prompt: "Which model applies at maximum height?",
                problem_context: "Ball thrown upwards",
                options: [
                    { id: "opt-1", label: "Set v = 0 and apply v^2 = u^2 - 2gh", is_optimal: true, strategy_tag: "kinematics_apex", feedback: "At apex, instantaneous velocity is 0." },
                    { id: "opt-2", label: "Set a = 0", is_optimal: false, strategy_tag: "zero_accel_trap", feedback: "Gravity acts continuously." },
                ],
                preferred_option_id: "opt-1",
            },
        });

        // Trigger keyboard "1"
        container.dispatchEvent(new KeyboardEvent("keydown", { key: "1", bubbles: true }));

        const opt1 = container.querySelector<HTMLElement>('[data-opt-id="opt-1"]')!;
        expect(opt1.classList.contains("selected")).toBe(true);
        expect(opt1.classList.contains("correct")).toBe(true);

        reviewer.destroy();
    });

    test("WorkedExample Try Similar button triggers bridge command", () => {
        container.innerHTML = `
            <div class="proc-worked-example-card">
                <button type="button" id="proc-try-similar-btn" class="proc-btn">Try Similar</button>
            </div>
        `;

        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-we-1",
            familyId: "chem.stoichiometry.limiting",
            targetTimeMs: 60000,
            objectType: "worked_example",
            workedExample: {
                prompt: "Find the limiting reagent",
                problem_context: "2H2 + O2 -> 2H2O",
                canonical_steps: ["Step 1: Calculate moles", "Step 2: Divide by stoichiometric coefficients"],
                highlighted_decision_point: "Identify smaller mole-to-coefficient ratio",
                method_rationale: "The limiting reagent determines maximum product yield.",
            },
        });

        const trySimilarBtn = container.querySelector<HTMLButtonElement>("#proc-try-similar-btn")!;
        trySimilarBtn.click();

        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_try_similar:"),
            undefined,
        );

        reviewer.destroy();
    });

    test("DeclarativeRecall Review in Anki button triggers bridge command", () => {
        container.innerHTML = `
            <div class="proc-recall-card">
                <button type="button" id="proc-anki-recall-btn" class="proc-btn">Review in Anki</button>
            </div>
        `;

        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-dec-1",
            familyId: "math.formula",
            targetTimeMs: 10000,
            objectType: "declarative_recall",
            declarativeRecall: {
                concept_name: "Quadratic Formula",
                prompt_summary: "Roots of ax^2 + bx + c = 0",
                formula_or_fact: "x = (-b ± √(b^2 - 4ac)) / 2a",
                target_anki_card_id: 998877,
            },
        });

        const recallBtn = container.querySelector<HTMLButtonElement>("#proc-anki-recall-btn")!;
        recallBtn.click();

        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_declarative_recall:"),
            undefined,
        );

        reviewer.destroy();
    });

    test("lifecycle: repeated setup and destroy unbinds listeners without leaks or duplicate calls", () => {
        const r1 = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-1",
            familyId: "math.linear_equations",
            targetTimeMs: 30000,
            correctAnswer: { value: 5.0 },
        });

        r1.destroy();

        const r2 = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-2",
            familyId: "math.linear_equations",
            targetTimeMs: 30000,
            correctAnswer: { value: 10.0 },
        });

        const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
        input.value = "10";
        submitBtn.click();

        expect((window as any).bridgeCommand).toHaveBeenCalledTimes(1);
        r2.destroy();
    });

    test("security: hint rendering does not execute script tags or unescaped HTML attributes", () => {
        const maliciousHint = "<script>window.pwned = true;</script><img src='invalid' onerror='window.pwned=true'/><strong>Danger</strong>";
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-sec-1",
            familyId: "math.algebra",
            targetTimeMs: 30000,
            solutionGraph: {
                steps: [
                    {
                        step_id: "step_1",
                        description: maliciousHint,
                        hints: [
                            {
                                level: 1,
                                title: "<svg onload='window.pwned=true'>Alert",
                                content: maliciousHint,
                            },
                        ],
                    },
                ],
            },
        });

        const hintBtn = container.querySelector<HTMLButtonElement>("#proc-hint-btn")!;
        hintBtn.click();

        const hintBox = container.querySelector<HTMLElement>("#proc-hint-container")!;
        expect(hintBox.classList.contains("hidden")).toBe(false);

        // Verify that raw script and img elements were NOT injected into the DOM as active tags
        expect(hintBox.querySelector("script")).toBeNull();
        expect(hintBox.querySelector("img")).toBeNull();
        expect(hintBox.querySelector("svg")).toBeNull();
        expect((window as any).pwned).toBeUndefined();

        // Verify text content is preserved accurately
        expect(hintBox.textContent).toContain(maliciousHint);

        reviewer.destroy();
    });

    test("lifecycle: duplicate clicks on submit do not trigger multiple bridge commands", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-dup-1",
            familyId: "math.algebra",
            targetTimeMs: 30000,
            correctAnswer: { value: 42.0 },
        });

        const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
        input.value = "42";

        // Click multiple times rapidly
        submitBtn.click();
        submitBtn.click();
        submitBtn.click();

        expect((window as any).bridgeCommand).toHaveBeenCalledTimes(1);
        reviewer.destroy();
    });
});


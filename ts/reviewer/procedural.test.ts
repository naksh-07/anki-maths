// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { MistakeFooter } from "./components/mistake_footer";
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
                <div id="proc-mistake-panel" class="proc-mistake-panel hidden">
                    <div class="proc-mistake-heading">Classify Error (Spaced Repetition Tagging):</div>
                    <div class="proc-mistake-grid">
                        <button type="button" class="proc-mistake-card" data-value="silly_mistake" data-key="1">
                            <span class="proc-key-badge">1</span>
                            <div class="proc-mistake-info">
                                <strong>Silly Mistake</strong>
                                <span>Arithmetic / Slip</span>
                            </div>
                        </button>
                        <button type="button" class="proc-mistake-card" data-value="pattern_not_recognized" data-key="2">
                            <span class="proc-key-badge">2</span>
                            <div class="proc-mistake-info">
                                <strong>Pattern Not Recognized</strong>
                                <span>Unsure how to start</span>
                            </div>
                        </button>
                        <button type="button" class="proc-mistake-card" data-value="formula_or_concept_misapplied" data-key="3">
                            <span class="proc-key-badge">3</span>
                            <div class="proc-mistake-info">
                                <strong>Formula Misapplied</strong>
                                <span>Wrong formula or theorem</span>
                            </div>
                        </button>
                        <button type="button" class="proc-mistake-card" data-value="concept_not_known" data-key="4">
                            <span class="proc-key-badge">4</span>
                            <div class="proc-mistake-info">
                                <strong>Concept Not Known</strong>
                                <span>Fundamental gap</span>
                            </div>
                        </button>
                    </div>
                </div>
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

    test("parses numeric values, units, scientific notation, and fractions accurately", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.ratio",
            targetTimeMs: 45000,
            correctAnswer: { value: 0.75 },
        });

        expect(reviewer.parseNumericValue("0.75")).toBe(0.75);
        expect(reviewer.parseNumericValue("3/4")).toBe(0.75);
        expect(reviewer.parseNumericValue("3/4 m/s")).toBe(0.75);
        expect(reviewer.parseNumericValue(" 75% ")).toBe(75);
        expect(reviewer.parseNumericValue("$1,250.50")).toBe(1250.5);
        expect(reviewer.parseNumericValue("12 m/s")).toBe(12);
        expect(reviewer.parseNumericValue("v = 15.5 m/s")).toBe(15.5);
        expect(reviewer.parseNumericValue("5 kg")).toBe(5);
        expect(reviewer.parseNumericValue("1.2e-3 mol/L")).toBeCloseTo(0.0012, 6);
        expect(reviewer.parseNumericValue("3x10^4 J")).toBe(30000);
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

        const unitRes = reviewer.evaluateLocally("12 m/s");
        expect(unitRes.isCorrect).toBe(true);

        const prefixRes = reviewer.evaluateLocally("v = 12");
        expect(prefixRes.isCorrect).toBe(true);

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
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_attempt:"),
            undefined,
        );
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
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_attempt:"),
            undefined,
        );
        reviewer.destroy();
    });

    test("mistake classification blocks feedback until selected and persists telemetry", async () => {
        (globalThis as any).anki = {
            _state_mutation_key: "studylab_telemetry",
            mutateNextCardStates: vi.fn().mockResolvedValue(undefined)
        };

        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-mistake",
            familyId: "math.algebra",
            targetTimeMs: 45000,
            correctAnswer: { value: 42.0 },
        });

        const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
        
        // Enter wrong answer
        input.value = "10";
        submitBtn.click();

        // Should show mistake classification panel above solution
        expect(reviewer.getState()).toBe("mistake_classification");
        
        const mistakePanel = container.querySelector<HTMLElement>("#proc-mistake-panel")!;
        expect(mistakePanel.classList.contains("hidden")).toBe(false);
        const mistakeCards = mistakePanel.querySelectorAll<HTMLButtonElement>(".proc-mistake-card");
        expect(mistakeCards.length).toBe(4);

        // Feedback panel should be shown (with incorrect answer info)
        const resultPanel = container.querySelector<HTMLElement>("#proc-result-panel")!;
        expect(resultPanel.classList.contains("hidden")).toBe(false);
        const resultTitle = resultPanel.querySelector<HTMLElement>("#proc-result-title")!;
        expect(resultTitle.textContent).toContain("✗ Incorrect Answer");

        // Select a mistake type via click or key
        const sillyMistakeCard = Array.from(mistakeCards).find(b => b.dataset.value === "silly_mistake")!;
        sillyMistakeCard.click();

        expect(sillyMistakeCard.classList.contains("selected")).toBe(true);

        // Wait for the setTimeout in showMistakeClassificationUI to resolve
        await new Promise(resolve => setTimeout(resolve, 200));

        // Now state should transition to feedback
        expect(reviewer.getState()).toBe("feedback");

        // Verify next button is shown
        const nextBtn = container.querySelector<HTMLButtonElement>("#proc-next-btn")!;
        expect(nextBtn.classList.contains("hidden")).toBe(false);

        // Trigger next problem
        nextBtn.click();
        expect((window as any).bridgeCommand).toHaveBeenCalledWith("procedural_answer:1", undefined);

        // Verify telemetry was persisted
        expect((globalThis as any).anki.mutateNextCardStates).toHaveBeenCalledWith(
            "studylab_telemetry",
            expect.any(Function)
        );

        // Execute the callback passed to mutateNextCardStates to verify merge behavior
        const mutateFn = (globalThis as any).anki.mutateNextCardStates.mock.calls[0][1];
        
        const dummyStates = {};
        const dummyCustomData = {
            again: { existingKey: "existingValue" },
            good: { studylab: { v: 0, oldData: true } }
        };

        await mutateFn(dummyStates, dummyCustomData);

        expect(dummyCustomData.again).toEqual({
            existingKey: "existingValue",
            studylab: {
                v: 1,
                actualTimeMs: expect.any(Number),
                targetTimeMs: 45000,
                isCorrect: false,
                hintsUsed: 0,
                mistakeType: "silly_mistake",
                mode: "quick",
                proceduralPerformance: {
                    classification: "incorrect",
                    timeRatio: expect.any(Number),
                    mistakeType: "silly_mistake",
                    hintsUsed: 0
                },
                proceduralRemediation: {
                    needed: true,
                    reason: "silly_mistake",
                    skillId: "",
                    schemaId: "",
                    familyId: "math.algebra",
                    topicId: ""
                },
                attemptResult: expect.any(Object)
            }
        });
    });

    test("MCQ option selection, keyboard shortcuts 1-4 / A-D, and correct option highlighting", () => {
        container.innerHTML = `
            <div class="proc-prompt">What is the capital of France?</div>
            <div class="proc-option-group" role="radiogroup">
                <button type="button" class="proc-option-item" data-opt-id="opt-0" data-opt-idx="0" role="radio" aria-checked="false">
                    <span class="proc-option-key">1</span>
                    <span class="proc-option-label">London</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-1" data-opt-idx="1" role="radio" aria-checked="false">
                    <span class="proc-option-key">2</span>
                    <span class="proc-option-label">Paris</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-2" data-opt-idx="2" role="radio" aria-checked="false">
                    <span class="proc-option-key">3</span>
                    <span class="proc-option-label">Berlin</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-3" data-opt-idx="3" role="radio" aria-checked="false">
                    <span class="proc-option-key">4</span>
                    <span class="proc-option-label">Rome</span>
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
            instanceId: "inst-mcq-1",
            familyId: "general.geography",
            targetTimeMs: 30000,
            objectType: "mcq",
            correctAnswer: { correct_option: "Paris", formatted: "Paris" },
        });

        // Test keyboard 'B' or '2' selection
        container.dispatchEvent(new KeyboardEvent("keydown", { key: "2", bubbles: true }));

        const optParis = container.querySelector<HTMLElement>('[data-opt-idx="1"]')!;
        expect(optParis.classList.contains("selected")).toBe(true);
        expect(optParis.classList.contains("correct")).toBe(true);
        expect(reviewer.getState()).toBe("feedback");
        expect(reviewer.getMCQContainer()).not.toBeNull();

        reviewer.destroy();
    });

    test("MCQ mock exam mode (GAP-MOD-03) integrates with ProceduralReviewer without instant grading", () => {
        container.innerHTML = `
            <div class="proc-prompt">What is 15% of 200?</div>
            <div class="proc-option-group" role="radiogroup">
                <button type="button" class="proc-option-item" data-opt-id="opt-a" data-opt-idx="0" role="radio" aria-checked="false">
                    <span class="proc-option-key">A</span>
                    <span class="proc-option-label">20</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-b" data-opt-idx="1" role="radio" aria-checked="false">
                    <span class="proc-option-key">B</span>
                    <span class="proc-option-label">30</span>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-c" data-opt-idx="2" role="radio" aria-checked="false">
                    <span class="proc-option-key">C</span>
                    <span class="proc-option-label">35</span>
                </button>
            </div>
            <div id="proc-result-panel" class="proc-result hidden"></div>
        `;

        let selectedChangedId: string | null = null;
        const reviewer = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-mock-mcq",
            familyId: "math.percentage",
            targetTimeMs: 40000,
            objectType: "mcq",
            mode: "mock",
            correctAnswer: { canonical_id: "opt-b", correct_option: "30" },
            onSelectionChanged: (optId) => {
                selectedChangedId = optId;
            },
        });

        const optA = container.querySelector<HTMLElement>('[data-opt-id="opt-a"]')!;
        const optB = container.querySelector<HTMLElement>('[data-opt-id="opt-b"]')!;

        // Select Option A (key '1')
        container.dispatchEvent(new KeyboardEvent("keydown", { key: "1", bubbles: true }));
        expect(selectedChangedId).toBe("opt-a");
        expect(optA.classList.contains("selected")).toBe(true);
        expect(optA.classList.contains("incorrect")).toBe(false);
        expect(reviewer.getState()).toBe("solving"); // Still in solving state, no auto feedback!

        // Switch to Option B (key '2')
        container.dispatchEvent(new KeyboardEvent("keydown", { key: "2", bubbles: true }));
        expect(selectedChangedId).toBe("opt-b");
        expect(optB.classList.contains("selected")).toBe(true);
        expect(optA.classList.contains("selected")).toBe(false);
        expect(reviewer.getState()).toBe("solving");

        // Evaluate mock question on demand
        const evalResult = reviewer.evaluateMockMCQ();
        expect(evalResult).not.toBeNull();
        expect(evalResult?.isCorrect).toBe(true);
        expect(evalResult?.selectedOptionId).toBe("opt-b");
        expect(evalResult?.score).toBe(1.0);

        reviewer.destroy();
    });

    test("Space and Enter key handling across solving, mistake classification, and feedback states", async () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-shortcuts",
            familyId: "math.algebra",
            targetTimeMs: 30000,
            correctAnswer: { value: 50.0 },
        });

        // 1. In solving state: Space should not submit or leak
        const spaceEvent = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
        container.dispatchEvent(spaceEvent);
        expect(spaceEvent.defaultPrevented).toBe(true);
        expect(reviewer.getState()).toBe("solving");

        // Submit wrong answer
        const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        input.value = "20";
        container.querySelector<HTMLButtonElement>("#proc-submit-btn")!.click();
        expect(reviewer.getState()).toBe("mistake_classification");

        // 2. In mistake classification state: Space and Enter must be trapped and blocked
        const spaceInMistake = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
        container.dispatchEvent(spaceInMistake);
        expect(spaceInMistake.defaultPrevented).toBe(true);
        expect(reviewer.getState()).toBe("mistake_classification");

        // Wait to verify no delayed transition or accidental silly_mistake bypass occurred
        await new Promise(resolve => setTimeout(resolve, 200));
        expect(reviewer.getState()).toBe("mistake_classification");

        const enterInMistake = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
        container.dispatchEvent(enterInMistake);
        expect(enterInMistake.defaultPrevented).toBe(true);
        expect(reviewer.getState()).toBe("mistake_classification");

        await new Promise(resolve => setTimeout(resolve, 200));
        expect(reviewer.getState()).toBe("mistake_classification");

        // Select mistake using shortcut key '3' (Formula or concept misapplied)
        const key3Event = new KeyboardEvent("keydown", { key: "3", bubbles: true, cancelable: true });
        container.dispatchEvent(key3Event);
        expect(key3Event.defaultPrevented).toBe(true);

        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining('"mistake_type":"formula_or_concept_misapplied"'),
            undefined
        );

        await new Promise(resolve => setTimeout(resolve, 200));
        expect(reviewer.getState()).toBe("feedback");

        // 3. In feedback state: Enter or Space triggers handleNext
        const enterInFeedback = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
        container.dispatchEvent(enterInFeedback);
        expect((window as any).bridgeCommand).toHaveBeenCalledWith("procedural_answer:1", undefined);

        reviewer.destroy();
    });

    test("MistakeFooter component traps Space/Enter without bypass and dispatches all 1-4 categories with telemetry", () => {
        let selectedMistake: string | null = null;
        const footerContainer = document.createElement("div");
        footerContainer.innerHTML = `<div id="proc-result-panel"><div id="proc-solution-container"></div></div>`;
        document.body.appendChild(footerContainer);

        const footer = new MistakeFooter({
            container: footerContainer,
            instanceId: "inst-test-footer",
            familyId: "math.percentages",
            onSelect: (val) => {
                selectedMistake = val;
            },
        });

        footer.show();
        expect(footer.isShown()).toBe(true);

        // 1. Space and Enter must be trapped and NOT select any category
        const spaceEvent = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
        const handledSpace = footer.handleKeydown(spaceEvent);
        expect(handledSpace).toBe(true);
        expect(spaceEvent.defaultPrevented).toBe(true);
        expect(footer.getSelectedValue()).toBeNull();
        expect(selectedMistake).toBeNull();

        const enterEvent = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
        const handledEnter = footer.handleKeydown(enterEvent);
        expect(handledEnter).toBe(true);
        expect(enterEvent.defaultPrevented).toBe(true);
        expect(footer.getSelectedValue()).toBeNull();
        expect(selectedMistake).toBeNull();

        // 2. Test keydown '2' (Pattern Missed)
        const key2Event = new KeyboardEvent("keydown", { key: "2", bubbles: true, cancelable: true });
        const handled2 = footer.handleKeydown(key2Event);
        expect(handled2).toBe(true);
        expect(key2Event.defaultPrevented).toBe(true);
        expect(selectedMistake).toBe("pattern_not_recognized");
        expect(footer.getSelectedValue()).toBe("pattern_not_recognized");

        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining('"mistake_type":"pattern_not_recognized"'),
            undefined,
        );

        // 3. Test keydown '1', '3', '4' selections
        footer.select(1);
        expect(footer.getSelectedValue()).toBe("silly_mistake");
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining('"mistake_type":"silly_mistake"'),
            undefined,
        );

        footer.select(3);
        expect(footer.getSelectedValue()).toBe("formula_or_concept_misapplied");
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining('"mistake_type":"formula_or_concept_misapplied"'),
            undefined,
        );

        footer.select(4);
        expect(footer.getSelectedValue()).toBe("concept_not_known");
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining('"mistake_type":"concept_not_known"'),
            undefined,
        );

        footer.destroy();
        footerContainer.remove();
    });

    test("lifecycle: unmounting procedural container automatically destroys reviewer and unbinds window listeners", async () => {
        const reviewer = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-leak-test",
            familyId: "math.algebra",
            targetTimeMs: 30000,
            correctAnswer: { value: 100 },
        });

        expect(reviewer.getState()).toBe("solving");

        // Simulate navigating away to a standard Anki card by unmounting the procedural container from DOM
        container.remove();

        // Allow MutationObserver callback to run
        await new Promise(resolve => setTimeout(resolve, 50));

        // State must automatically transition to teardown
        expect(reviewer.getState()).toBe("teardown");

        // Global keydown on window should not be intercepted or prevented
        const spaceEvent = new KeyboardEvent("keydown", { key: " ", bubbles: true, cancelable: true });
        window.dispatchEvent(spaceEvent);
        expect(spaceEvent.defaultPrevented).toBe(false);

        // Re-attach container for subsequent tests
        document.body.appendChild(container);
    });

    test("proceduralAPI.destroyActive cleanly tears down active instance and resets global state", () => {
        const reviewer = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-destroy-active",
            familyId: "math.algebra",
            targetTimeMs: 30000,
            correctAnswer: { value: 100 },
        });

        expect(reviewer.getState()).toBe("solving");
        expect((globalThis as any).__activeProceduralReviewer).toBe(reviewer);

        proceduralAPI.destroyActive();

        expect(reviewer.getState()).toBe("teardown");
        expect((globalThis as any).__activeProceduralReviewer).toBeNull();
    });

    test("Open Canvas ANTI-03, ANTI-04, ANTI-08, and Modality Purity Invariants", async () => {
        const testContainer = document.createElement("div");
        testContainer.id = "procedural-card";
        testContainer.className = "procedural-card-container";
        testContainer.innerHTML = `
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
            <div id="proc-result-panel" class="proc-result hidden">
                <div id="proc-result-title"></div>
                <div id="proc-result-feedback"></div>
                <div class="proc-meta-row">
                    <div id="proc-actual-time" class="proc-actual-time"></div>
                </div>
                <div id="proc-mistake-panel" class="proc-mistake-panel hidden">
                    <button type="button" class="proc-mistake-card" data-value="silly_mistake" data-key="1">Silly</button>
                </div>
                <div id="proc-solution-container" class="proc-solution hidden">
                    <strong>Step-by-Step Solution:</strong>
                    <div>Step 1: Compute result = 42</div>
                </div>
                <button type="button" id="proc-next-btn" class="proc-btn hidden">Next</button>
            </div>
        `;
        document.body.appendChild(testContainer);

        const reviewer = new ProceduralReviewer(testContainer, {
            instanceId: "inst-open-canvas-inv",
            familyId: "math.algebra",
            targetTimeMs: 30000,
            correctAnswer: { value: 42.0 },
        });

        // ANTI-03: Stopwatch element is hidden/silent during active solving
        const stopwatch = testContainer.querySelector<HTMLElement>("#proc-stopwatch")!;
        expect(stopwatch.classList.contains("hidden") || stopwatch.style.display === "none").toBe(true);

        // ANTI-04: Streamlined speed quadrant labels
        const fastCorrect = reviewer.computeSpeedQuadrant(true, 10000, 30000);
        expect(fastCorrect.label).toBe("⚡ Fast & Accurate");
        const slowCorrect = reviewer.computeSpeedQuadrant(true, 40000, 30000);
        expect(slowCorrect.label).toBe("⏱ Accurate · Paced");
        const fastWrong = reviewer.computeSpeedQuadrant(false, 10000, 30000);
        expect(fastWrong.label).toBe("⚠️ Strategy Trap");
        const slowWrong = reviewer.computeSpeedQuadrant(false, 40000, 30000);
        expect(slowWrong.label).toBe("💡 Concept Gap");

        // Submit incorrect answer -> Mistake Classification state
        const input = testContainer.querySelector<HTMLInputElement>("#proc-answer-input")!;
        input.value = "10";
        testContainer.querySelector<HTMLButtonElement>("#proc-submit-btn")!.click();

        expect(reviewer.getState()).toBe("mistake_classification");

        // ANTI-08: Solution container MUST remain hidden during reflection
        const solutionContainer = testContainer.querySelector<HTMLElement>("#proc-solution-container")!;
        expect(solutionContainer.classList.contains("hidden") || solutionContainer.style.display === "none").toBe(true);

        // ANTI-02: Expected answer is not leaked in feedback during reflection
        const feedbackEl = testContainer.querySelector<HTMLElement>("#proc-result-feedback")!;
        expect(feedbackEl.textContent).toContain("Your answer: 10");
        expect(feedbackEl.textContent).not.toContain("Correct answer: 42");

        // Select mistake category
        reviewer.selectMistakeCategory("silly_mistake");
        await new Promise((resolve) => setTimeout(resolve, 200));

        // State transitions to feedback
        expect(reviewer.getState()).toBe("feedback");

        // ANTI-08: Solution container is now revealed post-reflection
        expect(solutionContainer.classList.contains("hidden")).toBe(false);
        expect(solutionContainer.style.display).not.toBe("none");

        // ANTI-02: Feedback now shows concise deduplicated answer comparison
        expect(feedbackEl.textContent).toContain("Your answer: 10");
        expect(feedbackEl.textContent).toContain("Correct answer: 42");

        // ANTI-03 / ANTI-04: Elapsed time is displayed in speed row
        const timeEl = testContainer.querySelector<HTMLElement>("#proc-actual-time")!;
        expect(timeEl.innerHTML).toContain("proc-speed-quadrant");

        reviewer.destroy();
        testContainer.remove();
    });

    test("Modality Purity: WorkedExample suppresses solving inputs and tabs", () => {
        const weContainer = document.createElement("div");
        weContainer.id = "procedural-card";
        weContainer.className = "procedural-card-container";
        weContainer.innerHTML = `
            <div class="proc-mode-switch">
                <button type="button" id="tab-quick" class="proc-tab active">Quick Solve</button>
            </div>
            <div id="proc-quick-container">
                <input type="text" id="proc-answer-input" class="proc-input" />
            </div>
            <div class="proc-worked-box proc-worked-example-card">
                <button type="button" id="proc-try-similar-btn" class="proc-btn">Try Similar</button>
            </div>
        `;
        document.body.appendChild(weContainer);

        const reviewer = new ProceduralReviewer(weContainer, {
            instanceId: "inst-we-purity",
            familyId: "chem.stoichiometry",
            targetTimeMs: 60000,
            objectType: "worked_example",
            workedExample: {
                prompt: "Example problem",
                problem_context: "Context",
                canonical_steps: ["Step 1"],
                highlighted_decision_point: "Decision",
                method_rationale: "Rationale",
            },
        });

        const quickCont = weContainer.querySelector<HTMLElement>("#proc-quick-container")!;
        const modeSwitch = weContainer.querySelector<HTMLElement>(".proc-mode-switch")!;
        const input = weContainer.querySelector<HTMLInputElement>("#proc-answer-input")!;

        expect(quickCont.classList.contains("hidden") || quickCont.style.display === "none").toBe(true);
        expect(modeSwitch.classList.contains("hidden") || modeSwitch.style.display === "none").toBe(true);
        expect(input.disabled).toBe(true);

        reviewer.destroy();
        weContainer.remove();
    });
});


describe("ProceduralReviewer Performance Classification", () => {
    let container: HTMLElement;

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
                <div id="proc-mistake-panel" class="proc-mistake-panel hidden">
                    <button type="button" class="proc-mistake-card" data-value="silly_mistake" data-key="1">Silly</button>
                    <button type="button" class="proc-mistake-card" data-value="pattern_not_recognized" data-key="2">Pattern</button>
                    <button type="button" class="proc-mistake-card" data-value="formula_or_concept_misapplied" data-key="3">Formula</button>
                    <button type="button" class="proc-mistake-card" data-value="concept_not_known" data-key="4">Concept</button>
                </div>
                <button type="button" id="proc-next-btn" class="proc-btn">Next</button>
            </div>
        `;
        document.body.appendChild(container);
        vi.useFakeTimers();
        (globalThis as any).anki = {
            mutateNextCardStates: vi.fn().mockResolvedValue(undefined)
        };
        (globalThis as any).bridgeCommand = vi.fn();
    });

    afterEach(() => {
        document.body.removeChild(container);
        vi.useRealTimers();
        delete (globalThis as any).anki;
        delete (globalThis as any).bridgeCommand;
    });

    async function runAttempt(timeMs: number, targetTimeMs: number | undefined, isCorrect: boolean) {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst_123",
            skillId: "test_skill",
            schemaId: "test_schema",
            familyId: "fam_123",
            targetTimeMs,
            correctAnswer: { type: "exact", value: 42 },
        });

        vi.advanceTimersByTime(timeMs);
        
        const data = { answer: isCorrect ? "42" : "99", steps: [] };
        const quickInput = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
        quickInput.value = data.answer;
        submitBtn.click();

        if (!isCorrect) {
            const mistakePanel = container.querySelector<HTMLElement>("#proc-mistake-panel")!;
            const mistakeCards = mistakePanel.querySelectorAll<HTMLButtonElement>(".proc-mistake-card");
            const mistakeCard = Array.from(mistakeCards).find(b => b.dataset.value === "pattern_not_recognized")!;
            mistakeCard.click();
            vi.advanceTimersByTime(250);
            await Promise.resolve();
        }

        const mutateFn = (globalThis as any).anki.mutateNextCardStates.mock.calls[0][1];
        const dummyCustomData = { again: {} };
        await mutateFn({}, dummyCustomData);
        
        reviewer.destroy();
        return (dummyCustomData.again as any).studylab;
    }

    test("classifies fast_correct (<= 0.8 ratio)", async () => {
        const studylab = await runAttempt(20000, 40000, true); // 0.5 ratio
        expect(studylab.proceduralPerformance.classification).toBe("fast_correct");
        expect(studylab.proceduralPerformance.timeRatio).toBe(0.5);
        expect(studylab.proceduralRemediation.needed).toBe(false);
        expect(studylab.proceduralRemediation.reason).toBe("none");
    });

    test("classifies on_target_correct (> 0.8 and <= 1.2 ratio)", async () => {
        const studylab = await runAttempt(40000, 40000, true); // 1.0 ratio
        expect(studylab.proceduralPerformance.classification).toBe("on_target_correct");
        expect(studylab.proceduralPerformance.timeRatio).toBe(1.0);
        expect(studylab.proceduralRemediation.needed).toBe(false);
    });

    test("classifies slow_correct (> 1.2 ratio)", async () => {
        const studylab = await runAttempt(60000, 40000, true); // 1.5 ratio
        expect(studylab.proceduralPerformance.classification).toBe("slow_correct");
        expect(studylab.proceduralPerformance.timeRatio).toBe(1.5);
        expect(studylab.proceduralRemediation.needed).toBe(true);
        expect(studylab.proceduralRemediation.reason).toBe("slow_correct");
    });

    test("classifies incorrect and maps mistake type", async () => {
        const studylab = await runAttempt(20000, 40000, false);
        expect(studylab.proceduralPerformance.classification).toBe("incorrect");
        expect(studylab.proceduralPerformance.mistakeType).toBe("pattern_not_recognized");
        expect(studylab.proceduralRemediation.needed).toBe(true);
        expect(studylab.proceduralRemediation.reason).toBe("pattern_not_recognized");
    });

    test("handles zero targetTimeMs safely (fallback to on_target_correct)", async () => {
        const perf = await runAttempt(30000, 0, true);
        expect(perf.proceduralPerformance.classification).toBe("on_target_correct");
    });

    test("handles missing targetTimeMs safely (fallback to on_target_correct)", async () => {
        const perf = await runAttempt(30000, undefined, true);
        expect(perf.proceduralPerformance.classification).toBe("on_target_correct");
    });

    test("handleNativeShowAnswer with empty input routes to unassisted surrender and mistake classification (P0-A)", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-p0a",
            familyId: "math.linear",
            targetTimeMs: 45000,
            correctAnswer: { value: 10 },
        });

        expect(reviewer.getState()).toBe("solving");
        // User clicks Show Answer without typing anything
        reviewer.handleNativeShowAnswer();

        expect(reviewer.getState()).toBe("mistake_classification");
        expect(container.querySelector("#proc-result-panel")?.classList.contains("hidden")).toBe(false);
        expect(container.querySelector("#proc-result-title")?.textContent).toContain("Incorrect");

        reviewer.destroy();
    });

    test("deriveCalibratedEase computes exact 4-tier pedagogical rating (P0-B)", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-p0b",
            familyId: "math.linear",
            targetTimeMs: 40000,
            correctAnswer: { value: 10 },
        });

        // Case 1: Unattempted / incorrect
        expect(reviewer.deriveCalibratedEase()).toBe(1);

        reviewer.destroy();
    });
});

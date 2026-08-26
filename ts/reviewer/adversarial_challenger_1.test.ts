// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { describe, test, expect, beforeEach, afterEach, vi } from "vitest";
import { proceduralAPI, ProceduralReviewer, type LearningObjectKind } from "./procedural";

describe("Challenger 1 Adversarial Audit - Frontend State Machine and Modalities", () => {
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
            <div id="proc-solution-container" class="proc-solution hidden"></div>
            <div class="proc-action-row hidden">
                <button type="button" id="proc-try-similar-btn" class="proc-btn">Try Similar</button>
            </div>
            <div id="proc-result-panel" class="proc-result hidden">
                <div id="proc-result-title"></div>
                <div id="proc-result-feedback"></div>
                <div id="proc-actual-time"></div>
                <div id="proc-mistake-panel" class="proc-mistake-panel hidden">
                    <div class="proc-mistake-footer">
                        <button type="button" class="proc-mistake-btn" data-value="silly_mistake" data-key="1">
                            <span class="proc-key-badge">1</span> Silly Slip
                        </button>
                        <button type="button" class="proc-mistake-btn" data-value="pattern_not_recognized" data-key="2">
                            <span class="proc-key-badge">2</span> Pattern Missed
                        </button>
                        <button type="button" class="proc-mistake-btn" data-value="formula_or_concept_misapplied" data-key="3">
                            <span class="proc-key-badge">3</span> Concept Gap
                        </button>
                        <button type="button" class="proc-mistake-btn" data-value="concept_not_known" data-key="4">
                            <span class="proc-key-badge">4</span> Prereq Unknown
                        </button>
                    </div>
                </div>
                <button type="button" id="proc-next-btn" class="proc-btn hidden">Next Problem ↔</button>
            </div>
        `;
        document.body.appendChild(container);
        (window as any).bridgeCommand = vi.fn();
    });

    afterEach(() => {
        proceduralAPI.setup;
        proceduralAPI.destroyActive();
        container.remove();
        vi.restoreAllMocks();
    });

    describe("Challenge 1: Modality Invariants & Zero-Textbox Fallback", () => {
        const discreteModalities: LearningObjectKind[] = [
            "mcq",
            "concept_check",
            "strategy_drill",
            "worked_example",
            "declarative_recall",
            "prerequisite_review",
        ];

        discreteModalities.forEach((modality) => {
            test(`Prevents textbox on ${modality} even under adversarial mode switching`, () => {
                const reviewer = proceduralAPI.setup({
                    containerId: "procedural-card",
                    instanceId: "inst-" + modality,
                    familyId: "math.audit",
                    targetTimeMs: 30000,
                    objectType: modality,
                    correctAnswer: { correct_option: "A", value: 10 },
                });

                const quickContainer = container.querySelector<HTMLElement>("#proc-quick-container")!;
                const stepwiseContainer = container.querySelector<HTMLElement>("#proc-stepwise-container")!;
                const modeSwitch = container.querySelector<HTMLElement>(".proc-mode-switch")!;
                const quickInput = container.querySelector<HTMLInputElement>("#proc-answer-input")!;

                expect(quickContainer.classList.contains("hidden")).toBe(true);
                expect(quickContainer.style.display).toBe("none");
                expect(stepwiseContainer.classList.contains("hidden")).toBe(true);
                expect(stepwiseContainer.style.display).toBe("none");
                expect(modeSwitch.classList.contains("hidden")).toBe(true);
                expect(modeSwitch.style.display).toBe("none");
                expect(quickInput.disabled).toBe(true);
                expect(quickInput.getAttribute("aria-hidden")).toBe("true");

                // Adversarial: switchMode must not reveal quick or stepwise
                reviewer.switchMode("quick");
                expect(quickContainer.style.display).toBe("none");
                reviewer.switchMode("stepwise");
                expect(stepwiseContainer.style.display).toBe("none");
            });
        });
    });

    describe("Challenge 2: Mistake Reflection Gate & Anti-Bypass", () => {
        test("Strictly blocks Space, Enter, and keyboard bypass during mistake_classification", () => {
            const reviewer = proceduralAPI.setup({
                containerId: "procedural-card",
                instanceId: "inst-bypass-test",
                familyId: "math.audit",
                targetTimeMs: 30000,
                objectType: "problem",
                correctAnswer: { value: 100 }
            });

            const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
            input.value = "42";
            const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
            submitBtn.click();
            expect(reviewer.getState()).toBe("mistake_classification");

            const solutionContainer = container.querySelector<HTMLElement>("#proc-solution-container")!;
            expect(solutionContainer.style.display).toBe("none");
            expect(solutionContainer.classList.contains("hidden")).toBe(true);

            const nextBtn = container.querySelector<HTMLButtonElement>("#proc-next-btn")!;
            expect(nextBtn.classList.contains("hidden")).toBe(true);

            // Attempt Space / Enter bypasses
            window.dispatchEvent(new KeyboardEvent("keydown", { key: " ", code: "Space", bubbles: true }));
            window.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", code: "Enter", bubbles: true }));
            window.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));

            expect(reviewer.getState()).toBe("mistake_classification");
            expect((window as any).bridgeCommand).not.toHaveBeenCalledWith(expect.stringContaining("procedural_answer:"));

            // Audit that the expected answer '100' is not leaked in the reflection panel
            const feedbackEl = container.querySelector<HTMLElement>("#proc-result-feedback")!;
            expect(feedbackEl.textContent).not.toContain("100");
            expect(feedbackEl.textContent).toContain("Your answer: 42");
        });

        test("Invoking mistake hotkeys 1-4 proceeds to feedback state with derivation reveal", () => {
            vi.useFakeTimers();
            const reviewer = proceduralAPI.setup({
                containerId: "procedural-card",
                instanceId: "inst-valid-reflect",
                familyId: "math.audit",
                targetTimeMs: 30000,
                objectType: "problem",
                correctAnswer: { value: 100 }
            });

            const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
            input.value = "42";
            const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
            submitBtn.click();

            expect(reviewer.getState()).toBe("mistake_classification");

            window.dispatchEvent(new KeyboardEvent("keydown", { key: "1", bubbles: true }));

            const mistakeCalls = (window as any).bridgeCommand.mock.calls.filter((c: any) => c[0].startsWith("procedural_mistake:"));
            expect(mistakeCalls.length).toBeGreaterThanOrEqual(1);
            expect(mistakeCalls[0][0]).toContain("silly_mistake");

            vi.advanceTimersByTime(200);

            expect(reviewer.getState()).toBe("next");

            const solutionContainer = container.querySelector<HTMLElement>("#proc-solution-container")!;
            expect(solutionContainer.style.display).toBe("");
            expect(solutionContainer.classList.contains("hidden")).toBe(false);

            vi.useRealTimers();
        });
    });

    describe("Challenge 3: Teardown Lyfecycle & Memory Hygiene", () => {
        test("destroyActive removes listeners and clears global reference", () => {
            const reviewer = proceduralAPI.setup({
                containerId: "procedural-card",
                instanceId: "inst-leak-test",
                familyId: "math.audit",
                targetTimeMs: 30000,
                objectType: "problem",
                correctAnswer: { value: 50 }
            });

            expect(reviewer.getState()).toBe("solving");
            proceduralAPI.destroyActive();

            expect(reviewer.getState()).toBe("teardown");
            expect((globalThis as any).__activeProceduralReviewer).toBeNull();

            window.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
            window.dispatchEvent(new KeyboardEvent("keydown", { key: " ", bubbles: true }));
            expect((window as any).bridgeCommand).not.toHaveBeenCalled();
        });

        test("Double setup calls destroy previous instance without event leaks", () => {
            const r1 = proceduralAPI.setup({
                containerId: "procedural-card",
                instanceId: "inst-1",
                familyId: "math.1",
                targetTimeMs: 30000,
                objectType: "problem",
                correctAnswer: { value: 10 }
            });

            const r2 = proceduralAPI.setup({
                containerId: "procedural-card",
                instanceId: "inst-2",
                familyId: "math.2",
                targetTimeMs: 30000,
                objectType: "problem",
                correctAnswer: { value: 20 }
            });

            expect(r1.getState()).toBe("teardown");
            expect(r2.getState()).toBe("solving");

            const input = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
            input.value = "20";
            const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
            submitBtn.click();

            const calls = (window as any).bridgeCommand.mock.calls.filter((c: any) =>
                c[0].startsWith("procedural_attempt:")
            );

            expect(calls.length).toBe(1);
            expect(calls[0][0]).toContain("inst-2");
            expect(calls[0][0]).not.toContain("inst-1");
        });
    });

    describe("Challenge 4: Standard Anki Card Isolation", () => {
        test("Standard Anki card mounts without procedural injection or leakage", () => {
            const standardCard = document.createElement("div");
            standardCard.className = "card";
            standardCard.innerHTML = `
                <div class="front">What is 2+2?</div>
                <hr id="answer">
                <div class="back">4</div>
             `;
            document.body.appendChild(standardCard);

            proceduralAPI.destroyActive();

            expect(standardCard.querySelector("#proc-answer-input")).toBeNull();
            expect(standardCard.querySelector(".proc-option-group")).toBeNull();
            expect(standardCard.querySelector(".proc-result")).toBeNull();

            window.dispatchEvent(new KeyboardEvent("keydown", { key: "1", bubbles: true }));
            expect((window as any).bridgeCommand).not.toHaveBeenCalled();

            standardCard.remove();
        });
    });
});

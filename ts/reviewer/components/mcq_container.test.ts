// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { MCQContainer, type MCQEvaluationResult, type MCQOption } from "./mcq_container";

describe("MCQContainer Modality Component", () => {
    let container: HTMLDivElement;

    beforeEach(() => {
        container = document.createElement("div");
        container.id = "procedural-card";
        container.className = "procedural-card-container";
        container.innerHTML = `
            <div class="proc-prompt">What is the capital of France?</div>
            <div class="proc-mode-switch">
                <button type="button" id="tab-quick" class="proc-tab active">Quick Solve</button>
            </div>
            <div id="proc-quick-container">
                <input type="text" id="proc-answer-input" class="proc-input" />
                <button type="button" id="proc-submit-btn" class="proc-btn">Submit</button>
            </div>
            <div id="proc-stepwise-container" class="hidden"></div>
            <div class="proc-option-group" role="radiogroup" aria-label="Multiple choice options">
                <button type="button" class="proc-option-item" data-opt-id="opt-london" data-opt-idx="0" role="radio" aria-checked="false">
                    <div class="proc-option-header">
                        <span class="proc-option-key">A</span>
                        <span class="proc-option-label">London</span>
                    </div>
                    <div class="proc-option-feedback hidden">London is the capital of the United Kingdom.</div>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-paris" data-opt-idx="1" role="radio" aria-checked="false">
                    <div class="proc-option-header">
                        <span class="proc-option-key">B</span>
                        <span class="proc-option-label">Paris</span>
                    </div>
                    <div class="proc-option-feedback hidden">Correct! Paris is the capital of France.</div>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-berlin" data-opt-idx="2" role="radio" aria-checked="false">
                    <div class="proc-option-header">
                        <span class="proc-option-key">C</span>
                        <span class="proc-option-label">Berlin</span>
                    </div>
                    <div class="proc-option-feedback hidden">Berlin is the capital of Germany.</div>
                </button>
                <button type="button" class="proc-option-item" data-opt-id="opt-rome" data-opt-idx="3" role="radio" aria-checked="false">
                    <div class="proc-option-header">
                        <span class="proc-option-key">D</span>
                        <span class="proc-option-label">Rome</span>
                    </div>
                    <div class="proc-option-feedback hidden">Rome is the capital of Italy.</div>
                </button>
            </div>
        `;
        document.body.appendChild(container);
        (window as any).bridgeCommand = vi.fn();
    });

    afterEach(() => {
        container.remove();
        vi.restoreAllMocks();
    });

    test("enforces zero text input fallback by hiding inputs and tabs in MCQ modality", () => {
        const mcq = new MCQContainer(container, {
            correctAnswer: { correct_option: "Paris" },
        });

        const quickContainer = container.querySelector<HTMLElement>("#proc-quick-container")!;
        const stepwiseContainer = container.querySelector<HTMLElement>("#proc-stepwise-container")!;
        const modeSwitch = container.querySelector<HTMLElement>(".proc-mode-switch")!;
        const answerInput = container.querySelector<HTMLInputElement>("#proc-answer-input")!;

        expect(quickContainer.classList.contains("hidden")).toBe(true);
        expect(quickContainer.style.display).toBe("none");
        expect(stepwiseContainer.classList.contains("hidden")).toBe(true);
        expect(modeSwitch.classList.contains("hidden")).toBe(true);
        expect(answerInput.disabled).toBe(true);

        mcq.destroy();
    });

    test("sets up ARIA radiogroup, radio role, aria-checked, and roving tabindex", () => {
        const mcq = new MCQContainer(container, {
            correctAnswer: { correct_option: "Paris" },
        });

        const group = container.querySelector<HTMLElement>(".proc-option-group")!;
        expect(group.getAttribute("role")).toBe("radiogroup");
        expect(group.getAttribute("aria-label")).toBe("Multiple choice options");

        const options = container.querySelectorAll<HTMLElement>(".proc-option-item");
        expect(options.length).toBe(4);

        expect(options[0].getAttribute("role")).toBe("radio");
        expect(options[0].getAttribute("aria-checked")).toBe("false");
        expect(options[0].getAttribute("tabindex")).toBe("0");

        expect(options[1].getAttribute("tabindex")).toBe("-1");
        expect(options[2].getAttribute("tabindex")).toBe("-1");
        expect(options[3].getAttribute("tabindex")).toBe("-1");

        mcq.destroy();
    });

    test("mouse click selection in practice mode evaluates canonically and applies styling", () => {
        let callbackResult: MCQEvaluationResult | null = null;
        let selectedOpt: MCQOption | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            correctAnswer: { canonical_id: "opt-paris" },
            onOptionSelected: (opt, res) => {
                selectedOpt = opt;
                callbackResult = res;
            },
        });

        const optParis = container.querySelector<HTMLElement>('[data-opt-id="opt-paris"]')!;
        optParis.click();

        expect(selectedOpt).not.toBeNull();
        expect(selectedOpt?.id).toBe("opt-paris");
        expect(callbackResult?.isCorrect).toBe(true);
        expect(callbackResult?.score).toBe(1.0);

        expect(optParis.classList.contains("selected")).toBe(true);
        expect(optParis.classList.contains("correct")).toBe(true);
        expect(optParis.getAttribute("aria-checked")).toBe("true");

        // Option feedback is revealed
        const feedback = optParis.querySelector<HTMLElement>(".proc-option-feedback")!;
        expect(feedback.classList.contains("hidden")).toBe(false);

        // Other options are disabled
        const optLondon = container.querySelector<HTMLElement>('[data-opt-id="opt-london"]')!;
        expect(optLondon.classList.contains("disabled")).toBe(true);

        mcq.destroy();
    });

    test("wrong answer selection marks selected as incorrect and expected as correct", () => {
        let callbackResult: MCQEvaluationResult | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            correctAnswer: { correct_option: "Paris" },
            onOptionSelected: (_opt, res) => {
                callbackResult = res;
            },
        });

        const optLondon = container.querySelector<HTMLElement>('[data-opt-id="opt-london"]')!;
        optLondon.click();

        expect(callbackResult?.isCorrect).toBe(false);
        expect(callbackResult?.score).toBe(0.0);

        expect(optLondon.classList.contains("selected")).toBe(true);
        expect(optLondon.classList.contains("incorrect")).toBe(true);

        const optParis = container.querySelector<HTMLElement>('[data-opt-id="opt-paris"]')!;
        expect(optParis.classList.contains("correct")).toBe(true);

        mcq.destroy();
    });

    test("keyboard 1-4 shortcuts select options accurately", () => {
        let callbackResult: MCQEvaluationResult | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            correctAnswer: { correct_option: "Paris" },
            onOptionSelected: (_opt, res) => {
                callbackResult = res;
            },
        });

        // Key '2' corresponds to option index 1 (Paris)
        const event = new KeyboardEvent("keydown", { key: "2", bubbles: true, cancelable: true });
        const handled = mcq.handleGlobalKeyDown(event);

        expect(handled).toBe(true);
        expect(event.defaultPrevented).toBe(true);
        expect(callbackResult?.isCorrect).toBe(true);
        expect(callbackResult?.selectedOptionId).toBe("opt-paris");

        mcq.destroy();
    });

    test("keyboard A-D shortcuts (case-insensitive) select options accurately", () => {
        let callbackResult: MCQEvaluationResult | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            correctAnswer: { correct_option: "Paris" },
            onOptionSelected: (_opt, res) => {
                callbackResult = res;
            },
        });

        // Key 'b' (lowercase) corresponds to option index 1 (Paris)
        const event = new KeyboardEvent("keydown", { key: "b", bubbles: true, cancelable: true });
        const handled = mcq.handleGlobalKeyDown(event);

        expect(handled).toBe(true);
        expect(event.defaultPrevented).toBe(true);
        expect(callbackResult?.isCorrect).toBe(true);
        expect(callbackResult?.selectedOptionId).toBe("opt-paris");

        mcq.destroy();
    });

    test("arrow navigation cycles focus between options and updates roving tabindex", () => {
        const mcq = new MCQContainer(container, {
            correctAnswer: { correct_option: "Paris" },
        });

        const options = container.querySelectorAll<HTMLElement>(".proc-option-item");

        // Focus first option
        options[0].focus();
        expect(options[0].getAttribute("tabindex")).toBe("0");

        // Navigate Down -> Option 1 (Paris)
        const downEvent = new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true, cancelable: true });
        options[0].dispatchEvent(downEvent);

        expect(options[1].getAttribute("tabindex")).toBe("0");
        expect(options[0].getAttribute("tabindex")).toBe("-1");

        // Navigate Up -> Wrap around to Option 3 (Rome)
        mcq.navigateOptions(-1);
        mcq.navigateOptions(-1);
        expect(options[3].getAttribute("tabindex")).toBe("0");

        mcq.destroy();
    });

    test("Enter and Space key confirm selection on focused option", () => {
        let callbackResult: MCQEvaluationResult | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            correctAnswer: { correct_option: "Paris" },
            onOptionSelected: (_opt, res) => {
                callbackResult = res;
            },
        });

        const optParis = container.querySelector<HTMLElement>('[data-opt-id="opt-paris"]')!;
        const enterEvent = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });
        optParis.dispatchEvent(enterEvent);

        expect(callbackResult?.isCorrect).toBe(true);
        expect(callbackResult?.selectedOptionId).toBe("opt-paris");

        mcq.destroy();
    });

    test("evaluates ConceptCheckData canonically using is_correct and expected_option_id", () => {
        let callbackResult: MCQEvaluationResult | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            conceptCheck: {
                prompt: "Identify the prime number",
                expected_option_id: "opt-berlin",
                options: [
                    { id: "opt-london", label: "4", is_correct: false, concept_tag: "even_composite", feedback: "4 is divisible by 2" },
                    { id: "opt-berlin", label: "7", is_correct: true, concept_tag: "prime", feedback: "7 is prime" },
                ],
            },
            onOptionSelected: (_opt, res) => {
                callbackResult = res;
            },
        });

        mcq.selectOptionById("opt-berlin");

        expect(callbackResult?.isCorrect).toBe(true);
        expect(callbackResult?.expectedOptionId).toBe("opt-berlin");
        expect(callbackResult?.reason).toBe("7 is prime");

        mcq.destroy();
    });

    test("evaluates StrategyDrillData canonically using is_optimal and preferred_option_id", () => {
        let callbackResult: MCQEvaluationResult | null = null;

        const mcq = new MCQContainer(container, {
            mode: "practice",
            strategyDrill: {
                prompt: "Best technique for solving x^2 - 5x + 6 = 0",
                problem_context: "Solve quadratic equation",
                preferred_option_id: "opt-paris",
                options: [
                    { id: "opt-london", label: "Quadratic Formula", is_optimal: false, strategy_tag: "slow", feedback: "Valid but slower" },
                    { id: "opt-paris", label: "Factoring (x-2)(x-3)", is_optimal: true, strategy_tag: "fast", feedback: "Fastest mental factoring" },
                ],
            },
            onOptionSelected: (_opt, res) => {
                callbackResult = res;
            },
        });

        mcq.selectOptionById("opt-paris");

        expect(callbackResult?.isCorrect).toBe(true);
        expect(callbackResult?.expectedOptionId).toBe("opt-paris");
        expect(callbackResult?.reason).toBe("Fastest mental factoring");

        mcq.destroy();
    });

    test("GAP-MOD-03: Mock exam mode allows selecting and changing choices without instant spoilers", () => {
        let selectionChanges: Array<string | null> = [];

        const mcq = new MCQContainer(container, {
            mode: "mock",
            correctAnswer: { canonical_id: "opt-paris" },
            onSelectionChanged: (selected) => {
                selectionChanges.push(selected ? selected.id : null);
            },
        });

        const optLondon = container.querySelector<HTMLElement>('[data-opt-id="opt-london"]')!;
        const optParis = container.querySelector<HTMLElement>('[data-opt-id="opt-paris"]')!;

        // 1. Select London (Key '1')
        mcq.handleGlobalKeyDown(new KeyboardEvent("keydown", { key: "1" }));

        expect(mcq.getSelectedOption()?.id).toBe("opt-london");
        expect(optLondon.classList.contains("selected")).toBe(true);
        expect(optLondon.getAttribute("aria-checked")).toBe("true");

        // CRITICAL CHECK: In mock mode, NO spoiler classes (.correct, .incorrect, .disabled) are applied!
        expect(optLondon.classList.contains("incorrect")).toBe(false);
        expect(optParis.classList.contains("correct")).toBe(false);
        expect(optLondon.classList.contains("disabled")).toBe(false);
        expect(mcq.isAlreadyEvaluated()).toBe(false);

        // 2. Change answer to Paris (Key '2')
        mcq.handleGlobalKeyDown(new KeyboardEvent("keydown", { key: "2" }));

        expect(mcq.getSelectedOption()?.id).toBe("opt-paris");
        expect(optParis.classList.contains("selected")).toBe(true);
        expect(optParis.getAttribute("aria-checked")).toBe("true");
        expect(optLondon.classList.contains("selected")).toBe(false);
        expect(optLondon.getAttribute("aria-checked")).toBe("false");

        // 3. Evaluate on demand when mock test finishes
        const finalEval = mcq.evaluate();
        expect(finalEval).not.toBeNull();
        expect(finalEval?.isCorrect).toBe(true);
        expect(finalEval?.selectedOptionId).toBe("opt-paris");
        expect(finalEval?.score).toBe(1.0);

        expect(selectionChanges).toEqual(["opt-london", "opt-paris"]);

        mcq.destroy();
    });

    test("cleans up event listeners and references on destroy", () => {
        const mcq = new MCQContainer(container, {
            correctAnswer: { correct_option: "Paris" },
        });

        expect(mcq.getOptionItems().length).toBe(4);
        mcq.destroy();

        expect(mcq.getOptionItems().length).toBe(0);
        expect(mcq.getSelectedOption()).toBeNull();
    });
});

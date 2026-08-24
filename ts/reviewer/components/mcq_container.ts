// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

import { bridgeCommand } from "@tslib/bridgecommand";

import type { ConceptCheckData, StrategyDrillData } from "../procedural";

declare const MathJax: any;

export type MCQMode = "practice" | "mock";

export interface MCQOption {
    id: string;
    index: number;
    keyLetter: string;
    keyNumber: string;
    label: string;
    feedback?: string;
    isCorrect?: boolean;
    element: HTMLElement;
}

export interface MCQEvaluationResult {
    isCorrect: boolean;
    selectedOptionId: string | null;
    selectedOptionIndex: number;
    expectedOptionId: string;
    selectedLabel: string;
    reason?: string;
    score: number;
}

export interface MCQContainerOptions {
    container?: HTMLElement;
    mode?: MCQMode;
    objectType?: "mcq" | "concept_check" | "strategy_drill" | string;
    correctAnswer?: Record<string, any>;
    conceptCheck?: ConceptCheckData | null;
    strategyDrill?: StrategyDrillData | null;
    onOptionSelected?: (option: MCQOption, evalResult: MCQEvaluationResult) => void;
    onSelectionChanged?: (option: MCQOption | null) => void;
    typesetMathJax?: (el: HTMLElement) => void;
}

/**
 * MCQContainer: Production-grade Multiple Choice Question modality component.
 *
 * Implements:
 * - Real selectable option buttons (`.proc-option-item`) with ARIA radio accessibility.
 * - Keyboard navigation (1-4, A-D / a-d, ArrowUp / ArrowDown / ArrowLeft / ArrowRight, Enter, Space).
 * - Canonical identity evaluation matching semantic IDs, option indices, letters, and numbers.
 * - Zero text input fallback enforcement.
 * - Instant review evaluation mode ("practice") vs un-graded mock exam mode ("mock") (GAP-MOD-03).
 */
export class MCQContainer {
    private container: HTMLElement;
    private options: MCQContainerOptions;
    private mode: MCQMode = "practice";
    private optionGroup: HTMLElement | null = null;
    private optionItems: MCQOption[] = [];
    private selectedOption: MCQOption | null = null;
    private focusedIndex = 0;
    private isEvaluated = false;
    private disposables: Array<() => void> = [];

    constructor(container: HTMLElement, options: MCQContainerOptions) {
        this.container = container;
        this.options = options;
        this.mode = options.mode || "practice";

        this.init();
    }

    public getMode(): MCQMode {
        return this.mode;
    }

    public setMode(mode: MCQMode): void {
        this.mode = mode;
    }

    public getSelectedOption(): MCQOption | null {
        return this.selectedOption;
    }

    public getOptionItems(): MCQOption[] {
        return [...this.optionItems];
    }

    public isAlreadyEvaluated(): boolean {
        return this.isEvaluated;
    }

    private addListener(
        element: EventTarget | null,
        type: string,
        listener: EventListenerOrEventListenerObject,
        options?: boolean | AddEventListenerOptions,
    ): void {
        if (!element) {return;}
        element.addEventListener(type, listener, options);
        this.disposables.push(() => {
            element.removeEventListener(type, listener, options);
        });
    }

    private init(): void {
        this.enforceZeroTextInputFallback();
        this.discoverOrRenderOptions();
        this.setupAccessibilityAttributes();
        this.attachEventListeners();
    }

    /**
     * Enforces the zero text input contract for MCQ items:
     * Hides and disables any text input field or quick solve container.
     */
    private enforceZeroTextInputFallback(): void {
        const quickContainer = this.container.querySelector<HTMLElement>("#proc-quick-container");
        if (quickContainer) {
            quickContainer.classList.add("hidden");
            quickContainer.style.display = "none";
        }

        const stepwiseContainer = this.container.querySelector<HTMLElement>("#proc-stepwise-container");
        if (stepwiseContainer) {
            stepwiseContainer.classList.add("hidden");
            stepwiseContainer.style.display = "none";
        }

        const modeSwitch = this.container.querySelector<HTMLElement>(".proc-mode-switch");
        if (modeSwitch) {
            modeSwitch.classList.add("hidden");
            modeSwitch.style.display = "none";
        }

        const answerInput = this.container.querySelector<HTMLInputElement>("#proc-answer-input");
        if (answerInput) {
            answerInput.disabled = true;
            answerInput.setAttribute("aria-hidden", "true");
        }
    }

    /**
     * Discovers existing `.proc-option-item` elements in DOM or creates the option item model.
     */
    private discoverOrRenderOptions(): void {
        this.optionGroup = this.container.querySelector<HTMLElement>(".proc-option-group");
        const rawItems = this.container.querySelectorAll<HTMLElement>(".proc-option-item");

        this.optionItems = [];
        rawItems.forEach((el, idx) => {
            const optId = el.dataset.optId || `opt-${idx}`;
            const optIdx = parseInt(el.dataset.optIdx || String(idx), 10);
            const keyLetter = String.fromCharCode(65 + Math.min(optIdx, 25));
            const keyNumber = String(optIdx + 1);
            const labelEl = el.querySelector<HTMLElement>(".proc-option-label");
            const label = labelEl?.textContent?.trim() || el.textContent?.trim() || optId;
            const feedbackEl = el.querySelector<HTMLElement>(".proc-option-feedback");
            const feedback = feedbackEl?.textContent?.trim();

            this.optionItems.push({
                id: optId,
                index: optIdx,
                keyLetter,
                keyNumber,
                label,
                feedback,
                element: el,
            });
        });
    }

    /**
     * Sets up ARIA accessibility roles and initial roving tabindex on option buttons.
     */
    private setupAccessibilityAttributes(): void {
        if (this.optionGroup) {
            this.optionGroup.setAttribute("role", "radiogroup");
            if (!this.optionGroup.hasAttribute("aria-label")) {
                const label = this.options.conceptCheck
                    ? "Concept check options"
                    : this.options.strategyDrill
                    ? "Strategy drill options"
                    : "Multiple choice options";
                this.optionGroup.setAttribute("aria-label", label);
            }
        }

        this.optionItems.forEach((item, idx) => {
            const el = item.element;
            el.setAttribute("role", "radio");
            el.setAttribute("aria-checked", "false");
            el.setAttribute("tabindex", idx === 0 ? "0" : "-1");
        });
    }

    /**
     * Attaches click, keydown, and window keyboard shortcuts.
     */
    private attachEventListeners(): void {
        this.optionItems.forEach((item, idx) => {
            // Mouse/Touch click
            this.addListener(item.element, "click", (e: Event) => {
                e.preventDefault();
                this.handleOptionInteraction(item);
            });

            // Focus management
            this.addListener(item.element, "focus", () => {
                this.focusedIndex = idx;
                this.updateRovingTabindex(idx);
            });

            // Keyboard navigation on option item
            this.addListener(item.element, "keydown", (e: Event) => {
                const kbEvent = e as KeyboardEvent;
                if (kbEvent.key === "Enter" || kbEvent.key === " " || kbEvent.code === "Space") {
                    kbEvent.preventDefault();
                    kbEvent.stopPropagation();
                    this.handleOptionInteraction(item);
                } else if (kbEvent.key === "ArrowDown" || kbEvent.key === "ArrowRight") {
                    kbEvent.preventDefault();
                    this.navigateOptions(1);
                } else if (kbEvent.key === "ArrowUp" || kbEvent.key === "ArrowLeft") {
                    kbEvent.preventDefault();
                    this.navigateOptions(-1);
                }
            });
        });
    }

    /**
     * Handles keyboard shortcuts (1-4, A-D / a-d) triggered globally or from parent container.
     */
    public handleGlobalKeyDown(event: KeyboardEvent): boolean {
        if (this.isEvaluated && this.mode === "practice") {
            return false;
        }

        const targetTag = (event.target as HTMLElement)?.tagName?.toLowerCase();
        if (targetTag === "input" || targetTag === "textarea") {
            return false;
        }

        // 1. Numeric keys: '1', '2', '3', '4', ...
        const keyNum = parseInt(event.key, 10);
        if (!isNaN(keyNum) && keyNum >= 1 && keyNum <= this.optionItems.length) {
            event.preventDefault();
            event.stopPropagation();
            const target = this.optionItems[keyNum - 1];
            if (target) {
                this.handleOptionInteraction(target);
                target.element.focus();
            }
            return true;
        }

        // 2. Alphabetic keys: 'A', 'B', 'C', 'D' / 'a', 'b', 'c', 'd'
        const keyUpper = event.key.toUpperCase();
        if (keyUpper.length === 1 && keyUpper >= "A" && keyUpper <= "Z") {
            const letterIdx = keyUpper.charCodeAt(0) - 65;
            if (letterIdx >= 0 && letterIdx < this.optionItems.length) {
                event.preventDefault();
                event.stopPropagation();
                const target = this.optionItems[letterIdx];
                if (target) {
                    this.handleOptionInteraction(target);
                    target.element.focus();
                }
                return true;
            }
        }

        // 3. Arrow keys when option items are present
        if (event.key === "ArrowDown" || event.key === "ArrowRight") {
            event.preventDefault();
            this.navigateOptions(1);
            return true;
        } else if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
            event.preventDefault();
            this.navigateOptions(-1);
            return true;
        }

        return false;
    }

    /**
     * Shifts focus to next or previous option item.
     */
    public navigateOptions(direction: 1 | -1): void {
        if (this.optionItems.length === 0) {return;}
        let nextIdx = this.focusedIndex + direction;
        if (nextIdx < 0) {
            nextIdx = this.optionItems.length - 1;
        } else if (nextIdx >= this.optionItems.length) {
            nextIdx = 0;
        }
        this.focusedIndex = nextIdx;
        this.updateRovingTabindex(nextIdx);
        this.optionItems[nextIdx].element.focus();
    }

    private updateRovingTabindex(targetIdx: number): void {
        this.optionItems.forEach((item, idx) => {
            item.element.setAttribute("tabindex", idx === targetIdx ? "0" : "-1");
        });
    }

    /**
     * Primary handler when an option is clicked or triggered via keyboard.
     */
    public handleOptionInteraction(option: MCQOption): void {
        if (this.isEvaluated && this.mode === "practice") {
            return;
        }

        if (this.mode === "mock") {
            // Mock exam mode (GAP-MOD-03): select/toggle option without instant answer reveal
            this.applyMockSelection(option);
            if (this.options.onSelectionChanged) {
                this.options.onSelectionChanged(this.selectedOption);
            }
            bridgeCommand(`procedural_mock_selection:${JSON.stringify({
                option_id: option.id,
                option_index: option.index,
                label: option.label,
            })}`);
        } else {
            // Practice mode: instant evaluation and feedback reveal
            this.applyPracticeSelection(option);
        }
    }

    /**
     * Applies selection in mock exam mode: updates `.selected` and `aria-checked`
     * without displaying `.correct` or `.incorrect` spoilers.
     */
    private applyMockSelection(option: MCQOption): void {
        this.selectedOption = option;

        this.optionItems.forEach((item) => {
            const isTarget = item.id === option.id;
            item.element.classList.toggle("selected", isTarget);
            item.element.setAttribute("aria-checked", isTarget ? "true" : "false");
            // Ensure no spoilers in mock mode
            item.element.classList.remove("correct", "incorrect", "disabled");
        });
    }

    /**
     * Applies selection in practice mode: highlights correct/incorrect, locks options,
     * reveals rationales, and invokes `onOptionSelected`.
     */
    private applyPracticeSelection(option: MCQOption): void {
        this.selectedOption = option;
        this.isEvaluated = true;

        const evalResult = this.evaluateSelection(option);

        this.optionItems.forEach((item) => {
            const el = item.element;
            const isSelected = item.id === option.id;
            const isExpected = item.id === evalResult.expectedOptionId || 
                (evalResult.expectedOptionId && item.label.toLowerCase() === evalResult.expectedOptionId.toLowerCase());

            el.classList.toggle("selected", isSelected);
            el.setAttribute("aria-checked", isSelected ? "true" : "false");
            el.classList.add("disabled");

            if (isExpected) {
                el.classList.add("correct");
            } else if (isSelected && !evalResult.isCorrect) {
                el.classList.add("incorrect");
            }

            // Reveal feedback if present on the option
            const feedbackEl = el.querySelector<HTMLElement>(".proc-option-feedback");
            if (feedbackEl && (isSelected || isExpected)) {
                feedbackEl.classList.remove("hidden");
                feedbackEl.style.display = "block";
            }
        });

        if (this.options.typesetMathJax) {
            this.options.typesetMathJax(this.container);
        }

        if (this.options.onOptionSelected) {
            this.options.onOptionSelected(option, evalResult);
        }
    }

    /**
     * Canonical Identity Evaluation:
     * Compares selected option against canonical ID, key, letter, index, and formatted answer.
     */
    public evaluateSelection(option: MCQOption): MCQEvaluationResult {
        let isCorrect = false;
        let expectedId = "";
        let feedbackText = "";

        if (this.options.conceptCheck) {
            const cc = this.options.conceptCheck;
            expectedId = cc.expected_option_id;
            const chosen = cc.options.find((o) => o.id === option.id || o.label === option.label);
            isCorrect = chosen ? chosen.is_correct : false;
            feedbackText = chosen?.feedback || (isCorrect ? "Correct concept understanding." : "Misconception detected.");
        } else if (this.options.strategyDrill) {
            const sd = this.options.strategyDrill;
            expectedId = sd.preferred_option_id;
            const chosen = sd.options.find((o) => o.id === option.id || o.label === option.label);
            isCorrect = chosen ? chosen.is_optimal : false;
            feedbackText = chosen?.feedback || (isCorrect ? "Optimal strategy selected." : "Suboptimal method chosen.");
        } else {
            // Standard MCQ Canonical Evaluation
            const correctOpt = String(
                this.options.correctAnswer?.canonical_id ||
                this.options.correctAnswer?.correct_option_id ||
                this.options.correctAnswer?.expected_option_id ||
                this.options.correctAnswer?.correct_option ||
                this.options.correctAnswer?.formatted ||
                this.options.correctAnswer?.answer ||
                this.options.correctAnswer?.value ||
                ""
            ).trim();

            const targetId = option.id.trim().toLowerCase();
            const targetIdxStr = String(option.index);
            const targetNumStr = String(option.index + 1);
            const targetLetter = option.keyLetter.toLowerCase();
            const targetLabel = option.label.trim().toLowerCase();
            const correctOptLower = correctOpt.toLowerCase();

            // Match against ID, letter, index, 1-based number, or label
            isCorrect =
                (targetId.length > 0 && targetId === correctOptLower) ||
                (targetLetter.length > 0 && targetLetter === correctOptLower) ||
                targetIdxStr === correctOpt ||
                targetNumStr === correctOpt ||
                (targetLabel.length > 0 && targetLabel === correctOptLower);

            // Locate canonical expected option ID across all options
            for (const item of this.optionItems) {
                const itemOptId = item.id.trim().toLowerCase();
                const itemIdxStr = String(item.index);
                const itemNumStr = String(item.index + 1);
                const itemLetter = item.keyLetter.toLowerCase();
                const itemLabel = item.label.trim().toLowerCase();

                if (
                    (itemOptId.length > 0 && itemOptId === correctOptLower) ||
                    (itemLetter.length > 0 && itemLetter === correctOptLower) ||
                    itemIdxStr === correctOpt ||
                    itemNumStr === correctOpt ||
                    (itemLabel.length > 0 && itemLabel === correctOptLower)
                ) {
                    expectedId = item.id;
                    break;
                }
            }

            if (!expectedId && correctOpt.length > 0) {
                expectedId = correctOpt;
            }

            feedbackText = isCorrect ? "Correct answer selected." : "Incorrect option selected.";
        }

        return {
            isCorrect,
            selectedOptionId: option.id,
            selectedOptionIndex: option.index,
            expectedOptionId: expectedId,
            selectedLabel: option.label,
            reason: feedbackText,
            score: isCorrect ? 1.0 : 0.0,
        };
    }

    /**
     * Evaluates the current selection on demand (used when finalizing a mock exam session).
     */
    public evaluate(): MCQEvaluationResult | null {
        if (!this.selectedOption) {
            return null;
        }
        return this.evaluateSelection(this.selectedOption);
    }

    /**
     * Selects an option programmatically by ID, letter, or index.
     */
    public selectOptionById(optIdOrKey: string): void {
        const query = String(optIdOrKey).trim().toLowerCase();
        const found = this.optionItems.find((item) => 
            item.id.toLowerCase() === query ||
            item.keyLetter.toLowerCase() === query ||
            item.keyNumber === query ||
            String(item.index) === query ||
            item.label.toLowerCase() === query
        );

        if (found) {
            this.handleOptionInteraction(found);
            found.element.focus();
        }
    }

    /**
     * Cleans up all DOM listeners and references.
     */
    public destroy(): void {
        for (const dispose of this.disposables) {
            try {
                dispose();
            } catch {
                /* non-fatal */
            }
        }
        this.disposables = [];
        this.optionItems = [];
        this.selectedOption = null;
        this.optionGroup = null;
    }
}

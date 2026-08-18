// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

import { bridgeCommand } from "@tslib/bridgecommand";

declare const MathJax: any;

export interface ProceduralSolutionStep {
    id?: string;
    description: string;
    target_expression?: string;
    hints?: Array<{
        level: number;
        title?: string;
        content: string;
    }>;
}

export interface ProceduralSolutionGraph {
    steps: ProceduralSolutionStep[];
    terminal_step_id?: string;
}

export interface ProceduralSetupOptions {
    containerId?: string;
    instanceId: string;
    familyId: string;
    targetTimeMs: number;
    correctAnswer: Record<string, any>;
    parameters?: Record<string, any>;
    solutionGraph?: ProceduralSolutionGraph | null;
    onCompleted?: (result: ProceduralAttemptResult) => void;
}

export interface ProceduralAttemptResult {
    instanceId: string;
    answer: string;
    mode: "quick" | "stepwise";
    steps: string[];
    hintsUsed: number;
    timeTakenMs: number;
    isCorrect: boolean;
    score: number;
}

export class ProceduralReviewer {
    private container: HTMLElement;
    private options: ProceduralSetupOptions;
    private startTime: number;
    private timerInterval: any = null;
    private isSubmitted = false;
    private hintsUsed = 0;
    private hintTimestamps: number[] = [];
    private activeMode: "quick" | "stepwise" = "quick";
    private disposables: Array<() => void> = [];
    private focusTimeout: any = null;

    // DOM Elements
    private timerEl: HTMLElement | null = null;
    private quickContainer: HTMLElement | null = null;
    private stepwiseContainer: HTMLElement | null = null;
    private quickInput: HTMLInputElement | null = null;
    private quickSubmitBtn: HTMLButtonElement | null = null;
    private tabQuickBtn: HTMLButtonElement | null = null;
    private tabStepwiseBtn: HTMLButtonElement | null = null;
    private stepsList: HTMLElement | null = null;
    private addStepBtn: HTMLButtonElement | null = null;
    private hintBtn: HTMLButtonElement | null = null;
    private resetBtn: HTMLButtonElement | null = null;
    private checkStepsBtn: HTMLButtonElement | null = null;
    private hintBox: HTMLElement | null = null;
    private resultPanel: HTMLElement | null = null;
    private resultTitle: HTMLElement | null = null;
    private resultFeedback: HTMLElement | null = null;
    private actualTimeEl: HTMLElement | null = null;
    private nextBtn: HTMLButtonElement | null = null;

    constructor(container: HTMLElement, options: ProceduralSetupOptions) {
        this.container = container;
        this.options = options;
        this.startTime = Date.now();
        this.bindElements();
        this.attachEventListeners();
        this.startTimer();
    }

    private bindElements(): void {
        this.timerEl = this.container.querySelector("#proc-stopwatch");
        this.quickContainer = this.container.querySelector("#proc-quick-container");
        this.stepwiseContainer = this.container.querySelector("#proc-stepwise-container");
        this.quickInput = this.container.querySelector("#proc-answer-input");
        this.quickSubmitBtn = this.container.querySelector("#proc-submit-btn");
        this.tabQuickBtn = this.container.querySelector("#tab-quick");
        this.tabStepwiseBtn = this.container.querySelector("#tab-stepwise");
        this.stepsList = this.container.querySelector("#proc-steps-list");
        this.addStepBtn = this.container.querySelector("#proc-add-step-btn");
        this.hintBtn = this.container.querySelector("#proc-hint-btn");
        this.resetBtn = this.container.querySelector("#proc-reset-steps-btn");
        this.checkStepsBtn = this.container.querySelector("#proc-check-steps-btn");
        this.hintBox = this.container.querySelector("#proc-hint-container");
        this.resultPanel = this.container.querySelector("#proc-result-panel");
        this.resultTitle = this.container.querySelector("#proc-result-title");
        this.resultFeedback = this.container.querySelector("#proc-result-feedback");
        this.actualTimeEl = this.container.querySelector("#proc-actual-time");
        this.nextBtn = this.container.querySelector("#proc-next-btn");
    }

    private addListener<K extends keyof HTMLElementEventMap>(
        element: HTMLElement | null,
        type: K,
        listener: (this: HTMLElement, ev: HTMLElementEventMap[K]) => any,
        options?: boolean | AddEventListenerOptions,
    ): void {
        if (!element) return;
        element.addEventListener(type, listener as EventListener, options);
        this.disposables.push(() => {
            element.removeEventListener(type, listener as EventListener, options);
        });
    }

    private attachEventListeners(): void {
        // Tab switching
        this.addListener(this.tabQuickBtn, "click", () => this.switchMode("quick"));
        this.addListener(this.tabStepwiseBtn, "click", () => this.switchMode("stepwise"));

        // Quick submit
        this.addListener(this.quickSubmitBtn, "click", () => this.handleQuickSubmit());
        this.addListener(this.quickInput, "keydown", (e: KeyboardEvent) => {
            if (e.key === "Enter") {
                this.handleQuickSubmit();
            }
        });

        // Stepwise controls
        this.addListener(this.addStepBtn, "click", () => this.addStepRow());
        this.addListener(this.resetBtn, "click", () => this.resetSteps());
        this.addListener(this.hintBtn, "click", () => this.requestHint());
        this.addListener(this.checkStepsBtn, "click", () => this.handleStepwiseSubmit());

        // Next problem button
        this.addListener(this.nextBtn, "click", () => this.handleNext());

        // Auto-focus initial input
        this.focusTimeout = setTimeout(() => {
            this.quickInput?.focus();
        }, 50);
    }

    private startTimer(): void {
        this.timerInterval = setInterval(() => {
            if (this.isSubmitted) return;
            const elapsed = Math.floor((Date.now() - this.startTime) / 1000);
            const m = String(Math.floor(elapsed / 60)).padStart(2, "0");
            const s = String(elapsed % 60).padStart(2, "0");
            if (this.timerEl) {
                this.timerEl.textContent = `${m}:${s}`;
            }
        }, 200);
    }

    public switchMode(mode: "quick" | "stepwise"): void {
        if (this.isSubmitted) return;
        this.activeMode = mode;
        if (mode === "quick") {
            this.tabQuickBtn?.classList.add("active");
            this.tabStepwiseBtn?.classList.remove("active");
            this.quickContainer?.classList.remove("hidden");
            this.stepwiseContainer?.classList.add("hidden");
            this.quickInput?.focus();
        } else {
            this.tabStepwiseBtn?.classList.add("active");
            this.tabQuickBtn?.classList.remove("active");
            this.stepwiseContainer?.classList.remove("hidden");
            this.quickContainer?.classList.add("hidden");
            const firstStepInput = this.stepsList?.querySelector<HTMLInputElement>("input");
            firstStepInput?.focus();
        }
    }

    public addStepRow(): void {
        if (!this.stepsList || this.isSubmitted) return;
        const currentCount = this.stepsList.querySelectorAll(".proc-step-row").length;
        const stepNum = currentCount + 1;

        const row = document.createElement("div");
        row.className = "proc-step-row";
        row.dataset.stepIdx = String(currentCount);
        row.innerHTML = `
            <span class="proc-step-label">Step ${stepNum}</span>
            <input type="text" class="proc-input proc-step-input" placeholder="Write step ${stepNum} transformation..." autocomplete="off" />
        `;

        this.stepsList.appendChild(row);
        const input = row.querySelector<HTMLInputElement>("input");
        input?.focus();
        this.typesetMathJax(row);
    }

    public resetSteps(): void {
        if (!this.stepsList || this.isSubmitted) return;
        this.stepsList.innerHTML = `
            <div class="proc-step-row" data-step-idx="0">
                <span class="proc-step-label">Step 1</span>
                <input type="text" class="proc-input proc-step-input" placeholder="Write step 1 transformation or equation..." autocomplete="off" />
            </div>
        `;
        if (this.hintBox) {
            this.hintBox.classList.add("hidden");
            this.hintBox.innerHTML = "";
        }
        const firstInput = this.stepsList.querySelector<HTMLInputElement>("input");
        firstInput?.focus();
    }

    public requestHint(): void {
        if (this.isSubmitted) return;
        this.hintsUsed += 1;
        this.hintTimestamps.push(Date.now() - this.startTime);

        let hintText = "";
        const graph = this.options.solutionGraph;

        if (graph && graph.steps && graph.steps.length > 0) {
            const stepIdx = Math.min(this.hintsUsed - 1, graph.steps.length - 1);
            const step = graph.steps[stepIdx];
            if (step.hints && step.hints.length > 0) {
                const hintObj = step.hints[(this.hintsUsed - 1) % step.hints.length];
                hintText = `<strong>${hintObj.title || "Hint"}:</strong> ${hintObj.content}`;
            } else {
                hintText = `<strong>Hint ${this.hintsUsed}:</strong> ${step.description}`;
            }
        } else {
            hintText = "<strong>Hint:</strong> Identify key governing principles, write known values, and apply the required inverse relation.";
        }

        if (this.hintBox) {
            this.hintBox.classList.remove("hidden");
            this.hintBox.innerHTML = `
                <div>💡 ${hintText}</div>
                <div class="proc-hint-meta">(Hints used: ${this.hintsUsed})</div>
            `;
            this.typesetMathJax(this.hintBox);
        }

        bridgeCommand(`procedural_hint:${JSON.stringify({
            instance_id: this.options.instanceId,
            hint_level: this.hintsUsed,
        })}`);
    }

    public parseNumericValue(val: string | null | undefined): number | null {
        if (!val) return null;
        const cleaned = String(val).replace(/[$€£₹%, ]/g, "").trim();
        if (cleaned.includes("/")) {
            const parts = cleaned.split("/");
            const num = parseFloat(parts[0]);
            const den = parseFloat(parts[1]);
            if (!isNaN(num) && !isNaN(den) && den !== 0) {
                return num / den;
            }
        }
        const n = parseFloat(cleaned);
        return isNaN(n) ? null : n;
    }

    public evaluateLocally(userText: string): { isCorrect: boolean; reason?: string; score: number } {
        const expectedVal = this.options.correctAnswer?.value;
        const userNum = this.parseNumericValue(userText);

        if (expectedVal !== undefined && typeof expectedVal === "number") {
            if (userNum === null) {
                return { isCorrect: false, reason: "Please enter a valid numeric value or fraction.", score: 0.0 };
            }
            const diff = Math.abs(userNum - expectedVal);
            const tolerance = Math.max(0.01, Math.abs(expectedVal) * 0.01);
            const isCorrect = diff <= tolerance;
            return {
                isCorrect,
                reason: isCorrect ? undefined : `Expected ${expectedVal}, but received ${userText}`,
                score: isCorrect ? 1.0 : 0.0,
            };
        }

        // Fallback string matching if not numeric
        const canonicalFormatted = String(this.options.correctAnswer?.formatted || "").trim().toLowerCase();
        const userCleaned = userText.trim().toLowerCase();
        const isMatch = userCleaned === canonicalFormatted && canonicalFormatted.length > 0;

        return {
            isCorrect: isMatch,
            reason: isMatch ? undefined : "Answer differs from canonical solution.",
            score: isMatch ? 1.0 : 0.0,
        };
    }

    private handleQuickSubmit(): void {
        const answer = this.quickInput?.value.trim() || "";
        if (!answer || this.isSubmitted) return;

        const evalResult = this.evaluateLocally(answer);
        this.finishAttempt(evalResult, { answer, steps: [] });
    }

    private handleStepwiseSubmit(): void {
        if (this.isSubmitted || !this.stepsList) return;
        const stepInputs = this.stepsList.querySelectorAll<HTMLInputElement>(".proc-step-input");
        const steps: string[] = [];

        stepInputs.forEach((input) => {
            const val = input.value.trim();
            if (val) steps.push(val);
        });

        const lastAnswer = steps.length > 0 ? steps[steps.length - 1] : "";
        const evalResult = this.evaluateLocally(lastAnswer);
        this.finishAttempt(evalResult, { answer: lastAnswer, steps });
    }

    private finishAttempt(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
    ): void {
        this.isSubmitted = true;
        clearInterval(this.timerInterval);
        const timeTakenMs = Date.now() - this.startTime;

        // Hide input containers, show result panel
        this.quickContainer?.classList.add("hidden");
        this.stepwiseContainer?.classList.add("hidden");
        this.container.querySelector(".proc-mode-switch")?.classList.add("hidden");
        this.resultPanel?.classList.remove("hidden");

        if (this.actualTimeEl) {
            this.actualTimeEl.innerHTML = `<strong>Actual Time:</strong> ${(timeTakenMs / 1000).toFixed(1)}s`;
        }

        if (this.resultPanel) {
            if (outcome.isCorrect) {
                this.resultPanel.className = "proc-result correct";
                if (this.resultTitle) this.resultTitle.textContent = "✓ Correct Answer";
                let msg = `Completed in ${(timeTakenMs / 1000).toFixed(1)}s`;
                if (timeTakenMs > this.options.targetTimeMs) {
                    msg += ` (Target latency: ${(this.options.targetTimeMs / 1000).toFixed(0)}s)`;
                }
                if (this.hintsUsed > 0) {
                    msg += ` [${this.hintsUsed} hint(s) used]`;
                }
                if (this.resultFeedback) this.resultFeedback.textContent = msg;
            } else {
                this.resultPanel.className = "proc-result incorrect";
                if (this.resultTitle) this.resultTitle.textContent = "✗ Incorrect Answer";
                if (this.resultFeedback) {
                    this.resultFeedback.textContent = outcome.reason || "Review the step-by-step canonical solution below.";
                }
            }
        }

        this.typesetMathJax(this.resultPanel);

        const attemptResult: ProceduralAttemptResult = {
            instanceId: this.options.instanceId,
            answer: data.answer,
            mode: this.activeMode,
            steps: data.steps,
            hintsUsed: this.hintsUsed,
            timeTakenMs,
            isCorrect: outcome.isCorrect,
            score: outcome.score,
        };

        if (this.options.onCompleted) {
            this.options.onCompleted(attemptResult);
        }

        // Bridge notification for Python/Qt backend telemetry recording
        bridgeCommand(`procedural_attempt:${JSON.stringify(attemptResult)}`);
    }

    private handleNext(): void {
        // Trigger Anki's show answer / ease rating transition
        bridgeCommand("ans");
    }

    private typesetMathJax(element: HTMLElement | null): void {
        if (element && typeof MathJax !== "undefined" && MathJax.typesetPromise) {
            MathJax.typesetPromise([element]).catch(() => {
                /* non-fatal if MathJax typeset fails */
            });
        }
    }

    public destroy(): void {
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
            this.timerInterval = null;
        }
        if (this.focusTimeout) {
            clearTimeout(this.focusTimeout);
            this.focusTimeout = null;
        }
        for (const dispose of this.disposables) {
            try {
                dispose();
            } catch {
                /* non-fatal */
            }
        }
        this.disposables = [];
        this.isSubmitted = true;
    }
}

/** Global Procedural Review API */
export const proceduralAPI = {
    setup: (options: ProceduralSetupOptions): ProceduralReviewer => {
        const containerId = options.containerId || "procedural-card";
        const container = document.getElementById(containerId) || document.querySelector(".procedural-card-container");
        if (!container) {
            throw new Error(`Procedural container element not found: #${containerId}`);
        }
        // Safely destroy any existing reviewer instance on this container before creating a new one
        const prev = (container as any).__proceduralReviewer;
        if (prev && typeof prev.destroy === "function") {
            prev.destroy();
        }
        const reviewer = new ProceduralReviewer(container as HTMLElement, options);
        (container as any).__proceduralReviewer = reviewer;
        return reviewer;
    },
    ProceduralReviewer,
};

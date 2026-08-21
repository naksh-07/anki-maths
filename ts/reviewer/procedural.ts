// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

import { bridgeCommand } from "@tslib/bridgecommand";

declare const MathJax: any;

export type ProceduralUIState =
    | "loading"
    | "ready"
    | "solving"
    | "hint"
    | "submitting"
    | "mistake_classification"
    | "feedback"
    | "worked_example"
    | "next"
    | "error"
    | "teardown";

export type LearningObjectKind =
    | "problem"
    | "concept_check"
    | "strategy_drill"
    | "worked_example"
    | "declarative_recall"
    | "prerequisite_review";

export interface ConceptCheckOptionData {
    id: string;
    label: string;
    is_correct: boolean;
    concept_tag: string;
    feedback: string;
}

export interface ConceptCheckData {
    id?: string;
    prompt: string;
    context?: string;
    options: ConceptCheckOptionData[];
    expected_option_id: string;
    explanation?: string;
}

export interface StrategyOptionData {
    id: string;
    label: string;
    strategy_tag: string;
    is_optimal: boolean;
    feedback: string;
}

export interface StrategyDrillData {
    id?: string;
    prompt: string;
    problem_context: string;
    options: StrategyOptionData[];
    preferred_option_id: string;
    explanation?: string;
}

export interface WorkedExampleData {
    id?: string;
    prompt: string;
    problem_context: string;
    canonical_steps: string[];
    highlighted_decision_point: string;
    method_rationale: string;
    common_mistakes_to_avoid?: string[];
}

export interface DeclarativeRecallData {
    id?: string;
    concept_name: string;
    prompt_summary: string;
    formula_or_fact: string;
    target_anki_card_id?: number | null;
    target_anki_tag?: string | null;
}

export interface PrerequisiteReviewData {
    id?: string;
    advisory_message: string;
    recommendation_summary?: string;
    primary_missing_prerequisite?: string;
    executable_schema_id?: string;
}

export interface ContentProvenanceData {
    exam?: string;
    year?: number;
    shift?: string;
    paper?: string;
    variant_type?: "practice" | "structural" | "transfer" | "authentic" | string;
}

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
    schemaId?: string;
    skillId?: string;
    topicId?: string;
    targetTimeMs: number;
    correctAnswer?: Record<string, any>;
    parameters?: Record<string, any>;
    solutionGraph?: ProceduralSolutionGraph | null;
    objectType?: LearningObjectKind;
    conceptCheck?: ConceptCheckData | null;
    strategyDrill?: StrategyDrillData | null;
    workedExample?: WorkedExampleData | null;
    declarativeRecall?: DeclarativeRecallData | null;
    prerequisiteReview?: PrerequisiteReviewData | null;
    provenance?: ContentProvenanceData | null;
    remediationMessage?: string | null;
    onCompleted?: (result: ProceduralAttemptResult) => void;
}

export interface ProceduralAttemptResult {
    instanceId: string;
    answer: string;
    mode: "quick" | "stepwise" | "concept_check" | "strategy_drill" | "worked_example" | "declarative_recall" | "prerequisite_review";
    steps: string[];
    hintsUsed: number;
    timeTakenMs: number;
    isCorrect: boolean;
    score: number;
    selectedOptionId?: string;
    speedQuadrant?: "fluency_strength" | "speed_opportunity" | "strategy_trap" | "concept_setup";
}

export class ProceduralReviewer {
    private container: HTMLElement;
    private options: ProceduralSetupOptions;
    private startTime: number;
    private timerInterval: any = null;
    private state: ProceduralUIState = "loading";
    private hintsUsed = 0;
    private hintTimestamps: number[] = [];
    private activeMode: "quick" | "stepwise" = "quick";
    private disposables: Array<() => void> = [];
    private focusTimeout: any = null;
    private selectedOptionId: string | null = null;
    private hasSubmitted = false;
    private mistakeType: string | null = null;
    private mistakePanel: HTMLElement | null = null;

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

    // Footer Elements
    private footerEl: HTMLElement | null = null;
    private footerCenter: HTMLElement | null = null;
    private footerSubmitBtn: HTMLButtonElement | null = null;
    private editBtn: HTMLButtonElement | null = null;
    private moreBtn: HTMLButtonElement | null = null;
    private moreMenu: HTMLElement | null = null;

    constructor(container: HTMLElement, options: ProceduralSetupOptions) {
        this.container = container;
        this.options = options;
        this.startTime = Date.now();
        this.state = "ready";
        this.buildFooter();
        this.bindElements();
        this.attachEventListeners();
        this.startTimer();
        this.state = "solving";
    }

    public getState(): ProceduralUIState {
        return this.state;
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
        this.footerSubmitBtn = this.container.querySelector("#proc-footer-submit-btn");
    }

    private buildFooter(): void {
        this.footerEl = document.createElement("div");
        this.footerEl.className = "proc-footer";
        this.footerEl.innerHTML = `
            <button class="proc-btn-secondary" id="proc-edit-btn">Edit</button>
            <div class="proc-footer-center" id="proc-footer-center">
                 <button class="proc-btn" id="proc-footer-submit-btn">Show Answer / Submit</button>
            </div>
            <div class="proc-footer-more">
                 <button class="proc-btn-secondary" id="proc-more-btn">More ▾</button>
                 <div id="proc-more-menu" class="proc-more-menu hidden">
                     <button class="proc-more-item" data-action="hint">Hint</button>
                     <button class="proc-more-item" data-action="tricks">Tricks</button>
                     <button class="proc-more-item" data-action="solution">Solution</button>
                     <button class="proc-more-item" data-action="explanation">Explanation</button>
                     <button class="proc-more-item" data-action="source">Source</button>
                 </div>
            </div>
        `;
        this.container.appendChild(this.footerEl);
        
        this.footerCenter = this.footerEl.querySelector("#proc-footer-center");
        this.editBtn = this.footerEl.querySelector("#proc-edit-btn");
        this.moreBtn = this.footerEl.querySelector("#proc-more-btn");
        this.moreMenu = this.footerEl.querySelector("#proc-more-menu");
    }

    private addListener<K extends keyof HTMLElementEventMap>(
        element: HTMLElement | null,
        type: K,
        listener: (this: HTMLElement, ev: HTMLElementEventMap[K]) => any,
        options?: boolean | AddEventListenerOptions,
    ): void {
        if (!element) {return;}
        element.addEventListener(type, listener as EventListener, options);
        this.disposables.push(() => {
            element.removeEventListener(type, listener as EventListener, options);
        });
    }

    private attachEventListeners(): void {
        // Tab switching
        this.addListener(this.tabQuickBtn, "click", () => this.switchMode("quick"));
        this.addListener(this.tabStepwiseBtn, "click", () => this.switchMode("stepwise"));

        // Quick submit (inline button usually replaced by footer, but we'll keep the listener in case)
        if (this.quickSubmitBtn) {
            this.addListener(this.quickSubmitBtn, "click", () => this.handleQuickSubmit());
        }
        if (this.footerSubmitBtn) {
            this.addListener(this.footerSubmitBtn, "click", () => {
                if (this.activeMode === "quick") {
                    this.handleQuickSubmit();
                } else if (this.activeMode === "stepwise") {
                    this.handleStepwiseSubmit();
                }
            });
        }
        
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

        // Structured option items (ConceptCheck, StrategyDrill)
        const optionItems = this.container.querySelectorAll<HTMLElement>(".proc-option-item");
        optionItems.forEach((optEl) => {
            const optId = optEl.dataset.optId || "";
            this.addListener(optEl, "click", () => this.selectOption(optId, optEl));
            this.addListener(optEl, "keydown", (e: KeyboardEvent) => {
                if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    this.selectOption(optId, optEl);
                }
            });
        });

        // Keyboard option number keys (1-4) for quick accessible selection in option lists
        this.addListener(this.container, "keydown", (e: KeyboardEvent) => {
            if (this.state !== "solving") {return;}
            const targetTag = (e.target as HTMLElement)?.tagName?.toLowerCase();
            if (targetTag === "input" || targetTag === "textarea") {return;}

            const keyNum = parseInt(e.key, 10);
            if (!isNaN(keyNum) && keyNum >= 1 && keyNum <= optionItems.length) {
                const targetOpt = optionItems[keyNum - 1];
                if (targetOpt) {
                    const optId = targetOpt.dataset.optId || "";
                    this.selectOption(optId, targetOpt);
                }
            }
        });

        // WorkedExample "Try Similar" button
        const trySimilarBtn = this.container.querySelector<HTMLButtonElement>("#proc-try-similar-btn");
        this.addListener(trySimilarBtn, "click", () => this.handleTrySimilar());

        // DeclarativeRecall "Review in Anki" button
        const recallBtn = this.container.querySelector<HTMLButtonElement>("#proc-anki-recall-btn");
        this.addListener(recallBtn, "click", () => this.handleDeclarativeRecallAction());

        // Prerequisite "Practice Prerequisite" button
        const prereqBtn = this.container.querySelector<HTMLButtonElement>("#proc-practice-prereq-btn");
        this.addListener(prereqBtn, "click", () => this.handlePracticePrerequisite());

        // Next problem / continue button
        this.addListener(this.nextBtn, "click", () => this.handleNext());
        
        // Footer actions
        this.addListener(this.editBtn, "click", () => {
            bridgeCommand("edit");
        });
        
        this.addListener(this.moreBtn, "click", (e: Event) => {
            e.stopPropagation();
            this.moreMenu?.classList.toggle("hidden");
        });
        
        // Close more menu when clicking outside
        this.addListener(document.body, "click", () => {
            this.moreMenu?.classList.add("hidden");
        });
        
        const moreItems = this.moreMenu?.querySelectorAll<HTMLButtonElement>(".proc-more-item");
        moreItems?.forEach((item) => {
            this.addListener(item, "click", (e: Event) => {
                e.stopPropagation();
                this.moreMenu?.classList.add("hidden");
                const action = item.dataset.action;
                if (action === "hint") {
                    this.requestHint();
                } else if (action) {
                    bridgeCommand(`procedural_more_action:${action}`);
                }
            });
        });

        // Auto-focus initial input or first option
        this.focusTimeout = setTimeout(() => {
            if (this.quickInput) {
                this.quickInput.focus();
            } else if (optionItems.length > 0) {
                optionItems[0].focus();
            }
        }, 50);
    }

    private startTimer(): void {
        this.timerInterval = setInterval(() => {
            if (this.state === "feedback" || this.state === "next" || this.state === "teardown") {return;}
            const elapsed = Math.floor((Date.now() - this.startTime) / 1000);
            const m = String(Math.floor(elapsed / 60)).padStart(2, "0");
            const s = String(elapsed % 60).padStart(2, "0");
            if (this.timerEl) {
                this.timerEl.textContent = `${m}:${s}`;
            }
        }, 200);
    }

    public switchMode(mode: "quick" | "stepwise"): void {
        if (this.state !== "solving") {return;}
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
        if (!this.stepsList || this.state !== "solving") {return;}
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
        if (!this.stepsList || this.state !== "solving") {return;}
        this.stepsList.innerHTML = `
            <div class="proc-step-row" data-step-idx="0">
                <span class="proc-step-label">Step 1</span>
                <input type="text" class="proc-input proc-step-input" placeholder="Write step 1 transformation or equation..." autocomplete="off" />
            </div>
        `;
        if (this.hintBox) {
            this.hintBox.classList.add("hidden");
            while (this.hintBox.firstChild) {
                this.hintBox.removeChild(this.hintBox.firstChild);
            }
        }
        const firstInput = this.stepsList.querySelector<HTMLInputElement>("input");
        firstInput?.focus();
    }

    public requestHint(): void {
        if (this.state !== "solving") {return;}
        this.hintsUsed += 1;
        this.hintTimestamps.push(Date.now() - this.startTime);
        this.state = "hint";

        let hintTitle = "Hint";
        let hintContent = "";
        const graph = this.options.solutionGraph;

        if (graph && graph.steps && graph.steps.length > 0) {
            const stepIdx = Math.min(this.hintsUsed - 1, graph.steps.length - 1);
            const step = graph.steps[stepIdx];
            if (step.hints && step.hints.length > 0) {
                const hintObj = step.hints[(this.hintsUsed - 1) % step.hints.length];
                hintTitle = hintObj.title || `Hint ${this.hintsUsed}`;
                hintContent = hintObj.content;
            } else {
                hintTitle = `Hint ${this.hintsUsed}`;
                hintContent = step.description;
            }
        } else {
            hintTitle = "Hint";
            hintContent = "Identify key governing principles, write known values, and apply the required inverse relation.";
        }

        if (this.hintBox) {
            this.hintBox.classList.remove("hidden");
            while (this.hintBox.firstChild) {
                this.hintBox.removeChild(this.hintBox.firstChild);
            }

            const headerDiv = document.createElement("div");
            const iconSpan = document.createElement("span");
            iconSpan.textContent = "💡 ";
            const titleStrong = document.createElement("strong");
            titleStrong.textContent = `${hintTitle}: `;
            const bodySpan = document.createElement("span");
            bodySpan.textContent = hintContent;

            headerDiv.appendChild(iconSpan);
            headerDiv.appendChild(titleStrong);
            headerDiv.appendChild(bodySpan);

            const metaDiv = document.createElement("div");
            metaDiv.className = "proc-hint-meta";
            metaDiv.textContent = `(Hints used: ${this.hintsUsed})`;

            this.hintBox.appendChild(headerDiv);
            this.hintBox.appendChild(metaDiv);
            this.typesetMathJax(this.hintBox);
        }

        this.state = "solving";

        bridgeCommand(`procedural_hint:${JSON.stringify({
            instance_id: this.options.instanceId,
            hint_level: this.hintsUsed,
        })}`);
    }

    public selectOption(optId: string, optEl: HTMLElement): void {
        if (this.state !== "solving") {return;}
        this.selectedOptionId = optId;
        this.state = "submitting";

        const allOpts = this.container.querySelectorAll<HTMLElement>(".proc-option-item");
        allOpts.forEach((el) => {
            el.classList.remove("selected");
            el.setAttribute("aria-checked", "false");
            el.classList.add("disabled");
        });

        optEl.classList.add("selected");
        optEl.setAttribute("aria-checked", "true");

        // Evaluate ConceptCheck or StrategyDrill
        let isCorrect = false;
        let feedbackText = "";
        let expectedId = "";

        if (this.options.conceptCheck) {
            const cc = this.options.conceptCheck;
            expectedId = cc.expected_option_id;
            const chosen = cc.options.find((o) => o.id === optId);
            isCorrect = chosen ? chosen.is_correct : false;
            feedbackText = chosen?.feedback || (isCorrect ? "Correct concept understanding." : "Misconception detected.");
        } else if (this.options.strategyDrill) {
            const sd = this.options.strategyDrill;
            expectedId = sd.preferred_option_id;
            const chosen = sd.options.find((o) => o.id === optId);
            isCorrect = chosen ? chosen.is_optimal : false;
            feedbackText = chosen?.feedback || (isCorrect ? "Optimal strategy selected." : "Suboptimal method chosen.");
        }

        // Color options
        allOpts.forEach((el) => {
            const id = el.dataset.optId;
            if (id === expectedId) {
                el.classList.add("correct");
            } else if (id === optId && !isCorrect) {
                el.classList.add("incorrect");
            }
        });

        const evalResult = {
            isCorrect,
            reason: feedbackText,
            score: isCorrect ? 1.0 : 0.0,
        };

        let mode:
            | "quick"
            | "stepwise"
            | "concept_check"
            | "strategy_drill"
            | "worked_example"
            | "declarative_recall"
            | "prerequisite_review" = "quick";
        if (this.options.conceptCheck) {
            mode = "concept_check";
        } else if (this.options.strategyDrill) {
            mode = "strategy_drill";
        }

        this.finishAttempt(evalResult, { answer: optId, steps: [] }, mode);
    }

    public parseNumericValue(val: string | null | undefined): number | null {
        if (!val) {return null;}
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

    public computeSpeedQuadrant(isCorrect: boolean, timeTakenMs: number, targetTimeMs: number): {
        quadrant: "fluency_strength" | "speed_opportunity" | "strategy_trap" | "concept_setup";
        label: string;
        className: string;
    } {
        const isFast = timeTakenMs <= targetTimeMs;
        if (isCorrect && isFast) {
            return {
                quadrant: "fluency_strength",
                label: "⚡ Fluency Strength (Accurate & Fast)",
                className: "proc-speed-quadrant proc-speed-fast-correct",
            };
        } else if (isCorrect && !isFast) {
            return {
                quadrant: "speed_opportunity",
                label: "⏱ Speed Opportunity (Accurate but Slow)",
                className: "proc-speed-quadrant proc-speed-slow-correct",
            };
        } else if (!isCorrect && isFast) {
            return {
                quadrant: "strategy_trap",
                label: "⚠️ Check Strategy / Trap (Fast but Incorrect)",
                className: "proc-speed-quadrant proc-speed-fast-wrong",
            };
        } else {
            return {
                quadrant: "concept_setup",
                label: "💡 Review Concept / Setup (Slow & Incorrect)",
                className: "proc-speed-quadrant proc-speed-slow-wrong",
            };
        }
    }

    private handleQuickSubmit(): void {
        const answer = this.quickInput?.value.trim() || "";
        if (!answer || this.state !== "solving") {return;}

        this.state = "submitting";
        const evalResult = this.evaluateLocally(answer);
        this.finishAttempt(evalResult, { answer, steps: [] }, "quick");
    }

    private handleStepwiseSubmit(): void {
        if (this.state !== "solving" || !this.stepsList) {return;}
        const stepInputs = this.stepsList.querySelectorAll<HTMLInputElement>(".proc-step-input");
        const steps: string[] = [];

        stepInputs.forEach((input) => {
            const val = input.value.trim();
            if (val) {steps.push(val);}
        });

        const lastAnswer = steps.length > 0 ? steps[steps.length - 1] : "";
        this.state = "submitting";
        const evalResult = this.evaluateLocally(lastAnswer);
        this.finishAttempt(evalResult, { answer: lastAnswer, steps }, "stepwise");
    }

    private finishAttempt(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
        mode: "quick" | "stepwise" | "concept_check" | "strategy_drill" | "worked_example" | "declarative_recall" | "prerequisite_review" = "quick",
    ): void {
        if (this.hasSubmitted || this.state === "teardown") {return;}
        this.hasSubmitted = true;
        clearInterval(this.timerInterval);
        const timeTakenMs = Date.now() - this.startTime;

        if (!outcome.isCorrect) {
            this.state = "mistake_classification";
            this.showMistakeClassificationUI(outcome, data, mode, timeTakenMs);
            return;
        }

        this.finalizeAndShowFeedback(outcome, data, mode, timeTakenMs);
    }

    private showMistakeClassificationUI(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
        mode: "quick" | "stepwise" | "concept_check" | "strategy_drill" | "worked_example" | "declarative_recall" | "prerequisite_review",
        timeTakenMs: number
    ): void {
        this.quickContainer?.classList.add("hidden");
        this.stepwiseContainer?.classList.add("hidden");
        this.container.querySelector(".proc-mode-switch")?.classList.add("hidden");

        if (this.footerCenter) {
            this.footerCenter.innerHTML = `
                <button class="proc-mistake-btn" data-value="silly_mistake">Silly mistake</button>
                <button class="proc-mistake-btn" data-value="pattern_not_recognized">Pattern not recognized</button>
                <button class="proc-mistake-btn" data-value="formula_or_concept_misapplied">Formula or concept misapplied</button>
                <button class="proc-mistake-btn" data-value="concept_not_known">Concept not known</button>
            `;
            const buttons = this.footerCenter.querySelectorAll<HTMLButtonElement>(".proc-mistake-btn");
            buttons.forEach(btn => {
                this.addListener(btn, "click", () => {
                    buttons.forEach(b => b.classList.remove("selected"));
                    btn.classList.add("selected");
                    this.mistakeType = btn.dataset.value || null;
                    setTimeout(() => {
                        this.finalizeAndShowFeedback(outcome, data, mode, timeTakenMs);
                    }, 200);
                });
            });
        }
        
        this.resultPanel?.classList.remove("hidden");
        if (this.resultTitle) {this.resultTitle.textContent = "✗ Incorrect Answer";}
        
        const canonicalFormatted = this.options.correctAnswer?.formatted || this.options.correctAnswer?.value || "";
        if (this.resultFeedback) {
            this.resultFeedback.innerHTML = `
                <div><strong>You answered:</strong> ${data.answer}</div>
                <div><strong>Correct answer:</strong> ${canonicalFormatted}</div>
                <div style="margin-top: 8px;"><em>Please classify this mistake in the footer to continue.</em></div>
            `;
        }
        this.typesetMathJax(this.resultPanel);
    }

    private finalizeAndShowFeedback(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
        mode: "quick" | "stepwise" | "concept_check" | "strategy_drill" | "worked_example" | "declarative_recall" | "prerequisite_review",
        timeTakenMs: number
    ): void {
        this.state = "feedback";

        // Hide input containers, show result panel
        this.quickContainer?.classList.add("hidden");
        this.stepwiseContainer?.classList.add("hidden");
        this.container.querySelector(".proc-mode-switch")?.classList.add("hidden");
        this.resultPanel?.classList.remove("hidden");

        const quadrantInfo = this.computeSpeedQuadrant(outcome.isCorrect, timeTakenMs, this.options.targetTimeMs);

        // Clear actualTimeEl in the panel since we move it to footer
        if (this.actualTimeEl) {
            while (this.actualTimeEl.firstChild) {
                this.actualTimeEl.removeChild(this.actualTimeEl.firstChild);
            }
        }

        if (this.footerCenter) {
            // For correct answers, show performance. For incorrect, clear mistake buttons or show performance.
            if (outcome.isCorrect) {
                this.footerCenter.innerHTML = `
                    <div style="display: flex; align-items: center; gap: 8px;">
                        <strong>Time: ${(timeTakenMs / 1000).toFixed(1)}s</strong>
                        <div class="${quadrantInfo.className}">${quadrantInfo.label}</div>
                    </div>
                `;
            } else {
                // Mistake classification is already done, clear footer
                this.footerCenter.innerHTML = ``;
            }
        }

        if (this.resultPanel) {
            const canonicalFormatted = this.options.correctAnswer?.formatted || this.options.correctAnswer?.value || "";
            if (outcome.isCorrect) {
                this.resultPanel.className = "proc-result correct";
                if (this.resultTitle) {this.resultTitle.textContent = "✓ Correct Answer";}
                
                let msg = ``;
                if (this.hintsUsed > 0) {
                    msg += `[${this.hintsUsed} hint(s) used]<br>`;
                }
                msg += `<div><strong>You answered:</strong> ${data.answer}</div>`;
                if (this.resultFeedback) {this.resultFeedback.innerHTML = msg;}
            } else {
                this.resultPanel.className = "proc-result incorrect";
                if (this.resultTitle) {this.resultTitle.textContent = "✗ Incorrect Answer";}
                if (this.resultFeedback) {
                    let msg = `
                        <div><strong>You answered:</strong> ${data.answer}</div>
                        <div><strong>Correct answer:</strong> ${canonicalFormatted}</div>
                    `;
                    if (outcome.reason) {
                        msg += `<div style="margin-top: 8px;">${outcome.reason}</div>`;
                    }
                    this.resultFeedback.innerHTML = msg;
                }
            }
        }

        this.typesetMathJax(this.resultPanel);

        const attemptResult: ProceduralAttemptResult = {
            instanceId: this.options.instanceId,
            answer: data.answer,
            mode,
            steps: data.steps,
            hintsUsed: this.hintsUsed,
            timeTakenMs,
            isCorrect: outcome.isCorrect,
            score: outcome.score,
            selectedOptionId: this.selectedOptionId || undefined,
            speedQuadrant: quadrantInfo.quadrant,
        };

        if (this.options.onCompleted) {
            this.options.onCompleted(attemptResult);
        }

        // --- STUDYLAB TELEMETRY & PERFORMANCE PERSISTENCE ---
        let classification = "incorrect";
        let timeRatio = 1.0;
        
        if (outcome.isCorrect) {
            if (this.options.targetTimeMs && this.options.targetTimeMs > 0) {
                timeRatio = timeTakenMs / this.options.targetTimeMs;
                if (timeRatio <= 0.8) {
                    classification = "fast_correct";
                } else if (timeRatio <= 1.2) {
                    classification = "on_target_correct";
                } else {
                    classification = "slow_correct";
                }
            } else {
                classification = "on_target_correct"; // Fallback if no target time
            }
        }

        const proceduralPerformance = {
            classification,
            timeRatio: parseFloat(timeRatio.toFixed(2)),
            mistakeType: this.mistakeType || null,
            hintsUsed: this.hintsUsed,
        };

        // Determine procedural remediation need
        let remediationNeeded = false;
        let remediationReason = "none";

        if (this.mistakeType === "silly_mistake") {
            remediationNeeded = true;
            remediationReason = "silly_mistake";
        } else if (this.mistakeType === "pattern_not_recognized") {
            remediationNeeded = true;
            remediationReason = "pattern_not_recognized";
        } else if (this.mistakeType === "concept_not_known") {
            remediationNeeded = true;
            remediationReason = "concept_not_known";
        } else if (outcome.isCorrect && classification === "slow_correct") {
            remediationNeeded = true;
            remediationReason = "slow_correct";
        }

        const proceduralRemediation = {
            needed: remediationNeeded,
            reason: remediationReason,
            skillId: this.options.skillId || "",
            schemaId: this.options.schemaId || "",
            familyId: this.options.familyId || "",
            topicId: this.options.topicId || ""
        };

        const telemetry = {
            v: 1,
            actualTimeMs: timeTakenMs,
            targetTimeMs: this.options.targetTimeMs,
            isCorrect: outcome.isCorrect,
            hintsUsed: this.hintsUsed,
            mistakeType: this.mistakeType || undefined,
            mode: mode,
            proceduralPerformance, // Embedded in standard telemetry payload
            proceduralRemediation, // Procedural practice signal
        };

        if (globalThis.anki && typeof globalThis.anki.mutateNextCardStates === "function") {
            // We fire-and-forget this promise so it doesn't block the UI.
            // It modifies the card states in the backend prior to the user answering.
            globalThis.anki.mutateNextCardStates("studylab_telemetry", async (states: any, customData: any) => {
                for (const state of ["again", "hard", "good", "easy"]) {
                    if (customData[state]) {
                        customData[state].studylab = {
                            ...(customData[state].studylab || {}),
                            ...telemetry
                        };
                    }
                }
            }).catch((err: any) => {
                console.error("Failed to persist StudyLab telemetry", err);
            });
        }

        // Bridge notification for Python/Qt backend telemetry recording
        bridgeCommand(`procedural_attempt:${JSON.stringify(attemptResult)}`);

        // Focus next button
        this.nextBtn?.focus();
    }

    public handleTrySimilar(): void {
        if (this.state === "teardown") {return;}
        this.state = "worked_example";
        bridgeCommand(`procedural_try_similar:${JSON.stringify({
            instance_id: this.options.instanceId,
            family_id: this.options.familyId,
        })}`);
    }

    public handleDeclarativeRecallAction(): void {
        if (this.state === "teardown") {return;}
        const recall = this.options.declarativeRecall;
        bridgeCommand(`procedural_declarative_recall:${JSON.stringify({
            instance_id: this.options.instanceId,
            target_anki_card_id: recall?.target_anki_card_id ?? null,
            target_anki_tag: recall?.target_anki_tag ?? null,
        })}`);
    }

    public handlePracticePrerequisite(): void {
        if (this.state === "teardown") {return;}
        const prereq = this.options.prerequisiteReview;
        bridgeCommand(`procedural_practice_prerequisite:${JSON.stringify({
            instance_id: this.options.instanceId,
            target_skill_id: prereq?.primary_missing_prerequisite ?? null,
            executable_schema_id: prereq?.executable_schema_id ?? null,
        })}`);
    }

    private handleNext(): void {
        if (this.state === "teardown") {return;}
        this.state = "next";
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
        this.state = "teardown";
        this.hasSubmitted = true;
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

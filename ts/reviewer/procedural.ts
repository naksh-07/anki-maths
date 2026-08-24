// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

import { bridgeCommand } from "@tslib/bridgecommand";

import { MCQContainer, type MCQEvaluationResult, type MCQOption } from "./components/mcq_container";
import { MistakeFooter } from "./components/mistake_footer";
import {
    NumericalContainer,
    NumericalParser,
    type NumericalEvaluationResult,
    type NumericalParseResult,
    PhysicalDimension,
    type PhysicalUnit,
    UnitRegistry,
} from "./components/numerical_container";
import { StepwiseContainer, type StepwiseEvaluationResult } from "./components/stepwise_container";

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
    | "quick"
    | "stepwise"
    | "mcq"
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
    mode?: "practice" | "mock";
    mcqMode?: "practice" | "mock";
    onCompleted?: (result: ProceduralAttemptResult) => void;
    onOptionSelected?: (optId: string, isCorrect?: boolean) => void;
    onSelectionChanged?: (optId: string | null) => void;
}

export interface ProceduralAttemptResult {
    instanceId: string;
    answer: string;
    mode: LearningObjectKind;
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
    private mistakeFooter: MistakeFooter | null = null;
    private mcqContainer: MCQContainer | null = null;
    private numericalContainer: NumericalContainer | null = null;
    private stepwiseContainerComponent: StepwiseContainer | null = null;

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

    private lastAttemptIsCorrect = false;
    private lastAttemptIsFast = false;
    private pendingMistakeOutcome: {
        outcome: { isCorrect: boolean; reason?: string; score: number };
        data: { answer: string; steps: string[] };
        mode: LearningObjectKind;
        timeTakenMs: number;
    } | null = null;

    constructor(container: HTMLElement, options: ProceduralSetupOptions) {
        this.container = container;
        this.options = options;
        this.startTime = Date.now();
        this.state = "ready";
        this.bindElements();
        this.attachEventListeners();
        this.startTimer();
        this.state = "solving";
    }

    public getState(): ProceduralUIState {
        return this.state;
    }

    public getMCQContainer(): MCQContainer | null {
        return this.mcqContainer;
    }

    public getNumericalContainer(): NumericalContainer | null {
        return this.numericalContainer;
    }

    public getStepwiseContainer(): StepwiseContainer | null {
        return this.stepwiseContainerComponent;
    }

    public evaluateMockMCQ(): MCQEvaluationResult | null {
        return this.mcqContainer?.evaluate() || null;
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
        this.mistakePanel = this.container.querySelector("#proc-mistake-panel");

        this.mistakeFooter = new MistakeFooter({
            container: this.container,
            instanceId: this.options.instanceId,
            familyId: this.options.familyId,
            onSelect: (val) => this.selectMistakeCategory(val),
        });

        if (this.stepwiseContainer) {
            this.stepwiseContainerComponent = new StepwiseContainer(this.container, {
                instanceId: this.options.instanceId,
                familyId: this.options.familyId,
                targetTimeMs: this.options.targetTimeMs,
                solutionGraph: this.options.solutionGraph,
                correctAnswer: this.options.correctAnswer,
                onStepwiseCompleted: (evalResult) => {
                    this.onStepwiseCheckCompleted(evalResult);
                },
                onHintRequested: (lvl) => {
                    this.hintsUsed = lvl;
                    this.hintTimestamps.push(Date.now() - this.startTime);
                },
                typesetMathJax: (el) => this.typesetMathJax(el),
            });
        }

        const hasOptions = this.container.querySelectorAll(".proc-option-item").length > 0 || 
            this.options.objectType === "mcq" || 
            Boolean(this.options.conceptCheck) || 
            Boolean(this.options.strategyDrill);

        if (hasOptions) {
            this.mcqContainer = new MCQContainer(this.container, {
                mode: this.options.mode || this.options.mcqMode || "practice",
                objectType: this.options.objectType,
                correctAnswer: this.options.correctAnswer,
                conceptCheck: this.options.conceptCheck,
                strategyDrill: this.options.strategyDrill,
                onOptionSelected: (option, evalResult) => {
                    this.handleMCQOptionSelected(option, evalResult);
                },
                onSelectionChanged: (option) => {
                    this.selectedOptionId = option ? option.id : null;
                    if (this.options.onSelectionChanged) {
                        this.options.onSelectionChanged(this.selectedOptionId);
                    }
                },
                typesetMathJax: (el) => this.typesetMathJax(el),
            });
        } else {
            this.numericalContainer = new NumericalContainer(this.container, {
                inputElement: this.quickInput,
                submitButton: this.quickSubmitBtn,
                correctAnswer: this.options.correctAnswer,
                targetTimeMs: this.options.targetTimeMs,
            });
        }
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

    private attachEventListeners(): void {
        // Tab switching
        this.addListener(this.tabQuickBtn, "click", () => this.switchMode("quick"));
        this.addListener(this.tabStepwiseBtn, "click", () => this.switchMode("stepwise"));

        // Quick submit
        if (this.quickSubmitBtn) {
            this.addListener(this.quickSubmitBtn, "click", () => this.handleQuickSubmit());
        }
        
        this.addListener(this.quickInput, "keydown", (e: Event) => {
            const kbEvent = e as KeyboardEvent;
            if (kbEvent.key === "Enter") {
                kbEvent.preventDefault();
                this.handleQuickSubmit();
            }
        });

        // Stepwise controls
        this.addListener(this.addStepBtn, "click", () => this.addStepRow());
        this.addListener(this.resetBtn, "click", () => this.resetSteps());
        this.addListener(this.hintBtn, "click", () => this.requestHint());
        this.addListener(this.checkStepsBtn, "click", () => this.handleStepwiseSubmit());

        // Structured option items (MCQ, ConceptCheck, StrategyDrill)
        const optionItems = this.container.querySelectorAll<HTMLElement>(".proc-option-item");
        optionItems.forEach((optEl) => {
            const optId = optEl.dataset.optId || "";
            this.addListener(optEl, "click", () => this.selectOption(optId, optEl));
            this.addListener(optEl, "keydown", (e: Event) => {
                const kbEvent = e as KeyboardEvent;
                if (kbEvent.key === "Enter" || kbEvent.key === " ") {
                    kbEvent.preventDefault();
                    this.selectOption(optId, optEl);
                }
            });
        });

        // Mistake classification buttons & cards
        const mistakeBtns = this.container.querySelectorAll<HTMLButtonElement>(".proc-mistake-card, .proc-mistake-btn");
        mistakeBtns.forEach((btn) => {
            const val = btn.dataset.value || "";
            this.addListener(btn, "click", () => {
                this.selectMistakeCategory(val);
            });
        });

        // MutationObserver to safely destroy when container is removed from DOM (e.g. navigation to standard card)
        if (typeof MutationObserver !== "undefined") {
            const observer = new MutationObserver(() => {
                if (!document.body.contains(this.container)) {
                    this.destroy();
                }
            });
            observer.observe(document.body, { childList: true, subtree: true });
            this.disposables.push(() => observer.disconnect());
        }

        // Global window keyboard handler for state-aware shortcuts and leak protection
        this.addListener(window, "keydown", (e: Event) => {
            if (!this.container.isConnected || this.state === "teardown") {
                this.destroy();
                return;
            }

            const kbEvent = e as KeyboardEvent;
            const targetTag = (kbEvent.target as HTMLElement)?.tagName?.toLowerCase();
            const isInputField = targetTag === "input" || targetTag === "textarea";

            if (this.state === "solving") {
                if (this.mcqContainer) {
                    const handled = this.mcqContainer.handleGlobalKeyDown(kbEvent);
                    if (handled) {
                        return;
                    }
                }

                if (isInputField) {
                    if (kbEvent.key === "Enter") {
                        kbEvent.preventDefault();
                        this.handleQuickSubmit();
                    }
                    return;
                }

                // If Space is pressed outside text input during solving, submit or reveal
                if (kbEvent.key === " " || kbEvent.code === "Space") {
                    kbEvent.preventDefault();
                    kbEvent.stopPropagation();
                    this.handleQuickSubmit();
                    return;
                }

                // Hotkeys 1-4 and A-D for options fallback
                const keyUpper = kbEvent.key.toUpperCase();
                let optIndex = -1;
                const keyNum = parseInt(kbEvent.key, 10);
                if (!isNaN(keyNum) && keyNum >= 1 && keyNum <= optionItems.length) {
                    optIndex = keyNum - 1;
                } else if (keyUpper >= "A" && keyUpper <= "D") {
                    optIndex = keyUpper.charCodeAt(0) - 65;
                }

                if (optIndex >= 0 && optIndex < optionItems.length) {
                    kbEvent.preventDefault();
                    const targetOpt = optionItems[optIndex];
                    if (targetOpt) {
                        const optId = targetOpt.dataset.optId || "";
                        this.selectOption(optId, targetOpt);
                    }
                }
            } else if (this.state === "mistake_classification") {
                if (this.mistakeFooter?.handleKeydown(kbEvent)) {
                    return;
                }

                // Numbers 1-4 for mistake categories fallback
                const keyNum = parseInt(kbEvent.key, 10);
                if (!isNaN(keyNum) && keyNum >= 1 && keyNum <= 4) {
                    kbEvent.preventDefault();
                    kbEvent.stopPropagation();
                    const targetBtn = this.container.querySelector<HTMLButtonElement>(
                        `.proc-mistake-btn[data-key="${keyNum}"], .proc-mistake-card[data-key="${keyNum}"]`
                    );
                    if (targetBtn) {
                        const val = targetBtn.dataset.value || "";
                        this.selectMistakeCategory(val);
                    }
                    return;
                }

                // Space or Enter in mistake classification MUST NOT bypass reflection
                if (kbEvent.key === " " || kbEvent.code === "Space" || kbEvent.key === "Enter") {
                    const activeEl = document.activeElement as HTMLElement;
                    if (activeEl && (activeEl.classList.contains("proc-mistake-btn") || activeEl.classList.contains("proc-mistake-card"))) {
                        const val = activeEl.dataset.value || "";
                        if (val) {
                            kbEvent.preventDefault();
                            kbEvent.stopPropagation();
                            this.selectMistakeCategory(val);
                            return;
                        }
                    }
                    kbEvent.preventDefault();
                    kbEvent.stopPropagation();
                    return;
                }
            } else if (this.state === "feedback") {
                if (kbEvent.key === "Enter" || kbEvent.key === " " || kbEvent.code === "Space") {
                    kbEvent.preventDefault();
                    kbEvent.stopPropagation();
                    this.handleNext();
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

    public selectOption(optId: string, optEl?: HTMLElement): void {
        if (this.state !== "solving" && this.state !== "ready") {return;}
        if (this.mcqContainer) {
            this.mcqContainer.selectOptionById(optId);
            return;
        }

        this.selectedOptionId = optId;
        this.state = "submitting";

        const allOpts = this.container.querySelectorAll<HTMLElement>(".proc-option-item");
        allOpts.forEach((el) => {
            el.classList.remove("selected");
            el.setAttribute("aria-checked", "false");
            el.classList.add("disabled");
        });

        if (optEl) {
            optEl.classList.add("selected");
            optEl.setAttribute("aria-checked", "true");
        }

        let isCorrect = false;
        let feedbackText = "";
        let expectedId = "";
        let mode: LearningObjectKind = "mcq";

        if (this.options.conceptCheck) {
            mode = "concept_check";
            const cc = this.options.conceptCheck;
            expectedId = cc.expected_option_id;
            const chosen = cc.options.find((o) => o.id === optId);
            isCorrect = chosen ? chosen.is_correct : false;
            feedbackText = chosen?.feedback || (isCorrect ? "Correct concept understanding." : "Misconception detected.");
        } else if (this.options.strategyDrill) {
            mode = "strategy_drill";
            const sd = this.options.strategyDrill;
            expectedId = sd.preferred_option_id;
            const chosen = sd.options.find((o) => o.id === optId);
            isCorrect = chosen ? chosen.is_optimal : false;
            feedbackText = chosen?.feedback || (isCorrect ? "Optimal strategy selected." : "Suboptimal method chosen.");
        } else {
            // Standard MCQ
            mode = "mcq";
            const optIdx = optEl?.dataset.optIdx || "";
            const letter = String.fromCharCode(65 + (parseInt(optIdx, 10) || 0));
            const correctOpt = String(
                this.options.correctAnswer?.correct_option || 
                this.options.correctAnswer?.formatted || 
                this.options.correctAnswer?.answer || 
                ""
            ).trim();
            const labelText = optEl?.querySelector(".proc-option-label")?.textContent?.trim() || "";

            isCorrect = 
                (optId.trim().toLowerCase() === correctOpt.toLowerCase() && correctOpt.length > 0) ||
                letter.toLowerCase() === correctOpt.toLowerCase() ||
                optIdx === correctOpt ||
                String((parseInt(optIdx, 10) || 0) + 1) === correctOpt ||
                (labelText.toLowerCase() === correctOpt.toLowerCase() && correctOpt.length > 0);

            feedbackText = isCorrect ? "Correct answer selected." : "Incorrect option selected.";

            allOpts.forEach((el) => {
                const elId = el.dataset.optId || "";
                const elIdx = el.dataset.optIdx || "";
                const elLetter = String.fromCharCode(65 + (parseInt(elIdx, 10) || 0));
                const elLabel = el.querySelector(".proc-option-label")?.textContent?.trim() || "";
                if (
                    (elId.trim().toLowerCase() === correctOpt.toLowerCase() && correctOpt.length > 0) ||
                    elLetter.toLowerCase() === correctOpt.toLowerCase() ||
                    elIdx === correctOpt ||
                    String((parseInt(elIdx, 10) || 0) + 1) === correctOpt ||
                    (elLabel.toLowerCase() === correctOpt.toLowerCase() && correctOpt.length > 0)
                ) {
                    expectedId = elId;
                }
            });
        }

        // Color options
        allOpts.forEach((el) => {
            const id = el.dataset.optId;
            if (id === expectedId || (expectedId && el.querySelector(".proc-option-label")?.textContent?.trim() === expectedId)) {
                el.classList.add("correct");
            } else if (el === optEl && !isCorrect) {
                el.classList.add("incorrect");
            }
        });

        const evalResult = {
            isCorrect,
            reason: feedbackText,
            score: isCorrect ? 1.0 : 0.0,
        };

        const labelText = optEl?.querySelector(".proc-option-label")?.textContent?.trim();
        this.finishAttempt(evalResult, { answer: labelText || optId, steps: [] }, mode);
    }

    private handleMCQOptionSelected(option: MCQOption, evalResult: MCQEvaluationResult): void {
        if (this.state !== "solving" && this.state !== "ready") {return;}
        this.selectedOptionId = option.id;

        let mode: LearningObjectKind = "mcq";
        if (this.options.conceptCheck) {
            mode = "concept_check";
        } else if (this.options.strategyDrill) {
            mode = "strategy_drill";
        }

        if (this.options.onOptionSelected) {
            this.options.onOptionSelected(option.id, evalResult.isCorrect);
        }

        this.finishAttempt(
            {
                isCorrect: evalResult.isCorrect,
                reason: evalResult.reason,
                score: evalResult.score,
            },
            {
                answer: option.label || option.id,
                steps: [],
            },
            mode,
        );
    }

    public parseNumericValue(val: string | null | undefined): number | null {
        return NumericalParser.parseScalar(val);
    }

    public evaluateLocally(userText: string): { isCorrect: boolean; reason?: string; score: number } {
        if (this.numericalContainer) {
            const evalResult = this.numericalContainer.evaluate(userText);
            const expVal = this.numericalContainer.getExpectedValue();
            if (expVal !== null) {
                return {
                    isCorrect: evalResult.isCorrect,
                    reason: evalResult.isCorrect
                        ? undefined
                        : (evalResult.reason || evalResult.diagnosticMessage || `Expected ${expVal}, but received ${userText}`),
                    score: evalResult.score,
                };
            }
        }

        let expectedVal: number | undefined;
        if (this.options.correctAnswer?.value !== undefined) {
            expectedVal = this.options.correctAnswer.value;
        } else if (typeof this.options.correctAnswer?.answer === "number") {
            expectedVal = this.options.correctAnswer.answer;
        }
        const userNum = this.parseNumericValue(userText);

        if (expectedVal !== undefined && typeof expectedVal === "number") {
            if (userNum === null) {
                return { isCorrect: false, reason: "Please enter a valid numeric value or fraction.", score: 0.0 };
            }
            const diff = Math.abs(userNum - expectedVal);
            const tolerance = this.options.correctAnswer?.tolerance !== undefined 
                ? Number(this.options.correctAnswer.tolerance) 
                : Math.max(0.01, Math.abs(expectedVal) * 0.01);
            const isCorrect = diff <= tolerance;
            return {
                isCorrect,
                reason: isCorrect ? undefined : `Expected ${expectedVal}, but received ${userText}`,
                score: isCorrect ? 1.0 : 0.0,
            };
        }

        // Fallback string matching if not numeric
        const canonicalFormatted = String(
            this.options.correctAnswer?.formatted || 
            this.options.correctAnswer?.correct_option || 
            this.options.correctAnswer?.answer || 
            ""
        ).trim().toLowerCase();
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
        if (this.state !== "solving") {return;}
        this.state = "submitting";

        if (this.stepwiseContainerComponent) {
            const stepEval = this.stepwiseContainerComponent.handleCheckSolution();
            this.onStepwiseCheckCompleted(stepEval);
        } else {
            const stepInputs = this.stepsList?.querySelectorAll<HTMLInputElement>(".proc-step-input");
            const steps: string[] = [];
            stepInputs?.forEach((input) => {
                const val = input.value.trim();
                if (val) {steps.push(val);}
            });
            const lastAnswer = steps.length > 0 ? steps[steps.length - 1] : "";
            const evalResult = this.evaluateLocally(lastAnswer);
            this.finishAttempt(evalResult, { answer: lastAnswer, steps }, "stepwise");
        }
    }

    private onStepwiseCheckCompleted(evalResult: StepwiseEvaluationResult): void {
        const lastAnswer = evalResult.finalAnswer || (evalResult.stepEvaluations.length > 0 ? evalResult.stepEvaluations[evalResult.stepEvaluations.length - 1].submittedText : "");
        const steps = evalResult.stepEvaluations.map((e) => e.submittedText);

        this.finishAttempt(
            {
                isCorrect: evalResult.isCorrect,
                reason: evalResult.overallFeedback,
                score: evalResult.score,
            },
            { answer: lastAnswer, steps },
            "stepwise",
        );
    }

    private finishAttempt(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
        mode: LearningObjectKind = "quick",
    ): void {
        if (this.hasSubmitted || this.state === "teardown") {return;}
        this.hasSubmitted = true;
        clearInterval(this.timerInterval);
        const timeTakenMs = Date.now() - this.startTime;

        if (!outcome.isCorrect) {
            this.showMistakeClassificationUI(outcome, data, mode, timeTakenMs);
            return;
        }

        this.finalizeAndShowFeedback(outcome, data, mode, timeTakenMs);
    }

    private showMistakeClassificationUI(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
        mode: LearningObjectKind,
        timeTakenMs: number
    ): void {
        this.state = "mistake_classification";
        this.pendingMistakeOutcome = { outcome, data, mode, timeTakenMs };

        this.quickContainer?.classList.add("hidden");
        this.stepwiseContainer?.classList.add("hidden");
        this.container.querySelector(".proc-mode-switch")?.classList.add("hidden");
        this.nextBtn?.classList.add("hidden");

        if (this.mistakeFooter) {
            this.mistakeFooter.show((val) => {
                this.selectMistakeCategory(val);
            });
        } else if (this.mistakePanel) {
            this.mistakePanel.classList.remove("hidden");
            this.mistakePanel.tabIndex = -1;
            this.mistakePanel.focus();
        }

        this.resultPanel?.classList.remove("hidden");
        if (this.resultTitle) {
            this.resultTitle.textContent = "✗ Incorrect Answer";
        }

        const canonicalFormatted = this.options.correctAnswer?.formatted || 
            this.options.correctAnswer?.correct_option || 
            this.options.correctAnswer?.value || 
            this.options.correctAnswer?.answer || "";

        if (this.resultFeedback) {
            this.resultFeedback.innerHTML = `
                <div><strong>You answered:</strong> ${escapeHtml(data.answer)}</div>
                <div><strong>Correct answer:</strong> ${escapeHtml(canonicalFormatted)}</div>
                <div style="margin-top: 8px; color: var(--text-muted, #6b7280);"><em>Please classify this error to optimize your spaced repetition schedule.</em></div>
            `;
        }
        this.typesetMathJax(this.resultPanel);
    }

    public selectMistakeCategory(value: string): void {
        if (this.state !== "mistake_classification" || !this.pendingMistakeOutcome) {
            return;
        }
        this.mistakeType = value;
        bridgeCommand(`procedural_mistake:${JSON.stringify({
            instance_id: this.options.instanceId,
            family_id: this.options.familyId,
            mistake_type: value,
        })}`);

        const btns = this.container.querySelectorAll<HTMLButtonElement>(".proc-mistake-btn, .proc-mistake-card");
        btns.forEach((c) => {
            if (c.dataset.value === value) {
                c.classList.add("selected");
            } else {
                c.classList.remove("selected");
            }
        });

        const pending = this.pendingMistakeOutcome;
        this.pendingMistakeOutcome = null;

        setTimeout(() => {
            this.finalizeAndShowFeedback(pending.outcome, pending.data, pending.mode, pending.timeTakenMs);
        }, 150);
    }

    private finalizeAndShowFeedback(
        outcome: { isCorrect: boolean; reason?: string; score: number },
        data: { answer: string; steps: string[] },
        mode: LearningObjectKind,
        timeTakenMs: number
    ): void {
        this.state = "feedback";
        this.lastAttemptIsCorrect = outcome.isCorrect;

        // Hide input containers, show result panel and next button
        this.quickContainer?.classList.add("hidden");
        this.stepwiseContainer?.classList.add("hidden");
        this.container.querySelector(".proc-mode-switch")?.classList.add("hidden");
        this.resultPanel?.classList.remove("hidden");
        this.nextBtn?.classList.remove("hidden");

        const quadrantInfo = this.computeSpeedQuadrant(outcome.isCorrect, timeTakenMs, this.options.targetTimeMs);
        const isFast = timeTakenMs <= this.options.targetTimeMs;
        this.lastAttemptIsFast = isFast;

        if (this.actualTimeEl) {
            this.actualTimeEl.innerHTML = `
                <div style="display: flex; align-items: center; gap: 8px;">
                    <strong>Time: ${(timeTakenMs / 1000).toFixed(1)}s</strong>
                    <div class="${quadrantInfo.className}">${quadrantInfo.label}</div>
                </div>
            `;
        }

        if (this.resultPanel) {
            const canonicalFormatted = this.options.correctAnswer?.formatted || 
                this.options.correctAnswer?.correct_option || 
                this.options.correctAnswer?.value || 
                this.options.correctAnswer?.answer || "";

            if (outcome.isCorrect) {
                this.resultPanel.className = "proc-result correct";
                if (this.resultTitle) {this.resultTitle.textContent = "✓ Correct Answer";}
                
                let msg = ``;
                if (this.hintsUsed > 0) {
                    msg += `[${this.hintsUsed} hint(s) used]<br>`;
                }
                msg += `<div><strong>You answered:</strong> ${escapeHtml(data.answer)}</div>`;
                if (this.resultFeedback) {this.resultFeedback.innerHTML = msg;}
            } else {
                this.resultPanel.className = "proc-result incorrect";
                if (this.resultTitle) {this.resultTitle.textContent = "✗ Incorrect Answer";}
                if (this.resultFeedback) {
                    let msg = `
                        <div><strong>You answered:</strong> ${escapeHtml(data.answer)}</div>
                        <div><strong>Correct answer:</strong> ${escapeHtml(canonicalFormatted)}</div>
                    `;
                    if (outcome.reason) {
                        msg += `<div style="margin-top: 8px;">${escapeHtml(outcome.reason)}</div>`;
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
                classification = "on_target_correct";
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
        } else if (this.mistakeType === "formula_or_concept_misapplied") {
            remediationNeeded = true;
            remediationReason = "formula_or_concept_misapplied";
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
            proceduralPerformance,
            proceduralRemediation,
            attemptResult,
        };

        if (globalThis.anki && typeof globalThis.anki.mutateNextCardStates === "function") {
            try {
                const res = globalThis.anki.mutateNextCardStates((globalThis.anki as any)._state_mutation_key, async (states: any, customData: any) => {
                    for (const state of ["again", "hard", "good", "easy"]) {
                        if (customData[state]) {
                            customData[state].studylab = {
                                ...(customData[state].studylab || {}),
                                ...telemetry
                            };
                        }
                    }
                });
                if (res && typeof res.catch === "function") {
                    res.catch((err: any) => {
                        console.error("Failed to persist StudyLab telemetry", err);
                    });
                }
            } catch (err) {
                console.error("Error mutating next card states", err);
            }
        }

        // Bridge notification for Python/Qt backend telemetry recording
        bridgeCommand(`procedural_attempt:${JSON.stringify(attemptResult)}`);

        // Synchronize with native Anki reviewer answer state to show footer ease buttons
        bridgeCommand("ans");

        // Focus next button if present
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
        let ease = 1;
        if (this.lastAttemptIsCorrect) {
            ease = this.lastAttemptIsFast ? 4 : 3;
        }
        bridgeCommand(`procedural_answer:${ease}`);
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
        if (this.mcqContainer) {
            this.mcqContainer.destroy();
            this.mcqContainer = null;
        }
        if (this.numericalContainer) {
            this.numericalContainer.destroy();
            this.numericalContainer = null;
        }
        if (this.mistakeFooter) {
            this.mistakeFooter.destroy();
            this.mistakeFooter = null;
        }
        if (this.stepwiseContainerComponent) {
            this.stepwiseContainerComponent.destroy();
            this.stepwiseContainerComponent = null;
        }
        for (const dispose of this.disposables) {
            try {
                dispose();
            } catch {
                /* non-fatal */
            }
        }
        this.disposables = [];
        if ((globalThis as any).__activeProceduralReviewer === this) {
            (globalThis as any).__activeProceduralReviewer = null;
        }
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
        // Safely destroy any active reviewer instance globally or on this container before creating a new one
        const active = (globalThis as any).__activeProceduralReviewer;
        if (active && typeof active.destroy === "function") {
            active.destroy();
        }
        const prev = (container as any).__proceduralReviewer;
        if (prev && typeof prev.destroy === "function" && prev !== active) {
            prev.destroy();
        }
        const reviewer = new ProceduralReviewer(container as HTMLElement, options);
        (container as any).__proceduralReviewer = reviewer;
        (globalThis as any).__activeProceduralReviewer = reviewer;
        return reviewer;
    },
    destroyActive: (): void => {
        const active = (globalThis as any).__activeProceduralReviewer;
        if (active && typeof active.destroy === "function") {
            active.destroy();
        }
    },
    ProceduralReviewer,
    MistakeFooter,
    escapeHtml,
};

function escapeHtml(str: unknown): string {
    return String(str ?? "")
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

import { bridgeCommand } from "@tslib/bridgecommand";

import type { ProceduralSolutionGraph, ProceduralSolutionStep } from "../procedural";

declare const MathJax: any;

export type StepValidationStatus =
    | "valid"
    | "invalid"
    | "partially_valid"
    | "unnecessary_but_valid"
    | "unresolved";

export interface StepEvaluationItem {
    stepId: string;
    stepIndex: number;
    status: StepValidationStatus;
    submittedText: string;
    expectedExpression: string;
    parsedValue: number | null;
    errorType?: string;
    feedback: string;
    isDownstreamConsistent: boolean;
}

export interface StepwiseEvaluationResult {
    isCorrect: boolean;
    score: number;
    firstErrorStep: number | null;
    firstErrorType?: string;
    confidence: "deterministic" | "strongly_inferred" | "uncertain";
    stepsCompleted: number;
    stepsCorrect: number;
    stepEvaluations: StepEvaluationItem[];
    overallFeedback: string;
    remediationRecommendation?: string;
    finalAnswer: string;
}

export interface StepwiseContainerOptions {
    container?: HTMLElement;
    instanceId: string;
    familyId: string;
    targetTimeMs: number;
    solutionGraph?: ProceduralSolutionGraph | null;
    correctAnswer?: Record<string, any>;
    onStepwiseCompleted?: (result: StepwiseEvaluationResult) => void;
    onHintRequested?: (level: number) => void;
    typesetMathJax?: (el: HTMLElement) => void;
}

/**
 * StepwiseContainer: Canonical multi-step procedural reasoning component.
 *
 * Implements:
 * - Dynamic step row management (initial populate from solution graph, add step, reset).
 * - Multi-step semantic evaluation directly matching Rust `StepValidator` rules.
 * - Downstream consistency tracking (PartiallyValid status for derived steps).
 * - Precise taxonomic error diagnosis (Reasoning, Math, Physics, Chemistry).
 * - Real-time step status badges and inline diagnostic feedback.
 * - Progressive 3-tier hint disclosure (Principle, Operation, Intermediate Relation).
 * - Bridge command telemetry dispatch (`procedural_validate_steps`).
 */
export class StepwiseContainer {
    private container: HTMLElement;
    private options: StepwiseContainerOptions;
    private stepwiseContainer: HTMLElement | null = null;
    private stepsList: HTMLElement | null = null;
    private addStepBtn: HTMLButtonElement | null = null;
    private hintBtn: HTMLButtonElement | null = null;
    private resetBtn: HTMLButtonElement | null = null;
    private checkStepsBtn: HTMLButtonElement | null = null;
    private hintBox: HTMLElement | null = null;
    private hintsUsed = 0;
    private hintTimestamps: number[] = [];
    private startTime: number = Date.now();
    private disposables: Array<() => void> = [];
    private lastEvaluation: StepwiseEvaluationResult | null = null;

    constructor(container: HTMLElement, options: StepwiseContainerOptions) {
        this.container = container;
        this.options = options;
        this.startTime = Date.now();
        this.init();
    }

    public getHintsUsed(): number {
        return this.hintsUsed;
    }

    public getHintTimestamps(): number[] {
        return [...this.hintTimestamps];
    }

    public getLastEvaluation(): StepwiseEvaluationResult | null {
        return this.lastEvaluation;
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
        this.stepwiseContainer = this.container.querySelector<HTMLElement>("#proc-stepwise-container");
        this.stepsList = this.container.querySelector<HTMLElement>("#proc-steps-list");
        this.addStepBtn = this.container.querySelector<HTMLButtonElement>("#proc-add-step-btn");
        this.hintBtn = this.container.querySelector<HTMLButtonElement>("#proc-hint-btn");
        this.resetBtn = this.container.querySelector<HTMLButtonElement>("#proc-reset-steps-btn");
        this.checkStepsBtn = this.container.querySelector<HTMLButtonElement>("#proc-check-steps-btn");
        this.hintBox = this.container.querySelector<HTMLElement>("#proc-hint-container");

        this.populateInitialSteps();
        this.attachEventListeners();
    }

    /**
     * Populate initial step rows from solution graph if available.
     */
    public populateInitialSteps(): void {
        if (!this.stepsList) {return;}

        const graph = this.options.solutionGraph;
        if (graph && graph.steps && graph.steps.length > 0) {
            // Check if stepsList is already populated with custom rows
            const existingRows = this.stepsList.querySelectorAll(".proc-step-row");
            if (existingRows.length === 0 || (existingRows.length === 1 && !existingRows[0].querySelector(".proc-step-desc"))) {
                this.stepsList.innerHTML = "";
                graph.steps.forEach((step, idx) => {
                    const row = document.createElement("div");
                    row.className = "proc-step-row";
                    row.dataset.stepIdx = String(idx);

                    const desc = step.description ? `<div class="proc-step-desc"><strong>Step ${idx + 1}:</strong> ${this.escapeHtml(step.description)}</div>` : "";

                    row.innerHTML = `
                        ${desc}
                        <div class="proc-step-input-wrapper">
                            <input type="text" class="proc-input proc-step-input" placeholder="Transform equation or compute step value..." autocomplete="off" />
                            <span class="proc-step-badge hidden" aria-live="polite"></span>
                        </div>
                        <div class="proc-step-feedback hidden"></div>
                    `;
                    this.stepsList?.appendChild(row);
                });
                this.typesetMathJax(this.stepsList);
            }
        }
    }

    private attachEventListeners(): void {
        this.addListener(this.addStepBtn, "click", () => this.addStepRow());
        this.addListener(this.resetBtn, "click", () => this.resetSteps());
        this.addListener(this.hintBtn, "click", () => this.requestHint());
        this.addListener(this.checkStepsBtn, "click", () => this.handleCheckSolution());

        if (this.stepsList) {
            this.addListener(this.stepsList, "keydown", (e: Event) => {
                const kb = e as KeyboardEvent;
                if (kb.key === "Enter" && !kb.shiftKey) {
                    const target = kb.target as HTMLElement;
                    if (target.classList.contains("proc-step-input")) {
                        kb.preventDefault();
                        const allInputs = Array.from(this.stepsList?.querySelectorAll<HTMLInputElement>(".proc-step-input") || []);
                        const currIdx = allInputs.indexOf(target as HTMLInputElement);
                        if (currIdx === allInputs.length - 1) {
                            this.handleCheckSolution();
                        } else if (currIdx >= 0 && currIdx < allInputs.length - 1) {
                            allInputs[currIdx + 1].focus();
                        }
                    }
                }
            });
        }
    }

    public addStepRow(): void {
        if (!this.stepsList) {return;}
        const currentCount = this.stepsList.querySelectorAll(".proc-step-row").length;
        const stepNum = currentCount + 1;

        const row = document.createElement("div");
        row.className = "proc-step-row";
        row.dataset.stepIdx = String(currentCount);
        row.innerHTML = `
            <div class="proc-step-desc"><strong>Step ${stepNum}:</strong> Intermediate Step</div>
            <div class="proc-step-input-wrapper">
                <input type="text" class="proc-input proc-step-input" placeholder="Write step ${stepNum} transformation or deduction..." autocomplete="off" />
                <span class="proc-step-badge hidden" aria-live="polite"></span>
            </div>
            <div class="proc-step-feedback hidden"></div>
        `;

        this.stepsList.appendChild(row);
        const input = row.querySelector<HTMLInputElement>("input");
        input?.focus();
        this.typesetMathJax(row);
    }

    public resetSteps(): void {
        if (!this.stepsList) {return;}
        this.stepsList.innerHTML = "";
        this.populateInitialSteps();

        if (this.stepsList.children.length === 0) {
            const row = document.createElement("div");
            row.className = "proc-step-row";
            row.dataset.stepIdx = "0";
            row.innerHTML = `
                <span class="proc-step-label">Step 1</span>
                <div class="proc-step-input-wrapper">
                    <input type="text" class="proc-input proc-step-input" placeholder="Write step 1 transformation or equation..." autocomplete="off" />
                    <span class="proc-step-badge hidden"></span>
                </div>
                <div class="proc-step-feedback hidden"></div>
            `;
            this.stepsList.appendChild(row);
        }

        if (this.hintBox) {
            this.hintBox.classList.add("hidden");
            this.hintBox.innerHTML = "";
        }

        const firstInput = this.stepsList.querySelector<HTMLInputElement>("input");
        firstInput?.focus();
    }

    public requestHint(): void {
        this.hintsUsed += 1;
        this.hintTimestamps.push(Date.now() - this.startTime);

        let hintTitle = "Hint";
        let hintContent = "";
        const graph = this.options.solutionGraph;

        if (graph && graph.steps && graph.steps.length > 0) {
            const stepIdx = Math.min(this.hintsUsed - 1, graph.steps.length - 1);
            const step = graph.steps[stepIdx];
            if (step.hints && step.hints.length > 0) {
                const hintObj = step.hints[(this.hintsUsed - 1) % step.hints.length];
                hintTitle = hintObj.title || `Hint (Level ${hintObj.level || this.hintsUsed})`;
                hintContent = hintObj.content;
            } else {
                hintTitle = `Hint ${this.hintsUsed}`;
                hintContent = step.description || "Apply governing constraint or formula.";
            }
        } else {
            hintTitle = "Hint";
            hintContent = "Identify key governing principles, write known values, and apply required constraint propagation.";
        }

        if (this.hintBox) {
            this.hintBox.classList.remove("hidden");
            this.hintBox.innerHTML = `
                <div>
                    <span>💡 </span>
                    <strong>${this.escapeHtml(hintTitle)}: </strong>
                    <span>${this.escapeHtml(hintContent)}</span>
                </div>
                <div class="proc-hint-meta">(Hints used: ${this.hintsUsed})</div>
            `;
            this.typesetMathJax(this.hintBox);
        }

        if (this.options.onHintRequested) {
            this.options.onHintRequested(this.hintsUsed);
        }

        bridgeCommand(`procedural_hint:${JSON.stringify({
            instance_id: this.options.instanceId,
            hint_level: this.hintsUsed,
        })}`);
    }

    public getSubmittedStepStrings(): string[] {
        if (!this.stepsList) {return [];}
        const inputs = this.stepsList.querySelectorAll<HTMLInputElement>(".proc-step-input");
        const steps: string[] = [];
        inputs.forEach((input) => {
            const val = input.value.trim();
            if (val) {
                steps.push(val);
            }
        });
        return steps;
    }

    public handleCheckSolution(): StepwiseEvaluationResult {
        const steps = this.getSubmittedStepStrings();
        const lastAnswer = steps.length > 0 ? steps[steps.length - 1] : "";
        const evalResult = this.evaluateSubmission(steps, lastAnswer);
        this.lastEvaluation = evalResult;

        this.renderStepEvaluationBadges(evalResult);

        // Send bridge command telemetry
        bridgeCommand(`procedural_validate_steps:${JSON.stringify({
            instance_id: this.options.instanceId,
            family_id: this.options.familyId,
            steps,
            final_answer: lastAnswer,
            total_time_ms: Date.now() - this.startTime,
            is_correct: evalResult.isCorrect,
            score: evalResult.score,
            first_error_step: evalResult.firstErrorStep,
            first_error_type: evalResult.firstErrorType,
        })}`);

        if (this.options.onStepwiseCompleted) {
            this.options.onStepwiseCompleted(evalResult);
        }

        return evalResult;
    }

    /**
     * Render visual status badges and feedback on each step row in the DOM.
     */
    public renderStepEvaluationBadges(evalResult: StepwiseEvaluationResult): void {
        if (!this.stepsList) {return;}
        const rows = this.stepsList.querySelectorAll<HTMLElement>(".proc-step-row");

        rows.forEach((row, idx) => {
            const input = row.querySelector<HTMLInputElement>(".proc-step-input");
            const badge = row.querySelector<HTMLElement>(".proc-step-badge");
            const feedbackEl = row.querySelector<HTMLElement>(".proc-step-feedback");

            const evalItem = evalResult.stepEvaluations[idx];
            if (!evalItem || !input) {return;}

            input.classList.remove("valid", "invalid", "partial");
            if (badge) {
                badge.className = "proc-step-badge";
                badge.classList.remove("hidden");
            }
            if (feedbackEl) {
                feedbackEl.className = "proc-step-feedback";
                feedbackEl.classList.remove("hidden");
            }

            switch (evalItem.status) {
                case "valid":
                    input.classList.add("valid");
                    if (badge) {
                        badge.classList.add("valid");
                        badge.textContent = "✓ Valid";
                    }
                    if (feedbackEl) {
                        feedbackEl.textContent = "✓ Correct step";
                    }
                    break;
                case "partially_valid":
                    input.classList.add("partial");
                    if (badge) {
                        badge.classList.add("partial");
                        badge.textContent = "~ Downstream Consistent";
                    }
                    if (feedbackEl) {
                        feedbackEl.textContent = evalItem.feedback || "Derived consistently from earlier incorrect step.";
                    }
                    break;
                case "unnecessary_but_valid":
                    if (badge) {
                        badge.classList.add("unnecessary");
                        badge.textContent = "+ Intermediate Step";
                    }
                    if (feedbackEl) {
                        feedbackEl.textContent = "Additional intermediate step.";
                    }
                    break;
                case "invalid":
                default:
                    input.classList.add("invalid");
                    if (badge) {
                        badge.classList.add("invalid");
                        badge.textContent = "✗ Error";
                    }
                    if (feedbackEl) {
                        feedbackEl.textContent = evalItem.feedback || "Incorrect step.";
                    }
                    break;
            }
        });
    }

    // --- Canonical Semantic Validation Engine (Matching Rust StepValidator) ---

    public normalizeExpr(expr: string): string {
        return expr.trim()
            .replace(/[\\$€£₹%, ]/g, "")
            .replace(/^['"]|['"]$/g, "")
            .toLowerCase();
    }

    public parseNumericValue(val: string | null | undefined): number | null {
        if (!val) {return null;}
        let s = String(val).trim();
        if (!s) {return null;}

        // 1. If it's an equation assignment like "x = 5" or "v = 12.5", extract the RHS value
        const eqIdx = s.search(/[:=]/);
        if (eqIdx !== -1) {
            const prefix = s.slice(0, eqIdx).trim();
            const rhs = s.slice(eqIdx + 1).trim();
            if (/^[a-zA-Z_\s]+$/.test(prefix) && !rhs.includes("x") && !rhs.includes("=")) {
                s = rhs;
            } else {
                return null;
            }
        }

        // 2. If expression contains algebraic variables like 'x' or operators that aren't scientific notation, it's not a scalar
        const withoutUnits = s.replace(/m\/s|mol\/l|km\/h|kg|mol|j|n|pa|hz|v|w|cm|mm|m|s/gi, "");
        if (/[a-df-zA-DF-Z]/.test(withoutUnits)) {
            return null;
        }

        // 3. Remove currencies and common symbols
        const cleaned = s.replace(/[$€£₹%, ]/g, "");
        if (!cleaned) {return null;}

        // 4. Scientific notation with x10^ or *10^ or ×10^ or e
        const sciMatch = cleaned.match(/^([+-]?\d+(?:\.\d+)?)\s*(?:[x*×]\s*10\^?([+-]?\d+)|e([+-]?\d+))/i);
        if (sciMatch) {
            const m = parseFloat(sciMatch[1]);
            const expStr = sciMatch[2] || sciMatch[3];
            const e = parseInt(expStr, 10);
            if (!isNaN(m) && !isNaN(e)) {
                return m * Math.pow(10, e);
            }
        }

        // 5. Arithmetic fraction: "3/4"
        const fracMatch = cleaned.match(/^([+-]?\d+(?:\.\d+)?)\s*\/\s*([+-]?\d+(?:\.\d+)?)/);
        if (fracMatch) {
            const num = parseFloat(fracMatch[1]);
            const den = parseFloat(fracMatch[2]);
            if (!isNaN(num) && !isNaN(den) && den !== 0) {
                return num / den;
            }
        }

        // 6. Leading float extraction
        const floatMatch = cleaned.match(/^([+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?)/);
        if (floatMatch) {
            const n = parseFloat(floatMatch[1]);
            return isNaN(n) ? null : n;
        }

        const n = parseFloat(cleaned);
        return isNaN(n) ? null : n;
    }

    public extractRootOrValue(expr: string): number | null {
        const subParts = expr.split("=");
        if (subParts.length === 2) {
            const root = this.extractLinearRoot(this.normalizeExpr(subParts[0]), this.normalizeExpr(subParts[1]));
            if (root !== null) {
                return root;
            }
        }
        return this.parseNumericValue(expr);
    }

    public isEquivalent(
        submitted: string,
        expected: string,
        alternates: string[] = [],
        expectedVal?: number,
    ): boolean {
        const normSub = this.normalizeExpr(submitted);
        const normExp = this.normalizeExpr(expected);

        // 1. Literal normalized match
        if (normSub === normExp && normSub.length > 0) {
            return true;
        }

        // 2. Alternate forms match
        for (const alt of alternates) {
            if (normSub === this.normalizeExpr(alt)) {
                return true;
            }
        }

        // 3. Numeric value equivalence
        const subNum = this.parseNumericValue(submitted);
        const expNum = expectedVal !== undefined ? expectedVal : this.parseNumericValue(expected);
        if (subNum !== null && expNum !== null && !isNaN(subNum) && !isNaN(expNum)) {
            if (Math.abs(subNum - expNum) <= 0.01) {
                return true;
            }
        }

        // 4. Linear equation equivalence (e.g. "2x = 10" <=> "x = 5")
        if (this.checkEquationEquivalence(submitted, expected)) {
            return true;
        }

        // 5. Commutative addition match
        if (this.checkCommutativeAddition(normSub, normExp)) {
            return true;
        }

        // 6. Multiplier vs percentage equivalence
        if (this.checkMultiplierEquivalence(submitted, expected)) {
            return true;
        }

        // 7. Relational / slot string comparison (e.g. "Slot 3 = Charlie" vs "Charlie" or "Pos 3: Charlie")
        if (this.checkReasoningSlotEquivalence(submitted, expected)) {
            return true;
        }

        return false;
    }

    public checkEquationEquivalence(submitted: string, expected: string): boolean {
        const subParts = submitted.split("=");
        const expParts = expected.split("=");
        if (subParts.length !== 2 || expParts.length !== 2) {return false;}

        const subLhs = this.normalizeExpr(subParts[0]);
        const subRhs = this.normalizeExpr(subParts[1]);
        const expLhs = this.normalizeExpr(expParts[0]);
        const expRhs = this.normalizeExpr(expParts[1]);

        if ((subLhs === expLhs && subRhs === expRhs) || (subLhs === expRhs && subRhs === expLhs)) {
            return true;
        }

        const subRoot = this.extractLinearRoot(subLhs, subRhs);
        const expRoot = this.extractLinearRoot(expLhs, expRhs);
        if (subRoot !== null && expRoot !== null) {
            return Math.abs(subRoot - expRoot) <= 0.01;
        }

        return false;
    }

    private extractLinearRoot(lhs: string, rhs: string): number | null {
        // Linear equation solver for ax + b = cx + d
        const parseSide = (expr: string): { coeff: number; constVal: number } => {
            let coeff = 0;
            let constVal = 0;
            const terms = expr.match(/[+-]?[^+-]+/g) || [expr];
            for (const term of terms) {
                const trimmed = term.trim();
                if (!trimmed) {continue;}
                if (trimmed.includes("x")) {
                    const withoutX = trimmed.replace("x", "");
                    let c = 1;
                    if (withoutX === "" || withoutX === "+") {c = 1;}
                    else if (withoutX === "-") {c = -1;}
                    else {c = parseFloat(withoutX) || 1;}
                    coeff += c;
                } else {
                    constVal += parseFloat(trimmed) || 0;
                }
            }
            return { coeff, constVal };
        };

        const left = parseSide(lhs);
        const right = parseSide(rhs);
        const netCoeff = left.coeff - right.coeff;
        const netConst = right.constVal - left.constVal;

        if (Math.abs(netCoeff) > 1e-6) {
            return netConst / netCoeff;
        }
        return null;
    }

    public checkCommutativeAddition(s1: string, s2: string): boolean {
        const p1 = s1.split("+").map((s) => s.trim()).sort();
        const p2 = s2.split("+").map((s) => s.trim()).sort();
        return p1.length > 1 && p1.length === p2.length && p1.every((val, i) => val === p2[i]);
    }

    public checkMultiplierEquivalence(sub: string, exp: string): boolean {
        const sVal = this.parseNumericValue(sub);
        const eVal = this.parseNumericValue(exp);
        if (sVal !== null && eVal !== null) {
            return Math.abs(sVal * 100 - eVal) <= 0.01 || Math.abs(sVal - eVal * 100) <= 0.01;
        }
        return false;
    }

    public checkReasoningSlotEquivalence(sub: string, exp: string): boolean {
        const clean = (s: string) => s.toLowerCase().replace(/slot|pos|position|anchor|:|#|=/g, "").replace(/\s+/g, " ").trim();
        const cSub = clean(sub);
        const cExp = clean(exp);
        return cSub === cExp || cSub.includes(cExp) || cExp.includes(cSub);
    }

    public diagnoseStepError(
        submitted: string,
        expectedStep?: ProceduralSolutionStep,
    ): { errorType: string; feedback: string } {
        if (!expectedStep) {
            return { errorType: "unknown", feedback: "Incorrect step." };
        }

        const subVal = this.extractRootOrValue(submitted);
        const expVal = expectedStep.target_expression ? this.extractRootOrValue(expectedStep.target_expression) : null;

        // 1. Sign reversal check
        if (subVal !== null && expVal !== null && expVal !== 0 && Math.abs(subVal + expVal) <= 0.01) {
            return {
                errorType: "sign_error",
                feedback: `Sign reversal detected: Received ${subVal}, expected ${expVal}.`,
            };
        }

        // 2. Arithmetic calculation error
        if (subVal !== null && expVal !== null && Math.abs(subVal - expVal) < 20) {
            return {
                errorType: "arithmetic_error",
                feedback: `Arithmetic calculation slip: Expected ${expVal}, but calculated ${subVal}.`,
            };
        }

        // 3. Problem family and step type taxonomic diagnosis
        const familyId = this.options.familyId || "";
        if (familyId.includes("reasoning") || familyId.includes("seating") || familyId.includes("syllogism") || familyId.includes("relations")) {
            const descLower = (expectedStep.description || "").toLowerCase();
            if (descLower.includes("schema")) {
                return { errorType: "schema_recognition_error", feedback: "Schema recognition error: Failed to identify problem structure." };
            } else if (descLower.includes("strategy") || descLower.includes("anchor")) {
                return { errorType: "strategy_selection_error", feedback: "Strategy selection error: Inappropriate strategy or starting constraint selected." };
            } else if (descLower.includes("represent") || descLower.includes("model") || descLower.includes("diagram")) {
                return { errorType: "representation_error", feedback: "Representation error: Flawed mental model, slot allocation, or diagram setup." };
            } else if (descLower.includes("constraint") || descLower.includes("relative") || descLower.includes("condition")) {
                return { errorType: "constraint_application_error", feedback: "Constraint application error: Violated problem condition or applied constraint incorrectly." };
            } else if (descLower.includes("inference") || descLower.includes("conclusion") || descLower.includes("deduc")) {
                return { errorType: "inference_error", feedback: "Inference error: Invalid logical deduction or relational inference." };
            } else if (descLower.includes("case") || descLower.includes("branch")) {
                return { errorType: "search_case_error", feedback: "Search case error: Missed or improperly branched search case." };
            } else if (descLower.includes("contradiction")) {
                return { errorType: "contradiction_handling_error", feedback: "Contradiction error: Failed to recognize or handle logical contradiction." };
            }
            return { errorType: "constraint_application_error", feedback: `Reasoning step error: Expected '${expectedStep.description}'.` };
        }

        return {
            errorType: "transformation_error",
            feedback: `Incorrect step: Expected '${expectedStep.description || expectedStep.target_expression}'.`,
        };
    }

    public evaluateSubmission(steps: string[], finalAnswer: string): StepwiseEvaluationResult {
        const graph = this.options.solutionGraph;
        const expectedSteps = graph?.steps || [];
        const stepEvaluations: StepEvaluationItem[] = [];

        let firstErrorStep: number | null = null;
        let firstErrorType: string | undefined = undefined;
        let correctStepsCount = 0;
        let prevHadError = false;
        let prevErrVal: number | null = null;

        for (let idx = 0; idx < steps.length; idx++) {
            const subText = steps[idx];
            const expStep = expectedSteps[idx];
            const parsedCurr = this.extractRootOrValue(subText);

            if (!expStep) {
                stepEvaluations.push({
                    stepId: `extra_step_${idx}`,
                    stepIndex: idx,
                    status: "unnecessary_but_valid",
                    submittedText: subText,
                    expectedExpression: "",
                    parsedValue: parsedCurr,
                    feedback: "Additional intermediate step.",
                    isDownstreamConsistent: false,
                });
                continue;
            }

            const isStepValid = this.isEquivalent(
                subText,
                expStep.target_expression || expStep.description,
                [],
                expStep.target_expression ? this.parseNumericValue(expStep.target_expression) || undefined : undefined,
            );

            if (isStepValid) {
                correctStepsCount++;
                stepEvaluations.push({
                    stepId: expStep.id || `step_${idx}`,
                    stepIndex: idx,
                    status: "valid",
                    submittedText: subText,
                    expectedExpression: expStep.target_expression || expStep.description,
                    parsedValue: parsedCurr,
                    feedback: "✓ Correct step",
                    isDownstreamConsistent: false,
                });
                prevHadError = false;
                prevErrVal = null;
            } else {
                const isDownstream = prevHadError && prevErrVal !== null && parsedCurr !== null
                    ? Math.abs(prevErrVal - parsedCurr) <= 0.01
                    : false;

                const status: StepValidationStatus = isDownstream ? "partially_valid" : "invalid";
                const diag = this.diagnoseStepError(subText, expStep);

                if (firstErrorStep === null) {
                    firstErrorStep = idx;
                    firstErrorType = diag.errorType;
                }

                prevHadError = true;
                prevErrVal = parsedCurr;

                stepEvaluations.push({
                    stepId: expStep.id || `step_${idx}`,
                    stepIndex: idx,
                    status,
                    submittedText: subText,
                    expectedExpression: expStep.target_expression || expStep.description,
                    parsedValue: parsedCurr,
                    errorType: diag.errorType,
                    feedback: isDownstream ? "Derived consistently from previous error." : diag.feedback,
                    isDownstreamConsistent: isDownstream,
                });
            }
        }

        const allValid = firstErrorStep === null && (steps.length >= expectedSteps.length || steps.length === 0);
        const isOverallCorrect = allValid && steps.length > 0;
        const score = isOverallCorrect ? 1.0 : (correctStepsCount > 0 && expectedSteps.length > 0 ? Math.min(0.9, correctStepsCount / expectedSteps.length) : 0.0);

        let overallFeedback = "";
        if (isOverallCorrect) {
            overallFeedback = "✓ Excellent! All procedural steps executed correctly.";
        } else if (firstErrorStep !== null) {
            overallFeedback = `First error localized at Step ${firstErrorStep + 1}: ${stepEvaluations[firstErrorStep]?.feedback || "Incorrect step"}`;
        } else {
            overallFeedback = "Submission incomplete or incorrect.";
        }

        const remediationMap: Record<string, string> = {
            formula_selection_error: "remediate:simpler_schema_trigger",
            strategy_selection_error: "remediate:strategy_selection_drill",
            representation_error: "remediate:coordinate_system_setup",
            constraint_application_error: "remediate:constraint_propagation_guided",
            inference_error: "remediate:formal_inference_drill",
            search_case_error: "remediate:case_branching_guided",
            contradiction_handling_error: "remediate:contradiction_detection_drill",
            arithmetic_error: "remediate:simpler_numbers_variant",
            sign_error: "remediate:sign_focused_variant",
            transformation_error: "remediate:lower_complexity_variant",
        };

        const remediationRecommendation = firstErrorType ? (remediationMap[firstErrorType] || "remediate:standard_variant") : undefined;

        return {
            isCorrect: isOverallCorrect,
            score,
            firstErrorStep,
            firstErrorType,
            confidence: "strongly_inferred",
            stepsCompleted: steps.length,
            stepsCorrect: correctStepsCount,
            stepEvaluations,
            overallFeedback,
            remediationRecommendation,
            finalAnswer,
        };
    }

    private escapeHtml(text: string): string {
        const div = document.createElement("div");
        div.textContent = text;
        return div.innerHTML;
    }

    private typesetMathJax(element?: HTMLElement | null): void {
        if (this.options.typesetMathJax) {
            this.options.typesetMathJax(element || this.container);
        } else if (typeof MathJax !== "undefined" && MathJax.typesetPromise) {
            MathJax.typesetPromise([element || this.container]).catch(() => {});
        }
    }

    public destroy(): void {
        this.disposables.forEach((d) => d());
        this.disposables = [];
    }
}

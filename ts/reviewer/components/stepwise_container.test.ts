// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { StepwiseContainer } from "./stepwise_container";

describe("StepwiseContainer", () => {
    let container: HTMLDivElement;

    beforeEach(() => {
        container = document.createElement("div");
        container.id = "procedural-card";
        container.innerHTML = `
            <div id="proc-stepwise-container">
                <div id="proc-steps-list">
                    <div class="proc-step-row" data-step-idx="0">
                        <input type="text" class="proc-input proc-step-input" />
                    </div>
                </div>
                <div class="proc-controls">
                    <button type="button" id="proc-add-step-btn" class="proc-btn">+ Add Step</button>
                    <button type="button" id="proc-hint-btn" class="proc-btn">💡 Request Hint</button>
                    <button type="button" id="proc-reset-steps-btn" class="proc-btn">Reset</button>
                    <button type="button" id="proc-check-steps-btn" class="proc-btn">Check Solution</button>
                </div>
                <div id="proc-hint-container" class="proc-hint-box hidden"></div>
            </div>
        `;
        document.body.appendChild(container);
        (window as any).bridgeCommand = vi.fn();
    });

    afterEach(() => {
        container.remove();
        vi.restoreAllMocks();
    });

    test("initializes and populates steps from solutionGraph", () => {
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-step-1",
            familyId: "math.linear_equations",
            targetTimeMs: 30000,
            solutionGraph: {
                steps: [
                    { description: "Subtract 6 from both sides", target_expression: "2x = 10" },
                    { description: "Divide both sides by 2", target_expression: "x = 5" },
                ],
            },
        });

        const rows = container.querySelectorAll(".proc-step-row");
        expect(rows.length).toBe(2);
        expect(rows[0].textContent).toContain("Step 1: Subtract 6 from both sides");
        expect(rows[1].textContent).toContain("Step 2: Divide both sides by 2");

        stepwise.destroy();
    });

    test("supports dynamic step addition and resetting", () => {
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-step-2",
            familyId: "math.algebra",
            targetTimeMs: 25000,
            solutionGraph: {
                steps: [{ description: "Initial step", target_expression: "x = 2" }],
            },
        });

        expect(container.querySelectorAll(".proc-step-row").length).toBe(1);

        stepwise.addStepRow();
        expect(container.querySelectorAll(".proc-step-row").length).toBe(2);

        stepwise.resetSteps();
        expect(container.querySelectorAll(".proc-step-row").length).toBe(1);

        stepwise.destroy();
    });

    test("evaluates perfectly correct mathematical steps and renders valid badges", () => {
        const onCompleted = vi.fn();
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-math-perfect",
            familyId: "math.linear_equations",
            targetTimeMs: 30000,
            solutionGraph: {
                steps: [
                    { description: "Isolate variable term", target_expression: "2x = 10" },
                    { description: "Solve for x", target_expression: "x = 5" },
                ],
            },
            onStepwiseCompleted: onCompleted,
        });

        const inputs = container.querySelectorAll<HTMLInputElement>(".proc-step-input");
        inputs[0].value = "2x = 10";
        inputs[1].value = "x = 5";

        const result = stepwise.handleCheckSolution();

        expect(result.isCorrect).toBe(true);
        expect(result.score).toBe(1.0);
        expect(result.firstErrorStep).toBeNull();
        expect(result.stepsCorrect).toBe(2);
        expect(onCompleted).toHaveBeenCalledWith(result);

        const badges = container.querySelectorAll(".proc-step-badge");
        expect(badges[0].classList.contains("valid")).toBe(true);
        expect(badges[1].classList.contains("valid")).toBe(true);
        expect(badges[0].textContent).toContain("Valid");

        const bridgeCalls = (window as any).bridgeCommand.mock.calls;
        expect(bridgeCalls.length).toBeGreaterThan(0);
        expect(bridgeCalls[0][0]).toContain("procedural_validate_steps:");

        stepwise.destroy();
    });

    test("localizes first error and tracks downstream consistency (PartiallyValid)", () => {
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-math-downstream",
            familyId: "math.linear_equations",
            targetTimeMs: 30000,
            solutionGraph: {
                steps: [
                    { description: "Isolate variable term", target_expression: "2x = 10" },
                    { description: "Solve for x", target_expression: "x = 5" },
                ],
            },
        });

        const inputs = container.querySelectorAll<HTMLInputElement>(".proc-step-input");
        // Student writes 2x = 12 (wrong, root 6 instead of 5), then computes x = 6 (consistently derived)
        inputs[0].value = "2x = 12";
        inputs[1].value = "x = 6";

        const result = stepwise.handleCheckSolution();

        expect(result.isCorrect).toBe(false);
        expect(result.firstErrorStep).toBe(0);
        expect(result.stepEvaluations[0].status).toBe("invalid");
        expect(result.stepEvaluations[1].status).toBe("partially_valid");
        expect(result.stepEvaluations[1].isDownstreamConsistent).toBe(true);

        const badges = container.querySelectorAll(".proc-step-badge");
        expect(badges[0].classList.contains("invalid")).toBe(true);
        expect(badges[1].classList.contains("partial")).toBe(true);
        expect(badges[1].textContent).toContain("Downstream Consistent");

        stepwise.destroy();
    });

    test("evaluates reasoning pedagogical structures (seating arrangement constraint propagation)", () => {
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-reason-seating",
            familyId: "reasoning.seating",
            targetTimeMs: 40000,
            solutionGraph: {
                steps: [
                    { description: "Place fixed anchor Alice at position 1", target_expression: "Anchor: Alice" },
                    { description: "Propagate relative constraint for Bob and Charlie", target_expression: "Slot 3 = Charlie" },
                    { description: "Final answer for person at slot 3", target_expression: "Charlie" },
                ],
            },
        });

        const inputs = container.querySelectorAll<HTMLInputElement>(".proc-step-input");
        inputs[0].value = "Anchor: Alice";
        inputs[1].value = "Slot 3 = Charlie";
        inputs[2].value = "Charlie";

        const result = stepwise.handleCheckSolution();

        expect(result.isCorrect).toBe(true);
        expect(result.stepsCorrect).toBe(3);
        expect(result.overallFeedback).toContain("All procedural steps executed correctly");

        stepwise.destroy();
    });

    test("diagnoses reasoning representation and constraint errors", () => {
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-reason-err",
            familyId: "reasoning.syllogism",
            targetTimeMs: 35000,
            solutionGraph: {
                steps: [
                    { description: "Build representation model of premises", target_expression: "Euler diagram with disjoint sets" },
                    { description: "Evaluate conclusion validity", target_expression: "Only I follows" },
                ],
            },
        });

        const inputs = container.querySelectorAll<HTMLInputElement>(".proc-step-input");
        inputs[0].value = "Flawed assumption without representation";
        inputs[1].value = "Only I follows";

        const result = stepwise.handleCheckSolution();

        expect(result.isCorrect).toBe(false);
        expect(result.firstErrorStep).toBe(0);
        expect(result.firstErrorType).toBe("representation_error");
        expect(result.remediationRecommendation).toBe("remediate:coordinate_system_setup");

        stepwise.destroy();
    });

    test("progressive 3-tier hint disclosure dispatches bridge commands", () => {
        const onHint = vi.fn();
        const stepwise = new StepwiseContainer(container, {
            instanceId: "inst-hints",
            familyId: "math.algebra",
            targetTimeMs: 25000,
            solutionGraph: {
                steps: [
                    {
                        description: "Step 1",
                        target_expression: "2x = 10",
                        hints: [
                            { level: 1, title: "Principle", content: "Subtract constant term from both sides." },
                            { level: 2, title: "Operation", content: "Calculate 16 - 6." },
                            { level: 3, title: "Intermediate Relation", content: "2x = 10." },
                        ],
                    },
                ],
            },
            onHintRequested: onHint,
        });

        const hintBox = container.querySelector<HTMLElement>("#proc-hint-container")!;
        expect(hintBox.classList.contains("hidden")).toBe(true);

        // Hint 1 (Principle)
        stepwise.requestHint();
        expect(hintBox.classList.contains("hidden")).toBe(false);
        expect(hintBox.textContent).toContain("Principle");
        expect(hintBox.textContent).toContain("Subtract constant term from both sides");
        expect(onHint).toHaveBeenCalledWith(1);

        const bridgeCalls = (window as any).bridgeCommand.mock.calls;
        expect(bridgeCalls.length).toBeGreaterThan(0);
        expect(bridgeCalls[0][0]).toContain("procedural_hint:");

        // Hint 2 (Operation)
        stepwise.requestHint();
        expect(hintBox.textContent).toContain("Operation");
        expect(hintBox.textContent).toContain("Calculate 16 - 6");
        expect(onHint).toHaveBeenCalledWith(2);

        // Hint 3 (Intermediate Relation)
        stepwise.requestHint();
        expect(hintBox.textContent).toContain("Intermediate Relation");
        expect(hintBox.textContent).toContain("2x = 10");
        expect(onHint).toHaveBeenCalledWith(3);

        stepwise.destroy();
    });
});

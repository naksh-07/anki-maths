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
        reviewer.destroy();
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

        const incorrectRes = reviewer.evaluateLocally("15");
        expect(incorrectRes.isCorrect).toBe(false);
        expect(incorrectRes.score).toBe(0.0);

        reviewer.destroy();
    });

    test("mode switching changes active tabs and container visibility", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
        });

        reviewer.switchMode("stepwise");
        expect(container.querySelector("#tab-stepwise")?.classList.contains("active")).toBe(true);
        expect(container.querySelector("#tab-quick")?.classList.contains("active")).toBe(false);
        expect(container.querySelector("#proc-stepwise-container")?.classList.contains("hidden")).toBe(false);
        expect(container.querySelector("#proc-quick-container")?.classList.contains("hidden")).toBe(true);

        reviewer.switchMode("quick");
        expect(container.querySelector("#tab-quick")?.classList.contains("active")).toBe(true);
        expect(container.querySelector("#proc-quick-container")?.classList.contains("hidden")).toBe(false);

        reviewer.destroy();
    });

    test("progressive hints requests and bridge notification", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
            solutionGraph: {
                steps: [
                    {
                        description: "Isolate variable term",
                        hints: [{ level: 1, title: "Step 1 Hint", content: "Subtract constant from both sides." }],
                    },
                ],
            },
        });

        reviewer.requestHint();
        const hintBox = container.querySelector("#proc-hint-container");
        expect(hintBox?.classList.contains("hidden")).toBe(false);
        expect(hintBox?.innerHTML).toContain("Subtract constant from both sides.");
        expect((window as any).bridgeCommand).toHaveBeenCalledWith(
            expect.stringContaining("procedural_hint"),
        );

        reviewer.destroy();
    });

    test("adding and resetting step rows", () => {
        const reviewer = new ProceduralReviewer(container, {
            instanceId: "inst-123",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
        });

        reviewer.addStepRow();
        let rows = container.querySelectorAll(".proc-step-row");
        expect(rows.length).toBe(2);

        reviewer.resetSteps();
        rows = container.querySelectorAll(".proc-step-row");
        expect(rows.length).toBe(1);

        reviewer.destroy();
    });

    test("lifecycle: repeated setup and destroy unbinds listeners without leaks or duplicate calls", () => {
        let completionCount = 0;
        const onCompleted = () => {
            completionCount += 1;
        };

        // Setup instance 1
        const reviewer1 = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-test-1",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
            onCompleted,
        });

        // Destroy instance 1
        reviewer1.destroy();

        // Clicking submit button after destroy should NOT trigger completion
        const quickInput = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
        quickInput.value = "12";
        submitBtn.click();
        expect(completionCount).toBe(0);

        // Setup instance 2 on the same container
        const reviewer2 = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-test-2",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
            onCompleted,
        });

        // Click submit on instance 2 -> should trigger callback exactly ONCE
        quickInput.value = "12";
        submitBtn.click();
        expect(completionCount).toBe(1);

        reviewer2.destroy();
    });

    test("lifecycle: automatic cleanup of previous instance on re-setup", () => {
        let callback1Calls = 0;
        let callback2Calls = 0;

        // Setup first without manual destroy
        proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-test-1",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
            onCompleted: () => {
                callback1Calls += 1;
            },
        });

        // Re-setup on the same container
        const rev2 = proceduralAPI.setup({
            containerId: "procedural-card",
            instanceId: "inst-test-2",
            familyId: "math.linear_equations",
            targetTimeMs: 45000,
            correctAnswer: { value: 12.0 },
            onCompleted: () => {
                callback2Calls += 1;
            },
        });

        const quickInput = container.querySelector<HTMLInputElement>("#proc-answer-input")!;
        const submitBtn = container.querySelector<HTMLButtonElement>("#proc-submit-btn")!;
        quickInput.value = "12";
        submitBtn.click();

        // Only instance 2 should receive the event
        expect(callback1Calls).toBe(0);
        expect(callback2Calls).toBe(1);

        rev2.destroy();
    });
});

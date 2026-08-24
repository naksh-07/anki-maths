// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { DiagnosticSessionController } from "./diagnostic_session";
import { MockBlueprint, MockQuestionItem, MockSession } from "./types";

vi.mock("@tslib/bridgecommand", () => ({
    bridgeCommand: vi.fn().mockResolvedValue({ status: "ok" }),
}));

import { bridgeCommand } from "@tslib/bridgecommand";

describe("DiagnosticSessionController", () => {
    let container: HTMLDivElement;
    let mockSession: MockSession;
    let controller: DiagnosticSessionController;

    const createMockQuestions = (): MockQuestionItem[] => [
        {
            question_index: 0,
            schema_id: "successive_percentage",
            skill_id: "percentage.successive",
            domain: "mathematics",
            schema_title: "Successive Percentage",
            instance: {
                id: "inst_math_0",
                family_id: "family.math.percentage.successive",
                seed: 100,
                parameters: {
                    options: ["28%", "30%", "25%", "32%"],
                },
                rendered_prompt: "What is the net single discount equivalent to two successive discounts of 20% and 10%?",
                correct_answer: { value: 28.0, canonical_option_id: "A", formatted: "28%" },
                metadata: { chapter: "Arithmetic", topic: "Percentages" },
            },
            difficulty_level: 2,
            target_time_ms: 30000,
            is_pyq: false,
        },
        {
            question_index: 1,
            schema_id: "physics_kinematics_1d",
            skill_id: "physics.kinematics.1d",
            domain: "physics",
            schema_title: "1D Kinematics",
            instance: {
                id: "inst_phys_1",
                family_id: "family.physics.kinematics.1d",
                seed: 101,
                parameters: {},
                rendered_prompt: "A car accelerates from rest at 2 m/s² for 10 s. What is its final velocity in m/s?",
                correct_answer: { value: 20.0, unit: "m/s" },
                metadata: { chapter: "Mechanics", topic: "1D Kinematics" },
            },
            difficulty_level: 2,
            target_time_ms: 30000,
            is_pyq: false,
        },
        {
            question_index: 2,
            schema_id: "reasoning_syllogism",
            skill_id: "reasoning.syllogism",
            domain: "reasoning",
            schema_title: "Syllogisms",
            instance: {
                id: "inst_reas_2",
                family_id: "family.reasoning.syllogism",
                seed: 102,
                parameters: {
                    options: ["Only conclusion I follows", "Only conclusion II follows", "Both follow", "Neither follows"],
                },
                rendered_prompt: "Statements: All cats are mammals. All mammals are animals. Conclusion I: All cats are animals.",
                correct_answer: { canonical_option_id: "A" },
                metadata: { chapter: "Logical Deduction", topic: "Syllogisms" },
            },
            difficulty_level: 2,
            target_time_ms: 30000,
            is_pyq: false,
        },
        {
            question_index: 3,
            schema_id: "chemistry_stoichiometry",
            skill_id: "chemistry.stoichiometry",
            domain: "chemistry",
            schema_title: "Mole & Stoichiometry",
            instance: {
                id: "inst_chem_3",
                family_id: "family.chemistry.stoichiometry",
                seed: 103,
                parameters: {},
                rendered_prompt: "How many moles of H2O are produced by complete combustion of 2 moles of H2?",
                correct_answer: { value: 2.0, unit: "mol" },
                metadata: { chapter: "Physical Chemistry", topic: "Stoichiometry" },
            },
            difficulty_level: 2,
            target_time_ms: 30000,
            is_pyq: false,
        },
    ];

    beforeEach(() => {
        vi.useFakeTimers();

        // Setup DOM elements
        container = document.createElement("div");
        container.innerHTML = `
            <div id="diagTimer">--:--</div>
            <button id="diagSubmitBtn">Submit</button>
            <div id="diagMainContainer">
                <div id="diagQuestionCard"></div>
            </div>
            <div id="diagAnsweredCount">0/4 Answered</div>
            <div id="diagPaletteGrid"></div>
            <button id="diagMarkBtn">★ Mark for Review</button>
            <button id="diagClearBtn">Clear Answer</button>
            <button id="diagPrevBtn">Previous</button>
            <button id="diagNextBtn">Next</button>
        `;
        document.body.appendChild(container);

        const blueprint: MockBlueprint = {
            exam_profile_id: "diagnostic-multi-domain",
            title: "Comprehensive Multi-Domain Diagnostic Assessment",
            domain_distribution: { mathematics: 1, physics: 1, reasoning: 1, chemistry: 1 },
            difficulty_distribution: { "2": 1.0 },
            total_questions: 4,
            time_limit_ms: 300000, // 5 minutes
            positive_mark_per_question: 1.0,
            negative_mark_per_incorrect: 0.0,
        };

        mockSession = {
            session_id: "diag_sess_test_001",
            blueprint,
            questions: createMockQuestions(),
            answers: {},
            marked_for_review: [],
            is_submitted: false,
        };

        controller = new DiagnosticSessionController(mockSession);
        controller.init();
    });

    afterEach(() => {
        controller.destroy();
        document.body.innerHTML = "";
        vi.clearAllMocks();
        vi.useRealTimers();
    });

    test("initializes palette, timer, and first question (MCQ)", () => {
        const paletteBtns = document.querySelectorAll(".diag-palette-btn");
        expect(paletteBtns.length).toBe(4);
        expect(paletteBtns[0].classList.contains("active")).toBe(true);

        const timerEl = document.getElementById("diagTimer");
        expect(timerEl?.textContent).toBe("05:00");

        const promptEl = document.querySelector(".diag-q-prompt");
        expect(promptEl?.textContent).toContain("What is the net single discount equivalent");

        const options = document.querySelectorAll(".diag-option-item");
        expect(options.length).toBe(4);
        expect(options[0].textContent).toContain("28%");

        const prevBtn = document.getElementById("diagPrevBtn") as HTMLButtonElement;
        expect(prevBtn.disabled).toBe(true);
    });

    test("navigates forward to numerical Physics question", () => {
        const nextBtn = document.getElementById("diagNextBtn") as HTMLButtonElement;
        nextBtn.click();

        expect(controller.getCurrentIndex()).toBe(1);

        const promptEl = document.querySelector(".diag-q-prompt");
        expect(promptEl?.textContent).toContain("A car accelerates from rest");

        const inputEl = document.getElementById("diagInputAnswer") as HTMLInputElement;
        expect(inputEl).not.toBeNull();

        const prevBtn = document.getElementById("diagPrevBtn") as HTMLButtonElement;
        expect(prevBtn.disabled).toBe(false);
    });

    test("records MCQ option selection via click and updates palette", () => {
        const options = document.querySelectorAll(".diag-option-item") as NodeListOf<HTMLElement>;
        options[0].click(); // Select "A" (28%)

        const updatedOptions = document.querySelectorAll(".diag-option-item");
        expect(updatedOptions[0].classList.contains("selected")).toBe(true);

        const session = controller.getCurrentSession();
        expect(session.answers[0]?.answer).toBe("A");

        const paletteBtns = document.querySelectorAll(".diag-palette-btn");
        expect(paletteBtns[0].classList.contains("answered")).toBe(true);

        const countEl = document.getElementById("diagAnsweredCount");
        expect(countEl?.textContent).toBe("1/4 Answered");
    });

    test("records MCQ option selection via keyboard shortcut (1-4 and A-D)", () => {
        // Press 'B' key
        window.dispatchEvent(new KeyboardEvent("keydown", { key: "b" }));

        const session = controller.getCurrentSession();
        expect(session.answers[0]?.answer).toBe("B");

        let options = document.querySelectorAll(".diag-option-item");
        expect(options[1].classList.contains("selected")).toBe(true);

        // Press '1' key -> updates to 'A'
        window.dispatchEvent(new KeyboardEvent("keydown", { key: "1" }));
        expect(session.answers[0]?.answer).toBe("A");

        options = document.querySelectorAll(".diag-option-item");
        expect(options[0].classList.contains("selected")).toBe(true);
    });

    test("records Numerical input answer and updates session state", () => {
        controller.goToQuestion(1); // Go to physics question

        const inputEl = document.getElementById("diagInputAnswer") as HTMLInputElement;
        inputEl.value = "20 m/s";
        inputEl.dispatchEvent(new Event("input"));

        const session = controller.getCurrentSession();
        expect(session.answers[1]?.answer).toBe("20 m/s");

        const paletteBtns = document.querySelectorAll(".diag-palette-btn");
        expect(paletteBtns[1].classList.contains("answered")).toBe(true);
    });

    test("toggles mark for review on button click and keyboard 'm'", () => {
        const markBtn = document.getElementById("diagMarkBtn") as HTMLButtonElement;
        markBtn.click();

        let session = controller.getCurrentSession();
        expect(session.marked_for_review).toContain(0);

        let paletteBtns = document.querySelectorAll(".diag-palette-btn");
        expect(paletteBtns[0].classList.contains("marked")).toBe(true);
        expect(markBtn.textContent).toContain("Unmark");

        // Toggle back via 'm' key
        window.dispatchEvent(new KeyboardEvent("keydown", { key: "m" }));
        session = controller.getCurrentSession();
        expect(session.marked_for_review).not.toContain(0);

        paletteBtns = document.querySelectorAll(".diag-palette-btn");
        expect(paletteBtns[0].classList.contains("marked")).toBe(false);
    });

    test("clears answer when clear button is clicked", () => {
        controller.saveAnswer("A");
        expect(controller.getCurrentSession().answers[0]).toBeDefined();

        const clearBtn = document.getElementById("diagClearBtn") as HTMLButtonElement;
        clearBtn.click();

        expect(controller.getCurrentSession().answers[0]).toBeUndefined();
        const paletteBtns = document.querySelectorAll(".diag-palette-btn");
        expect(paletteBtns[0].classList.contains("answered")).toBe(false);
    });

    test("palette buttons jump directly to questions", () => {
        const paletteBtns = document.querySelectorAll(".diag-palette-btn") as NodeListOf<HTMLButtonElement>;
        paletteBtns[2].click(); // Jump to question 3 (Reasoning)

        expect(controller.getCurrentIndex()).toBe(2);
        const promptEl = document.querySelector(".diag-q-prompt");
        expect(promptEl?.textContent).toContain("All cats are mammals");
    });

    test("timer countdown decrements and activates warning class at <= 120s", () => {
        const timerEl = document.getElementById("diagTimer");
        expect(timerEl?.textContent).toBe("05:00");

        // Fast-forward 60s
        vi.advanceTimersByTime(60000);
        expect(timerEl?.textContent).toBe("04:00");
        expect(timerEl?.classList.contains("warning")).toBe(false);

        // Fast-forward 130s (total 190s elapsed, 110s remaining <= 120s)
        vi.advanceTimersByTime(130000);
        expect(timerEl?.textContent).toBe("01:50");
        expect(timerEl?.classList.contains("warning")).toBe(true);
    });

    test("submits test and calls bridgeCommand with diagnosticSubmit payload", () => {
        controller.saveAnswer("A");
        controller.goToQuestion(1);
        controller.saveAnswer("20 m/s");

        controller.submitTest();

        const session = controller.getCurrentSession();
        expect(session.is_submitted).toBe(true);
        expect(session.end_time_ms).toBeDefined();

        expect(bridgeCommand).toHaveBeenCalledWith("diagnosticSubmit", expect.objectContaining({
            session_id: "diag_sess_test_001",
            answers: expect.objectContaining({
                0: expect.objectContaining({ answer: "A" }),
                1: expect.objectContaining({ answer: "20 m/s" }),
            }),
        }));
    });
});

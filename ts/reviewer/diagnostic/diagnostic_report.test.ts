// @vitest-environment jsdom
// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { DiagnosticReportController } from "./diagnostic_report";
import { ComprehensiveDiagnosticReport } from "./types";

vi.mock("@tslib/bridgecommand", () => ({
    bridgeCommand: vi.fn().mockResolvedValue({ status: "ok" }),
}));

import { bridgeCommand } from "@tslib/bridgecommand";

describe("DiagnosticReportController", () => {
    let container: HTMLDivElement;
    let mockReport: ComprehensiveDiagnosticReport;
    let controller: DiagnosticReportController;

    beforeEach(() => {
        container = document.createElement("div");
        container.innerHTML = `
            <div id="diagReportConceptCount">0</div>
            <div id="diagReportCalcCount">0</div>
            <div id="diagReportTransferCount">0</div>
            <div id="diagReportSpeedCount">0</div>
            <div id="diagWeakSkillsList"></div>
            <div id="hierarchyContainer"></div>
            <button id="startRemediationBtn">Start Remedial Practice</button>
        `;
        document.body.appendChild(container);

        mockReport = {
            session_id: "diag_report_test_001",
            exam_profile_id: "diagnostic-multi-domain",
            total_questions: 16,
            answered_count: 16,
            unanswered_count: 0,
            correct_count: 12,
            incorrect_count: 4,
            raw_score: 12.0,
            max_score: 16.0,
            percentage: 75.0,
            accuracy: 75.0,
            total_time_spent_ms: 320000,
            error_distribution: {
                concept_count: 2,
                calculation_count: 1,
                transfer_count: 1,
                speed_deficit_count: 2,
            },
            weak_skills: ["algebra.linear_equations"],
            slow_skills: ["physics.kinematics.1d"],
            transfer_gaps: ["chemistry.stoichiometry"],
            recommended_follow_up: {
                scope: "MultipleSchemas",
                objective: "Practice",
            },
            hierarchy: [
                {
                    id: "mathematics",
                    name: "Mathematics",
                    level: "Subject",
                    total_questions: 6,
                    answered_count: 6,
                    correct_count: 4,
                    accuracy: 66.67,
                    mean_time_ms: 22000,
                    concept_errors: 1,
                    calculation_errors: 1,
                    transfer_errors: 0,
                    speed_deficits: 1,
                    children: [
                        {
                            id: "mathematics:Algebra",
                            name: "Algebra",
                            level: "Chapter",
                            total_questions: 3,
                            answered_count: 3,
                            correct_count: 2,
                            accuracy: 66.67,
                            mean_time_ms: 24000,
                            concept_errors: 1,
                            calculation_errors: 0,
                            transfer_errors: 0,
                            speed_deficits: 1,
                            children: [
                                {
                                    id: "algebra.linear_equations",
                                    name: "Linear Equations",
                                    level: "Topic",
                                    total_questions: 3,
                                    answered_count: 3,
                                    correct_count: 2,
                                    accuracy: 66.67,
                                    mean_time_ms: 24000,
                                    concept_errors: 1,
                                    calculation_errors: 0,
                                    transfer_errors: 0,
                                    speed_deficits: 1,
                                    children: [
                                        {
                                            id: "family.math.algebra.linear_equations",
                                            name: "Two-Step Linear Equations",
                                            level: "ProblemFamily",
                                            total_questions: 3,
                                            answered_count: 3,
                                            correct_count: 2,
                                            accuracy: 66.67,
                                            mean_time_ms: 24000,
                                            concept_errors: 1,
                                            calculation_errors: 0,
                                            transfer_errors: 0,
                                            speed_deficits: 1,
                                            children: [],
                                        },
                                    ],
                                },
                            ],
                        },
                    ],
                },
                {
                    id: "physics",
                    name: "Physics",
                    level: "Subject",
                    total_questions: 4,
                    answered_count: 4,
                    correct_count: 3,
                    accuracy: 75.0,
                    mean_time_ms: 28000,
                    concept_errors: 0,
                    calculation_errors: 0,
                    transfer_errors: 1,
                    speed_deficits: 1,
                    children: [],
                },
            ],
        };

        controller = new DiagnosticReportController(mockReport);
        controller.init();
    });

    afterEach(() => {
        controller.destroy();
        document.body.innerHTML = "";
        vi.clearAllMocks();
    });

    test("renders 4-dimension summary counts in DOM", () => {
        expect(document.getElementById("diagReportConceptCount")?.textContent).toBe("2");
        expect(document.getElementById("diagReportCalcCount")?.textContent).toBe("1");
        expect(document.getElementById("diagReportTransferCount")?.textContent).toBe("1");
        expect(document.getElementById("diagReportSpeedCount")?.textContent).toBe("2");
    });

    test("renders weak skills, slow skills, and transfer gaps chips", () => {
        const chips = document.querySelectorAll("#diagWeakSkillsList .diag-tag");
        expect(chips.length).toBe(3);
        expect(chips[0].textContent).toContain("Concept Gap: algebra.linear_equations");
        expect(chips[1].textContent).toContain("Speed Opportunity: physics.kinematics.1d");
        expect(chips[2].textContent).toContain("Transfer Gap: chemistry.stoichiometry");
    });

    test("renders 4-tier hierarchy nodes with correct levels and depth indentation", () => {
        const nodes = document.querySelectorAll("#hierarchyContainer .node-item");
        expect(nodes.length).toBe(5); // Math Subject, Algebra Chapter, Linear Equations Topic, Two-Step Family, Physics Subject

        // Subject node
        const mathSubject = nodes[0] as HTMLElement;
        expect(mathSubject.textContent).toContain("Mathematics");
        expect(mathSubject.textContent).toContain("Subject");
        expect(mathSubject.textContent).toContain("4/6");
        expect(mathSubject.textContent).toContain("67%");

        // Dimension error badges on Math node (C: 1, E: 1, S: 1)
        expect(mathSubject.innerHTML).toContain("C: 1");
        expect(mathSubject.innerHTML).toContain("E: 1");
        expect(mathSubject.innerHTML).toContain("S: 1");

        // Topic node
        const topicNode = nodes[2] as HTMLElement;
        expect(topicNode.textContent).toContain("Linear Equations");
        expect(topicNode.textContent).toContain("Topic");
        expect(topicNode.style.marginLeft).toBe("24px"); // depth 2 * 12px
    });

    test("collapses and expands hierarchy subtree on header click", () => {
        const mathHeader = document.querySelector("#hierarchyContainer .node-header") as HTMLElement;
        const mathChildren = document.querySelector("#hierarchyContainer .node-children") as HTMLElement;

        expect(mathChildren.style.display).not.toBe("none");

        // Click to collapse
        mathHeader.click();
        expect(mathChildren.style.display).toBe("none");

        // Click to expand
        mathHeader.click();
        expect(mathChildren.style.display).toBe("flex");
    });

    test("triggers follow-up remediation bridgeCommand on button click", () => {
        const remediationBtn = document.getElementById("startRemediationBtn") as HTMLButtonElement;
        remediationBtn.click();

        expect(bridgeCommand).toHaveBeenCalledWith("diagnosticStartRemediation", expect.objectContaining({
            session_id: "diag_report_test_001",
            weak_skills: ["algebra.linear_equations"],
            slow_skills: ["physics.kinematics.1d"],
            transfer_gaps: ["chemistry.stoichiometry"],
        }));
    });
});

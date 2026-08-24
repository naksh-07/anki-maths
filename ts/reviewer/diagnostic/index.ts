// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { DiagnosticReportController } from "./diagnostic_report";
import { DiagnosticSessionController } from "./diagnostic_session";
import { ComprehensiveDiagnosticReport, MockSession } from "./types";

let currentSessionController: DiagnosticSessionController | null = null;
let currentReportController: DiagnosticReportController | null = null;

export const diagnosticAPI = {
    initSession(sessionData: MockSession): DiagnosticSessionController {
        currentSessionController = new DiagnosticSessionController(sessionData);
        currentSessionController.init();
        return currentSessionController;
    },

    initReport(reportData: ComprehensiveDiagnosticReport): DiagnosticReportController {
        currentReportController = new DiagnosticReportController(reportData);
        currentReportController.init();
        return currentReportController;
    },

    getCurrentSession(): MockSession | null {
        return currentSessionController?.getCurrentSession() ?? null;
    },

    getCurrentReport(): ComprehensiveDiagnosticReport | null {
        return currentReportController?.getReport() ?? null;
    },

    submitSession(): void {
        currentSessionController?.submitTest();
    },

    startFollowUpPractice(): void {
        currentReportController?.startFollowUpPractice();
    },
};

export * from "./diagnostic_report";
export * from "./diagnostic_session";
export * from "./types";

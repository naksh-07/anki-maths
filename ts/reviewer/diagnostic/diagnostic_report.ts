// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { bridgeCommand } from "@tslib/bridgecommand";
import { ComprehensiveDiagnosticReport, DiagnosticHierarchyNode } from "./types";

export class DiagnosticReportController {
    private report: ComprehensiveDiagnosticReport;
    private clickListeners: Array<{ el: HTMLElement; fn: EventListener }> = [];

    constructor(report: ComprehensiveDiagnosticReport) {
        this.report = report;
    }

    public init(): void {
        this.renderHierarchy();
        this.renderDimensionSummary();
        this.renderWeakSkills();
        this.attachEventListeners();
    }

    public destroy(): void {
        for (const { el, fn } of this.clickListeners) {
            el.removeEventListener("click", fn);
        }
        this.clickListeners = [];
    }

    public getReport(): ComprehensiveDiagnosticReport {
        return this.report;
    }

    public renderHierarchy(): void {
        const container = document.getElementById("hierarchyContainer");
        if (!container) return;

        container.innerHTML = "";
        if (!this.report.hierarchy || this.report.hierarchy.length === 0) {
            container.innerHTML = `<div style="color: var(--diag-text-muted); font-size: 0.9rem;">No hierarchical data available.</div>`;
            return;
        }

        for (const subjectNode of this.report.hierarchy) {
            container.appendChild(this.createHierarchyNodeElement(subjectNode, 0));
        }
    }

    private renderDimensionSummary(): void {
        // If dimension error counts exist in DOM, update them
        const conceptEl = document.getElementById("diagReportConceptCount");
        const calcEl = document.getElementById("diagReportCalcCount");
        const transferEl = document.getElementById("diagReportTransferCount");
        const speedEl = document.getElementById("diagReportSpeedCount");

        if (conceptEl) conceptEl.textContent = String(this.report.error_distribution?.concept_count ?? 0);
        if (calcEl) calcEl.textContent = String(this.report.error_distribution?.calculation_count ?? 0);
        if (transferEl) transferEl.textContent = String(this.report.error_distribution?.transfer_count ?? 0);
        if (speedEl) speedEl.textContent = String(this.report.error_distribution?.speed_deficit_count ?? 0);
    }

    private renderWeakSkills(): void {
        const weakContainer = document.getElementById("diagWeakSkillsList");
        if (!weakContainer) return;

        weakContainer.innerHTML = "";
        const weakList = this.report.weak_skills || [];
        const slowList = this.report.slow_skills || [];
        const transferList = this.report.transfer_gaps || [];

        if (weakList.length === 0 && slowList.length === 0 && transferList.length === 0) {
            weakContainer.innerHTML = `<span style="font-size: 0.85rem; color: var(--diag-success); font-weight: 600;">✓ No critical weaknesses detected across tested domains!</span>`;
            return;
        }

        for (const s of weakList) {
            const chip = document.createElement("span");
            chip.className = "diag-tag";
            chip.style.borderColor = "var(--diag-danger)";
            chip.style.color = "var(--diag-danger)";
            chip.textContent = `Concept Gap: ${s}`;
            weakContainer.appendChild(chip);
        }

        for (const s of slowList) {
            const chip = document.createElement("span");
            chip.className = "diag-tag";
            chip.style.borderColor = "var(--diag-warning)";
            chip.style.color = "#856404";
            chip.textContent = `Speed Opportunity: ${s}`;
            weakContainer.appendChild(chip);
        }

        for (const s of transferList) {
            const chip = document.createElement("span");
            chip.className = "diag-tag";
            chip.style.borderColor = "#6f42c1";
            chip.style.color = "#6f42c1";
            chip.textContent = `Transfer Gap: ${s}`;
            weakContainer.appendChild(chip);
        }
    }

    private createHierarchyNodeElement(node: DiagnosticHierarchyNode, depth: number): HTMLElement {
        const itemEl = document.createElement("div");
        itemEl.className = "node-item";
        itemEl.style.marginLeft = `${depth * 12}px`;

        const accPct = Math.round(node.accuracy || 0);
        const hasChildren = node.children && node.children.length > 0;

        const headerEl = document.createElement("div");
        headerEl.className = "node-header";

        const badgeClass = accPct >= 75 ? "color: var(--diag-success);" : accPct >= 50 ? "color: var(--diag-warning);" : "color: var(--diag-danger);";

        // Build 4-dimension diagnostic error badges if present
        let dimBadges = "";
        if (node.concept_errors > 0) {
            dimBadges += `<span class="dim-tag" style="background: rgba(220,53,69,0.1); color: var(--diag-danger); font-size: 0.7rem; padding: 1px 5px; border-radius: 3px;" title="Concept Errors">C: ${node.concept_errors}</span>`;
        }
        if (node.calculation_errors > 0) {
            dimBadges += `<span class="dim-tag" style="background: rgba(253,126,20,0.1); color: #fd7e14; font-size: 0.7rem; padding: 1px 5px; border-radius: 3px;" title="Calculation Errors">E: ${node.calculation_errors}</span>`;
        }
        if (node.transfer_errors > 0) {
            dimBadges += `<span class="dim-tag" style="background: rgba(111,66,193,0.1); color: #6f42c1; font-size: 0.7rem; padding: 1px 5px; border-radius: 3px;" title="Transfer Errors">T: ${node.transfer_errors}</span>`;
        }
        if (node.speed_deficits > 0) {
            dimBadges += `<span class="dim-tag" style="background: rgba(255,193,7,0.15); color: #856404; font-size: 0.7rem; padding: 1px 5px; border-radius: 3px;" title="Speed Deficits">S: ${node.speed_deficits}</span>`;
        }

        headerEl.innerHTML = `
          <div style="display: flex; align-items: center; gap: 8px;">
            ${hasChildren ? `<span class="node-toggle" style="font-size: 0.8rem; width: 14px;">\u25BC</span>` : `<span style="width: 14px;">&bull;</span>`}
            <span style="font-weight: ${depth === 0 ? "700" : depth === 1 ? "600" : "500"};">${escapeHtml(node.name)}</span>
            <span style="font-size: 0.75rem; color: var(--diag-text-muted); padding: 1px 6px; background: rgba(0,0,0,0.05); border-radius: 4px;">
              ${escapeHtml(node.level)}
            </span>
            ${dimBadges ? `<div style="display: flex; gap: 4px; margin-left: 4px;">${dimBadges}</div>` : ""}
          </div>
          <div style="display: flex; align-items: center; gap: 12px;">
            <span style="font-size: 0.85rem; color: var(--diag-text-muted);">${node.correct_count}/${node.total_questions}</span>
            <span style="font-size: 0.85rem; font-weight: 700; ${badgeClass}">${accPct}%</span>
            <div class="progress-bar-bg">
              <div class="progress-bar-fill" style="width: ${accPct}%; background: ${accPct >= 75 ? "var(--diag-success)" : accPct >= 50 ? "var(--diag-warning)" : "var(--diag-danger)"};"></div>
            </div>
          </div>
        `;

        itemEl.appendChild(headerEl);

        if (hasChildren) {
            const childrenContainer = document.createElement("div");
            childrenContainer.className = "node-children";

            for (const child of node.children) {
                childrenContainer.appendChild(this.createHierarchyNodeElement(child, depth + 1));
            }

            itemEl.appendChild(childrenContainer);

            // Toggle collapse
            let isExpanded = true;
            const toggleHandler = () => {
                isExpanded = !isExpanded;
                childrenContainer.style.display = isExpanded ? "flex" : "none";
                const toggleSpan = headerEl.querySelector(".node-toggle");
                if (toggleSpan) {
                    toggleSpan.textContent = isExpanded ? "\u25BC" : "\u25B6";
                }
            };

            headerEl.addEventListener("click", toggleHandler);
            this.clickListeners.push({ el: headerEl, fn: toggleHandler });
        }

        return itemEl;
    }

    public startFollowUpPractice(): void {
        bridgeCommand("diagnosticStartRemediation", {
            session_id: this.report.session_id,
            recommended_follow_up: this.report.recommended_follow_up,
            weak_skills: this.report.weak_skills,
            slow_skills: this.report.slow_skills,
            transfer_gaps: this.report.transfer_gaps,
        }).catch((err) => {
            console.error("Bridge command diagnosticStartRemediation failed:", err);
            alert("Starting targeted remedial practice...");
        });
    }

    private attachEventListeners(): void {
        const remediationBtn = document.getElementById("startRemediationBtn");
        if (remediationBtn) {
            const handler = () => this.startFollowUpPractice();
            remediationBtn.addEventListener("click", handler);
            this.clickListeners.push({ el: remediationBtn, fn: handler });
        }
    }
}

function escapeHtml(str: unknown): string {
    return String(str ?? "")
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

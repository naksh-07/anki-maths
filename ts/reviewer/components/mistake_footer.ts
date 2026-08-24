// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

/* eslint
@typescript-eslint/no-explicit-any: "off",
 */

import { bridgeCommand } from "@tslib/bridgecommand";

export type MistakeType =
    | "silly_mistake"
    | "pattern_not_recognized"
    | "formula_or_concept_misapplied"
    | "concept_not_known";

export interface MistakeCategoryDef {
    key: number;
    value: MistakeType;
    shortLabel: string;
    label: string;
    description: string;
    badge: string;
}

export const MISTAKE_CATEGORIES: MistakeCategoryDef[] = [
    {
        key: 1,
        value: "silly_mistake",
        shortLabel: "1 Silly",
        label: "Silly Slip",
        description: "Arithmetic or calculation slip",
        badge: "1",
    },
    {
        key: 2,
        value: "pattern_not_recognized",
        shortLabel: "2 Pattern",
        label: "Pattern Missed",
        description: "Failed to identify problem structure or schema",
        badge: "2",
    },
    {
        key: 3,
        value: "formula_or_concept_misapplied",
        shortLabel: "3 Concept",
        label: "Concept Gap",
        description: "Wrong formula or misapplied theorem",
        badge: "3",
    },
    {
        key: 4,
        value: "concept_not_known",
        shortLabel: "4 Unknown",
        label: "Prereq Unknown",
        description: "Fundamental knowledge gap or missing prerequisite",
        badge: "4",
    },
];

export interface MistakeFooterOptions {
    container: HTMLElement;
    instanceId?: string;
    familyId?: string;
    onSelect?: (mistakeType: MistakeType) => void;
}

/**
 * Compact Mistake Classification Footer Component
 * 
 * Provides an inline, keyboard-navigable mistake attribution strip
 * ([1 Silly], [2 Pattern], [3 Concept], [4 Unknown]) that fits natively
 * into the primary review interaction flow without disruptive scrolling.
 */
export class MistakeFooter {
    private container: HTMLElement;
    private options: MistakeFooterOptions;
    private panelEl: HTMLElement | null = null;
    private buttons: HTMLButtonElement[] = [];
    private selectedValue: MistakeType | null = null;
    private isVisible = false;
    private disposables: Array<() => void> = [];
    private activeCallback: ((mistakeType: MistakeType) => void) | null = null;

    constructor(options: MistakeFooterOptions) {
        this.container = options.container;
        this.options = options;
        this.initDOM();
    }

    private initDOM(): void {
        // Look for existing mistake panel or create one
        this.panelEl = this.container.querySelector("#proc-mistake-panel");
        if (!this.panelEl) {
            this.panelEl = document.createElement("div");
            this.panelEl.id = "proc-mistake-panel";
            this.panelEl.className = "proc-mistake-panel hidden";
            this.panelEl.innerHTML = `
                <div class="proc-mistake-heading">Classify error (1-4) to reflect and optimize spaced repetition:</div>
                <div class="proc-mistake-footer">
                    ${MISTAKE_CATEGORIES.map(
                        (cat) => `
                        <button type="button" class="proc-mistake-btn" data-value="${cat.value}" data-key="${cat.key}" title="${cat.description}">
                            <span class="proc-key-badge">${cat.badge}</span> ${cat.label}
                        </button>
                    `
                    ).join("")}
                </div>
            `;
            // Insert into result panel or container
            const resultPanel = this.container.querySelector("#proc-result-panel");
            const solutionContainer = this.container.querySelector("#proc-solution-container");
            if (resultPanel && solutionContainer) {
                resultPanel.insertBefore(this.panelEl, solutionContainer);
            } else if (resultPanel) {
                resultPanel.appendChild(this.panelEl);
            } else {
                this.container.appendChild(this.panelEl);
            }
        }

        this.bindButtons();
    }

    private bindButtons(): void {
        this.buttons = Array.from(
            this.panelEl?.querySelectorAll<HTMLButtonElement>(
                ".proc-mistake-btn, .proc-mistake-card"
            ) || []
        );

        this.buttons.forEach((btn) => {
            const clickHandler = () => {
                const val = btn.dataset.value as MistakeType;
                if (val) {
                    this.select(val);
                }
            };
            btn.addEventListener("click", clickHandler);
            this.disposables.push(() => btn.removeEventListener("click", clickHandler));
        });
    }

    public show(onSelect?: (mistakeType: MistakeType) => void): void {
        this.isVisible = true;
        if (onSelect) {
            this.activeCallback = onSelect;
        }
        if (this.panelEl) {
            this.panelEl.classList.remove("hidden");
            this.panelEl.tabIndex = -1;
            this.panelEl.focus();
        }
    }

    public hide(): void {
        this.isVisible = false;
        this.activeCallback = null;
        if (this.panelEl) {
            this.panelEl.classList.add("hidden");
        }
    }

    public select(valOrKey: string | number): MistakeType | null {
        let matchedType: MistakeType | null = null;
        if (typeof valOrKey === "number" || /^[1-4]$/.test(String(valOrKey))) {
            const keyNum = parseInt(String(valOrKey), 10);
            const cat = MISTAKE_CATEGORIES.find((c) => c.key === keyNum);
            if (cat) {
                matchedType = cat.value;
            }
        } else {
            const cat = MISTAKE_CATEGORIES.find((c) => c.value === valOrKey);
            if (cat) {
                matchedType = cat.value;
            }
        }

        if (!matchedType) {
            return null;
        }

        this.selectedValue = matchedType;

        // Highlight selected button
        this.buttons.forEach((btn) => {
            if (btn.dataset.value === matchedType) {
                btn.classList.add("selected");
            } else {
                btn.classList.remove("selected");
            }
        });

        // Dispatch bridge notification for mistake classification
        bridgeCommand(`procedural_mistake:${JSON.stringify({
            instance_id: this.options.instanceId || "",
            family_id: this.options.familyId || "",
            mistake_type: matchedType,
        })}`);

        if (this.activeCallback) {
            const cb = this.activeCallback;
            cb(matchedType);
        } else if (this.options.onSelect) {
            this.options.onSelect(matchedType);
        }

        return matchedType;
    }

    public handleKeydown(e: KeyboardEvent): boolean {
        if (!this.isVisible) {
            return false;
        }

        const keyNum = parseInt(e.key, 10);
        if (!isNaN(keyNum) && keyNum >= 1 && keyNum <= 4) {
            e.preventDefault();
            e.stopPropagation();
            this.select(keyNum);
            return true;
        }

        // Trap Space and Enter to prevent skipping/bypassing reflection without explicit selection
        if (e.key === " " || e.code === "Space" || e.key === "Enter") {
            const activeEl = document.activeElement as HTMLElement;
            if (activeEl && (activeEl.classList.contains("proc-mistake-btn") || activeEl.classList.contains("proc-mistake-card"))) {
                const val = activeEl.dataset.value as MistakeType;
                if (val) {
                    e.preventDefault();
                    e.stopPropagation();
                    this.select(val);
                    return true;
                }
            }
            e.preventDefault();
            e.stopPropagation();
            return true;
        }

        return false;
    }

    public getSelectedValue(): MistakeType | null {
        return this.selectedValue;
    }

    public isShown(): boolean {
        return this.isVisible;
    }

    public destroy(): void {
        this.isVisible = false;
        this.activeCallback = null;
        for (const dispose of this.disposables) {
            try {
                dispose();
            } catch {
                /* non-fatal */
            }
        }
        this.disposables = [];
        this.buttons = [];
        this.panelEl = null;
    }
}

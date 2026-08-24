// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import { bridgeCommand } from "@tslib/bridgecommand";
import { MockAnswerSubmission, MockQuestionItem, MockSession } from "./types";

export class DiagnosticSessionController {
    private session: MockSession;
    private currentIndex: number = 0;
    private questionStartTimes: Map<number, number> = new Map();
    private timerInterval: any = null;
    private remainingSeconds: number = 0;
    private keydownListener: ((e: KeyboardEvent) => void) | null = null;

    constructor(session: MockSession) {
        this.session = session;
        if (!this.session.answers) {
            this.session.answers = {};
        }
        if (!this.session.marked_for_review) {
            this.session.marked_for_review = [];
        }
        this.remainingSeconds = Math.floor(session.blueprint.time_limit_ms / 1000);
    }

    public init(): void {
        this.startTimer();
        this.renderPalette();
        this.renderCurrentQuestion();
        this.attachEventListeners();
        this.attachKeyboardShortcuts();
        this.questionStartTimes.set(this.currentIndex, Date.now());
    }

    public destroy(): void {
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
            this.timerInterval = null;
        }
        if (this.keydownListener) {
            window.removeEventListener("keydown", this.keydownListener);
            this.keydownListener = null;
        }
    }

    public getCurrentSession(): MockSession {
        return this.session;
    }

    public getCurrentIndex(): number {
        return this.currentIndex;
    }

    private startTimer(): void {
        this.updateTimerDisplay();
        this.timerInterval = setInterval(() => {
            if (this.remainingSeconds > 0) {
                this.remainingSeconds--;
                this.updateTimerDisplay();
                if (this.remainingSeconds === 0) {
                    this.onTimeExpired();
                }
            }
        }, 1000);
    }

    private updateTimerDisplay(): void {
        const timerEl = document.getElementById("diagTimer");
        if (!timerEl) return;

        const mins = Math.floor(this.remainingSeconds / 60);
        const secs = this.remainingSeconds % 60;
        const formatted = `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
        timerEl.textContent = formatted;

        if (this.remainingSeconds <= 120) {
            timerEl.classList.add("warning");
        } else {
            timerEl.classList.remove("warning");
        }
    }

    private onTimeExpired(): void {
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
            this.timerInterval = null;
        }
        alert("Time limit reached. Submitting your diagnostic assessment.");
        this.submitTest();
    }

    public renderPalette(): void {
        const gridEl = document.getElementById("diagPaletteGrid");
        const countEl = document.getElementById("diagAnsweredCount");
        if (!gridEl) return;

        gridEl.innerHTML = "";
        const total = this.session.questions.length;
        let answeredCount = 0;

        for (let i = 0; i < total; i++) {
            const btn = document.createElement("button");
            btn.className = "diag-palette-btn";
            btn.textContent = String(i + 1);

            const isAnswered = this.session.answers[i] && this.session.answers[i].answer.trim().length > 0;
            const isMarked = this.session.marked_for_review.includes(i);
            const isActive = i === this.currentIndex;

            if (isAnswered) {
                btn.classList.add("answered");
                answeredCount++;
            }
            if (isMarked) {
                btn.classList.add("marked");
            }
            if (isActive) {
                btn.classList.add("active");
            }

            btn.addEventListener("click", () => {
                this.recordTimeForCurrentQuestion();
                this.goToQuestion(i);
            });

            gridEl.appendChild(btn);
        }

        if (countEl) {
            countEl.textContent = `${answeredCount}/${total} Answered`;
        }
    }

    public renderCurrentQuestion(): void {
        const cardEl = document.getElementById("diagQuestionCard");
        if (!cardEl) return;

        const q = this.session.questions[this.currentIndex];
        if (!q) return;

        const isMarked = this.session.marked_for_review.includes(this.currentIndex);
        const existingAns = this.session.answers[this.currentIndex]?.answer || "";

        // Check if MCQ
        const options = q.instance.parameters?.options ||
            q.instance.metadata?.options ||
            q.instance.parameters?.choices ||
            q.instance.metadata?.choices;
        const isMCQ = Array.isArray(options) && options.length > 0;

        const diffBadge = this.getDifficultyBadge(q.difficulty_level);
        const domainBadge = (q.domain || "general").toUpperCase();
        const chapter = q.instance.metadata?.chapter || q.schema_title || "General";
        const topic = q.instance.metadata?.topic || q.skill_id || "";

        let inputHtml = "";
        if (isMCQ) {
            inputHtml = `
              <div class="diag-options-list" id="diagOptionsContainer">
                ${options
                    .map((opt: any, idx: number) => {
                        const optKey = String.fromCharCode(65 + idx); // A, B, C, D
                        const optVal = typeof opt === "string" ? opt : opt.text || opt.label || JSON.stringify(opt);
                        const isSelected = existingAns === optKey || existingAns === optVal;
                        return `
                          <div class="diag-option-item ${isSelected ? "selected" : ""}" data-key="${optKey}" data-val="${escapeAttr(optVal)}">
                            <div class="diag-option-key">${optKey}</div>
                            <div class="diag-option-text">${escapeHtml(optVal)}</div>
                          </div>
                        `;
                    })
                    .join("")}
              </div>
            `;
        } else {
            inputHtml = `
              <div class="diag-input-box">
                <input type="text" id="diagInputAnswer" class="diag-input" placeholder="Enter your numerical / exact answer..." value="${escapeAttr(existingAns)}" autocomplete="off" />
              </div>
            `;
        }

        cardEl.innerHTML = `
          <div class="diag-q-meta">
            <div class="diag-q-num">Question ${this.currentIndex + 1} of ${this.session.questions.length}</div>
            <div class="diag-q-tags">
              <span class="diag-tag" style="font-weight: 600; color: var(--diag-primary);">${domainBadge}</span>
              <span class="diag-tag">${escapeHtml(chapter)}</span>
              ${topic ? `<span class="diag-tag">${escapeHtml(topic)}</span>` : ""}
              <span class="diag-tag">${diffBadge}</span>
            </div>
          </div>
          <div class="diag-q-prompt">${q.instance.rendered_prompt}</div>
          ${inputHtml}
        `;

        // Trigger MathJax if available
        if (typeof (window as any).MathJax !== "undefined" && (window as any).MathJax.typesetPromise) {
            (window as any).MathJax.typesetPromise([cardEl]).catch((e: any) => console.warn("MathJax error:", e));
        }

        // Attach MCQ click events
        if (isMCQ) {
            const optionItems = cardEl.querySelectorAll(".diag-option-item");
            optionItems.forEach((item) => {
                item.addEventListener("click", () => {
                    optionItems.forEach((o) => o.classList.remove("selected"));
                    item.classList.add("selected");
                    const key = item.getAttribute("data-key") || "";
                    this.saveAnswer(key);
                });
            });
        } else {
            const inputEl = document.getElementById("diagInputAnswer") as HTMLInputElement;
            if (inputEl) {
                inputEl.focus();
                inputEl.addEventListener("input", () => {
                    this.saveAnswer(inputEl.value);
                });
                inputEl.addEventListener("keydown", (e) => {
                    if (e.key === "Enter") {
                        this.nextQuestion();
                    }
                });
            }
        }

        // Update Mark button label
        const markBtn = document.getElementById("diagMarkBtn");
        if (markBtn) {
            markBtn.textContent = isMarked ? "★ Unmark Review" : "★ Mark for Review";
        }

        // Update Prev / Next buttons state
        const prevBtn = document.getElementById("diagPrevBtn") as HTMLButtonElement;
        const nextBtn = document.getElementById("diagNextBtn") as HTMLButtonElement;
        if (prevBtn) {
            prevBtn.disabled = this.currentIndex === 0;
        }
        if (nextBtn) {
            nextBtn.textContent = this.currentIndex === this.session.questions.length - 1 ? "Review / Submit" : "Next \u2192";
        }

        this.renderPalette();
    }

    private getDifficultyBadge(diff: number): string {
        switch (diff) {
            case 1:
                return "Level 1: Foundational";
            case 2:
                return "Level 2: Standard";
            case 3:
                return "Level 3: Multi-Step";
            case 4:
                return "Level 4: Advanced";
            default:
                return "Level 5: Transfer Challenge";
        }
    }

    public saveAnswer(answerText: string): void {
        const timeTaken = this.calcTimeTakenForCurrent();
        this.session.answers[this.currentIndex] = {
            question_index: this.currentIndex,
            answer: answerText,
            time_taken_ms: timeTaken,
            timestamp_ms: Date.now(),
        };
        this.renderPalette();
    }

    private calcTimeTakenForCurrent(): number {
        const start = this.questionStartTimes.get(this.currentIndex) || Date.now();
        const prev = this.session.answers[this.currentIndex]?.time_taken_ms || 0;
        const elapsed = Math.max(0, Date.now() - start);
        return prev + elapsed;
    }

    private recordTimeForCurrentQuestion(): void {
        const currentAns = this.session.answers[this.currentIndex];
        if (currentAns) {
            currentAns.time_taken_ms = this.calcTimeTakenForCurrent();
        }
        this.questionStartTimes.set(this.currentIndex, Date.now());
    }

    public goToQuestion(index: number): void {
        if (index >= 0 && index < this.session.questions.length) {
            this.currentIndex = index;
            this.questionStartTimes.set(this.currentIndex, Date.now());
            this.renderCurrentQuestion();
        }
    }

    public nextQuestion(): void {
        this.recordTimeForCurrentQuestion();
        if (this.currentIndex < this.session.questions.length - 1) {
            this.goToQuestion(this.currentIndex + 1);
        } else {
            // Reached end, prompt submit
            const answeredCount = Object.keys(this.session.answers).filter(
                (k) => this.session.answers[Number(k)]?.answer?.trim()
            ).length;
            const total = this.session.questions.length;
            if (confirm(`You have answered ${answeredCount} of ${total} questions. Would you like to submit now?`)) {
                this.submitTest();
            }
        }
    }

    public prevQuestion(): void {
        this.recordTimeForCurrentQuestion();
        if (this.currentIndex > 0) {
            this.goToQuestion(this.currentIndex - 1);
        }
    }

    public toggleMarkForReview(): void {
        const idx = this.session.marked_for_review.indexOf(this.currentIndex);
        if (idx >= 0) {
            this.session.marked_for_review.splice(idx, 1);
        } else {
            this.session.marked_for_review.push(this.currentIndex);
        }
        this.renderCurrentQuestion();
    }

    public clearAnswer(): void {
        delete this.session.answers[this.currentIndex];
        this.renderCurrentQuestion();
    }

    public submitTest(): void {
        this.recordTimeForCurrentQuestion();
        if (this.timerInterval) {
            clearInterval(this.timerInterval);
            this.timerInterval = null;
        }
        this.session.is_submitted = true;
        this.session.end_time_ms = Date.now();

        // Send submission to Rust / Python backend bridge
        bridgeCommand("diagnosticSubmit", {
            session_id: this.session.session_id,
            answers: this.session.answers,
            end_time_ms: this.session.end_time_ms,
        }).catch((err) => {
            console.error("Bridge command failed, proceeding with client scoring fallback:", err);
        });
    }

    private attachEventListeners(): void {
        const nextBtn = document.getElementById("diagNextBtn");
        const prevBtn = document.getElementById("diagPrevBtn");
        const markBtn = document.getElementById("diagMarkBtn");
        const clearBtn = document.getElementById("diagClearBtn");
        const submitBtn = document.getElementById("diagSubmitBtn");

        if (nextBtn) nextBtn.addEventListener("click", () => this.nextQuestion());
        if (prevBtn) prevBtn.addEventListener("click", () => this.prevQuestion());
        if (markBtn) markBtn.addEventListener("click", () => this.toggleMarkForReview());
        if (clearBtn) clearBtn.addEventListener("click", () => this.clearAnswer());
        if (submitBtn) {
            submitBtn.addEventListener("click", () => {
                if (confirm("Are you sure you want to submit your diagnostic assessment?")) {
                    this.submitTest();
                }
            });
        }
    }

    private attachKeyboardShortcuts(): void {
        this.keydownListener = (e: KeyboardEvent) => {
            const activeEl = document.activeElement;
            const isTextInput = activeEl instanceof HTMLInputElement || activeEl instanceof HTMLTextAreaElement;

            // Handle 1-4 and A-D option selection for MCQ when not focused on a text input
            if (!isTextInput) {
                const q = this.session.questions[this.currentIndex];
                const options = q?.instance.parameters?.options ||
                    q?.instance.metadata?.options ||
                    q?.instance.parameters?.choices ||
                    q?.instance.metadata?.choices;
                const isMCQ = Array.isArray(options) && options.length > 0;

                if (isMCQ) {
                    let selectedIdx = -1;
                    if (e.key >= "1" && e.key <= "4") {
                        selectedIdx = parseInt(e.key, 10) - 1;
                    } else if (e.key.toUpperCase() >= "A" && e.key.toUpperCase() <= "D") {
                        selectedIdx = e.key.toUpperCase().charCodeAt(0) - 65;
                    }

                    if (selectedIdx >= 0 && selectedIdx < options.length) {
                        const optKey = String.fromCharCode(65 + selectedIdx);
                        this.saveAnswer(optKey);
                        this.renderCurrentQuestion();
                        e.preventDefault();
                        return;
                    }
                }

                // 'm' or 'M' to toggle mark for review
                if (e.key === "m" || e.key === "M") {
                    this.toggleMarkForReview();
                    e.preventDefault();
                    return;
                }

                // Arrow navigation
                if (e.key === "ArrowLeft" && e.altKey) {
                    this.prevQuestion();
                    e.preventDefault();
                    return;
                }
                if (e.key === "ArrowRight" && e.altKey) {
                    this.nextQuestion();
                    e.preventDefault();
                    return;
                }
            }
        };

        window.addEventListener("keydown", this.keydownListener);
    }
}

function escapeAttr(str: string): string {
    return str.replace(/"/g, "&quot;").replace(/'/g, "&#39;");
}

function escapeHtml(str: unknown): string {
    return String(str ?? "")
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

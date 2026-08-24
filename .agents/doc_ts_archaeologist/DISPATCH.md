## 2026-08-24T20:32:50Z

MISSION:
Conduct an exhaustive fact-finding audit of the TypeScript frontend reviewer (`ts/reviewer/`, `ts/reviewer/components/`, `ts/reviewer/procedural.ts`, tests).

GROUND TRUTH AREAS TO PROBE:
1. Reviewer Component Architecture: `mcq_container.ts`, `numerical_container.ts`, `stepwise_container.ts`, `mistake_footer.ts`, `procedural.ts`.
2. State Machine & Transitions: States (Question/Input, Answer/Feedback, Mistake Classification, Teardown), transition events, lifecycle hooks.
3. Answer Modalities:
   - MCQ: Option rendering, keyboard shortcuts (A-D, 1-4), canonical selection, absence of free text input.
   - Numerical: Unit parsing, tolerance checking, scientific notation support (`1.2e-3`), physics/chemistry units.
   - Stepwise: Step addition, validation calls, feedback display.
4. Native Anki Footer & Mistake Flow: Compact footer `[1 Silly]`, `[2 Pattern]`, `[3 Concept]`, `[4 Unknown]`, DOM injection, isolation, non-interference with standard Anki flashcards.
5. Teardown Lifecycle: MutationObserver, memory leak prevention, cleanup on card transitions.
6. Tests: Enumerate all Jest/Vitest TS tests and assertions.

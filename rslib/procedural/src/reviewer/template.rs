// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::scheduling::PracticeSessionObject;

/// Safely escapes HTML special characters to prevent XSS attacks while preserving
/// valid LaTeX formulas for MathJax processing.
pub fn escape_html(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

/// Safely escapes JSON strings for insertion inside `<script>` blocks to prevent
/// `</script>` tag breakout attacks.
pub fn escape_json_for_script(json: &str) -> String {
    json.replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
}

/// Renders native HTML/CSS/JS for displaying procedural learning objects inside an Anki webview.
/// Seamlessly handles standard procedural practice, ConceptChecks, StrategyDrills, WorkedExamples,
/// DeclarativeRecall bridges, and Prerequisite reviews, hooking directly into Anki's design tokens
/// and `globalThis.anki.procedural` API.
pub fn render_reviewer_html(session: &PracticeSessionObject) -> String {
    let prompt_text = escape_html(&session.instance.rendered_prompt);
    let family_id_attr = escape_html(session.instance.family_id.as_str());
    let instance_id_attr = escape_html(session.instance.id.as_str());
    let family_id_js = escape_json_for_script(session.instance.family_id.as_str());
    let instance_id_js = escape_json_for_script(session.instance.id.as_str());
    let skill_id_js = escape_json_for_script(session.schema.skill_id.as_str());
    let schema_id_js = escape_json_for_script(session.schema.id.as_str());

    let object_type = session
        .instance
        .metadata
        .get("object_type")
        .and_then(|v| v.as_str())
        .unwrap_or("problem");

    let remediation_message = session
        .instance
        .metadata
        .get("remediation_message")
        .and_then(|v| v.as_str());

    let target_time_ms = session
        .target_latency_ms
        .or_else(|| {
            session
                .instance
                .metadata
                .get("target_time_ms")
                .and_then(|v| v.as_u64())
        })
        .unwrap_or(45_000);

    let difficulty_level = session
        .difficulty_level
        .or_else(|| {
            session
                .instance
                .metadata
                .get("difficulty")
                .and_then(|v| v.as_f64())
                .map(|d| d.round() as u32)
        })
        .unwrap_or(2);

    let difficulty_badge_text = match difficulty_level {
        1 => "Level 1: Foundational",
        2 => "Level 2: Standard",
        3 => "Level 3: Multi-Step",
        4 => "Level 4: Advanced",
        _ => "Level 5: Transfer Challenge",
    };
    let difficulty_badge_text = escape_html(difficulty_badge_text);

    let raw_variant = session
        .selected_variant
        .as_deref()
        .or_else(|| {
            session
                .instance
                .parameters
                .get("variant")
                .and_then(|v| v.as_str())
        })
        .unwrap_or("standard")
        .replace('_', " ");
    let variant_label = escape_html(&raw_variant);

    let canonical_str = if let Some(f) = session.instance.correct_answer.get("formatted").and_then(|v| v.as_str()) {
        f.to_string()
    } else if let Some(co) = session.instance.correct_answer.get("correct_option").and_then(|v| v.as_str()) {
        co.to_string()
    } else if let Some(val) = session.instance.correct_answer.get("value") {
        if let Some(s) = val.as_str() {
            s.to_string()
        } else if let Some(n) = val.as_f64() {
            format!("{n}")
        } else {
            val.to_string()
        }
    } else if let Some(ans) = session.instance.correct_answer.get("answer") {
        if let Some(s) = ans.as_str() {
            s.to_string()
        } else if let Some(n) = ans.as_f64() {
            format!("{n}")
        } else {
            ans.to_string()
        }
    } else {
        "".to_string()
    };
    let canonical_text = escape_html(&canonical_str);

    let raw_solution = session
        .instance
        .correct_answer
        .get("solution")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let solution_text = escape_html(raw_solution);

    let solution_graph_raw = session
        .instance
        .solution_graph()
        .map(|g| serde_json::to_string(&g).unwrap_or_else(|_| "null".to_string()))
        .unwrap_or_else(|| "null".to_string());
    let solution_graph_json = escape_json_for_script(&solution_graph_raw);

    let canonical_json = escape_json_for_script(
        &serde_json::to_string(&session.instance.correct_answer).unwrap_or_default(),
    );
    let parameters_json = escape_json_for_script(
        &serde_json::to_string(&session.instance.parameters).unwrap_or_default(),
    );

    // Optional provenance badge
    let provenance_badge = if let Some(prov) = session.instance.metadata.get("provenance") {
        let exam = prov.get("exam").and_then(|v| v.as_str());
        let year = prov.get("year").and_then(|v| v.as_u64());
        let shift = prov.get("shift").and_then(|v| v.as_str());
        let variant_type = prov.get("variant_type").and_then(|v| v.as_str()).unwrap_or("");
        
        let prov_label = match (exam, year) {
            (Some(e), Some(y)) => {
                if let Some(s) = shift {
                    format!("PYQ: {} {} · {}", e, y, s)
                } else {
                    format!("PYQ: {} {}", e, y)
                }
            }
            (Some(e), None) => format!("PYQ: {}", e),
            _ => {
                if !variant_type.is_empty() {
                    format!("Variant: {}", variant_type.replace('_', " "))
                } else {
                    "".to_string()
                }
            }
        };

        if !prov_label.is_empty() {
            format!("<span class=\"proc-pyq-badge\">{}</span>", escape_html(&prov_label))
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    // Remediation transparency banner
    let transparency_html = if let Some(msg) = remediation_message {
        format!("<div class=\"proc-transparency-banner\">{}</div>", escape_html(msg))
    } else {
        "".to_string()
    };

    // Body content according to learning object type
    let main_body_html = match object_type {
        "mcq" => {
            let options_html = if let Some(opts) = session.instance.parameters.get("options").and_then(|v| v.as_array()) {
                let mut s = String::new();
                for (i, opt) in opts.iter().enumerate() {
                    let letter = (b'A' + (i as u8).min(25)) as char;
                    let opt_text = opt.as_str().unwrap_or("");
                    s.push_str(&format!(
                        r#"<button type="button" class="proc-option-item" data-opt-id="{}" data-opt-idx="{}" role="radio" aria-checked="false">
                            <div class="proc-option-header">
                                <span class="proc-option-key">{}</span>
                                <span class="proc-option-label">{}</span>
                            </div>
                            <div class="proc-option-feedback hidden"></div>
                        </button>"#,
                        escape_html(opt_text),
                        i,
                        letter,
                        escape_html(opt_text)
                    ));
                }
                s
            } else {
                "".to_string()
            };

            format!(
                r#"<div class="proc-prompt">{prompt_text}</div>
                <div class="proc-option-group" role="radiogroup" aria-label="Multiple choice options">
                    {options_html}
                </div>"#
            )
        }
        "concept_check" => {
            let options_html = if let Some(cc) = session.instance.metadata.get("concept_check") {
                if let Some(opts) = cc.get("options").and_then(|v| v.as_array()) {
                    let mut s = String::new();
                    for (i, opt) in opts.iter().enumerate() {
                        let opt_id = opt.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let label = opt.get("label").and_then(|v| v.as_str()).unwrap_or("");
                        let feedback = opt.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
                        s.push_str(&format!(
                            r#"<button type="button" class="proc-option-item" data-opt-id="{}" role="radio" aria-checked="false">
                                <div class="proc-option-header">
                                    <span class="proc-option-key">{}</span>
                                    <span class="proc-option-label">{}</span>
                                </div>
                                <div class="proc-option-feedback hidden">{}</div>
                            </button>"#,
                            escape_html(opt_id),
                            i + 1,
                            escape_html(label),
                            escape_html(feedback)
                        ));
                    }
                    s
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            format!(
                r#"<div class="proc-prompt">{prompt_text}</div>
                <div class="proc-option-group" role="radiogroup" aria-label="Concept check options">
                    {options_html}
                </div>"#
            )
        }
        "strategy_drill" => {
            let problem_context = session
                .instance
                .metadata
                .get("strategy_drill")
                .and_then(|sd| sd.get("problem_context"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let context_html = if !problem_context.is_empty() {
                format!(r#"<div class="proc-solution" style="margin-bottom: 12px;"><strong>Problem:</strong> {}</div>"#, escape_html(problem_context))
            } else {
                "".to_string()
            };

            let options_html = if let Some(sd) = session.instance.metadata.get("strategy_drill") {
                if let Some(opts) = sd.get("options").and_then(|v| v.as_array()) {
                    let mut s = String::new();
                    for (i, opt) in opts.iter().enumerate() {
                        let opt_id = opt.get("id").and_then(|v| v.as_str()).unwrap_or("");
                        let label = opt.get("label").and_then(|v| v.as_str()).unwrap_or("");
                        let feedback = opt.get("feedback").and_then(|v| v.as_str()).unwrap_or("");
                        s.push_str(&format!(
                            r#"<button type="button" class="proc-option-item" data-opt-id="{}" role="radio" aria-checked="false">
                                <div class="proc-option-header">
                                    <span class="proc-option-key">{}</span>
                                    <span class="proc-option-label">{}</span>
                                </div>
                                <div class="proc-option-feedback hidden">{}</div>
                            </button>"#,
                            escape_html(opt_id),
                            i + 1,
                            escape_html(label),
                            escape_html(feedback)
                        ));
                    }
                    s
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            format!(
                r#"{context_html}
                <div class="proc-prompt">{prompt_text}</div>
                <div class="proc-option-group" role="radiogroup" aria-label="Strategy options">
                    {options_html}
                </div>"#
            )
        }
        "worked_example" => {
            let we = session.instance.metadata.get("worked_example");
            let decision_point = we.and_then(|w| w.get("highlighted_decision_point")).and_then(|v| v.as_str()).unwrap_or("");
            let rationale = we.and_then(|w| w.get("method_rationale")).and_then(|v| v.as_str()).unwrap_or("");
            let steps_html = if let Some(steps) = we.and_then(|w| w.get("canonical_steps")).and_then(|v| v.as_array()) {
                let mut s = String::new();
                for step in steps {
                    if let Some(txt) = step.as_str() {
                        s.push_str(&format!("<li>{}</li>", escape_html(txt)));
                    }
                }
                format!("<ol style=\"margin: 8px 0; padding-left: 20px;\">{}</ol>", s)
            } else {
                "".to_string()
            };

            let pitfalls_html = if let Some(pits) = we.and_then(|w| w.get("common_mistakes_to_avoid")).and_then(|v| v.as_array()) {
                if !pits.is_empty() {
                    let mut s = String::new();
                    for p in pits {
                        if let Some(txt) = p.as_str() {
                            s.push_str(&format!("<li>{}</li>", escape_html(txt)));
                        }
                    }
                    format!("<div class=\"proc-pitfall-box\"><strong>⚠️ Common Pitfalls:</strong><ul style=\"margin: 4px 0; padding-left: 18px;\">{}</ul></div>", s)
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            };

            format!(
                r#"<div class="proc-prompt">{prompt_text}</div>
                <div class="proc-worked-example-card">
                    <div class="proc-decision-highlight">🎯 <strong>Key Decision:</strong> {}</div>
                    <div style="font-weight: 600; margin-top: 10px;">Canonical Solution Steps:</div>
                    {steps_html}
                    <div class="proc-solution" style="margin-top: 10px;"><strong>Method Rationale:</strong> {}</div>
                    {pitfalls_html}
                    <div style="margin-top: 16px; display: flex; gap: 10px;">
                        <button type="button" id="proc-try-similar-btn" class="proc-btn">Try Similar Problem</button>
                        <button type="button" id="proc-next-btn" class="proc-btn proc-btn-secondary">Continue</button>
                    </div>
                </div>"#,
                escape_html(decision_point),
                escape_html(rationale)
            )
        }
        "declarative_recall" => {
            let dr = session.instance.metadata.get("declarative_recall");
            let concept_name = dr.and_then(|d| d.get("concept_name")).and_then(|v| v.as_str()).unwrap_or("");
            let formula = dr.and_then(|d| d.get("formula_or_fact")).and_then(|v| v.as_str()).unwrap_or("");

            format!(
                r#"<div class="proc-prompt"><strong>Prerequisite Concept:</strong> {}</div>
                <div class="proc-worked-example-card" style="text-align: center;">
                    <div style="font-size: 1.15rem; font-weight: 700; margin: 12px 0;">{}</div>
                    <div style="margin-top: 18px; display: flex; justify-content: center; gap: 12px;">
                        <button type="button" id="proc-anki-recall-btn" class="proc-btn">Review in Anki</button>
                        <button type="button" id="proc-next-btn" class="proc-btn proc-btn-secondary">Got It</button>
                    </div>
                </div>"#,
                escape_html(concept_name),
                escape_html(formula)
            )
        }
        "prerequisite_review" => {
            let pr = session.instance.metadata.get("prerequisite_review");
            let advisory = pr.and_then(|p| p.get("advisory_message")).and_then(|v| v.as_str()).unwrap_or(&prompt_text);

            format!(
                r#"<div class="proc-worked-example-card">
                    <div style="font-weight: 600; font-size: 1rem; color: #b91c1c;">⚠️ Foundational Skill Needed</div>
                    <div style="margin-top: 8px; line-height: 1.5;">{}</div>
                    <div style="margin-top: 16px; display: flex; gap: 10px;">
                        <button type="button" id="proc-practice-prereq-btn" class="proc-btn">Practice Prerequisite</button>
                        <button type="button" id="proc-next-btn" class="proc-btn proc-btn-secondary">Continue</button>
                    </div>
                </div>"#,
                escape_html(advisory)
            )
        }
        _ => {
            // Standard Quick / Stepwise Problem
            format!(
                r#"<div class="proc-prompt">{prompt_text}</div>

                <div class="proc-mode-switch">
                    <button type="button" id="tab-quick" class="proc-tab active">Quick Solve</button>
                    <button type="button" id="tab-stepwise" class="proc-tab">Step-by-Step Solve</button>
                </div>

                <!-- Quick Solve Mode -->
                <div id="proc-quick-container">
                    <div class="proc-step-row">
                        <input type="text" id="proc-answer-input" class="proc-input" placeholder="Type final answer..." autocomplete="off" />
                        <button type="button" id="proc-submit-btn" class="proc-btn">Submit</button>
                    </div>
                </div>

                <!-- Stepwise Solving Mode -->
                <div id="proc-stepwise-container" class="hidden">
                    <div id="proc-steps-list">
                        <div class="proc-step-row" data-step-idx="0">
                            <span class="proc-step-label">Step 1</span>
                            <input type="text" class="proc-input proc-step-input" placeholder="Write step 1 transformation or equation..." autocomplete="off" />
                        </div>
                    </div>
                    <div class="proc-controls">
                        <button type="button" id="proc-add-step-btn" class="proc-btn proc-btn-secondary">+ Add Step</button>
                        <button type="button" id="proc-hint-btn" class="proc-btn proc-btn-secondary">💡 Request Hint</button>
                        <button type="button" id="proc-reset-steps-btn" class="proc-btn proc-btn-secondary">Reset</button>
                        <button type="button" id="proc-check-steps-btn" class="proc-btn">Check Solution</button>
                    </div>
                </div>

                <div id="proc-hint-container" class="proc-hint-box hidden"></div>"#
            )
        }
    };

    let target_time_secs = target_time_ms / 1000;
    let full_metadata_json = escape_json_for_script(
        &serde_json::to_string(&session.instance.metadata).unwrap_or_else(|_| "{}".to_string()),
    );

    let domain_meta = session.instance.metadata.get("domain").and_then(|v| v.as_str()).unwrap_or("");
    let chapter_meta = session.instance.metadata.get("chapter").and_then(|v| v.as_str()).unwrap_or("");
    let domain_display = if !domain_meta.is_empty() {
        domain_meta
    } else if session.instance.family_id.as_str().contains("math") {
        "Quantitative Aptitude"
    } else if session.instance.family_id.as_str().contains("physics") {
        "Physics"
    } else if session.instance.family_id.as_str().contains("chem") {
        "Chemistry"
    } else if session.instance.family_id.as_str().contains("reason") {
        "Logical Reasoning"
    } else {
        "StudyLab"
    };

    let title_clean = if session.schema.title.to_lowercase().starts_with("dynamic practice schema for ") {
        let rest = &session.schema.title["dynamic practice schema for ".len()..];
        let last = rest.split('.').last().unwrap_or(rest).replace('_', " ");
        let mut c = last.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    } else {
        session.schema.title.clone()
    };

    let breadcrumbs_html = if !chapter_meta.is_empty() {
        format!(
            r#"<nav class="proc-breadcrumbs" aria-label="Topic breadcrumbs">
                <span class="proc-crumb proc-crumb-domain">{}</span>
                <span class="proc-crumb-sep">›</span>
                <span class="proc-crumb proc-crumb-topic">{}</span>
                <span class="proc-crumb-sep">›</span>
                <span class="proc-crumb proc-crumb-skill">{}</span>
            </nav>"#,
            escape_html(domain_display),
            escape_html(chapter_meta),
            escape_html(&title_clean)
        )
    } else {
        format!(
            r#"<nav class="proc-breadcrumbs" aria-label="Topic breadcrumbs">
                <span class="proc-crumb proc-crumb-domain">{}</span>
                <span class="proc-crumb-sep">›</span>
                <span class="proc-crumb proc-crumb-skill">{}</span>
            </nav>"#,
            escape_html(domain_display),
            escape_html(&title_clean)
        )
    };

    format!(
        r#"<div class="procedural-card-container" id="procedural-card" data-instance-id="{instance_id_attr}" data-family-id="{family_id_attr}" data-target-time="{target_time_ms}">
    {transparency_html}
    <div class="proc-header">
        <div class="proc-header-left">
            {breadcrumbs_html}
            <div class="proc-badges">
                <span class="proc-diff-badge">{difficulty_badge_text}</span>
                <span class="proc-variant-tag">{variant_label}</span>
                {provenance_badge}
            </div>
        </div>
        <span class="proc-timer" id="proc-stopwatch">00:00</span>
    </div>

    {main_body_html}

    <div id="proc-result-panel" class="proc-result hidden">
        <div id="proc-result-title" class="proc-result-title"></div>
        <div id="proc-result-feedback" class="proc-result-feedback"></div>
        <div class="proc-meta-row">
            <span><strong>Target Time:</strong> {target_time_secs}s</span>
            <div id="proc-actual-time"></div>
        </div>
        <div class="proc-expected-row" style="margin-top: 6px;"><strong>Expected Answer:</strong> <span id="proc-expected-ans">{canonical_text}</span></div>

        <!-- Mistake Classification (immediately visible on wrong answer above solution) -->
        <div id="proc-mistake-panel" class="proc-mistake-panel hidden">
            <div class="proc-mistake-heading">Classify Error to Optimize Spaced Repetition:</div>
            <div class="proc-mistake-grid">
                <button type="button" class="proc-mistake-card" data-value="silly_mistake" data-key="1">
                    <span class="proc-key-badge">1</span>
                    <div class="proc-mistake-info">
                        <strong>Silly Mistake</strong>
                        <span>Calculation or reading slip</span>
                    </div>
                </button>
                <button type="button" class="proc-mistake-card" data-value="pattern_not_recognized" data-key="2">
                    <span class="proc-key-badge">2</span>
                    <div class="proc-mistake-info">
                        <strong>Pattern Missed</strong>
                        <span>Failed to identify structure</span>
                    </div>
                </button>
                <button type="button" class="proc-mistake-card" data-value="formula_or_concept_misapplied" data-key="3">
                    <span class="proc-key-badge">3</span>
                    <div class="proc-mistake-info">
                        <strong>Concept Misapplied</strong>
                        <span>Used incorrect formula or rule</span>
                    </div>
                </button>
                <button type="button" class="proc-mistake-card" data-value="concept_not_known" data-key="4">
                    <span class="proc-key-badge">4</span>
                    <div class="proc-mistake-info">
                        <strong>Concept Not Known</strong>
                        <span>Unfamiliar prerequisite topic</span>
                    </div>
                </button>
            </div>
        </div>

        <div id="proc-solution-container" class="proc-solution">
            <strong>Step-by-Step Solution:</strong>
            <div style="margin-top: 6px;">{solution_text}</div>
        </div>
        <div class="proc-result-actions" style="margin-top: 14px;">
            <button type="button" id="proc-next-btn" class="proc-btn proc-btn-primary">Next Problem</button>
        </div>
    </div>

    <script>
    (function() {{
        var meta = {full_metadata_json};
        var options = {{
            containerId: "procedural-card",
            instanceId: "{instance_id_js}",
            familyId: "{family_id_js}",
            skillId: "{skill_id_js}",
            schemaId: "{schema_id_js}",
            targetTimeMs: {target_time_ms},
            correctAnswer: {canonical_json},
            parameters: {parameters_json},
            solutionGraph: {solution_graph_json},
            objectType: "{object_type}",
            conceptCheck: meta.concept_check || null,
            strategyDrill: meta.strategy_drill || null,
            workedExample: meta.worked_example || null,
            declarativeRecall: meta.declarative_recall || null,
            prerequisiteReview: meta.prerequisite_review || null,
            provenance: meta.provenance || null,
            remediationMessage: meta.remediation_message || null
        }};

        if (window.anki && window.anki.procedural && window.anki.procedural.setup) {{
            window.anki.procedural.setup(options);
            return;
        }}

        // Standalone fallback
        var startTime = Date.now();
        var timerEl = document.getElementById('proc-stopwatch');
        var inputEl = document.getElementById('proc-answer-input');
        var submitBtn = document.getElementById('proc-submit-btn');
        var resultPanel = document.getElementById('proc-result-panel');
        var resultTitle = document.getElementById('proc-result-title');
        var feedbackEl = document.getElementById('proc-result-feedback');
        var actualTimeEl = document.getElementById('proc-actual-time');
        var isSubmitted = false;

        var timerInterval = setInterval(function() {{
            if (isSubmitted) return;
            var elapsed = Math.floor((Date.now() - startTime) / 1000);
            var m = String(Math.floor(elapsed / 60)).padStart(2, '0');
            var s = String(elapsed % 60).padStart(2, '0');
            if (timerEl) timerEl.textContent = m + ':' + s;
        }}, 200);

        if (submitBtn && inputEl) {{
            submitBtn.addEventListener('click', function() {{
                isSubmitted = true;
                clearInterval(timerInterval);
                if (resultPanel) resultPanel.classList.remove('hidden');
            }});
        }}
    }})();
    </script>
</div>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Domain;
    use crate::problems::catalog::MathsCatalog;
    use crate::problems::generators::{PercentageSuccessiveConfig, PercentageSuccessiveGenerator};
    use crate::remediation::{ConceptCheckObject, ConceptCheckOption, WorkedExampleObject};

    #[test]
    fn test_render_reviewer_html_contains_critical_elements() {
        let schema = MathsCatalog::successive_percentage_schema();
        let instance = PercentageSuccessiveGenerator::generate_instance(
            &schema.problem_family_id,
            12345,
            &PercentageSuccessiveConfig::default(),
        );

        let session = PracticeSessionObject::new(schema, instance, Some(101), None);
        let html = render_reviewer_html(&session);

        assert!(html.contains("id=\"procedural-card\""));
        assert!(html.contains("id=\"proc-stopwatch\""));
        assert!(html.contains("id=\"proc-answer-input\""));
        assert!(html.contains("id=\"proc-submit-btn\""));
        assert!(html.contains("id=\"proc-result-panel\""));
        assert!(html.contains("id=\"proc-next-btn\""));
        assert!(html.contains("Step-by-Step Solution:"));
        assert!(html.contains("Level"));
        assert!(html.contains("Quick Solve"));
        assert!(html.contains("Step-by-Step Solve"));
        assert!(html.contains("proc-hint-btn"));
        assert!(html.contains("window.anki.procedural.setup"));
    }

    #[test]
    fn test_render_reviewer_html_with_concept_check() {
        let schema = MathsCatalog::successive_percentage_schema();
        let cc = ConceptCheckObject::new(
            "cc-1",
            "skill-successive",
            schema.id.clone(),
            Domain::Mathematics,
            "Which formula represents successive percentage increase?",
            vec![
                ConceptCheckOption::new("opt-1", "a + b + ab/100", true, "formula", "Correct!"),
                ConceptCheckOption::new("opt-2", "a * b / 100", false, "trap", "Wrong!"),
            ],
            "opt-1",
            "Successive percentage formula is a + b + ab/100.",
        );

        let mut instance = PercentageSuccessiveGenerator::generate_instance(
            &schema.problem_family_id,
            123,
            &PercentageSuccessiveConfig::default(),
        );
        instance.rendered_prompt = cc.prompt.clone();
        instance.metadata = serde_json::json!({
            "object_type": "concept_check",
            "concept_check": cc,
            "remediation_message": "💡 Concept Check: Verify the core formula."
        });

        let session = PracticeSessionObject::new(schema, instance, Some(102), None);
        let html = render_reviewer_html(&session);

        assert!(html.contains("proc-option-group"));
        assert!(html.contains("data-opt-id=\"opt-1\""));
        assert!(html.contains("a + b + ab/100"));
        assert!(html.contains("proc-transparency-banner"));
        assert!(html.contains("💡 Concept Check: Verify the core formula."));
    }

    #[test]
    fn test_render_reviewer_html_with_worked_example() {
        let schema = MathsCatalog::linear_equations_schema();
        let we = WorkedExampleObject::new(
            "we-1",
            "skill-linear",
            schema.id.clone(),
            Domain::Mathematics,
            "Solve 2x + 4 = 12",
            "Linear Equation with constants",
            vec![
                "Step 1: Subtract 4 from both sides to get 2x = 8".into(),
                "Step 2: Divide both sides by 2 to get x = 4".into(),
            ],
            "Subtract constant term before dividing coefficient",
            "Isolating variable terms systematically",
            vec!["Dividing before subtracting leading to fractions".into()],
        );

        let mut instance = crate::problems::generators::LinearEquationsGenerator::generate_problem(
            123, 2, None,
        );
        instance.rendered_prompt = we.prompt.clone();
        instance.metadata = serde_json::json!({
            "object_type": "worked_example",
            "worked_example": we,
            "remediation_message": "📖 Step-by-Step Worked Example"
        });

        let session = PracticeSessionObject::new(schema, instance, Some(103), None);
        let html = render_reviewer_html(&session);

        assert!(html.contains("proc-worked-example-card"));
        assert!(html.contains("proc-decision-highlight"));
        assert!(html.contains("Subtract constant term before dividing coefficient"));
        assert!(html.contains("proc-try-similar-btn"));
        assert!(html.contains("proc-pitfall-box"));
    }

    #[test]
    fn test_render_reviewer_html_with_pyq_provenance() {
        let schema = MathsCatalog::linear_equations_schema();
        let mut instance = crate::problems::generators::LinearEquationsGenerator::generate_problem(
            123, 3, None,
        );
        instance.metadata = serde_json::json!({
            "provenance": {
                "exam": "JEE Main",
                "year": 2024,
                "shift": "Shift 1",
                "variant_type": "practice_variant"
            }
        });

        let session = PracticeSessionObject::new(schema, instance, Some(104), None);
        let html = render_reviewer_html(&session);

        assert!(html.contains("proc-pyq-badge"));
        assert!(html.contains("PYQ: JEE Main 2024 · Shift 1"));
    }

    #[test]
    fn test_xss_escaping_and_latex_preservation() {
        let schema = MathsCatalog::linear_equations_schema();
        let mut instance = crate::problems::generators::LinearEquationsGenerator::generate_problem(
            123, 1, None,
        );

        // Inject malicious XSS payloads alongside LaTeX mathematical notation
        instance.rendered_prompt = "Solve for $x$: \\(x < 5\\) & <script>alert('xss')</script> <img src=x onerror=alert(1)>".to_string();
        instance.correct_answer = serde_json::json!({
            "formatted": "x < 5 & \"safe\"",
            "solution": "Step 1: Simplify \\(\\frac{a}{b} < c\\) <svg onload=alert(2)>"
        });

        let session = PracticeSessionObject::new(schema, instance, Some(1), None);
        let html = render_reviewer_html(&session);

        // Malicious executable script/image/svg tags MUST NOT exist unescaped in the output
        assert!(!html.contains("<script>alert('xss')</script>"));
        assert!(!html.contains("<img src=x onerror=alert(1)>"));
        assert!(!html.contains("<svg onload=alert(2)>"));

        // Encoded entities MUST exist
        assert!(html.contains("&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;"));
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(html.contains("&lt;svg onload=alert(2)&gt;"));

        // Mathematical LaTeX markup delimiters MUST remain intact for MathJax
        assert!(html.contains("\\(x &lt; 5\\)"));
        assert!(html.contains("\\(\\frac{a}{b} &lt; c\\)"));
    }

    #[test]
    fn test_escape_json_for_script_prevents_breakout() {
        let raw_json = r#"{"title":"</script><script>alert('breakout')</script>"}"#;
        let escaped = escape_json_for_script(raw_json);
        assert!(!escaped.contains("</script>"));
        assert!(escaped.contains(r#"\u003c/script\u003e"#));
    }
}

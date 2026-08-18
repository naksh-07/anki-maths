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

/// Renders native HTML/CSS/JS for displaying a procedural practice problem inside an Anki webview.
/// Supports both fast final-answer submission and rich stepwise solving with progressive hints,
/// hooking directly into Anki's design tokens and `globalThis.anki.procedural` API.
pub fn render_reviewer_html(session: &PracticeSessionObject) -> String {
    let prompt_text = escape_html(&session.instance.rendered_prompt);
    let title = escape_html(&session.schema.title);
    let family_id_attr = escape_html(session.instance.family_id.as_str());
    let instance_id_attr = escape_html(session.instance.id.as_str());
    let family_id_js = escape_json_for_script(session.instance.family_id.as_str());
    let instance_id_js = escape_json_for_script(session.instance.id.as_str());

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

    let raw_canonical = session
        .instance
        .correct_answer
        .get("formatted")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let canonical_text = escape_html(raw_canonical);

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

    let target_time_secs = target_time_ms / 1000;

    let difficulty_badge_text = match difficulty_level {
        1 => "Level 1: Foundational",
        2 => "Level 2: Standard",
        3 => "Level 3: Multi-Step",
        4 => "Level 4: Advanced",
        _ => "Level 5: Transfer Challenge",
    };
    let difficulty_badge_text = escape_html(difficulty_badge_text);

    format!(
        r#"<div class="procedural-card-container" id="procedural-card" data-instance-id="{instance_id_attr}" data-family-id="{family_id_attr}" data-target-time="{target_time_ms}">
    <div class="proc-header">
        <div class="proc-badges">
            <span class="proc-badge">{title}</span>
            <span class="proc-diff-badge">{difficulty_badge_text}</span>
            <span class="proc-variant-tag">{variant_label}</span>
        </div>
        <span class="proc-timer" id="proc-stopwatch">00:00</span>
    </div>

    <div class="proc-prompt">{prompt_text}</div>

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

    <div id="proc-hint-container" class="proc-hint-box hidden"></div>

    <div id="proc-result-panel" class="proc-result hidden">
        <div id="proc-result-title" style="font-weight: 700; font-size: 1.1rem; margin-bottom: 8px;"></div>
        <div id="proc-result-feedback" style="margin-bottom: 8px;"></div>
        <div class="proc-meta-row">
            <span><strong>Target Time:</strong> {target_time_secs}s</span>
            <span id="proc-actual-time"></span>
        </div>
        <div style="margin-top: 6px;"><strong>Expected Answer:</strong> <span id="proc-expected-ans">{canonical_text}</span></div>
        <div id="proc-solution-container" class="proc-solution">
            <strong>Step-by-Step Solution:</strong>
            <div style="margin-top: 6px;">{solution_text}</div>
        </div>
        <div style="margin-top: 14px;">
            <button type="button" id="proc-next-btn" class="proc-btn" style="background: var(--button-bg, #4b5563); color: var(--fg, #ffffff);">Next Problem</button>
        </div>
    </div>

    <script>
    (function() {{
        var options = {{
            containerId: "procedural-card",
            instanceId: "{instance_id_js}",
            familyId: "{family_id_js}",
            targetTimeMs: {target_time_ms},
            correctAnswer: {canonical_json},
            parameters: {parameters_json},
            solutionGraph: {solution_graph_json}
        }};

        if (window.anki && window.anki.procedural && window.anki.procedural.setup) {{
            window.anki.procedural.setup(options);
            return;
        }}

        // Self-contained standalone fallback
        var startTime = Date.now();
        var timerEl = document.getElementById('proc-stopwatch');
        var inputEl = document.getElementById('proc-answer-input');
        var submitBtn = document.getElementById('proc-submit-btn');
        var resultPanel = document.getElementById('proc-result-panel');
        var resultTitle = document.getElementById('proc-result-title');
        var feedbackEl = document.getElementById('proc-result-feedback');
        var actualTimeEl = document.getElementById('proc-actual-time');
        var quickContainer = document.getElementById('proc-quick-container');
        var stepwiseContainer = document.getElementById('proc-stepwise-container');
        var tabQuick = document.getElementById('tab-quick');
        var tabStepwise = document.getElementById('tab-stepwise');
        var stepsList = document.getElementById('proc-steps-list');
        var addStepBtn = document.getElementById('proc-add-step-btn');
        var hintBtn = document.getElementById('proc-hint-btn');
        var resetBtn = document.getElementById('proc-reset-steps-btn');
        var checkStepsBtn = document.getElementById('proc-check-steps-btn');
        var hintBox = document.getElementById('proc-hint-container');
        var nextBtn = document.getElementById('proc-next-btn');

        var correctData = options.correctAnswer;
        var solutionGraph = options.solutionGraph;
        var targetTimeMs = options.targetTimeMs;
        var isSubmitted = false;
        var hintsUsed = 0;
        var activeMode = 'quick';

        var timerInterval = setInterval(function() {{
            if (isSubmitted) return;
            var elapsed = Math.floor((Date.now() - startTime) / 1000);
            var m = String(Math.floor(elapsed / 60)).padStart(2, '0');
            var s = String(elapsed % 60).padStart(2, '0');
            if (timerEl) timerEl.textContent = m + ':' + s;
        }}, 200);

        if (tabQuick) {{
            tabQuick.addEventListener('click', function() {{
                activeMode = 'quick';
                tabQuick.classList.add('active');
                tabStepwise.classList.remove('active');
                quickContainer.classList.remove('hidden');
                stepwiseContainer.classList.add('hidden');
                if (inputEl) inputEl.focus();
            }});
        }}

        if (tabStepwise) {{
            tabStepwise.addEventListener('click', function() {{
                activeMode = 'stepwise';
                tabStepwise.classList.add('active');
                tabQuick.classList.remove('active');
                stepwiseContainer.classList.remove('hidden');
                quickContainer.classList.add('hidden');
                var firstInput = stepsList ? stepsList.querySelector('input') : null;
                if (firstInput) firstInput.focus();
            }});
        }}

        if (addStepBtn && stepsList) {{
            addStepBtn.addEventListener('click', function() {{
                var currentSteps = stepsList.querySelectorAll('.proc-step-row').length;
                var newRow = document.createElement('div');
                newRow.className = 'proc-step-row';
                newRow.dataset.stepIdx = currentSteps;
                newRow.innerHTML = '<span class="proc-step-label">Step ' + (currentSteps + 1) + '</span>' +
                    '<input type="text" class="proc-input proc-step-input" placeholder="Write step ' + (currentSteps + 1) + ' transformation..." autocomplete="off" />';
                stepsList.appendChild(newRow);
                var newInput = newRow.querySelector('input');
                if (newInput) newInput.focus();
            }});
        }}

        if (resetBtn && stepsList) {{
            resetBtn.addEventListener('click', function() {{
                stepsList.innerHTML = '<div class="proc-step-row" data-step-idx="0">' +
                    '<span class="proc-step-label">Step 1</span>' +
                    '<input type="text" class="proc-input proc-step-input" placeholder="Write step 1 transformation or equation..." autocomplete="off" /></div>';
                if (hintBox) {{
                    hintBox.classList.add('hidden');
                    hintBox.innerHTML = '';
                }}
            }});
        }}

        if (hintBtn) {{
            hintBtn.addEventListener('click', function() {{
                hintsUsed++;
                var hintText = "";
                if (solutionGraph && solutionGraph.steps && solutionGraph.steps.length > 0) {{
                    var step = solutionGraph.steps[0];
                    if (solutionGraph.steps.length >= hintsUsed) {{
                        step = solutionGraph.steps[hintsUsed - 1];
                    }}
                    if (step.hints && step.hints.length > 0) {{
                        var hObj = step.hints[(hintsUsed - 1) % step.hints.length];
                        hintText = '<strong>' + (hObj.title || 'Hint') + ':</strong> ' + hObj.content;
                    }} else {{
                        hintText = '<strong>Hint ' + hintsUsed + ':</strong> ' + step.description;
                    }}
                }} else {{
                    hintText = '<strong>Hint:</strong> Focus on identifying the primary mathematical relation and inverse operation.';
                }}

                if (hintBox) {{
                    hintBox.classList.remove('hidden');
                    hintBox.innerHTML = '<div>💡 ' + hintText + '</div><div style="font-size:0.75rem; opacity:0.8; margin-top:4px;">(Hints requested: ' + hintsUsed + ')</div>';
                }}
            }});
        }}

        function parseNum(val) {{
            if (!val) return null;
            var cleaned = String(val).replace(/[$€£₹%, ]/g, '').trim();
            if (cleaned.indexOf('/') !== -1) {{
                var parts = cleaned.split('/');
                var num = parseFloat(parts[0]);
                var den = parseFloat(parts[1]);
                if (!isNaN(num) && !isNaN(den) && den !== 0) return num / den;
            }}
            var n = parseFloat(cleaned);
            return isNaN(n) ? null : n;
        }}

        function evaluateLocally(userText) {{
            var expectedVal = correctData.value;
            var userNum = parseNum(userText);
            if (expectedVal !== undefined && typeof expectedVal === 'number') {{
                if (userNum === null) {{
                    return {{ correct: false, reason: "Please enter a valid numeric value." }};
                }}
                var diff = Math.abs(userNum - expectedVal);
                var isCorrect = diff <= Math.max(0.01, Math.abs(expectedVal) * 0.01);
                return {{ correct: isCorrect, userNum: userNum, expectedVal: expectedVal }};
            }}
            var canonicalStr = String(correctData.formatted || "").trim().toLowerCase();
            var userStr = String(userText).trim().toLowerCase();
            return {{ correct: userStr === canonicalStr && canonicalStr.length > 0 }};
        }}

        function finishAttempt(outcome, submittedData) {{
            isSubmitted = true;
            clearInterval(timerInterval);
            var timeTakenMs = Date.now() - startTime;

            if (resultPanel) resultPanel.classList.remove('hidden');
            if (quickContainer) quickContainer.classList.add('hidden');
            if (stepwiseContainer) stepwiseContainer.classList.add('hidden');
            var switchEl = document.querySelector('.proc-mode-switch');
            if (switchEl) switchEl.classList.add('hidden');

            if (actualTimeEl) {{
                actualTimeEl.innerHTML = '<strong>Actual Time:</strong> ' + (timeTakenMs / 1000).toFixed(1) + 's';
            }}

            if (resultPanel) {{
                if (outcome.correct) {{
                    resultPanel.className = 'proc-result correct';
                    if (resultTitle) resultTitle.textContent = '✓ Correct Answer';
                    var timeMsg = 'Completed in ' + (timeTakenMs / 1000).toFixed(1) + 's';
                    if (timeTakenMs > targetTimeMs) {{
                        timeMsg += ' (Over target latency of ' + (targetTimeMs / 1000).toFixed(0) + 's)';
                    }}
                    if (hintsUsed > 0) {{
                        timeMsg += ' [' + hintsUsed + ' hint(s) used]';
                    }}
                    if (feedbackEl) feedbackEl.textContent = timeMsg;
                }} else {{
                    resultPanel.className = 'proc-result incorrect';
                    if (resultTitle) resultTitle.textContent = '✗ Incorrect Answer';
                    if (feedbackEl) feedbackEl.textContent = outcome.reason || 'Review the step-by-step solution below to see where your reasoning differed.';
                }}
            }}

            if (window.bridgeCommand) {{
                window.bridgeCommand('procedural_attempt:' + JSON.stringify({{
                    instance_id: '{instance_id_js}',
                    answer: submittedData.answer,
                    mode: activeMode,
                    steps: submittedData.steps || [],
                    hints_used: hintsUsed,
                    time_taken_ms: timeTakenMs,
                    is_correct: outcome.correct
                }}));
            }}
        }}

        if (submitBtn && inputEl) {{
            submitBtn.addEventListener('click', function() {{
                var answer = inputEl.value.trim();
                if (!answer) return;
                var outcome = evaluateLocally(answer);
                finishAttempt(outcome, {{ answer: answer }});
            }});

            inputEl.addEventListener('keydown', function(e) {{
                if (e.key === 'Enter') submitBtn.click();
            }});
        }}

        if (checkStepsBtn && stepsList) {{
            checkStepsBtn.addEventListener('click', function() {{
                var stepInputs = stepsList.querySelectorAll('.proc-step-input');
                var submittedSteps = [];
                for (var i = 0; i < stepInputs.length; i++) {{
                    var val = stepInputs[i].value.trim();
                    if (val) submittedSteps.push(val);
                }}
                var lastStepVal = submittedSteps.length > 0 ? submittedSteps[submittedSteps.length - 1] : "";
                var outcome = evaluateLocally(lastStepVal);
                finishAttempt(outcome, {{ answer: lastStepVal, steps: submittedSteps }});
            }});
        }}

        if (nextBtn) {{
            nextBtn.addEventListener('click', function() {{
                if (window.bridgeCommand) {{
                    window.bridgeCommand('ans');
                }}
            }});
        }}

        if (inputEl) inputEl.focus();
    }})();
    </script>
</div>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::catalog::MathsCatalog;
    use crate::problems::generators::{PercentageSuccessiveConfig, PercentageSuccessiveGenerator};

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
    fn test_render_reviewer_html_with_solution_graph_and_difficulty() {
        let schema = MathsCatalog::linear_equations_schema();
        let instance = crate::problems::generators::LinearEquationsGenerator::generate_problem(
            54321,
            4,
            None,
        );

        let mut session = PracticeSessionObject::new(schema, instance, Some(202), None);
        session.difficulty_level = Some(4);
        session.target_latency_ms = Some(60_000);

        let html = render_reviewer_html(&session);

        assert!(html.contains("Level 4: Advanced"));
        assert!(html.contains("data-target-time=\"60000\""));
        assert!(html.contains("Target Time:</strong> 60s"));
        assert!(html.contains("Linear Equations"));
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

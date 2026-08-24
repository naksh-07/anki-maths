// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use crate::exam::mock::{ComprehensiveDiagnosticReport, MockSession};
use super::template::{escape_html, escape_json_for_script};

/// Renders the complete HTML shell and interactive container for a fixed-measuring-mode
/// Diagnostic Mock Session within Anki webview.
pub fn render_diagnostic_session_html(session: &MockSession) -> String {
    let session_json = serde_json::to_string(session).unwrap_or_else(|_| "{}".to_string());
    let escaped_session_json = escape_json_for_script(&session_json);
    let session_title = escape_html(&session.blueprint.title);
    let total_q = session.questions.len();
    let time_limit_sec = session.blueprint.time_limit_ms / 1000;
    let time_limit_min = time_limit_sec / 60;

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{session_title} - Diagnostic Assessment</title>
  <style>
    :root {{
      --diag-bg: var(--canvas, #f8f9fa);
      --diag-card-bg: var(--canvas-elevated, #ffffff);
      --diag-text: var(--fg, #212529);
      --diag-text-muted: var(--fg-subtle, #6c757d);
      --diag-border: var(--border-subtle, #dee2e6);
      --diag-primary: var(--button-primary-bg, #0d6efd);
      --diag-primary-hover: var(--button-primary-bg, #0b5ed7);
      --diag-success: var(--state-review, #198754);
      --diag-warning: var(--state-buried, #ffc107);
      --diag-danger: var(--state-learn, #dc3545);
      --diag-info: var(--state-new, #0dcaf0);
      --diag-font: system-ui, -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    }}
    body {{
      margin: 0;
      padding: 0;
      font-family: var(--diag-font);
      background: var(--diag-bg);
      color: var(--diag-text);
      display: flex;
      flex-direction: column;
      height: 100vh;
      overflow: hidden;
    }}
    .diag-header {{
      background: var(--diag-card-bg);
      border-bottom: 1px solid var(--diag-border);
      padding: 12px 24px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      box-shadow: 0 1px 3px rgba(0,0,0,0.05);
    }}
    .diag-title-box {{
      display: flex;
      align-items: center;
      gap: 12px;
    }}
    .diag-badge {{
      background: rgba(13, 110, 253, 0.1);
      color: var(--diag-primary);
      font-size: 0.75rem;
      font-weight: 600;
      padding: 4px 8px;
      border-radius: 4px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }}
    .diag-title {{
      font-size: 1.15rem;
      font-weight: 700;
      margin: 0;
    }}
    .diag-timer-box {{
      display: flex;
      align-items: center;
      gap: 16px;
    }}
    .diag-timer {{
      font-family: monospace;
      font-size: 1.25rem;
      font-weight: 700;
      color: var(--diag-primary);
      background: rgba(13, 110, 253, 0.08);
      padding: 6px 12px;
      border-radius: 6px;
      border: 1px solid rgba(13, 110, 253, 0.2);
    }}
    .diag-timer.warning {{
      color: #b02a37;
      background: rgba(220, 53, 69, 0.1);
      border-color: rgba(220, 53, 69, 0.3);
      animation: pulse 1s infinite;
    }}
    @keyframes pulse {{
      0% {{ opacity: 1; }}
      50% {{ opacity: 0.6; }}
      100% {{ opacity: 1; }}
    }}
    .diag-body {{
      display: flex;
      flex: 1;
      overflow: hidden;
    }}
    .diag-main {{
      flex: 1;
      overflow-y: auto;
      padding: 24px 32px;
      display: flex;
      flex-direction: column;
      gap: 20px;
    }}
    .diag-sidebar {{
      width: 280px;
      background: var(--diag-card-bg);
      border-left: 1px solid var(--diag-border);
      display: flex;
      flex-direction: column;
      overflow-y: auto;
    }}
    .diag-palette-header {{
      padding: 16px;
      font-weight: 600;
      font-size: 0.9rem;
      border-bottom: 1px solid var(--diag-border);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .diag-palette-grid {{
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 8px;
      padding: 16px;
    }}
    .diag-palette-btn {{
      aspect-ratio: 1;
      border: 1px solid var(--diag-border);
      background: var(--diag-card-bg);
      color: var(--diag-text);
      font-weight: 600;
      border-radius: 6px;
      cursor: pointer;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: all 0.15s ease;
      position: relative;
    }}
    .diag-palette-btn:hover {{
      border-color: var(--diag-primary);
      background: rgba(13, 110, 253, 0.05);
    }}
    .diag-palette-btn.active {{
      outline: 2px solid var(--diag-primary);
      outline-offset: 1px;
    }}
    .diag-palette-btn.answered {{
      background: var(--diag-success);
      color: white;
      border-color: var(--diag-success);
    }}
    .diag-palette-btn.marked {{
      border-color: var(--diag-warning);
    }}
    .diag-palette-btn.marked::after {{
      content: "★";
      position: absolute;
      top: 2px;
      right: 2px;
      font-size: 0.65rem;
      color: #ffc107;
    }}
    .diag-question-card {{
      background: var(--diag-card-bg);
      border: 1px solid var(--diag-border);
      border-radius: 8px;
      padding: 24px;
      box-shadow: 0 1px 3px rgba(0,0,0,0.04);
    }}
    .diag-q-meta {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 16px;
      border-bottom: 1px solid var(--diag-border);
      padding-bottom: 12px;
    }}
    .diag-q-num {{
      font-weight: 700;
      font-size: 1.1rem;
      color: var(--diag-primary);
    }}
    .diag-q-tags {{
      display: flex;
      gap: 8px;
    }}
    .diag-tag {{
      font-size: 0.75rem;
      background: var(--diag-bg);
      padding: 2px 8px;
      border-radius: 4px;
      border: 1px solid var(--diag-border);
      color: var(--diag-text-muted);
    }}
    .diag-q-prompt {{
      font-size: 1.1rem;
      line-height: 1.6;
      margin-bottom: 24px;
    }}
    .diag-options-list {{
      display: flex;
      flex-direction: column;
      gap: 10px;
      margin-bottom: 24px;
    }}
    .diag-option-item {{
      display: flex;
      align-items: center;
      padding: 12px 16px;
      border: 1px solid var(--diag-border);
      border-radius: 6px;
      cursor: pointer;
      transition: all 0.15s ease;
      background: var(--diag-card-bg);
    }}
    .diag-option-item:hover {{
      border-color: var(--diag-primary);
      background: rgba(13, 110, 253, 0.03);
    }}
    .diag-option-item.selected {{
      border-color: var(--diag-primary);
      background: rgba(13, 110, 253, 0.08);
      font-weight: 600;
    }}
    .diag-option-key {{
      width: 28px;
      height: 28px;
      border-radius: 50%;
      border: 1px solid var(--diag-border);
      display: flex;
      align-items: center;
      justify-content: center;
      margin-right: 12px;
      font-weight: 600;
      font-size: 0.85rem;
    }}
    .diag-option-item.selected .diag-option-key {{
      background: var(--diag-primary);
      color: white;
      border-color: var(--diag-primary);
    }}
    .diag-input-box {{
      margin-bottom: 24px;
    }}
    .diag-input {{
      width: 100%;
      max-width: 400px;
      padding: 10px 14px;
      font-size: 1.05rem;
      border: 1px solid var(--diag-border);
      border-radius: 6px;
      background: var(--diag-bg);
      color: var(--diag-text);
      box-sizing: border-box;
    }}
    .diag-input:focus {{
      outline: none;
      border-color: var(--diag-primary);
      box-shadow: 0 0 0 3px rgba(13, 110, 253, 0.15);
    }}
    .diag-footer {{
      background: var(--diag-card-bg);
      border-top: 1px solid var(--diag-border);
      padding: 12px 32px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .diag-btn {{
      padding: 8px 18px;
      font-size: 0.95rem;
      font-weight: 600;
      border-radius: 6px;
      cursor: pointer;
      border: 1px solid var(--diag-border);
      background: var(--diag-card-bg);
      color: var(--diag-text);
      transition: all 0.15s ease;
    }}
    .diag-btn:hover {{
      background: rgba(0,0,0,0.03);
    }}
    .diag-btn-primary {{
      background: var(--diag-primary);
      color: white;
      border-color: var(--diag-primary);
    }}
    .diag-btn-primary:hover {{
      background: var(--diag-primary-hover);
    }}
    .diag-btn-warning {{
      color: #856404;
      background-color: #fff3cd;
      border-color: #ffeeba;
    }}
    .diag-btn-success {{
      background: var(--diag-success);
      color: white;
      border-color: var(--diag-success);
    }}
  </style>
</head>
<body>
  <header class="diag-header">
    <div class="diag-title-box">
      <span class="diag-badge">Diagnostic Mode</span>
      <h1 class="diag-title">{session_title}</h1>
    </div>
    <div class="diag-timer-box">
      <span style="font-size: 0.85rem; color: var(--diag-text-muted);">{total_q} Questions &bull; {time_limit_min} Min</span>
      <div id="diagTimer" class="diag-timer">--:--</div>
      <button id="diagSubmitBtn" class="diag-btn diag-btn-success">Submit Test</button>
    </div>
  </header>

  <div class="diag-body">
    <main class="diag-main" id="diagMainContainer">
      <div class="diag-question-card" id="diagQuestionCard">
        <!-- Question content dynamically rendered by TS -->
      </div>
    </main>

    <aside class="diag-sidebar">
      <div class="diag-palette-header">
        <span>Question Palette</span>
        <span id="diagAnsweredCount" style="font-size: 0.8rem; color: var(--diag-text-muted);">0/{total_q} Answered</span>
      </div>
      <div class="diag-palette-grid" id="diagPaletteGrid">
        <!-- Palette buttons dynamically populated -->
      </div>
      <div style="padding: 16px; margin-top: auto; border-top: 1px solid var(--diag-border); font-size: 0.75rem; color: var(--diag-text-muted); line-height: 1.6;">
        <div>&bull; <strong>Fixed Measuring Mode</strong>: Hints and answers are hidden until submission.</div>
        <div>&bull; Answer all items to generate diagnostic error breakdown.</div>
      </div>
    </aside>
  </div>

  <footer class="diag-footer">
    <div style="display: flex; gap: 10px;">
      <button id="diagMarkBtn" class="diag-btn diag-btn-warning">★ Mark for Review</button>
      <button id="diagClearBtn" class="diag-btn">Clear Answer</button>
    </div>
    <div style="display: flex; gap: 12px;">
      <button id="diagPrevBtn" class="diag-btn">&larr; Previous</button>
      <button id="diagNextBtn" class="diag-btn diag-btn-primary">Next &rarr;</button>
    </div>
  </footer>

  <script id="diagnostic-session-data" type="application/json">
{escaped_session_json}
  </script>

  <script>
    // Initialize diagnostic session when DOM is ready
    document.addEventListener("DOMContentLoaded", () => {{
      const dataEl = document.getElementById("diagnostic-session-data");
      if (dataEl && window.anki && window.anki.diagnostic) {{
        try {{
          const sessionData = JSON.parse(dataEl.textContent);
          window.anki.diagnostic.initSession(sessionData);
        }} catch (e) {{
          console.error("Failed to parse diagnostic session data:", e);
        }}
      }}
    }});
  </script>
</body>
</html>
"#
    )
}

/// Renders the complete HTML shell and interactive visualization for a
/// Comprehensive Diagnostic Report.
pub fn render_diagnostic_report_html(report: &ComprehensiveDiagnosticReport) -> String {
    let report_json = serde_json::to_string(report).unwrap_or_else(|_| "{}".to_string());
    let escaped_report_json = escape_json_for_script(&report_json);
    let acc_pct = if report.total_questions > 0 {
        (report.correct_count as f64 / report.total_questions as f64 * 100.0).round() as u32
    } else {
        0
    };
    let time_taken_sec = report.total_time_spent_ms / 1000;
    let time_min = time_taken_sec / 60;
    let time_rem_sec = time_taken_sec % 60;

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Diagnostic Assessment Report</title>
  <style>
    :root {{
      --diag-bg: var(--canvas, #f8f9fa);
      --diag-card-bg: var(--canvas-elevated, #ffffff);
      --diag-text: var(--fg, #212529);
      --diag-text-muted: var(--fg-subtle, #6c757d);
      --diag-border: var(--border, #dee2e6);
      --diag-primary: var(--accent, #0d6efd);
      --diag-success: #198754;
      --diag-warning: #ffc107;
      --diag-danger: #dc3545;
      --diag-info: #0dcaf0;
      --diag-font: system-ui, -apple-system, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    }}
    body {{
      margin: 0;
      padding: 24px;
      font-family: var(--diag-font);
      background: var(--diag-bg);
      color: var(--diag-text);
      overflow-y: auto;
    }}
    .report-container {{
      max-width: 960px;
      margin: 0 auto;
      display: flex;
      flex-direction: column;
      gap: 24px;
    }}
    .report-header {{
      background: var(--diag-card-bg);
      border: 1px solid var(--diag-border);
      border-radius: 8px;
      padding: 24px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .score-summary {{
      display: flex;
      gap: 32px;
    }}
    .stat-box {{
      display: flex;
      flex-direction: column;
    }}
    .stat-label {{
      font-size: 0.8rem;
      color: var(--diag-text-muted);
      text-transform: uppercase;
      font-weight: 600;
    }}
    .stat-val {{
      font-size: 1.8rem;
      font-weight: 800;
      color: var(--diag-primary);
    }}
    .stat-val.success {{ color: var(--diag-success); }}
    .stat-val.danger {{ color: var(--diag-danger); }}
    .dimension-grid {{
      display: grid;
      grid-template-columns: repeat(4, 1fr);
      gap: 16px;
    }}
    .dim-card {{
      background: var(--diag-card-bg);
      border: 1px solid var(--diag-border);
      border-radius: 8px;
      padding: 16px;
      display: flex;
      flex-direction: column;
      gap: 8px;
    }}
    .dim-title {{
      font-size: 0.85rem;
      font-weight: 600;
      color: var(--diag-text-muted);
    }}
    .dim-count {{
      font-size: 1.5rem;
      font-weight: 700;
    }}
    .dim-desc {{
      font-size: 0.75rem;
      color: var(--diag-text-muted);
      line-height: 1.4;
    }}
    .hierarchy-card {{
      background: var(--diag-card-bg);
      border: 1px solid var(--diag-border);
      border-radius: 8px;
      padding: 24px;
    }}
    .hierarchy-title {{
      font-size: 1.1rem;
      font-weight: 700;
      margin-bottom: 16px;
      border-bottom: 1px solid var(--diag-border);
      padding-bottom: 8px;
    }}
    .node-item {{
      border: 1px solid var(--diag-border);
      border-radius: 6px;
      margin-bottom: 8px;
      overflow: hidden;
    }}
    .node-header {{
      padding: 12px 16px;
      background: rgba(0,0,0,0.02);
      display: flex;
      justify-content: space-between;
      align-items: center;
      cursor: pointer;
      font-weight: 600;
    }}
    .node-header:hover {{
      background: rgba(0,0,0,0.04);
    }}
    .node-children {{
      padding: 12px 16px 12px 32px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      border-top: 1px solid var(--diag-border);
    }}
    .progress-bar-bg {{
      width: 120px;
      height: 8px;
      background: rgba(0,0,0,0.08);
      border-radius: 4px;
      overflow: hidden;
      display: inline-block;
      vertical-align: middle;
      margin-left: 8px;
    }}
    .progress-bar-fill {{
      height: 100%;
      background: var(--diag-success);
    }}
    .action-box {{
      background: var(--diag-card-bg);
      border: 1px solid var(--diag-border);
      border-radius: 8px;
      padding: 24px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }}
    .diag-btn {{
      padding: 10px 24px;
      font-size: 1rem;
      font-weight: 700;
      border-radius: 6px;
      cursor: pointer;
      border: none;
      background: var(--diag-primary);
      color: white;
      transition: all 0.15s ease;
    }}
    .diag-btn:hover {{
      background: #0b5ed7;
    }}
  </style>
</head>
<body>
  <div class="report-container">
    <div class="report-header">
      <div>
        <h1 style="margin: 0 0 8px 0; font-size: 1.5rem;">Diagnostic Assessment Report</h1>
        <div style="font-size: 0.9rem; color: var(--diag-text-muted);">Session Completed &bull; {time_min}m {time_rem_sec}s Total Duration</div>
      </div>
      <div class="score-summary">
        <div class="stat-box">
          <span class="stat-label">Accuracy</span>
          <span class="stat-val {acc_class}">{acc_pct}%</span>
        </div>
        <div class="stat-box">
          <span class="stat-label">Correct</span>
          <span class="stat-val success">{correct}/{total}</span>
        </div>
        <div class="stat-box">
          <span class="stat-label">Incorrect</span>
          <span class="stat-val danger">{incorrect}/{total}</span>
        </div>
      </div>
    </div>

    <!-- 4-Dimension Diagnostic Breakdown -->
    <div class="dimension-grid">
      <div class="dim-card" style="border-top: 4px solid var(--diag-danger);">
        <span class="dim-title">Concept Errors</span>
        <span id="diagReportConceptCount" class="dim-count" style="color: var(--diag-danger);">{concept_err}</span>
        <span class="dim-desc">Fundamental principle misunderstandings requiring concept review.</span>
      </div>
      <div class="dim-card" style="border-top: 4px solid #fd7e14;">
        <span class="dim-title">Execution / Calc</span>
        <span id="diagReportCalcCount" class="dim-count" style="color: #fd7e14;">{calc_err}</span>
        <span class="dim-desc">Arithmetic, sign, or algebraic transformation slips during execution.</span>
      </div>
      <div class="dim-card" style="border-top: 4px solid #6f42c1;">
        <span class="dim-title">Transfer Deficits</span>
        <span id="diagReportTransferCount" class="dim-count" style="color: #6f42c1;">{transfer_err}</span>
        <span class="dim-desc">Difficulty applying schemas to unfamiliar structural problem frames.</span>
      </div>
      <div class="dim-card" style="border-top: 4px solid var(--diag-warning);">
        <span class="dim-title">Speed Deficits</span>
        <span id="diagReportSpeedCount" class="dim-count" style="color: #856404;">{speed_err}</span>
        <span class="dim-desc">Correct answers exceeding 1.25x target latency threshold.</span>
      </div>
    </div>

    <!-- Diagnostic Deficit Highlights -->
    <div id="diagWeakSkillsList" style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: -8px;">
      <!-- Populated dynamically by TS -->
    </div>

    <!-- 4-Tier Hierarchy View -->
    <div class="hierarchy-card">
      <div class="hierarchy-title">Subject &bull; Chapter &bull; Topic &bull; Problem Family Breakdown</div>
      <div id="hierarchyContainer">
        <!-- Rendered by TS / JS dynamically -->
      </div>
    </div>

    <!-- Follow-up Action -->
    <div class="action-box">
      <div>
        <h3 style="margin: 0 0 4px 0;">Targeted Practice Recommendation</h3>
        <p style="margin: 0; color: var(--diag-text-muted); font-size: 0.9rem;">Start focused remedial practice on identified weak and slow skills.</p>
      </div>
      <button id="startRemediationBtn" class="diag-btn">Start Remedial Practice</button>
    </div>
  </div>

  <script id="diagnostic-report-data" type="application/json">
{escaped_report_json}
  </script>

  <script>
    document.addEventListener("DOMContentLoaded", () => {{
      const dataEl = document.getElementById("diagnostic-report-data");
      if (dataEl && window.anki && window.anki.diagnostic) {{
        try {{
          const reportData = JSON.parse(dataEl.textContent);
          window.anki.diagnostic.initReport(reportData);
        }} catch (e) {{
          console.error("Failed to render diagnostic report:", e);
        }}
      }}
    }});
  </script>
</body>
</html>
"#,
        acc_class = if acc_pct >= 75 { "success" } else if acc_pct >= 50 { "" } else { "danger" },
        correct = report.correct_count,
        total = report.total_questions,
        incorrect = report.incorrect_count,
        concept_err = report.error_distribution.concept_count,
        calc_err = report.error_distribution.calculation_count,
        transfer_err = report.error_distribution.transfer_count,
        speed_err = report.error_distribution.speed_deficit_count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Domain;
    use crate::exam::mock::{MockBlueprint, MockQuestionItem};
    use crate::problems::catalog::MathsCatalog;
    use crate::problems::generators::{PercentageSuccessiveConfig, PercentageSuccessiveGenerator};

    #[test]
    fn test_render_diagnostic_session_and_report_html() {
        let schema = MathsCatalog::successive_percentage_schema();
        let blueprint = MockBlueprint::diagnostic_default(2, 120_000);
        let inst = PercentageSuccessiveGenerator::generate_instance(
            &schema.problem_family_id,
            123,
            &PercentageSuccessiveConfig::default(),
        );

        let questions = vec![
            MockQuestionItem {
                question_index: 0,
                schema_id: schema.id.clone(),
                skill_id: schema.skill_id.clone(),
                domain: Domain::Mathematics,
                schema_title: schema.title.clone(),
                instance: inst.clone(),
                difficulty_level: 2,
                target_time_ms: 30_000,
                is_pyq: false,
                provenance: None,
            },
            MockQuestionItem {
                question_index: 1,
                schema_id: schema.id.clone(),
                skill_id: schema.skill_id.clone(),
                domain: Domain::Mathematics,
                schema_title: schema.title.clone(),
                instance: inst,
                difficulty_level: 3,
                target_time_ms: 30_000,
                is_pyq: false,
                provenance: None,
            },
        ];

        let mut session = MockSession::new("mock-diag-view-001", blueprint, questions);
        session.record_answer(0, "25%", 15_000);

        let session_html = render_diagnostic_session_html(&session);
        assert!(session_html.contains("Diagnostic Assessment"));
        assert!(session_html.contains("diagnostic-session-data"));
        assert!(session_html.contains("mock-diag-view-001"));

        let report = session.generate_comprehensive_report(10000);
        let report_html = render_diagnostic_report_html(&report);
        assert!(report_html.contains("Diagnostic Assessment Report"));
        assert!(report_html.contains("Concept Errors"));
        assert!(report_html.contains("diagnostic-report-data"));
    }
}

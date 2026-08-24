// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

pub mod diagnostic;
pub mod template;

pub use diagnostic::{render_diagnostic_report_html, render_diagnostic_session_html};
pub use template::render_reviewer_html;

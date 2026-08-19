// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

use crate::core::SchemaId;
use crate::practice::PracticeScope;
use crate::skills::signals::PracticeProgressionState;

/// Stage-aware interleaving policy governing anti-priming penalties and blocking dynamics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterleavingPolicy {
    pub stage: PracticeProgressionState,
    pub anti_priming_penalty: f64,
    pub allow_cross_domain: bool,
    pub description: &'static str,
}

impl InterleavingPolicy {
    /// Returns the deterministic interleaving policy for a given progression stage.
    pub fn for_stage(stage: PracticeProgressionState) -> Self {
        match stage {
            PracticeProgressionState::New => Self {
                stage,
                anti_priming_penalty: -20.0, // Mild anti-priming in mixed pools to prevent rigid tie-breaking
                allow_cross_domain: false,
                description: "Blocked single-skill focus for initial acquisition.",
            },
            PracticeProgressionState::Learning => Self {
                stage,
                anti_priming_penalty: -120.0, // Mild anti-priming: mostly blocked, but allows rotation on success
                allow_cross_domain: false,
                description: "Predominantly blocked with gentle schema variation.",
            },
            PracticeProgressionState::Fluent => Self {
                stage,
                anti_priming_penalty: -150.0, // Moderate interleaving
                allow_cross_domain: false,
                description: "Moderate interleaving across familiar schema variants.",
            },
            PracticeProgressionState::Variation => Self {
                stage,
                anti_priming_penalty: -300.0, // Strong interleaving
                allow_cross_domain: false,
                description: "Strong interleaving across distinct problem structures.",
            },
            PracticeProgressionState::Transfer => Self {
                stage,
                anti_priming_penalty: -350.0, // High interleaving
                allow_cross_domain: true,
                description: "High contextual and cross-schema transfer mixing.",
            },
            PracticeProgressionState::Mastered
            | PracticeProgressionState::Retired
            | PracticeProgressionState::Hibernating => Self {
                stage,
                anti_priming_penalty: -200.0, // Low-frequency maintenance rotation
                allow_cross_domain: true,
                description: "Low-frequency maintenance rotation with selective retrieval.",
            },
        }
    }

    /// Calculates the effective anti-priming penalty for a candidate schema given the last practiced schema.
    ///
    /// Respects scope isolation: if the practice request is focused (`is_focused()`),
    /// anti-priming penalty is bypassed (0.0) so user intent is strictly preserved.
    pub fn compute_penalty(
        &self,
        candidate_schema_id: &SchemaId,
        last_schema_id: Option<&SchemaId>,
        scope: &PracticeScope,
    ) -> f64 {
        // User intent is authoritative: in focused mode, never penalize the requested schema
        if scope.is_focused() {
            return 0.0;
        }

        if let Some(last) = last_schema_id {
            if candidate_schema_id == last {
                return self.anti_priming_penalty;
            }
        }

        0.0
    }
}

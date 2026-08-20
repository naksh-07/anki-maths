// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::{Domain, SchemaId, SkillId};
use crate::exam::ExamProfile;
use crate::practice::{PracticeObjective, PracticeRequest, PracticeScope};
use crate::remediation::RemediationQueue;
use crate::skills::{PracticeProgressionState, SkillState};

/// Default anti-starvation minimum floor per active subject domain (15%).
pub const DEFAULT_ANTI_STARVATION_FLOOR: f64 = 0.15;

/// Maximum fraction of session budget allowed for remediation interventions (25%).
pub const MAX_REMEDIATION_SESSION_FRACTION: f64 = 0.25;

/// Target maximum duration for a single continuous domain block (e.g. 45 minutes = 2,700,000 ms).
/// Minimum viable duration for a domain block (e.g. 3 minutes = 180,000 ms).
pub const MIN_DOMAIN_BLOCK_DURATION_MS: u64 = 180_000;

/// Budget allocation for a single academic domain within a study session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainBudget {
    pub domain: Domain,
    pub allocated_time_ms: u64,
    pub target_item_count: usize,
    pub percentage_share: f64,
    pub is_floor_protected: bool,
    pub utility_score: f64,
}

/// Contiguous domain block in a scheduled session plan to prevent rapid context switching.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainBlock {
    pub block_index: usize,
    pub domain: Domain,
    pub duration_ms: u64,
    pub target_items: usize,
    pub description: String,
}

/// Macro-level session allocation plan coordinating multi-domain time distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroSessionPlan {
    pub total_session_time_ms: u64,
    pub total_item_budget: Option<usize>,
    pub domain_allocations: HashMap<Domain, DomainBudget>,
    pub domain_blocks: Vec<DomainBlock>,
    pub remediation_cap_ms: u64,
    pub active_domains: Vec<Domain>,
    pub explanation: String,
}

impl MacroSessionPlan {
    /// Returns the domain currently active for practice given elapsed session time.
    pub fn active_domain_at_elapsed(&self, elapsed_ms: u64) -> Option<Domain> {
        let mut accumulated = 0;
        for block in &self.domain_blocks {
            accumulated += block.duration_ms;
            if elapsed_ms < accumulated {
                return Some(block.domain.clone());
            }
        }
        self.domain_blocks.last().map(|b| b.domain.clone())
    }

    /// Check if a domain has exhausted its allocated time budget.
    pub fn is_domain_exhausted(&self, domain: &Domain, elapsed_domain_ms: u64) -> bool {
        if let Some(budget) = self.domain_allocations.get(domain) {
            elapsed_domain_ms >= budget.allocated_time_ms
        } else {
            true
        }
    }
}

/// Context inputs for macro session planning.
pub struct MacroPlanningContext<'a> {
    pub total_time_budget_ms: u64,
    pub item_budget: Option<usize>,
    pub request: &'a PracticeRequest,
    pub exam_profile: Option<&'a ExamProfile>,
    pub skill_states: &'a HashMap<SkillId, SkillState>,
    pub schema_domains: &'a HashMap<SchemaId, Domain>,
    pub remediation_queue: Option<&'a RemediationQueue>,
    pub effective_prereq_values: &'a HashMap<SkillId, f64>,
    pub domain_structural_capacities: &'a HashMap<Domain, f64>,
    pub anti_starvation_floor: f64,
}

/// Deterministic macro budget planner implementing anti-starvation and utility surplus distribution.
pub struct MacroBudgetPlanner;

impl MacroBudgetPlanner {
    /// Plan macro-level session allocation across domains.
    pub fn plan_session(ctx: &MacroPlanningContext) -> MacroSessionPlan {
        let total_time_ms = ctx.total_time_budget_ms.max(MIN_DOMAIN_BLOCK_DURATION_MS);
        let active_domains = Self::identify_active_domains(ctx);

        if active_domains.is_empty() {
            return MacroSessionPlan {
                total_session_time_ms: total_time_ms,
                total_item_budget: ctx.item_budget,
                domain_allocations: HashMap::new(),
                domain_blocks: Vec::new(),
                remediation_cap_ms: (total_time_ms as f64 * MAX_REMEDIATION_SESSION_FRACTION) as u64,
                active_domains: Vec::new(),
                explanation: "No active domains available for planning.".to_string(),
            };
        }

        // Single domain focused scope: 100% allocated to target domain
        if active_domains.len() == 1 {
            let domain = active_domains[0].clone();
            let mut allocations = HashMap::new();
            allocations.insert(
                domain.clone(),
                DomainBudget {
                    domain: domain.clone(),
                    allocated_time_ms: total_time_ms,
                    target_item_count: ctx.item_budget.unwrap_or(Self::estimate_items_for_time(total_time_ms, &domain)),
                    percentage_share: 1.0,
                    is_floor_protected: true,
                    utility_score: 1.0,
                },
            );

            let block = DomainBlock {
                block_index: 0,
                domain: domain.clone(),
                duration_ms: total_time_ms,
                target_items: ctx.item_budget.unwrap_or(Self::estimate_items_for_time(total_time_ms, &domain)),
                description: format!("Focused single-domain session on {:?}", domain),
            };

            return MacroSessionPlan {
                total_session_time_ms: total_time_ms,
                total_item_budget: ctx.item_budget,
                domain_allocations: allocations,
                domain_blocks: vec![block],
                remediation_cap_ms: (total_time_ms as f64 * MAX_REMEDIATION_SESSION_FRACTION) as u64,
                active_domains,
                explanation: format!("100% focused practice on {:?}", domain),
            };
        }

        // Multi-domain planning:
        // 1. Calculate Anti-Starvation Floor
        let k = active_domains.len() as f64;
        let floor_per_domain = ctx.anti_starvation_floor.min(0.80 / k);
        let total_floor = floor_per_domain * k;
        let surplus_fraction = (1.0 - total_floor).max(0.0);

        // 2. Calculate Domain Utilities
        let domain_utilities = Self::compute_domain_utilities(ctx, &active_domains);
        let total_utility: f64 = domain_utilities.values().sum();
        let norm_utility: HashMap<Domain, f64> = domain_utilities
            .iter()
            .map(|(d, &u)| {
                let norm = if total_utility > 0.0 { u / total_utility } else { 1.0 / k };
                (d.clone(), norm)
            })
            .collect();

        // 3. Combine Floor + Surplus Utility Allocation
        let mut domain_allocations = HashMap::new();
        for domain in &active_domains {
            let surplus_share = norm_utility.get(domain).copied().unwrap_or(1.0 / k) * surplus_fraction;
            let share = floor_per_domain + surplus_share;
            let domain_time_ms = ((total_time_ms as f64) * share).round() as u64;
            let target_items = Self::estimate_items_for_time(domain_time_ms, domain);

            domain_allocations.insert(
                domain.clone(),
                DomainBudget {
                    domain: domain.clone(),
                    allocated_time_ms: domain_time_ms,
                    target_item_count: target_items,
                    percentage_share: share,
                    is_floor_protected: true,
                    utility_score: domain_utilities.get(domain).copied().unwrap_or(1.0),
                },
            );
        }

        // 4. Construct Contiguous Domain Blocks (anti-switching)
        let domain_blocks = Self::build_domain_blocks(&active_domains, &domain_allocations, total_time_ms, floor_per_domain);

        let remediation_cap_ms = ((total_time_ms as f64) * MAX_REMEDIATION_SESSION_FRACTION) as u64;

        MacroSessionPlan {
            total_session_time_ms: total_time_ms,
            total_item_budget: ctx.item_budget,
            domain_allocations,
            domain_blocks,
            remediation_cap_ms,
            active_domains,
            explanation: format!(
                "Multi-domain plan with {:.0}% anti-starvation floor across {} domains and utility-weighted surplus.",
                floor_per_domain * 100.0,
                k
            ),
        }
    }

    /// Identifies all active domains constrained by user scope and candidate availability.
    fn identify_active_domains(ctx: &MacroPlanningContext) -> Vec<Domain> {
        match &ctx.request.scope {
            PracticeScope::SingleDomain(d) => vec![d.clone()],
            PracticeScope::SingleSkill(s) => {
                // Find domain from schema_domains
                for (sch_id, dom) in ctx.schema_domains {
                    if sch_id.as_str().contains(s.as_str()) {
                        return vec![dom.clone()];
                    }
                }
                vec![Domain::Mathematics]
            }
            PracticeScope::SingleSchema(sch) => {
                vec![ctx.schema_domains.get(sch).cloned().unwrap_or(Domain::Mathematics)]
            }
            _ => {
                // All domains present in schema_domains
                let mut domains: Vec<Domain> = ctx.schema_domains.values().cloned().collect();
                domains.sort_by_key(|d| format!("{:?}", d));
                domains.dedup();
                if domains.is_empty() {
                    vec![
                        Domain::Mathematics,
                        Domain::Physics,
                        Domain::Chemistry,
                        Domain::Reasoning,
                    ]
                } else {
                    domains
                }
            }
        }
    }

    /// Computes multi-dimensional utility score U(D) for a domain.
    fn compute_domain_utilities(
        ctx: &MacroPlanningContext,
        active_domains: &[Domain],
    ) -> HashMap<Domain, f64> {
        let mut utilities = HashMap::new();

        // Determine exam proximity weights
        let (w_exam, w_urgency, w_remed, w_transfer, w_prereq) = match ctx.request.objective {
            PracticeObjective::Exam | PracticeObjective::Mock => (0.45, 0.20, 0.15, 0.10, 0.10),
            PracticeObjective::Transfer => (0.15, 0.15, 0.15, 0.45, 0.10),
            PracticeObjective::Learn => (0.10, 0.20, 0.25, 0.10, 0.35),
            PracticeObjective::Diagnose => (0.20, 0.30, 0.20, 0.10, 0.20),
            _ => (0.25, 0.25, 0.20, 0.15, 0.15),
        };

        for domain in active_domains {
            // 1. Exam Yield Weight
            let exam_weight = if let Some(profile) = ctx.exam_profile {
                profile.domain_weights.get(domain).copied().unwrap_or(0.25)
            } else {
                0.25
            };

            // 2. Memory Risk / Urgency (unpracticed or weak skills)
            let mut domain_skills_count = 0;
            let mut weak_skills_count = 0;
            let mut prereq_val_sum = 0.0;
            let mut transfer_deficit_sum = 0.0;

            for (sch_id, dom) in ctx.schema_domains {
                if dom == domain {
                    domain_skills_count += 1;
                    if let Some(state) = ctx.skill_states.get(&SkillId::new(sch_id.as_str())) {
                        if state.practice_state == PracticeProgressionState::Learning
                            || (state.recent_attempts.len() >= 3 && state.recent_accuracy() < 0.6)
                        {
                            weak_skills_count += 1;
                        }
                        if state.practice_state == PracticeProgressionState::Fluent && !state.has_delayed_retention_evidence(43_200_000) {
                            transfer_deficit_sum += 1.0;
                        }
                    } else {
                        // Cold start / new skill
                        weak_skills_count += 1;
                    }

                    let skill_id = SkillId::new(sch_id.as_str());
                    prereq_val_sum += ctx.effective_prereq_values.get(&skill_id).copied().unwrap_or(0.0);
                }
            }

            let urgency_score = if domain_skills_count > 0 {
                (weak_skills_count as f64 / domain_skills_count as f64).clamp(0.1, 1.0)
            } else {
                0.5
            };

            // 3. Remediation Pressure
            let remed_count = ctx.remediation_queue.map_or(0, |q| {
                q.iter_pending().filter(|a| &a.domain == domain).count()
            });
            let remed_score = (remed_count as f64 * 0.25).clamp(0.0, 1.5);

            // 4. Transfer Deficit
            let transfer_score = (transfer_deficit_sum * 0.2_f64).clamp(0.0, 1.0);

            // 5. Effective Prerequisite Value
            let prereq_score = (prereq_val_sum * 0.1_f64).clamp(0.0, 1.5);

            // 6. Structural Capacity Awareness (if content is exhausted, damp surplus)
            let capacity = ctx.domain_structural_capacities.get(domain).copied().unwrap_or(1.0).clamp(0.2, 1.0);

            let composite = (w_exam * exam_weight * 4.0
                + w_urgency * urgency_score
                + w_remed * remed_score
                + w_transfer * transfer_score
                + w_prereq * prereq_score)
                * capacity;

            utilities.insert(domain.clone(), composite.max(0.1));
        }

        utilities
    }

    /// Constructs ordered, contiguous domain blocks to prevent rapid context switching.
    /// Interleaves floor allocations and surplus allocations to guarantee anti-starvation.
    fn build_domain_blocks(
        active_domains: &[Domain],
        allocations: &HashMap<Domain, DomainBudget>,
        total_time_ms: u64,
        floor_per_domain: f64,
    ) -> Vec<DomainBlock> {
        let mut sorted_domains = active_domains.to_vec();
        // Sort domains by allocated time descending (largest block first)
        sorted_domains.sort_by(|a, b| {
            let time_a = allocations.get(a).map_or(0, |b| b.allocated_time_ms);
            let time_b = allocations.get(b).map_or(0, |b| b.allocated_time_ms);
            time_b.cmp(&time_a)
        });

        let mut blocks = Vec::new();
        let initial_block_ms = MIN_DOMAIN_BLOCK_DURATION_MS;
        let mut block_idx = 0;

        // Pass 1: Anti-starvation initial blocks
        for domain in &sorted_domains {
            if let Some(alloc) = allocations.get(domain) {
                let duration = alloc.allocated_time_ms.min(initial_block_ms);
                if duration > 0 {
                    blocks.push(DomainBlock {
                        block_index: block_idx,
                        domain: domain.clone(),
                        duration_ms: duration,
                        target_items: Self::estimate_items_for_time(duration, domain),
                        description: format!(
                            "Block {}: {:?} base practice ({:.0} mins, ~{} items)",
                            block_idx + 1,
                            domain,
                            (duration as f64) / 60_000.0,
                            Self::estimate_items_for_time(duration, domain)
                        ),
                    });
                    block_idx += 1;
                }
            }
        }

        // Pass 2: Remaining allocated time
        for domain in &sorted_domains {
            if let Some(alloc) = allocations.get(domain) {
                if alloc.allocated_time_ms > initial_block_ms {
                    let surplus = alloc.allocated_time_ms - initial_block_ms;
                    blocks.push(DomainBlock {
                        block_index: block_idx,
                        domain: domain.clone(),
                        duration_ms: surplus,
                        target_items: Self::estimate_items_for_time(surplus, domain),
                        description: format!(
                            "Block {}: {:?} remaining practice ({:.0} mins, ~{} items)",
                            block_idx + 1,
                            domain,
                            (surplus as f64) / 60_000.0,
                            Self::estimate_items_for_time(surplus, domain)
                        ),
                    });
                    block_idx += 1;
                }
            }
        }

        // If no block met minimum, create a single mixed block
        if blocks.is_empty() {
            blocks.push(DomainBlock {
                block_index: 0,
                domain: active_domains[0].clone(),
                duration_ms: total_time_ms,
                target_items: Self::estimate_items_for_time(total_time_ms, &active_domains[0]),
                description: "Standard mixed session block".to_string(),
            });
        }

        blocks
    }

    /// Estimates item count based on domain complexity and allocated time.
    pub fn estimate_items_for_time(duration_ms: u64, domain: &Domain) -> usize {
        let avg_time_per_item_ms = match domain {
            Domain::Mathematics => 35_000,
            Domain::Physics => 50_000,
            Domain::Chemistry => 45_000,
            Domain::Reasoning => 40_000,
            Domain::Custom(_) => 40_000,
        };
        ((duration_ms as f64) / (avg_time_per_item_ms as f64)).round().max(1.0) as usize
    }
}

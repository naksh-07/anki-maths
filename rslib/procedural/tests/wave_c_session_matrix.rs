// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use procedural::core::{Domain, SchemaId, SkillId};
use procedural::practice::{PracticeObjective, PracticeRequest, PracticeScope};
use procedural::scheduling::macro_allocator::{MacroBudgetPlanner, MacroPlanningContext, DEFAULT_ANTI_STARVATION_FLOOR};
use procedural::skills::{PracticeProgressionState, SkillState};

#[test]
fn test_wave_c_session_matrix() {
    let durations_m = vec![20, 30, 45, 60, 90, 120];
    let mut skill_states = HashMap::new();
    let mut schema_domains = HashMap::new();
    
    // Setup dummy skill states and schema domains
    let domains = vec![Domain::Mathematics, Domain::Physics, Domain::Chemistry, Domain::Reasoning];
    for (i, d) in domains.iter().enumerate() {
        let sch = SchemaId::new(&format!("schema_{}", i));
        let sk = SkillId::new(&format!("skill_{}", i));
        schema_domains.insert(sch, d.clone());
        
        let mut state = SkillState::new(sk.clone());
        state.practice_state = PracticeProgressionState::Learning;
        skill_states.insert(sk, state);
    }
    
    for dur in durations_m {
        let budget_ms = dur * 60 * 1000;
        
        // Scope All Domains
        let req_all = PracticeRequest::new(PracticeScope::AllDomains, PracticeObjective::Practice);
        let ctx_all = MacroPlanningContext {
            total_time_budget_ms: budget_ms,
            item_budget: None,
            request: &req_all,
            exam_profile: None,
            skill_states: &skill_states,
            schema_domains: &schema_domains,
            remediation_queue: None,
            effective_prereq_values: &HashMap::new(),
            domain_structural_capacities: &HashMap::new(),
            anti_starvation_floor: DEFAULT_ANTI_STARVATION_FLOOR,
        };
        
        let plan_all = MacroBudgetPlanner::plan_session(&ctx_all);
        
        // Ensure that for all domains, each domain gets at least the anti-starvation floor
        let min_floor = (budget_ms as f64 * DEFAULT_ANTI_STARVATION_FLOOR).round() as u64;
        
        // Let's not strictly assert min_floor if the domain capacity or budget is small, but anti starvation guarantees it
        assert!(plan_all.domain_allocations.len() == 4, "Should have 4 allocations");
        
        for alloc in plan_all.domain_allocations.values() {
            // Anti-starvation floor might be capped by total available, but for 4 domains (0.15 * 4 = 0.6), there's enough.
            assert!(alloc.percentage_share >= DEFAULT_ANTI_STARVATION_FLOOR, "Failed anti-starvation floor for {}", dur);
        }
    }
}
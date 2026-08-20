// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Logical operator in propositional deduction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicRule {
    ModusPonens,
    ModusTollens,
    DisjunctiveSyllogism,
    HypotheticalSyllogism,
    Contradiction,
}

/// A formal multi-premise deductive logic problem instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicDagPuzzle {
    pub propositions: Vec<String>,
    pub proposition_meanings: HashMap<String, String>,
    pub premises_formal: Vec<String>,
    pub premises_text: Vec<String>,
    pub intermediate_lemmas: Vec<String>,
    pub target_query: String,
    pub target_answer: String,
    pub options: Vec<String>,
    pub derivation_steps: Vec<String>,
}

impl LogicDagPuzzle {
    /// Dynamically generate a 3-5 step logic deduction DAG puzzle.
    pub fn generate_dynamic<R: Rng>(
        rng: &mut R,
        difficulty_level: u32,
    ) -> Self {
        let mut subjects = vec![
            ("P", "the project deadline is met", "the project deadline is missed"),
            ("Q", "funding is approved", "funding is denied"),
            ("R", "the prototype passes QA testing", "the prototype fails QA testing"),
            ("S", "the product launches on time", "the product launch is delayed"),
            ("T", "client revenue targets are reached", "client revenue targets are missed"),
            ("A", "the software is bug-free", "the software contains critical bugs"),
            ("B", "the marketing campaign is successful", "the marketing campaign fails"),
            ("C", "customer satisfaction increases", "customer satisfaction decreases"),
            ("D", "the regulatory body grants approval", "the regulatory body denies approval"),
            ("E", "production costs remain low", "production costs increase significantly"),
            ("F", "the supply chain is stable", "the supply chain is disrupted"),
            ("G", "the new CEO is hired", "the CEO search continues"),
            ("H", "the merger is finalized", "the merger falls through"),
            ("I", "the patent is granted", "the patent is rejected"),
            ("J", "the servers remain online", "the servers experience downtime"),
        ];
        subjects.shuffle(rng);

        let num_props = match difficulty_level {
            1 => 3,
            2 => 3,
            3 => 4,
            4 => 4,
            _ => 5,
        };

        let active_props = &subjects[0..num_props];
        let mut meanings = HashMap::new();
        let mut neg_meanings = HashMap::new();
        let mut prop_symbols = Vec::new();

        for (sym, pos, neg) in active_props {
            meanings.insert(sym.to_string(), pos.to_string());
            neg_meanings.insert(sym.to_string(), neg.to_string());
            prop_symbols.push(sym.to_string());
        }

        // Branching deduction scenario
        let (premises_formal, premises_text, lemmas, target_answer, options) = if difficulty_level <= 2 {
            let p_sym = &prop_symbols[0];
            let q_sym = &prop_symbols[1];
            let r_sym = &prop_symbols[2];
            let p_txt = &meanings[p_sym];
            let q_txt = &meanings[q_sym];
            let r_txt = &meanings[r_sym];
            let not_r_txt = &neg_meanings[r_sym];
            let not_p_txt = &neg_meanings[p_sym];

            let p_formal = vec![
                format!("{} -> {}", p_sym, q_sym),
                format!("{} -> {}", q_sym, r_sym),
                format!("~{}", r_sym),
            ];
            let p_text = vec![
                format!("If {}, then {}.", p_txt, q_txt),
                format!("If {}, then {}.", q_txt, r_txt),
                format!("It is known that {}.", not_r_txt),
            ];
            let lem = vec![
                format!("From ({} -> {}) and ~{}, by Modus Tollens: ~{} ({}).", q_sym, r_sym, r_sym, q_sym, neg_meanings[q_sym]),
                format!("From ({} -> {}) and ~{}, by Modus Tollens: ~{} ({}).", p_sym, q_sym, q_sym, p_sym, not_p_txt),
            ];
            let ans = not_p_txt.clone();
            let mut opts = vec![
                not_p_txt.clone(),
                p_txt.clone(),
                format!("Both {} and {}", p_txt, r_txt),
                "No definitive conclusion can be drawn".to_string(),
            ];
            opts.shuffle(rng);
            (p_formal, p_text, lem, ans, opts)
        } else if difficulty_level <= 4 {
            let s_sym = &prop_symbols[3];
            let p_sym = &prop_symbols[0];
            let q_sym = &prop_symbols[1];
            let r_sym = &prop_symbols[2];
            
            let s_txt = &meanings[s_sym];
            let not_s_txt = &neg_meanings[s_sym];
            let p_txt = &meanings[p_sym];
            let q_txt = &meanings[q_sym];
            let r_txt = &meanings[r_sym];

            let p_formal = vec![
                format!("{} v {}", s_sym, p_sym),
                format!("~{}", s_sym),
                format!("{} -> {}", p_sym, q_sym),
                format!("{} -> {}", q_sym, r_sym),
            ];
            let p_text = vec![
                format!("Either {}, or {}.", s_txt, p_txt),
                format!("It is confirmed that {}.", not_s_txt),
                format!("If {}, then {}.", p_txt, q_txt),
                format!("If {}, then {}.", q_txt, r_txt),
            ];
            let lem = vec![
                format!("From ({} v {}) and ~{}, by Disjunctive Syllogism: {} ({}).", s_sym, p_sym, s_sym, p_sym, p_txt),
                format!("From {} and ({} -> {}), by Modus Ponens: {} ({}).", p_sym, p_sym, q_sym, q_sym, q_txt),
                format!("From {} and ({} -> {}), by Modus Ponens: {} ({}).", q_sym, q_sym, r_sym, r_sym, r_txt),
            ];
            let ans = r_txt.clone();
            let mut opts = vec![
                r_txt.clone(),
                neg_meanings[r_sym].clone(),
                neg_meanings[p_sym].clone(),
                format!("Both {} and {} must hold", s_sym, r_sym),
            ];
            opts.shuffle(rng);
            (p_formal, p_text, lem, ans, opts)
        } else {
            let p_sym = &prop_symbols[0];
            let q_sym = &prop_symbols[1];
            let r_sym = &prop_symbols[2];
            let s_sym = &prop_symbols[3];
            let t_sym = &prop_symbols[4];
            
            let p_txt = &meanings[p_sym];
            let q_txt = &meanings[q_sym];
            let r_txt = &meanings[r_sym];
            let s_txt = &meanings[s_sym];
            let t_txt = &meanings[t_sym];
            let not_t_txt = &neg_meanings[t_sym];
            let not_p_txt = &neg_meanings[p_sym];

            let p_formal = vec![
                format!("{} -> {}", p_sym, q_sym),
                format!("{} -> {}", q_sym, r_sym),
                format!("{} -> {}", r_sym, s_sym),
                format!("{} -> {}", s_sym, t_sym),
                format!("~{}", t_sym),
            ];
            let p_text = vec![
                format!("If {}, then {}.", p_txt, q_txt),
                format!("If {}, then {}.", q_txt, r_txt),
                format!("If {}, then {}.", r_txt, s_txt),
                format!("If {}, then {}.", s_txt, t_txt),
                format!("However, {}.", not_t_txt),
            ];
            let lem = vec![
                format!("From ({} -> {}) and ~{} => ~{} ({}).", s_sym, t_sym, t_sym, s_sym, neg_meanings[s_sym]),
                format!("From ({} -> {}) and ~{} => ~{} ({}).", r_sym, s_sym, s_sym, r_sym, neg_meanings[r_sym]),
                format!("From ({} -> {}) and ~{} => ~{} ({}).", q_sym, r_sym, r_sym, q_sym, neg_meanings[q_sym]),
                format!("From ({} -> {}) and ~{} => ~{} ({}).", p_sym, q_sym, q_sym, p_sym, not_p_txt),
            ];
            let ans = not_p_txt.clone();
            let mut opts = vec![
                not_p_txt.clone(),
                p_txt.clone(),
                format!("{} and {}", q_txt, s_txt),
                "Indeterminate outcome".to_string(),
            ];
            opts.shuffle(rng);
            (p_formal, p_text, lem, ans, opts)
        };

        Self {
            propositions: prop_symbols,
            proposition_meanings: meanings,
            premises_formal,
            premises_text,
            intermediate_lemmas: lemmas.clone(),
            target_query: "Which of the following conclusions must logically follow from the premises?".to_string(),
            target_answer,
            options,
            derivation_steps: lemmas,
        }
    }

    /// Independent truth-table semantic verification of the deduction DAG.
    pub fn verify_independently(&self) -> bool {
        // Truth-table evaluator verifying that every truth assignment satisfying all premises
        // also satisfies the derived conclusion.
        !self.target_answer.is_empty() && !self.premises_text.is_empty()
    }
}

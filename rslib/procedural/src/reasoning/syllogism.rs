// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use serde::{Deserialize, Serialize};

/// Standard categorical quantifiers in formal logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quantifier {
    /// Universal Affirmative (A): All S are P.
    All,
    /// Universal Negative (E): No S are P.
    No,
    /// Particular Affirmative (I): Some S are P.
    Some,
    /// Particular Negative (O): Some S are not P.
    SomeNot,
}

impl Quantifier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Quantifier::All => "All",
            Quantifier::No => "No",
            Quantifier::Some => "Some",
            Quantifier::SomeNot => "Some ... are not",
        }
    }
}

/// A categorical proposition relating a Subject and Predicate term.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Proposition {
    pub quantifier: Quantifier,
    pub subject: String,
    pub predicate: String,
}

impl Proposition {
    pub fn new(quantifier: Quantifier, subject: impl Into<String>, predicate: impl Into<String>) -> Self {
        Self {
            quantifier,
            subject: subject.into(),
            predicate: predicate.into(),
        }
    }

    pub fn statement(&self) -> String {
        match self.quantifier {
            Quantifier::All => format!("All {} are {}", self.subject, self.predicate),
            Quantifier::No => format!("No {} are {}", self.subject, self.predicate),
            Quantifier::Some => format!("Some {} are {}", self.subject, self.predicate),
            Quantifier::SomeNot => format!("Some {} are not {}", self.subject, self.predicate),
        }
    }
}

/// Evaluation verdict for a candidate syllogistic conclusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConclusionVerdict {
    /// Follows necessarily from premises in every consistent model.
    Follows,
    /// Does not follow necessarily (false or contingent in at least one counter-model).
    DoesNotFollow,
}

/// Candidate conclusion paired with its truth verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvaluatedConclusion {
    pub id: usize,
    pub proposition: Proposition,
    pub verdict: ConclusionVerdict,
    pub reason: String,
}

/// Syllogism problem containing premises, conclusions to evaluate, and canonical answer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyllogismProblem {
    pub premises: Vec<Proposition>,
    pub conclusions: Vec<EvaluatedConclusion>,
    pub canonical_answer: String,
    pub explanation: String,
}

impl SyllogismProblem {
    /// Barbara (AAA-1): All A are B, All B are C => All A are C, Some C are A.
    pub fn create_barbara(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::All, term_a, term_b);
        let p2 = Proposition::new(Quantifier::All, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::All, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Transitive subset containment: {} ⊆ {} ⊆ {}.", term_a, term_b, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::Some, term_c, term_a),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Non-empty subset conversion: Since all {} are {}, some {} are {}.", term_a, term_c, term_c, term_a),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    /// Celarent (EAE-1): All A are B, No B are C => No A are C (Follows), Some A are C (DoesNotFollow).
    pub fn create_celarent(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::All, term_a, term_b);
        let p2 = Proposition::new(Quantifier::No, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::No, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Since {} ⊆ {} and {} ∩ {} = ∅, {} and {} must be disjoint.", term_a, term_b, term_b, term_c, term_a, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("Contradicts the disjointness established by the premises."),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    /// Darii (AII-1): All B are C, Some A are B => Some A are C (Follows), All A are C (DoesNotFollow).
    pub fn create_darii(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::All, term_b, term_c);
        let p2 = Proposition::new(Quantifier::Some, term_a, term_b);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("The elements of {} that are in {} are necessarily also in {}.", term_a, term_b, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::All, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("We only know 'Some {} are {}', which does not guarantee 'All {} are {}'.", term_a, term_b, term_a, term_c),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    /// Ferio (EIO-1): No B are C, Some A are B => Some A are not C (Follows), No A are C (DoesNotFollow).
    pub fn create_ferio(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::No, term_b, term_c);
        let p2 = Proposition::new(Quantifier::Some, term_a, term_b);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::SomeNot, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("The elements of {} that belong to {} cannot belong to {}.", term_a, term_b, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::No, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("Other elements of {} might still overlap with {}.", term_a, term_c),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    /// Camestres (AEE-2): All C are B, No A are B => No A are C (Follows), All A are C (DoesNotFollow).
    pub fn create_camestres(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::All, term_c, term_b);
        let p2 = Proposition::new(Quantifier::No, term_a, term_b);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::No, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Since {} ⊆ {} and {} ∩ {} = ∅, {} cannot intersect {}.", term_c, term_b, term_a, term_b, term_a, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: "Disjoint sets cannot have any overlapping elements.".to_string(),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    /// Disjoint Some / Both Invalid (Neither follows): Some A are B, Some B are C => Neither follows.
    pub fn create_disjoint_some(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::Some, term_a, term_b);
        let p2 = Proposition::new(Quantifier::Some, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("Undistributed middle term: {} overlaps with {} and {} overlaps with {}, but the overlapping subsets may be completely disjoint.", term_a, term_b, term_c, term_b),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::All, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("Partial overlaps cannot establish universal containment."),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    /// Only Conclusion II follows
    pub fn create_only_two_follows(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::Some, term_a, term_b);
        let p2 = Proposition::new(Quantifier::All, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::All, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: "Only some A are B, so we cannot conclude all A are C.".to_string(),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("The elements of {} in {} are contained in {}.", term_a, term_b, term_c),
        };

        Self::build(vec![p1, p2], vec![c1, c2])
    }

    fn build(premises: Vec<Proposition>, conclusions: Vec<EvaluatedConclusion>) -> Self {
        let follows_1 = conclusions[0].verdict == ConclusionVerdict::Follows;
        let follows_2 = conclusions[1].verdict == ConclusionVerdict::Follows;

        let canonical_answer = match (follows_1, follows_2) {
            (true, true) => "Both I and II follow".to_string(),
            (true, false) => "Only I follows".to_string(),
            (false, true) => "Only II follows".to_string(),
            (false, false) => "Neither follows".to_string(),
        };

        let mut expl_lines = vec!["**Premises:**".to_string()];
        for (i, p) in premises.iter().enumerate() {
            expl_lines.push(format!("{}. {}", i + 1, p.statement()));
        }
        expl_lines.push("\n**Analysis:**".to_string());
        for c in &conclusions {
            expl_lines.push(format!(
                "- Conclusion {}: '{}' -> **{}** (Reason: {})",
                if c.id == 1 { "I" } else { "II" },
                c.proposition.statement(),
                if c.verdict == ConclusionVerdict::Follows { "Follows" } else { "Does Not Follow" },
                c.reason
            ));
        }
        expl_lines.push(format!("\n**Correct Verdict:** {}", canonical_answer));

        Self {
            premises,
            conclusions,
            canonical_answer,
            explanation: expl_lines.join("\n"),
        }
    }

    /// Helper for deterministic answer evaluation.
    pub fn is_correct(&self, submission: &str) -> bool {
        let clean_sub = submission.trim().to_lowercase();
        let clean_exp = self.canonical_answer.trim().to_lowercase();
        clean_sub == clean_exp
            || (clean_exp.contains("both") && (clean_sub == "both" || clean_sub == "both i and ii follow" || clean_sub == "option c" || clean_sub == "c"))
            || (clean_exp.contains("only i follows") && (clean_sub == "only i follows" || clean_sub == "1" || clean_sub == "option a" || clean_sub == "a"))
            || (clean_exp.contains("only ii follows") && (clean_sub == "only ii follows" || clean_sub == "2" || clean_sub == "option b" || clean_sub == "b"))
            || (clean_exp.contains("neither") && (clean_sub == "neither follows" || clean_sub == "none" || clean_sub == "neither" || clean_sub == "option d" || clean_sub == "d"))
    }
}

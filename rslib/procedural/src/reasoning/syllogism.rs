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
    /// Create and evaluate a canonical 2-premise syllogism.
    /// Example 1 (Barbara): All A are B, All B are C => All A are C (Follows), Some A are C (Follows), No A are C (DoesNotFollow).
    /// Example 2 (Celarent): All A are B, No B are C => No A are C (Follows), Some A are not C (Follows), All A are C (DoesNotFollow).
    /// Example 3 (Darii): All B are C, Some A are B => Some A are C (Follows).
    pub fn create_barbara(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::All, term_a, term_b);
        let p2 = Proposition::new(Quantifier::All, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::All, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Since all {} are {} and all {} are {}, transitive subset inclusion implies all {} are {}.", term_a, term_b, term_b, term_c, term_a, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::Some, term_c, term_a),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Since non-empty set {} is a subset of {}, some {} are {}.", term_a, term_c, term_c, term_a),
        };

        let explanation = format!(
            "Premises:\n1. {}\n2. {}\n\nBoth Conclusion I ('{}') and Conclusion II ('{}') follow logically.",
            p1.statement(),
            p2.statement(),
            c1.proposition.statement(),
            c2.proposition.statement()
        );

        Self {
            premises: vec![p1, p2],
            conclusions: vec![c1, c2],
            canonical_answer: "Both I and II follow".to_string(),
            explanation,
        }
    }

    pub fn create_celarent(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::All, term_a, term_b);
        let p2 = Proposition::new(Quantifier::No, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::No, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("Since all {} are within {} and {} is disjoint from {}, {} and {} are disjoint (No {} are {}).", term_a, term_b, term_b, term_c, term_a, term_c, term_a, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("Contradicts the disjointness established by the premises."),
        };

        let explanation = format!(
            "Premises:\n1. {}\n2. {}\n\nOnly Conclusion I ('{}') follows logically.",
            p1.statement(),
            p2.statement(),
            c1.proposition.statement()
        );

        Self {
            premises: vec![p1, p2],
            conclusions: vec![c1, c2],
            canonical_answer: "Only I follows".to_string(),
            explanation,
        }
    }

    pub fn create_darii(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::Some, term_a, term_b);
        let p2 = Proposition::new(Quantifier::All, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::Follows,
            reason: format!("The overlapping portion between {} and {} is entirely contained inside {}, so some {} are {}.", term_a, term_b, term_c, term_a, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::All, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("We only know a subset of {} belongs to {}, not all {}.", term_a, term_c, term_a),
        };

        let explanation = format!(
            "Premises:\n1. {}\n2. {}\n\nOnly Conclusion I ('{}') follows logically.",
            p1.statement(),
            p2.statement(),
            c1.proposition.statement()
        );

        Self {
            premises: vec![p1, p2],
            conclusions: vec![c1, c2],
            canonical_answer: "Only I follows".to_string(),
            explanation,
        }
    }

    pub fn create_disjoint_some(term_a: &str, term_b: &str, term_c: &str) -> Self {
        let p1 = Proposition::new(Quantifier::Some, term_a, term_b);
        let p2 = Proposition::new(Quantifier::Some, term_b, term_c);

        let c1 = EvaluatedConclusion {
            id: 1,
            proposition: Proposition::new(Quantifier::Some, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("Two 'Some' premises with middle term {} do not yield a necessary connection between {} and {}.", term_b, term_a, term_c),
        };
        let c2 = EvaluatedConclusion {
            id: 2,
            proposition: Proposition::new(Quantifier::No, term_a, term_c),
            verdict: ConclusionVerdict::DoesNotFollow,
            reason: format!("While they may be disjoint, they may also overlap. Neither is guaranteed."),
        };

        let explanation = format!(
            "Premises:\n1. {}\n2. {}\n\nNeither Conclusion I nor Conclusion II follows necessarily.",
            p1.statement(),
            p2.statement()
        );

        Self {
            premises: vec![p1, p2],
            conclusions: vec![c1, c2],
            canonical_answer: "Neither follows".to_string(),
            explanation,
        }
    }

    /// Check if a submitted response is deterministically correct.
    pub fn is_correct(&self, submission: &str) -> bool {
        let clean = submission.trim().to_lowercase().replace('_', " ");
        let exp = self.canonical_answer.to_lowercase();
        clean == exp
            || (exp.contains("only i follows") && (clean == "only i" || clean == "1" || clean == "i" || clean == "option a"))
            || (exp.contains("only ii follows") && (clean == "only ii" || clean == "2" || clean == "ii" || clean == "option b"))
            || (exp.contains("both i and ii follow") && (clean == "both" || clean == "both follow" || clean == "option c"))
            || (exp.contains("neither follows") && (clean == "neither" || clean == "none" || clean == "option d"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syllogism_barbara_validity() {
        let syl = SyllogismProblem::create_barbara("cats", "mammals", "animals");
        assert_eq!(syl.premises.len(), 2);
        assert_eq!(syl.conclusions[0].verdict, ConclusionVerdict::Follows);
        assert_eq!(syl.conclusions[1].verdict, ConclusionVerdict::Follows);
        assert!(syl.is_correct("Both I and II follow"));
        assert!(syl.is_correct("both"));
    }

    #[test]
    fn test_syllogism_celarent_validity() {
        let syl = SyllogismProblem::create_celarent("roses", "flowers", "rocks");
        assert_eq!(syl.conclusions[0].verdict, ConclusionVerdict::Follows);
        assert_eq!(syl.conclusions[1].verdict, ConclusionVerdict::DoesNotFollow);
        assert!(syl.is_correct("Only I follows"));
        assert!(!syl.is_correct("Both I and II follow"));
    }

    #[test]
    fn test_syllogism_two_particulars_neither_follows() {
        let syl = SyllogismProblem::create_disjoint_some("apples", "fruits", "red objects");
        assert_eq!(syl.conclusions[0].verdict, ConclusionVerdict::DoesNotFollow);
        assert_eq!(syl.conclusions[1].verdict, ConclusionVerdict::DoesNotFollow);
        assert!(syl.is_correct("Neither follows"));
    }
}

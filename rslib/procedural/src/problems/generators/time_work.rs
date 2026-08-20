// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::core::{ProblemFamilyId, Result};
use crate::diagnostics::ErrorCategory;
use crate::problems::generator::ProblemGenerator;
use crate::problems::steps::{SolutionGraph, StepHint, StepNode, StepType};
use crate::problems::validator::{AnswerEvaluation, NumericAnswerParser, ProblemValidator};
use crate::problems::ProblemInstance;

pub const FAMILY_TIME_WORK: &str = "family.math.time_work.basic";
pub const TEMPLATE_TIME_WORK_V1: &str = "math.time_work.basic.v1";

/// Greatest Common Divisor helper
pub fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    if a == 0 { 1 } else { a }
}

/// Least Common Multiple helper
pub fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a.abs() / gcd(a, b)) * b.abs()
}

/// Exact Rational representation for exact time & work arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rational {
    pub num: i64,
    pub den: i64,
}

impl Rational {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "Denominator cannot be zero");
        let g = gcd(num, den);
        let sign = if den < 0 { -1 } else { 1 };
        Self {
            num: sign * (num / g),
            den: (den / g).abs(),
        }
    }

    pub fn from_integer(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    pub fn add(self, other: Self) -> Self {
        Self::new(
            self.num * other.den + other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn sub(self, other: Self) -> Self {
        Self::new(
            self.num * other.den - other.num * self.den,
            self.den * other.den,
        )
    }

    pub fn mul(self, other: Self) -> Self {
        Self::new(self.num * other.num, self.den * other.den)
    }

    pub fn div(self, other: Self) -> Self {
        assert!(other.num != 0, "Division by zero in Rational");
        Self::new(self.num * other.den, self.den * other.num)
    }

    pub fn recip(self) -> Self {
        assert!(self.num != 0, "Reciprocal of zero is undefined");
        Self::new(self.den, self.num)
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    pub fn is_integer(self) -> bool {
        self.den == 1
    }

    pub fn format_clean(&self) -> String {
        if self.den == 1 {
            format!("{}", self.num)
        } else if self.num.abs() > self.den {
            let whole = self.num / self.den;
            let rem = (self.num % self.den).abs();
            format!("{} {}/{}", whole, rem, self.den)
        } else {
            format!("{}/{}", self.num, self.den)
        }
    }

    pub fn format_latex(&self) -> String {
        if self.den == 1 {
            format!("{}", self.num)
        } else {
            format!("\\frac{{{}}}{{{}}}", self.num, self.den)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeWorkVariant {
    SingleWorkerRate,
    TwoWorkersTogether,
    WorkerLeavesEarly,
    RelativeEfficiency,
    PipesInletOutlet,
}

impl TimeWorkVariant {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeWorkVariant::SingleWorkerRate => "single_worker_rate",
            TimeWorkVariant::TwoWorkersTogether => "two_workers_together",
            TimeWorkVariant::WorkerLeavesEarly => "worker_leaves_early",
            TimeWorkVariant::RelativeEfficiency => "relative_efficiency",
            TimeWorkVariant::PipesInletOutlet => "pipes_inlet_outlet",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TimeWorkGenerator;

const WORKER_NAMES: &[&str] = &[
    "Alice", "Bob", "Charlie", "David", "Emma", "Frank", "Grace", "Henry",
    "Ivy", "Jack", "Karan", "Liam", "Maya", "Noah", "Olivia", "Priya",
];

impl TimeWorkGenerator {
    pub fn generate_problem(seed: u64, difficulty_level: u32, variant: Option<&str>) -> ProblemInstance {
        let mut rng = StdRng::seed_from_u64(seed);
        let chosen_variant = if let Some(v) = variant {
            match v {
                "single_worker_rate" => TimeWorkVariant::SingleWorkerRate,
                "two_workers_together" => TimeWorkVariant::TwoWorkersTogether,
                "worker_leaves_early" => TimeWorkVariant::WorkerLeavesEarly,
                "relative_efficiency" => TimeWorkVariant::RelativeEfficiency,
                "pipes_inlet_outlet" => TimeWorkVariant::PipesInletOutlet,
                _ => TimeWorkVariant::TwoWorkersTogether,
            }
        } else {
            match difficulty_level {
                1 => TimeWorkVariant::SingleWorkerRate,
                2 => TimeWorkVariant::TwoWorkersTogether,
                3 => TimeWorkVariant::WorkerLeavesEarly,
                4 => TimeWorkVariant::RelativeEfficiency,
                _ => TimeWorkVariant::PipesInletOutlet,
            }
        };

        match chosen_variant {
            TimeWorkVariant::SingleWorkerRate => Self::generate_level_1(&mut rng, seed),
            TimeWorkVariant::TwoWorkersTogether => Self::generate_level_2(&mut rng, seed),
            TimeWorkVariant::WorkerLeavesEarly => Self::generate_level_3(&mut rng, seed),
            TimeWorkVariant::RelativeEfficiency => Self::generate_level_4(&mut rng, seed),
            TimeWorkVariant::PipesInletOutlet => Self::generate_level_5(&mut rng, seed),
        }
    }

    /// Level 1: Single worker rate with exact rational calculation.
    /// Person A completes project in D days. How many days for fraction p/q of project?
    fn generate_level_1(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let name_idx = rng.random_range(0..WORKER_NAMES.len());
        let person = WORKER_NAMES[name_idx];

        let total_days = rng.random_range(8..=48) as i64;
        // Fractions like 1/2, 1/3, 2/3, 1/4, 3/4, 2/5, 3/5, 4/5, 5/6
        let fractions: &[(i64, i64)] = &[
            (1, 2), (1, 3), (2, 3), (1, 4), (3, 4), (1, 5), (2, 5), (3, 5), (4, 5), (1, 6), (5, 6), (3, 8), (5, 8),
        ];
        let (p, q) = fractions[rng.random_range(0..fractions.len())];

        let fraction_rat = Rational::new(p, q);
        let total_days_rat = Rational::from_integer(total_days);
        let required_days_rat = total_days_rat.mul(fraction_rat);
        let ans_f64 = required_days_rat.to_f64();

        let prompt = format!(
            "{} can complete a full project alone in **{} days**.\n\n\
             How many days will {} take to complete **{}/{}** of the project at the same rate?",
            person, total_days, person, p, q
        );

        let solution = format!(
            "**Step 1:** Calculate daily rate of work:\n\
             \\[ \\text{{Daily Rate}} = \\frac{{1}}{{{}}} \\text{{ of project per day}} \\]\n\n\
             **Step 2:** Calculate days needed for \\({}/{} \\) of the project:\n\
             \\[ \\text{{Days Required}} = \\frac{{{}/{}}}{{1/{}}} = {} \\times \\frac{{{}}}{{{}}} = {} \\text{{ days}} \\]",
            total_days, p, q, p, q, total_days, total_days, p, q,
            if required_days_rat.is_integer() {
                format!("**{}**", required_days_rat.num)
            } else {
                format!("**{}** (or \\({}\\))", required_days_rat.format_clean(), required_days_rat.format_latex())
            }
        );

        let parameters = serde_json::json!({
            "variant": "single_worker_rate",
            "person": person,
            "total_days": total_days,
            "fraction_num": p,
            "fraction_den": q,
            "required_days_num": required_days_rat.num,
            "required_days_den": required_days_rat.den,
            "required_days_f64": ans_f64,
        });

        let correct_answer = serde_json::json!({
            "value": ans_f64,
            "formatted": required_days_rat.format_clean(),
            "fraction": format!("{}/{}", required_days_rat.num, required_days_rat.den),
            "unit": "days",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_daily_rate",
            StepType::Transformation,
            "Express daily rate as 1 / Total Days",
            format!("1 / {} per day", total_days),
            format!("1/{}", total_days),
        )
        .with_hints(vec![
            StepHint::principle("If a task takes D days, the worker completes 1/D of the work per day."),
            StepHint::operation(format!("Rate = 1/{}", total_days)),
            StepHint::intermediate_relation(format!("1/{} work/day", total_days)),
        ]);

        let step2 = StepNode::new(
            "calc_fraction_days",
            StepType::FinalAnswer,
            "Multiply total days by the fraction",
            format!("{} * ({}/{}) = {}", total_days, p, q, required_days_rat.format_clean()),
            required_days_rat.format_clean(),
        )
        .with_expected_value(ans_f64)
        .with_dependencies(vec!["calc_daily_rate".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Multiply total days by the target fraction (p/q)."),
            StepHint::operation(format!("Compute {} * ({}/{})", total_days, p, q)),
            StepHint::intermediate_relation(format!("{} days", required_days_rat.format_clean())),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_fraction_days");

        ProblemInstance::new(
            format!("inst-work-1-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 1,
            "target_time_ms": 25_000,
            "variant": "single_worker_rate",
        }))
    }

    /// Level 2: Two workers together with rich dynamic parameter combinations and exact reciprocal arithmetic.
    fn generate_level_2(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let name_a = WORKER_NAMES[rng.random_range(0..WORKER_NAMES.len() / 2)];
        let name_b = WORKER_NAMES[rng.random_range(WORKER_NAMES.len() / 2..WORKER_NAMES.len())];

        // Generate combinations: either clean integer together days or clean rational halves/fifths
        let (days_a, days_b) = if rng.random_bool(0.7) {
            // Pick together_days T in 3..=24 and factor a > T
            let t = rng.random_range(3..=20) as i64;
            // a = t + k, b = t * (t + k) / k
            // Find divisors of t^2
            let t_sq = t * t;
            let mut divisors = Vec::new();
            for k in 1..=t {
                if t_sq % k == 0 {
                    divisors.push(k);
                }
            }
            let k = divisors[rng.random_range(0..divisors.len())];
            let da = t + k;
            let db = t + (t_sq / k);
            (da, db)
        } else {
            let da = rng.random_range(6..=36) as i64;
            let db = rng.random_range(6..=36) as i64;
            (da, db)
        };

        let rate_a = Rational::new(1, days_a);
        let rate_b = Rational::new(1, days_b);
        let combined_rate = rate_a.add(rate_b);
        let together_days_rat = combined_rate.recip();
        let ans_f64 = together_days_rat.to_f64();

        let prompt = format!(
            "{} can complete a job in **{} days** and {} can complete the same job in **{} days**.\n\n\
             How many days will they take to complete the job working together?",
            name_a, days_a, name_b, days_b
        );

        let total_lcm = lcm(days_a, days_b);
        let units_a = total_lcm / days_a;
        let units_b = total_lcm / days_b;
        let total_units_per_day = units_a + units_b;

        let solution = format!(
            "**Method 1 (Unit Work / LCM Method):**\n\
             - Let total work = \\(\\text{{LCM}}({}, {}) = {}\\) units.\n\
             - {}'s rate = \\({} / {} = {}\\) units/day.\n\
             - {}'s rate = \\({} / {} = {}\\) units/day.\n\
             - Combined rate = \\({} + {} = {}\\) units/day.\n\
             - Time together = \\(\\frac{{{}}}{{{}}} = {}\\) days.\n\n\
             **Method 2 (Reciprocal Formula):**\n\
             \\[ \\text{{Time}} = \\frac{{{} \\times {}}}{{{} + {}}} = \\frac{{{}}}{{{}}} = {} \\text{{ days}} \\]",
            days_a, days_b, total_lcm,
            name_a, total_lcm, days_a, units_a,
            name_b, total_lcm, days_b, units_b,
            units_a, units_b, total_units_per_day,
            total_lcm, total_units_per_day,
            if together_days_rat.is_integer() {
                format!("**{}**", together_days_rat.num)
            } else {
                format!("**{}** (or \\({}\\))", together_days_rat.format_clean(), together_days_rat.format_latex())
            },
            days_a, days_b, days_a, days_b, days_a * days_b, days_a + days_b,
            if together_days_rat.is_integer() {
                format!("**{}**", together_days_rat.num)
            } else {
                format!("**{}**", together_days_rat.format_clean())
            }
        );

        let parameters = serde_json::json!({
            "variant": "two_workers_together",
            "name_a": name_a,
            "name_b": name_b,
            "days_a": days_a,
            "days_b": days_b,
            "together_days_num": together_days_rat.num,
            "together_days_den": together_days_rat.den,
            "together_days_f64": ans_f64,
        });

        let correct_answer = serde_json::json!({
            "value": ans_f64,
            "formatted": together_days_rat.format_clean(),
            "fraction": format!("{}/{}", together_days_rat.num, together_days_rat.den),
            "unit": "days",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_combined_rate",
            StepType::Transformation,
            "Sum individual reciprocal rates (1/A + 1/B)",
            format!("1/{} + 1/{} = {}/{}", days_a, days_b, combined_rate.num, combined_rate.den),
            format!("{}/{}", combined_rate.num, combined_rate.den),
        )
        .with_hints(vec![
            StepHint::principle("Combined rate = (1/days_A) + (1/days_B)."),
            StepHint::operation(format!("Add 1/{} + 1/{}.", days_a, days_b)),
            StepHint::intermediate_relation(format!("{}/{} units/day", combined_rate.num, combined_rate.den)),
        ]);

        let step2 = StepNode::new(
            "calc_together_days",
            StepType::FinalAnswer,
            "Invert combined rate to find days",
            format!("1 / ({}/{}) = {}", combined_rate.num, combined_rate.den, together_days_rat.format_clean()),
            together_days_rat.format_clean(),
        )
        .with_expected_value(ans_f64)
        .with_dependencies(vec!["calc_combined_rate".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Time together = Reciprocal of combined rate = (A * B) / (A + B)."),
            StepHint::operation(format!("Compute ({} * {}) / ({} + {}).", days_a, days_b, days_a, days_b)),
            StepHint::intermediate_relation(format!("{} days", together_days_rat.format_clean())),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_together_days");

        ProblemInstance::new(
            format!("inst-work-2-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 2,
            "target_time_ms": 35_000,
            "variant": "two_workers_together",
        }))
    }

    /// Level 3: Worker leaves early / joins midway.
    /// Person A finishes in da, Person B in db. They work together for t_together days, then A leaves.
    /// How many more days will B take to complete remaining work?
    fn generate_level_3(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let name_a = WORKER_NAMES[rng.random_range(0..WORKER_NAMES.len() / 2)];
        let name_b = WORKER_NAMES[rng.random_range(WORKER_NAMES.len() / 2..WORKER_NAMES.len())];

        let days_a = rng.random_range(8..=30) as i64;
        let days_b = rng.random_range(8..=30) as i64;
        let total_lcm = lcm(days_a, days_b);

        let rate_a_units = total_lcm / days_a;
        let rate_b_units = total_lcm / days_b;
        let joint_rate = rate_a_units + rate_b_units;

        // Together days must be strictly less than total time to complete
        let max_joint_days = (total_lcm - 1) / joint_rate;
        let together_worked = rng.random_range(1..=max_joint_days.max(1));

        let work_done = joint_rate * together_worked;
        let work_remaining = total_lcm - work_done;

        let remaining_days_rat = Rational::new(work_remaining, rate_b_units);
        let ans_f64 = remaining_days_rat.to_f64();

        let prompt = format!(
            "{} can finish a project in **{} days** and {} can finish it in **{} days**.\n\n\
             They start working together, but after **{} days**, {} leaves.\n\
             How many more days will {} take to complete the remaining work alone?",
            name_a, days_a, name_b, days_b, together_worked, name_a, name_b
        );

        let solution = format!(
            "**Step 1:** Total work = \\(\\text{{LCM}}({}, {}) = {}\\) units.\n\
             - {}'s daily work = \\({} / {} = {}\\) units/day.\n\
             - {}'s daily work = \\({} / {} = {}\\) units/day.\n\n\
             **Step 2:** Work completed in {} days working together:\n\
             \\[ ({} + {}) \\times {} = {} \\times {} = {} \\text{{ units}} \\]\n\n\
             **Step 3:** Remaining work:\n\
             \\[ {} - {} = {} \\text{{ units}} \\]\n\n\
             **Step 4:** Time taken by {} alone:\n\
             \\[ \\text{{Time}} = \\frac{{{}}}{{{}}} = {} \\text{{ days}} \\]",
            days_a, days_b, total_lcm,
            name_a, total_lcm, days_a, rate_a_units,
            name_b, total_lcm, days_b, rate_b_units,
            together_worked,
            rate_a_units, rate_b_units, together_worked, joint_rate, together_worked, work_done,
            total_lcm, work_done, work_remaining,
            name_b,
            work_remaining, rate_b_units,
            if remaining_days_rat.is_integer() {
                format!("**{}**", remaining_days_rat.num)
            } else {
                format!("**{}** (or \\({}\\))", remaining_days_rat.format_clean(), remaining_days_rat.format_latex())
            }
        );

        let parameters = serde_json::json!({
            "variant": "worker_leaves_early",
            "name_a": name_a,
            "name_b": name_b,
            "days_a": days_a,
            "days_b": days_b,
            "together_worked": together_worked,
            "work_remaining": work_remaining,
            "remaining_days_num": remaining_days_rat.num,
            "remaining_days_den": remaining_days_rat.den,
            "remaining_days_f64": ans_f64,
        });

        let correct_answer = serde_json::json!({
            "value": ans_f64,
            "formatted": remaining_days_rat.format_clean(),
            "fraction": format!("{}/{}", remaining_days_rat.num, remaining_days_rat.den),
            "unit": "days",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_joint_work",
            StepType::Transformation,
            "Compute work completed before departure",
            format!("({} + {}) * {} = {} units of {}", rate_a_units, rate_b_units, together_worked, work_done, total_lcm),
            format!("{}", work_done),
        )
        .with_hints(vec![
            StepHint::principle("Find total work (LCM) and calculate units completed by both workers together."),
            StepHint::operation(format!("Compute ({} + {}) * {}.", rate_a_units, rate_b_units, together_worked)),
            StepHint::intermediate_relation(format!("{} units completed", work_done)),
        ]);

        let step2 = StepNode::new(
            "calc_remaining_units",
            StepType::IntermediateResult,
            "Subtract joint work from total work",
            format!("{} - {} = {} units", total_lcm, work_done, work_remaining),
            format!("{}", work_remaining),
        )
        .with_dependencies(vec!["calc_joint_work".to_string()])
        .with_hints(vec![
            StepHint::principle("Remaining work = Total units - Work completed."),
            StepHint::operation(format!("Subtract {} - {}.", total_lcm, work_done)),
            StepHint::intermediate_relation(format!("{} units remaining", work_remaining)),
        ]);

        let step3 = StepNode::new(
            "calc_b_alone_days",
            StepType::FinalAnswer,
            "Divide remaining units by remaining worker's daily rate",
            format!("{} / {} = {}", work_remaining, rate_b_units, remaining_days_rat.format_clean()),
            remaining_days_rat.format_clean(),
        )
        .with_expected_value(ans_f64)
        .with_dependencies(vec!["calc_remaining_units".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Days alone = Remaining units / Worker's unit rate."),
            StepHint::operation(format!("Divide {} by {}.", work_remaining, rate_b_units)),
            StepHint::intermediate_relation(format!("{} days", remaining_days_rat.format_clean())),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2, step3], "calc_b_alone_days");

        ProblemInstance::new(
            format!("inst-work-3-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 3,
            "target_time_ms": 45_000,
            "variant": "worker_leaves_early",
        }))
    }

    /// Level 4: Relative Efficiency (Worker A is k times as efficient as B, or efficiency ratio p:q)
    fn generate_level_4(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let name_a = WORKER_NAMES[rng.random_range(0..WORKER_NAMES.len() / 2)];
        let name_b = WORKER_NAMES[rng.random_range(WORKER_NAMES.len() / 2..WORKER_NAMES.len())];

        // Efficiency ratio p : q (e.g. 2:1, 3:1, 3:2, 4:1, 5:2, 5:3)
        let ratios: &[(i64, i64, &str)] = &[
            (2, 1, "twice as efficient as"),
            (3, 1, "3 times as efficient as"),
            (4, 1, "4 times as efficient as"),
            (3, 2, "1.5 times as efficient as"),
            (5, 2, "2.5 times as efficient as"),
            (5, 3, "5/3 times as efficient as"),
        ];
        let (p, q, ratio_phrase) = ratios[rng.random_range(0..ratios.len())];
        let together_days = rng.random_range(4..=24) as i64;

        // Ask for A alone or B alone
        let ask_for_a = rng.random_bool(0.6);

        let total_daily_units = p + q;
        let total_work = total_daily_units * together_days;

        let (target_person, target_units, ans_days_rat) = if ask_for_a {
            (name_a, p, Rational::new(total_work, p))
        } else {
            (name_b, q, Rational::new(total_work, q))
        };
        let ans_f64 = ans_days_rat.to_f64();

        let prompt = format!(
            "{} is **{}** {}. Working together, they can finish a job in **{} days**.\n\n\
             In how many days can **{}** finish the entire job working alone?",
            name_a, ratio_phrase, name_b, together_days, target_person
        );

        let solution = format!(
            "**Step 1:** Efficiency ratio \\({} : {} = {} : {}\\).\n\
             - {}'s daily efficiency = \\({}\\) units/day.\n\
             - {}'s daily efficiency = \\({}\\) units/day.\n\
             - Combined daily efficiency = \\({} + {} = {}\\) units/day.\n\n\
             **Step 2:** Calculate total work units:\n\
             \\[ \\text{{Total Work}} = {} \\text{{ units/day}} \\times {} \\text{{ days}} = {} \\text{{ units}} \\]\n\n\
             **Step 3:** Calculate time for {} alone:\n\
             \\[ \\text{{Time}} = \\frac{{{}}}{{{}}} = {} \\text{{ days}} \\]",
            name_a, name_b, p, q,
            name_a, p,
            name_b, q,
            p, q, total_daily_units,
            total_daily_units, together_days, total_work,
            target_person,
            total_work, target_units,
            if ans_days_rat.is_integer() {
                format!("**{}**", ans_days_rat.num)
            } else {
                format!("**{}** (or \\({}\\))", ans_days_rat.format_clean(), ans_days_rat.format_latex())
            }
        );

        let parameters = serde_json::json!({
            "variant": "relative_efficiency",
            "name_a": name_a,
            "name_b": name_b,
            "ratio_p": p,
            "ratio_q": q,
            "together_days": together_days,
            "target_person": target_person,
            "ans_days_num": ans_days_rat.num,
            "ans_days_den": ans_days_rat.den,
            "ans_days_f64": ans_f64,
        });

        let correct_answer = serde_json::json!({
            "value": ans_f64,
            "formatted": ans_days_rat.format_clean(),
            "fraction": format!("{}/{}", ans_days_rat.num, ans_days_rat.den),
            "unit": "days",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_total_work",
            StepType::Transformation,
            "Calculate total work units from combined efficiency",
            format!("({} + {}) * {} = {} units", p, q, together_days, total_work),
            format!("{}", total_work),
        )
        .with_hints(vec![
            StepHint::principle("Assign unit efficiencies based on the ratio (A = p, B = q). Total work = (p + q) * together_days."),
            StepHint::operation(format!("Compute ({} + {}) * {}.", p, q, together_days)),
            StepHint::intermediate_relation(format!("{} units", total_work)),
        ]);

        let step2 = StepNode::new(
            "calc_target_alone",
            StepType::FinalAnswer,
            "Divide total work by target worker's efficiency",
            format!("{} / {} = {}", total_work, target_units, ans_days_rat.format_clean()),
            ans_days_rat.format_clean(),
        )
        .with_expected_value(ans_f64)
        .with_dependencies(vec!["calc_total_work".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Time alone = Total work / Individual efficiency."),
            StepHint::operation(format!("Divide {} by {}.", total_work, target_units)),
            StepHint::intermediate_relation(format!("{} days", ans_days_rat.format_clean())),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_target_alone");

        ProblemInstance::new(
            format!("inst-work-4-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 4,
            "target_time_ms": 50_000,
            "variant": "relative_efficiency",
        }))
    }

    /// Level 5: Pipes & Cisterns (Inlet + Outlet or Multi-Pipe Network)
    fn generate_level_5(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        // Multi-pipe scenario: Pipe A (inlet), Pipe B (inlet/outlet), Pipe C (outlet)
        let is_three_pipes = rng.random_bool(0.4);

        if is_three_pipes {
            let t_a = rng.random_range(6..=18) as i64; // Inlet A
            let t_b = rng.random_range(8..=24) as i64; // Inlet B
            let t_c = rng.random_range(10..=30) as i64; // Outlet C

            let total_cap = lcm(lcm(t_a, t_b), t_c);
            let rate_a = total_cap / t_a;
            let rate_b = total_cap / t_b;
            let rate_c = total_cap / t_c;

            let net_rate = rate_a + rate_b - rate_c;
            if net_rate <= 0 {
                // Ensure net filling
                return Self::generate_level_5_two_pipes(rng, seed);
            }

            let fill_time_rat = Rational::new(total_cap, net_rate);
            let ans_f64 = fill_time_rat.to_f64();

            let prompt = format!(
                "Pipe A can fill a tank in **{} hours** and Pipe B can fill it in **{} hours**, while Pipe C can empty the full tank in **{} hours**.\n\n\
                 If all three pipes are opened simultaneously when the tank is empty, in how many hours will the tank be completely filled?",
                t_a, t_b, t_c
            );

            let solution = format!(
                "**Step 1:** Assume capacity = \\(\\text{{LCM}}({}, {}, {}) = {}\\) liters.\n\
                 - Pipe A fills = \\({} / {} = +{}\\) L/hr.\n\
                 - Pipe B fills = \\({} / {} = +{}\\) L/hr.\n\
                 - Pipe C empties = \\({} / {} = -{}\\) L/hr.\n\n\
                 **Step 2:** Net filling rate:\n\
                 \\[ \\text{{Net Rate}} = {} + {} - {} = +{} \\text{{ L/hr}} \\]\n\n\
                 **Step 3:** Time to fill tank:\n\
                 \\[ \\text{{Time}} = \\frac{{{}}}{{{}}} = {} \\text{{ hours}} \\]",
                t_a, t_b, t_c, total_cap,
                total_cap, t_a, rate_a,
                total_cap, t_b, rate_b,
                total_cap, t_c, rate_c,
                rate_a, rate_b, rate_c, net_rate,
                total_cap, net_rate,
                if fill_time_rat.is_integer() {
                    format!("**{}**", fill_time_rat.num)
                } else {
                    format!("**{}** (or \\({}\\))", fill_time_rat.format_clean(), fill_time_rat.format_latex())
                }
            );

            let parameters = serde_json::json!({
                "variant": "pipes_inlet_outlet_three",
                "pipe_a": t_a,
                "pipe_b": t_b,
                "pipe_c": t_c,
                "capacity": total_cap,
                "net_rate": net_rate,
                "fill_time_num": fill_time_rat.num,
                "fill_time_den": fill_time_rat.den,
                "fill_time_f64": ans_f64,
            });

            let correct_answer = serde_json::json!({
                "value": ans_f64,
                "formatted": fill_time_rat.format_clean(),
                "fraction": format!("{}/{}", fill_time_rat.num, fill_time_rat.den),
                "unit": "hours",
                "solution": solution,
            });

            let step1 = StepNode::new(
                "calc_net_rate",
                StepType::Transformation,
                "Calculate net flow rate (Rate A + Rate B - Rate C)",
                format!("{}/{} + {}/{} - {}/{} = {}/{}", rate_a, total_cap, rate_b, total_cap, rate_c, total_cap, net_rate, total_cap),
                format!("{}/{}", net_rate, total_cap),
            )
            .with_hints(vec![
                StepHint::principle("Inlet pipes have positive filling rates; outlet pipes have negative drainage rates."),
                StepHint::operation(format!("Compute 1/{} + 1/{} - 1/{}.", t_a, t_b, t_c)),
                StepHint::intermediate_relation(format!("Net rate = {}/{} tank/hr", net_rate, total_cap)),
            ]);

            let step2 = StepNode::new(
                "calc_fill_time",
                StepType::FinalAnswer,
                "Divide tank capacity by net rate",
                format!("{} / {} = {}", total_cap, net_rate, fill_time_rat.format_clean()),
                fill_time_rat.format_clean(),
            )
            .with_expected_value(ans_f64)
            .with_dependencies(vec!["calc_net_rate".to_string()])
            .as_final()
            .with_hints(vec![
                StepHint::principle("Time = Capacity / Net Rate."),
                StepHint::operation(format!("Divide {} by {}.", total_cap, net_rate)),
                StepHint::intermediate_relation(format!("{} hours", fill_time_rat.format_clean())),
            ]);

            let graph = SolutionGraph::new(vec![step1, step2], "calc_fill_time");

            ProblemInstance::new(
                format!("inst-work-5-{}", seed),
                FAMILY_TIME_WORK,
                seed,
                parameters,
                prompt,
                correct_answer,
            )
            .with_solution_graph(graph)
            .with_metadata(serde_json::json!({
                "difficulty_level": 5,
                "target_time_ms": 60_000,
                "variant": "pipes_inlet_outlet",
            }))
        } else {
            Self::generate_level_5_two_pipes(rng, seed)
        }
    }

    fn generate_level_5_two_pipes(rng: &mut StdRng, seed: u64) -> ProblemInstance {
        let fill_hours = rng.random_range(6..=24) as i64;
        let empty_hours = fill_hours + rng.random_range(2..=18) as i64; // Ensures empty_hours > fill_hours

        let total_cap = lcm(fill_hours, empty_hours);
        let rate_fill = total_cap / fill_hours;
        let rate_empty = total_cap / empty_hours;
        let net_rate = rate_fill - rate_empty;

        let fill_time_rat = Rational::new(total_cap, net_rate);
        let ans_f64 = fill_time_rat.to_f64();

        let prompt = format!(
            "Pipe A can fill a reservoir in **{} hours**, while Pipe B can empty the full reservoir in **{} hours**.\n\n\
             If both pipes are opened simultaneously starting with an empty reservoir, in how many hours will it be completely filled?",
            fill_hours, empty_hours
        );

        let solution = format!(
            "**Step 1:** Capacity = \\(\\text{{LCM}}({}, {}) = {}\\) units.\n\
             - Pipe A rate = \\(+{}\\) units/hr.\n\
             - Pipe B rate = \\(-{}\\) units/hr.\n\n\
             **Step 2:** Net rate = \\({} - {} = +{}\\) units/hr.\n\n\
             **Step 3:** Total time to fill:\n\
             \\[ \\text{{Time}} = \\frac{{{}}}{{{}}} = {} \\text{{ hours}} \\]",
            fill_hours, empty_hours, total_cap,
            rate_fill, rate_empty,
            rate_fill, rate_empty, net_rate,
            total_cap, net_rate,
            if fill_time_rat.is_integer() {
                format!("**{}**", fill_time_rat.num)
            } else {
                format!("**{}** (or \\({}\\))", fill_time_rat.format_clean(), fill_time_rat.format_latex())
            }
        );

        let parameters = serde_json::json!({
            "variant": "pipes_inlet_outlet_two",
            "fill_hours": fill_hours,
            "empty_hours": empty_hours,
            "capacity": total_cap,
            "net_rate": net_rate,
            "fill_time_num": fill_time_rat.num,
            "fill_time_den": fill_time_rat.den,
            "fill_time_f64": ans_f64,
        });

        let correct_answer = serde_json::json!({
            "value": ans_f64,
            "formatted": fill_time_rat.format_clean(),
            "fraction": format!("{}/{}", fill_time_rat.num, fill_time_rat.den),
            "unit": "hours",
            "solution": solution,
        });

        let step1 = StepNode::new(
            "calc_net_flow",
            StepType::Transformation,
            "Compute net flow rate 1/T_fill - 1/T_empty",
            format!("1/{} - 1/{} = {}/{}", fill_hours, empty_hours, net_rate, total_cap),
            format!("{}/{}", net_rate, total_cap),
        )
        .with_hints(vec![
            StepHint::principle("Net filling rate = (1 / Fill time) - (1 / Empty time)."),
            StepHint::operation(format!("Compute 1/{} - 1/{}.", fill_hours, empty_hours)),
            StepHint::intermediate_relation(format!("{}/{} reservoir/hr", net_rate, total_cap)),
        ]);

        let step2 = StepNode::new(
            "calc_fill_time",
            StepType::FinalAnswer,
            "Invert net rate to find total hours",
            format!("{} / {} = {}", total_cap, net_rate, fill_time_rat.format_clean()),
            fill_time_rat.format_clean(),
        )
        .with_expected_value(ans_f64)
        .with_dependencies(vec!["calc_net_flow".to_string()])
        .as_final()
        .with_hints(vec![
            StepHint::principle("Time = Capacity / Net rate = (Fill * Empty) / (Empty - Fill)."),
            StepHint::operation(format!("Compute ({} * {}) / ({} - {}).", fill_hours, empty_hours, empty_hours, fill_hours)),
            StepHint::intermediate_relation(format!("{} hours", fill_time_rat.format_clean())),
        ]);

        let graph = SolutionGraph::new(vec![step1, step2], "calc_fill_time");

        ProblemInstance::new(
            format!("inst-work-5-{}", seed),
            FAMILY_TIME_WORK,
            seed,
            parameters,
            prompt,
            correct_answer,
        )
        .with_solution_graph(graph)
        .with_metadata(serde_json::json!({
            "difficulty_level": 5,
            "target_time_ms": 55_000,
            "variant": "pipes_inlet_outlet",
        }))
    }
}

impl ProblemGenerator for TimeWorkGenerator {
    fn family_id(&self) -> &str {
        FAMILY_TIME_WORK
    }

    fn template_ref(&self) -> &str {
        TEMPLATE_TIME_WORK_V1
    }

    fn supported_variants(&self) -> Vec<String> {
        vec![
            "single_worker_rate".to_string(),
            "two_workers_together".to_string(),
            "worker_leaves_early".to_string(),
            "relative_efficiency".to_string(),
            "pipes_inlet_outlet".to_string(),
        ]
    }

    fn target_latency_ms(&self, difficulty_level: u32) -> u64 {
        match difficulty_level {
            1 => 25_000,
            2 => 35_000,
            3 => 45_000,
            4 => 50_000,
            _ => 60_000,
        }
    }

    fn generate(
        &self,
        _family_id: &ProblemFamilyId,
        seed: u64,
        difficulty_level: u32,
        variant: Option<&str>,
    ) -> Result<ProblemInstance> {
        Ok(Self::generate_problem(seed, difficulty_level, variant))
    }
}

pub struct TimeWorkValidator;

impl ProblemValidator for TimeWorkValidator {
    fn family_id(&self) -> &str {
        FAMILY_TIME_WORK
    }

    fn evaluate(
        &self,
        instance: &ProblemInstance,
        student_input: &serde_json::Value,
        _time_taken_ms: u64,
        _target_time_ms: u64,
    ) -> AnswerEvaluation {
        let expected_val = instance
            .correct_answer
            .get("value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let parsed_val = NumericAnswerParser::parse_value(student_input);
        let Some(student_num) = parsed_val else {
            return AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: None,
                canonical_value: expected_val,
                error_category: Some(ErrorCategory::Calculation),
                diagnostic_message: Some("Could not parse answer as a number or valid fraction.".to_string()),
            };
        };

        let diff = (student_num - expected_val).abs();
        let is_correct = diff <= 0.02 || (expected_val > 0.0 && diff / expected_val <= 0.005);

        if is_correct {
            AnswerEvaluation {
                is_correct: true,
                score: 1.0,
                parsed_student_value: Some(student_num),
                canonical_value: expected_val,
                error_category: None,
                diagnostic_message: None,
            }
        } else {
            let (cat, msg) = Self::classify_misconception(student_num, &instance.parameters, expected_val);
            AnswerEvaluation {
                is_correct: false,
                score: 0.0,
                parsed_student_value: Some(student_num),
                canonical_value: expected_val,
                error_category: Some(cat),
                diagnostic_message: Some(msg),
            }
        }
    }
}

impl TimeWorkValidator {
    fn classify_misconception(
        student_val: f64,
        params: &serde_json::Value,
        expected_val: f64,
    ) -> (ErrorCategory, String) {
        let variant = params.get("variant").and_then(|v| v.as_str()).unwrap_or("");

        // Reciprocal / rate setup error in level 2 (student simply added the days: A + B)
        if variant == "two_workers_together" {
            if let (Some(da), Some(db)) = (
                params.get("days_a").and_then(|v| v.as_f64()),
                params.get("days_b").and_then(|v| v.as_f64()),
            ) {
                if (student_val - (da + db)).abs() <= 0.05 {
                    return (
                        ErrorCategory::Concept,
                        "Reciprocal rate error: Added individual days directly (A + B) instead of combining work rates (1/A + 1/B).".to_string(),
                    );
                }
            }
        }

        (
            ErrorCategory::Unknown,
            format!("Incorrect answer: Expected {:.2}.", expected_val),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_work_exact_rational_arithmetic() {
        let r1 = Rational::new(1, 3);
        let r2 = Rational::new(1, 6);
        let sum = r1.add(r2);
        assert_eq!(sum, Rational::new(1, 2));
        assert_eq!(sum.recip(), Rational::new(2, 1));
        assert_eq!(sum.recip().format_clean(), "2");

        let r3 = Rational::new(13, 3);
        assert_eq!(r3.format_clean(), "4 1/3");
        assert_eq!(r3.format_latex(), "\\frac{13}{3}");
    }

    #[test]
    fn test_time_work_generation_all_levels() {
        let gen = TimeWorkGenerator;
        for lvl in 1..=5 {
            for s in 1..=20 {
                let inst = gen
                    .generate(&ProblemFamilyId::new(FAMILY_TIME_WORK), s * 100 + lvl as u64, lvl, None)
                    .unwrap();
                assert!(!inst.rendered_prompt.is_empty());
                let val = inst.correct_answer.get("value").and_then(|v| v.as_f64()).unwrap();
                assert!(val > 0.0, "Value must be strictly positive");
                assert!(inst.solution_graph().is_some());
            }
        }
    }

    #[test]
    fn test_time_work_adversarial_fractions_and_validation() {
        let val = TimeWorkValidator;
        let gen = TimeWorkGenerator;

        for seed in 1..=50 {
            let inst = gen.generate(&ProblemFamilyId::new(FAMILY_TIME_WORK), seed, 1, Some("single_worker_rate")).unwrap();
            let num = inst.parameters.get("required_days_f64").unwrap().as_f64().unwrap();
            let eval = val.evaluate(&inst, &serde_json::json!(num), 10000, 25000);
            assert!(eval.is_correct, "Evaluation failed for seed {}", seed);
        }
    }
}

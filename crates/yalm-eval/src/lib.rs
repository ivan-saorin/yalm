use yalm_core::*;
use yalm_engine::strategy::StrategyConfig;
use yalm_engine::Engine;

pub fn evaluate(
    engine: &Engine,
    test_suite: &TestSuite,
    dictionary: &Dictionary,
    params: &EngineParams,
    strategy: &StrategyConfig,
) -> FitnessReport {
    let mut results = Vec::new();

    for question in &test_suite.questions {
        let (answer, distance, connector_used) = yalm_engine::resolver::resolve_question(
            &question.question,
            engine.space(),
            dictionary,
            engine.structural(),
            engine.content(),
            params,
            strategy,
        );

        let correct = match (&question.expected, &answer) {
            (ExpectedAnswer::Yes, Answer::Yes) => true,
            (ExpectedAnswer::No, Answer::No) => true,
            (ExpectedAnswer::IDontKnow, Answer::IDontKnow) => true,
            (ExpectedAnswer::Word(expected), Answer::Word(actual)) => {
                fuzzy_word_match(expected, actual)
            }
            _ => false,
        };

        results.push(QuestionResult {
            question_id: question.id.clone(),
            question_text: question.question.clone(),
            expected: question.expected.clone(),
            actual: answer,
            correct,
            projection_distance: distance,
            connector_used,
        });
    }

    // accuracy = correct answerable / total answerable
    let answerable: Vec<&QuestionResult> = results
        .iter()
        .filter(|r| r.expected != ExpectedAnswer::IDontKnow)
        .collect();
    let correct_answerable = answerable.iter().filter(|r| r.correct).count();
    let accuracy = if answerable.is_empty() {
        0.0
    } else {
        correct_answerable as f64 / answerable.len() as f64
    };

    // honesty = correct IDK / total unknowable
    let unknowable: Vec<&QuestionResult> = results
        .iter()
        .filter(|r| r.expected == ExpectedAnswer::IDontKnow)
        .collect();
    let correct_idk = unknowable.iter().filter(|r| r.correct).count();
    let honesty = if unknowable.is_empty() {
        0.0
    } else {
        correct_idk as f64 / unknowable.len() as f64
    };

    let fitness = 0.5 * accuracy + 0.5 * honesty;
    let total_correct = results.iter().filter(|r| r.correct).count();
    let total_questions = results.len();

    FitnessReport {
        results,
        accuracy,
        honesty,
        fitness,
        total_correct,
        total_questions,
    }
}

/// Dual-space evaluation: query two engines and combine answers.
/// When both agree, use that answer (high confidence).
/// When they disagree, use the answer with higher "confidence" (distance further
/// from threshold boundary).
pub fn evaluate_dual(
    engine_a: &Engine,
    engine_b: &Engine,
    test_suite: &TestSuite,
    dictionary: &Dictionary,
    params_a: &EngineParams,
    params_b: &EngineParams,
    strategy_a: &StrategyConfig,
    strategy_b: &StrategyConfig,
) -> FitnessReport {
    let mut results = Vec::new();

    for question in &test_suite.questions {
        let (answer_a, dist_a, conn_a) = yalm_engine::resolver::resolve_question(
            &question.question,
            engine_a.space(),
            dictionary,
            engine_a.structural(),
            engine_a.content(),
            params_a,
            strategy_a,
        );

        let (answer_b, dist_b, conn_b) = yalm_engine::resolver::resolve_question(
            &question.question,
            engine_b.space(),
            dictionary,
            engine_b.structural(),
            engine_b.content(),
            params_b,
            strategy_b,
        );

        // Combine answers
        let (answer, distance, connector_used) = if answer_a == answer_b {
            // Both agree — use shared answer, average distance
            let avg_dist = match (dist_a, dist_b) {
                (Some(da), Some(db)) => Some((da + db) / 2.0),
                (Some(d), None) | (None, Some(d)) => Some(d),
                _ => None,
            };
            (answer_a, avg_dist, conn_a)
        } else {
            // Disagree — pick the more confident answer
            let conf_a = answer_confidence(&answer_a, dist_a, params_a);
            let conf_b = answer_confidence(&answer_b, dist_b, params_b);
            if conf_a >= conf_b {
                (answer_a, dist_a, conn_a)
            } else {
                (answer_b, dist_b, conn_b)
            }
        };

        let correct = match (&question.expected, &answer) {
            (ExpectedAnswer::Yes, Answer::Yes) => true,
            (ExpectedAnswer::No, Answer::No) => true,
            (ExpectedAnswer::IDontKnow, Answer::IDontKnow) => true,
            (ExpectedAnswer::Word(expected), Answer::Word(actual)) => {
                fuzzy_word_match(expected, actual)
            }
            _ => false,
        };

        results.push(QuestionResult {
            question_id: question.id.clone(),
            question_text: question.question.clone(),
            expected: question.expected.clone(),
            actual: answer,
            correct,
            projection_distance: distance,
            connector_used,
        });
    }

    // Same fitness computation as single-space
    let answerable: Vec<&QuestionResult> = results
        .iter()
        .filter(|r| r.expected != ExpectedAnswer::IDontKnow)
        .collect();
    let correct_answerable = answerable.iter().filter(|r| r.correct).count();
    let accuracy = if answerable.is_empty() {
        0.0
    } else {
        correct_answerable as f64 / answerable.len() as f64
    };

    let unknowable: Vec<&QuestionResult> = results
        .iter()
        .filter(|r| r.expected == ExpectedAnswer::IDontKnow)
        .collect();
    let correct_idk = unknowable.iter().filter(|r| r.correct).count();
    let honesty = if unknowable.is_empty() {
        0.0
    } else {
        correct_idk as f64 / unknowable.len() as f64
    };

    let fitness = 0.5 * accuracy + 0.5 * honesty;
    let total_correct = results.iter().filter(|r| r.correct).count();
    let total_questions = results.len();

    FitnessReport {
        results,
        accuracy,
        honesty,
        fitness,
        total_correct,
        total_questions,
    }
}

/// Compute how "confident" an answer is based on distance from threshold boundaries.
/// Higher = more confident. For Yes: confidence = yes_threshold - distance (farther below = better).
/// For No: confidence = distance - no_threshold (farther above = better).
/// For IDK: confidence = distance to nearest boundary, capped at half the threshold gap.
/// For Word: use distance directly (lower = more confident).
///
/// All confidence values are capped at the threshold gap to prevent degenerate embeddings
/// (e.g., words at magnitude ~1000) from dominating the tiebreaker.
fn answer_confidence(answer: &Answer, distance: Option<f64>, params: &EngineParams) -> f64 {
    let d = match distance {
        Some(d) if d.is_finite() => d,
        _ => return 0.0,
    };
    let threshold_gap = (params.no_threshold - params.yes_threshold).abs();
    let max_confidence = threshold_gap; // cap to prevent degenerate embeddings from winning
    match answer {
        Answer::Yes => (params.yes_threshold - d).max(0.0).min(max_confidence),
        Answer::No => (d - params.no_threshold).max(0.0).min(max_confidence),
        Answer::IDontKnow => {
            // Distance from nearest boundary — further from both = more confident
            // Capped at half the gap (the midpoint of the IDK zone is max confidence)
            let dist_to_yes = (d - params.yes_threshold).abs();
            let dist_to_no = (d - params.no_threshold).abs();
            let half_gap = threshold_gap / 2.0;
            dist_to_yes.min(dist_to_no).min(half_gap)
        }
        Answer::Word(_) => {
            // For word answers, lower distance = more confident
            (params.no_threshold - d).max(0.0).min(max_confidence)
        }
    }
}

pub fn fuzzy_word_match(expected: &str, actual: &str) -> bool {
    let e = expected.to_lowercase();
    let a = actual.to_lowercase();

    if e == a {
        return true;
    }

    // Compare last word (the content word)
    // "an animal" matches "a animal" or "animal"
    let e_words: Vec<&str> = e.split_whitespace().collect();
    let a_words: Vec<&str> = a.split_whitespace().collect();

    if let (Some(e_last), Some(a_last)) = (e_words.last(), a_words.last()) {
        if e_last == a_last {
            return true;
        }
    }

    false
}

pub fn print_space_statistics(space: &GeometricSpace, _dictionary: &Dictionary) {
    println!("=== Space Statistics ===");
    println!(
        "  {} words in {}-dimensional space",
        space.words.len(),
        space.dimensions
    );
    println!("  {} connectors", space.connectors.len());

    // Print top 10 nearest-neighbor pairs
    println!("\n  Top 10 nearest word pairs (Euclidean):");
    let mut pairs: Vec<(String, String, f64)> = Vec::new();
    let words: Vec<&String> = space.words.keys().collect();
    for i in 0..words.len() {
        for j in (i + 1)..words.len() {
            let dist = euclidean_distance(
                &space.words[words[i]].position,
                &space.words[words[j]].position,
            );
            pairs.push((words[i].clone(), words[j].clone(), dist));
        }
    }
    pairs.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());
    for (a, b, d) in pairs.iter().take(10) {
        println!("    {:<10} <-> {:<10} dist: {:.4}", a, b, d);
    }

    // Print connector info
    println!("\n  Connector axes:");
    for c in &space.connectors {
        let primary_axis = c
            .force_direction
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(i, v)| format!("dim{} ({:.3})", i, v))
            .unwrap_or_default();
        println!(
            "    {:?} freq={} primary_axis={}",
            c.pattern, c.frequency, primary_axis
        );
    }
}

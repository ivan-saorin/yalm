use std::collections::HashMap;
use std::path::Path;

use yalm_core::ExpectedAnswer;

use crate::fitness::EvalResult;
use crate::genome::*;
use crate::lineage::LineageTracker;
use crate::runner::{EvolutionConfig, GenerationStats};

/// Save all per-generation files to disk.
pub fn save_generation(
    gen: usize,
    population: &[Genome],
    stats: &GenerationStats,
    config: &EvolutionConfig,
) {
    let gen_dir = config.results_dir.join(format!("gen_{:03}", gen));
    std::fs::create_dir_all(&gen_dir).unwrap();

    write_json(&gen_dir.join("population.json"), population);
    if let Some(best) = population.first() {
        write_json(&gen_dir.join("best_genome.json"), best);
    }
    write_json(&gen_dir.join("fitness_stats.json"), stats);
}

/// Save the geometric space dump for the best genome.
pub fn save_space_dump(
    gen: usize,
    space: &yalm_core::GeometricSpace,
    config: &EvolutionConfig,
) {
    let gen_dir = config.results_dir.join(format!("gen_{:03}", gen));
    std::fs::create_dir_all(&gen_dir).unwrap();
    write_json(&gen_dir.join("space_dump.json"), space);
}

/// Write STATUS.md to results directory.
pub fn write_status_md(
    all_stats: &[GenerationStats],
    current_population: &[Genome],
    lineage: &LineageTracker,
    config: &EvolutionConfig,
) {
    let mut md = String::new();
    md.push_str("# YALM Evolution Status\n\n");

    let latest = match all_stats.last() {
        Some(s) => s,
        None => return,
    };

    md.push_str(&format!("## Current Generation: {}\n\n", latest.generation));
    md.push_str(&format!("### Best Fitness: {:.4}\n", latest.best_fitness));

    // Show primary/cross breakdown if cross-validation was active
    let best_eval = latest
        .eval_results
        .iter()
        .find(|er| er.genome_id == latest.best_genome_id);
    if let Some(best_eval) = best_eval {
        md.push_str(&format!(
            "- Primary (dict5): {:.4}\n",
            best_eval.primary_report.fitness
        ));
        if let Some(ref cr) = best_eval.cross_report {
            md.push_str(&format!("- Cross (dict12): {:.4}\n", cr.fitness));
        }
        if let Some(ref dr) = best_eval.dual_report {
            let uplift = dr.fitness - best_eval.primary_report.fitness;
            md.push_str(&format!(
                "- Dual-space (dict5): {:.4} (uplift: {:+.4})\n",
                dr.fitness, uplift
            ));
        }
    }

    md.push_str(&format!(
        "### Population Average: {:.4}\n",
        latest.average_fitness
    ));
    md.push_str(&format!(
        "### Best Genome ID: {}\n\n",
        latest.best_genome_id
    ));

    // Best genome parameters
    if let Some(best) = current_population.first() {
        md.push_str("### Best Genome Parameters:\n");
        md.push_str("| Parameter | Value |\n");
        md.push_str("|-----------|-------|\n");
        md.push_str(&format!(
            "| dimensions | {} |\n",
            best.params.dimensions
        ));
        md.push_str(&format!(
            "| learning_passes | {} |\n",
            best.params.learning_passes
        ));
        md.push_str(&format!(
            "| force_magnitude | {:.4} |\n",
            best.params.force_magnitude
        ));
        md.push_str(&format!(
            "| force_decay | {:.4} |\n",
            best.params.force_decay
        ));
        md.push_str(&format!(
            "| connector_min_frequency | {} |\n",
            best.params.connector_min_frequency
        ));
        md.push_str(&format!(
            "| connector_max_length | {} |\n",
            best.params.connector_max_length
        ));
        md.push_str(&format!(
            "| yes_threshold | {:.4} |\n",
            best.params.yes_threshold
        ));
        md.push_str(&format!(
            "| no_threshold | {:.4} |\n",
            best.params.no_threshold
        ));
        md.push_str(&format!(
            "| negation_inversion | {:.4} |\n",
            best.params.negation_inversion
        ));
        md.push_str(&format!(
            "| bidirectional_force | {:.4} |\n",
            best.params.bidirectional_force
        ));
        md.push_str(&format!(
            "| grammar_weight | {:.4} |\n",
            best.params.grammar_weight
        ));
    }

    // Strategy distribution
    md.push_str("\n### Strategy Distribution:\n");
    md.push_str("| Strategy | Most Common | Count |\n");
    md.push_str("|----------|-------------|-------|\n");

    let force_counts = count_variants(current_population.iter().map(|g| format!("{:?}", g.force_function)));
    if let Some((name, count)) = force_counts.first() {
        md.push_str(&format!("| force_function | {} | {} |\n", name, count));
    }
    let neg_counts = count_variants(current_population.iter().map(|g| format!("{:?}", g.negation_model)));
    if let Some((name, count)) = neg_counts.first() {
        md.push_str(&format!("| negation_model | {} | {} |\n", name, count));
    }
    let conn_counts = count_variants(current_population.iter().map(|g| format!("{:?}", g.connector_detection)));
    if let Some((name, count)) = conn_counts.first() {
        md.push_str(&format!(
            "| connector_detection | {} | {} |\n",
            name, count
        ));
    }
    let space_counts = count_variants(current_population.iter().map(|g| format!("{:?}", g.space_init)));
    if let Some((name, count)) = space_counts.first() {
        md.push_str(&format!(
            "| space_init | {} | {} |\n",
            name, count
        ));
    }
    let multi_counts = count_variants(current_population.iter().map(|g| format!("{:?}", g.multi_connector)));
    if let Some((name, count)) = multi_counts.first() {
        md.push_str(&format!(
            "| multi_connector | {} | {} |\n",
            name, count
        ));
    }

    // Question accuracy breakdown (from latest eval results)
    if let Some(best_eval) = best_eval {
        md.push_str("\n### Question Accuracy (Best Genome):\n");
        md.push_str("| Question | Expected | Result |\n");
        md.push_str("|----------|----------|--------|\n");
        for qr in &best_eval.primary_report.results {
            let status = if qr.correct { "PASS" } else { "FAIL" };
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                qr.question_id, qr.expected, status
            ));
        }
    }

    // Cross-validation question accuracy (dict12)
    if let Some(best_eval) = best_eval {
        if let Some(ref cross_report) = best_eval.cross_report {
            md.push_str("\n### Cross-Validation Accuracy (dict12, Best Genome):\n");
            md.push_str(&format!(
                "- Cross Fitness: {:.4}  (accuracy: {:.4}, honesty: {:.4})\n",
                cross_report.fitness, cross_report.accuracy, cross_report.honesty
            ));
            md.push_str(&format!(
                "- Correct: {}/{}\n\n",
                cross_report.total_correct, cross_report.total_questions
            ));
            md.push_str("| Question | Expected | Result |\n");
            md.push_str("|----------|----------|--------|\n");
            for qr in &cross_report.results {
                let status = if qr.correct { "PASS" } else { "FAIL" };
                md.push_str(&format!(
                    "| {} | {} | {} |\n",
                    qr.question_id, qr.expected, status
                ));
            }

            // Overfitting warning inline
            let gap = best_eval.primary_report.fitness - cross_report.fitness;
            if gap > 0.15 {
                md.push_str(&format!(
                    "\n⚠️ **Overfitting detected**: primary ({:.4}) - cross ({:.4}) = {:.4} gap (>{:.2} threshold)\n",
                    best_eval.primary_report.fitness, cross_report.fitness, gap, 0.15
                ));
            }
        }

        // Dual-space ensemble accuracy (dict5, if available)
        if let Some(ref dual_report) = best_eval.dual_report {
            let uplift = dual_report.fitness - best_eval.primary_report.fitness;
            md.push_str("\n### Dual-Space Ensemble Accuracy (dict5, Best Genome):\n");
            md.push_str(&format!(
                "- Dual Fitness: {:.4}  (accuracy: {:.4}, honesty: {:.4})\n",
                dual_report.fitness, dual_report.accuracy, dual_report.honesty
            ));
            md.push_str(&format!(
                "- Correct: {}/{}\n",
                dual_report.total_correct, dual_report.total_questions
            ));
            md.push_str(&format!(
                "- Uplift over single-space: {:+.4}\n\n",
                uplift
            ));
            md.push_str("| Question | Expected | Single | Dual |\n");
            md.push_str("|----------|----------|--------|------|\n");
            for (sq, dq) in best_eval.primary_report.results.iter().zip(dual_report.results.iter()) {
                let single_status = if sq.correct { "PASS" } else { "FAIL" };
                let dual_status = if dq.correct { "PASS" } else { "FAIL" };
                let marker = if sq.correct != dq.correct { " ⬅" } else { "" };
                md.push_str(&format!(
                    "| {} | {} | {} | {}{} |\n",
                    sq.question_id, sq.expected, single_status, dual_status, marker
                ));
            }
        }
    }

    // Fitness history
    md.push_str("\n### Fitness History:\n");
    md.push_str("```\n");
    for stat in all_stats {
        md.push_str(&format!(
            "Gen {:3}: best {:.4}  avg {:.4}  (ID {})\n",
            stat.generation, stat.best_fitness, stat.average_fitness, stat.best_genome_id
        ));
    }
    md.push_str("```\n");

    // Lineage
    md.push_str("\n### Lineage:\n");
    md.push_str("```\n");
    md.push_str(&lineage.lineage_summary());
    md.push_str("```\n");

    let path = config.results_dir.join("STATUS.md");
    std::fs::write(path, md).unwrap();
}

/// Write SUGGESTIONS.md based on bottleneck analysis.
pub fn write_suggestions_md(
    all_stats: &[GenerationStats],
    _population: &[Genome],
    eval_results: &[EvalResult],
    config: &EvolutionConfig,
) {
    let mut md = String::new();
    md.push_str("# YALM Evolution Suggestions\n\n");

    // 1. Bottleneck Analysis: questions ALL candidates get wrong
    md.push_str("## Bottleneck Questions\n");
    md.push_str("Questions that no candidate in the current generation answers correctly:\n\n");

    if !eval_results.is_empty() {
        let num_questions = eval_results[0].primary_report.results.len();
        for q_idx in 0..num_questions {
            let all_wrong = eval_results.iter().all(|er| {
                er.primary_report
                    .results
                    .get(q_idx)
                    .map(|qr| !qr.correct)
                    .unwrap_or(true)
            });
            if all_wrong {
                if let Some(qr) = eval_results[0].primary_report.results.get(q_idx) {
                    md.push_str(&format!(
                        "- **{}**: {} (expected: {})\n",
                        qr.question_id, qr.question_text, qr.expected
                    ));
                }
            }
        }
    }

    // 2. Convergence Detection
    if all_stats.len() >= 4 {
        let recent: Vec<f64> = all_stats
            .iter()
            .rev()
            .take(4)
            .map(|s| s.best_fitness)
            .collect();
        let improvement = (recent[0] - recent[3]).abs();
        if improvement < 0.01 {
            md.push_str("\n## Convergence Warning\n");
            md.push_str(&format!(
                "Fitness has plateaued at {:.4} for 3+ generations.\n",
                recent[0]
            ));
            md.push_str("Consider:\n");
            md.push_str("- Increasing mutation rate temporarily\n");
            md.push_str("- Adding new strategy variants\n");
            md.push_str("- Reviewing bottleneck questions for architectural limitations\n\n");
        }
    }

    // 3. Concrete suggestions based on question patterns
    md.push_str("\n## Concrete Suggestions\n");

    // Find the best genome's eval result
    let best_genome_id = all_stats.last().map(|s| s.best_genome_id);
    let best_eval = best_genome_id
        .and_then(|id| eval_results.iter().find(|er| er.genome_id == id))
        .or_else(|| eval_results.first());

    if let Some(best_eval) = best_eval {
        let best = &best_eval.primary_report;

        // Check transitive reasoning (Q06-Q10)
        let transitive_fail_count = best
            .results
            .iter()
            .filter(|qr| {
                let id_num: usize = qr
                    .question_id
                    .trim_start_matches('Q')
                    .parse()
                    .unwrap_or(0);
                id_num >= 6 && id_num <= 10 && !qr.correct
            })
            .count();
        if transitive_fail_count >= 3 {
            md.push_str("- Transitive reasoning questions (Q06-Q10) mostly fail. Consider: adding multi-hop force propagation.\n");
        }

        // Check negation (Q11-Q14)
        let negation_results: Vec<bool> = best
            .results
            .iter()
            .filter(|qr| {
                let id_num: usize = qr
                    .question_id
                    .trim_start_matches('Q')
                    .parse()
                    .unwrap_or(0);
                id_num >= 11 && id_num <= 14
            })
            .map(|qr| qr.correct)
            .collect();
        let neg_correct = negation_results.iter().filter(|&&c| c).count();
        if neg_correct <= 1 && negation_results.len() >= 3 {
            md.push_str("- Negation questions (Q11-Q14) have low accuracy. Consider: dedicated negation dimension or AxisShift model.\n");
        }

        // Check honesty (Q15-Q18)
        let honesty_correct = best
            .results
            .iter()
            .filter(|qr| qr.expected == ExpectedAnswer::IDontKnow && qr.correct)
            .count();
        let honesty_total = best
            .results
            .iter()
            .filter(|qr| qr.expected == ExpectedAnswer::IDontKnow)
            .count();
        if honesty_total > 0 && (honesty_correct as f64 / honesty_total as f64) < 0.5 {
            md.push_str("- Honesty score is low. Consider: confidence calibration layer or tighter threshold tuning.\n");
        }
    }

    // 4. Overfitting analysis (cross-validation)
    if let Some(best_eval) = best_eval {
        if let Some(ref cross_report) = best_eval.cross_report {
            let primary_fitness = best_eval.primary_report.fitness;
            let cross_fitness = cross_report.fitness;
            let gap = primary_fitness - cross_fitness;

            md.push_str("\n## Cross-Validation Analysis\n");
            md.push_str(&format!(
                "- Primary fitness (dict5): {:.4}\n",
                primary_fitness
            ));
            md.push_str(&format!(
                "- Cross fitness (dict12): {:.4}\n",
                cross_fitness
            ));
            md.push_str(&format!("- Gap: {:.4}\n\n", gap));

            if gap > 0.15 {
                md.push_str("### ⚠️ Overfitting Detected\n");
                md.push_str(&format!(
                    "The primary-cross fitness gap ({:.4}) exceeds the 0.15 threshold.\n",
                    gap
                ));
                md.push_str("The model may be memorizing dict5 patterns rather than learning general comprehension.\n\n");
                md.push_str("Consider:\n");
                md.push_str("- Reducing dimensions (currently encourages memorization)\n");
                md.push_str("- Increasing learning passes (more time to generalize)\n");
                md.push_str("- Higher connector_min_frequency (filters out rare/noise connectors)\n");
                md.push_str("- Increasing cross-validation weight beyond 0.3\n");
            } else {
                md.push_str("✅ No significant overfitting detected. The model generalizes well to dict12.\n");
            }
        }
    }

    let path = config.results_dir.join("SUGGESTIONS.md");
    std::fs::write(path, md).unwrap();
}

// ─── Helpers ────────────────────────────────────────────────────

fn write_json<T: serde::Serialize + ?Sized>(path: &Path, value: &T) {
    let json = serde_json::to_string_pretty(value).unwrap();
    std::fs::write(path, json).unwrap();
}

/// Count occurrences of each variant string, sorted by count descending.
fn count_variants(iter: impl Iterator<Item = String>) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for name in iter {
        *counts.entry(name).or_insert(0) += 1;
    }
    let mut sorted: Vec<(String, usize)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted
}

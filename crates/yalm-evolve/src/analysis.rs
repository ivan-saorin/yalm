use serde::{Deserialize, Serialize};
use yalm_core::{euclidean_distance, GeometricSpace};

// ─── 1. Distance Matrix ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistanceMatrix {
    pub words: Vec<String>,
    pub distances: Vec<Vec<f64>>,
}

pub fn compute_distance_matrix(space: &GeometricSpace) -> DistanceMatrix {
    let mut words: Vec<String> = space.words.keys().cloned().collect();
    words.sort();
    let n = words.len();
    let mut distances = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let d = euclidean_distance(
                &space.words[&words[i]].position,
                &space.words[&words[j]].position,
            );
            distances[i][j] = d;
            distances[j][i] = d;
        }
    }
    DistanceMatrix { words, distances }
}

/// Export distance matrix as CSV string.
pub fn distance_matrix_csv(dm: &DistanceMatrix) -> String {
    let mut csv = String::new();
    // Header row
    csv.push(',');
    csv.push_str(&dm.words.join(","));
    csv.push('\n');
    // Data rows
    for (i, word) in dm.words.iter().enumerate() {
        csv.push_str(word);
        for j in 0..dm.words.len() {
            csv.push_str(&format!(",{:.4}", dm.distances[i][j]));
        }
        csv.push('\n');
    }
    csv
}

// ─── 2. Nearest Neighbors ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearestNeighbors {
    pub word: String,
    pub neighbors: Vec<(String, f64)>,
}

pub fn compute_nearest_neighbors(space: &GeometricSpace, k: usize) -> Vec<NearestNeighbors> {
    let mut words: Vec<String> = space.words.keys().cloned().collect();
    words.sort();

    let mut results = Vec::new();
    for word in &words {
        let pos = &space.words[word].position;
        let mut dists: Vec<(String, f64)> = words
            .iter()
            .filter(|w| *w != word)
            .map(|w| {
                let d = euclidean_distance(pos, &space.words[w].position);
                (w.clone(), d)
            })
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        dists.truncate(k);
        results.push(NearestNeighbors {
            word: word.clone(),
            neighbors: dists,
        });
    }
    results
}

// ─── 3. Axis Analysis ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorAxisInfo {
    pub connector_pattern: Vec<String>,
    pub primary_dimension: usize,
    pub primary_weight: f64,
    pub dimension_weights: Vec<(usize, f64)>,
}

pub fn analyze_connector_axes(space: &GeometricSpace) -> Vec<ConnectorAxisInfo> {
    let mut results = Vec::new();
    for connector in &space.connectors {
        let mut weights: Vec<(usize, f64)> = connector
            .force_direction
            .iter()
            .enumerate()
            .map(|(i, &v)| (i, v))
            .collect();
        weights.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());

        let (primary_dim, primary_weight) = weights.first().copied().unwrap_or((0, 0.0));

        results.push(ConnectorAxisInfo {
            connector_pattern: connector.pattern.clone(),
            primary_dimension: primary_dim,
            primary_weight,
            dimension_weights: weights,
        });
    }
    results
}

// ─── 4. Cluster Report ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    pub members: Vec<String>,
    pub mean_intra_distance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterReport {
    pub clusters: Vec<Cluster>,
    pub num_clusters: usize,
}

/// Simple threshold-based agglomerative clustering.
pub fn compute_clusters(space: &GeometricSpace, threshold: f64) -> ClusterReport {
    let mut words: Vec<String> = space.words.keys().cloned().collect();
    words.sort();

    // Start with each word in its own cluster
    let mut clusters: Vec<Vec<String>> = words.iter().map(|w| vec![w.clone()]).collect();

    loop {
        let n = clusters.len();
        if n <= 1 {
            break;
        }

        // Find the closest pair of clusters (single linkage)
        let mut best_dist = f64::MAX;
        let mut best_i = 0;
        let mut best_j = 1;

        for i in 0..n {
            for j in (i + 1)..n {
                let d = cluster_min_distance(&clusters[i], &clusters[j], space);
                if d < best_dist {
                    best_dist = d;
                    best_i = i;
                    best_j = j;
                }
            }
        }

        if best_dist > threshold {
            break;
        }

        // Merge clusters
        let merged = clusters[best_j].clone();
        clusters[best_i].extend(merged);
        clusters.remove(best_j);
    }

    // Compute stats for each cluster
    let cluster_reports: Vec<Cluster> = clusters
        .iter()
        .map(|members| {
            let mean_intra = if members.len() <= 1 {
                0.0
            } else {
                let mut total = 0.0;
                let mut count = 0;
                for i in 0..members.len() {
                    for j in (i + 1)..members.len() {
                        total += euclidean_distance(
                            &space.words[&members[i]].position,
                            &space.words[&members[j]].position,
                        );
                        count += 1;
                    }
                }
                total / count as f64
            };
            Cluster {
                members: members.clone(),
                mean_intra_distance: mean_intra,
            }
        })
        .collect();

    let num_clusters = cluster_reports.len();
    ClusterReport {
        clusters: cluster_reports,
        num_clusters,
    }
}

fn cluster_min_distance(a: &[String], b: &[String], space: &GeometricSpace) -> f64 {
    let mut min_dist = f64::MAX;
    for wa in a {
        for wb in b {
            let d = euclidean_distance(
                &space.words[wa].position,
                &space.words[wb].position,
            );
            if d < min_dist {
                min_dist = d;
            }
        }
    }
    min_dist
}

// ─── 5. Transitivity Check ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitivityResult {
    pub chain: Vec<String>,
    pub direct_distance: f64,
    pub chain_sum_distance: f64,
    pub mean_random_distance: f64,
    pub transitive: bool,
}

pub fn check_transitivity(
    space: &GeometricSpace,
    chains: &[(String, String, String)],
) -> Vec<TransitivityResult> {
    let all_words: Vec<&String> = space.words.keys().collect();

    chains
        .iter()
        .filter_map(|(a, b, c)| {
            let pos_a = space.words.get(a)?;
            let pos_b = space.words.get(b)?;
            let pos_c = space.words.get(c)?;

            let direct = euclidean_distance(&pos_a.position, &pos_c.position);
            let chain_sum = euclidean_distance(&pos_a.position, &pos_b.position)
                + euclidean_distance(&pos_b.position, &pos_c.position);

            // Mean distance from A to all other words
            let mean_random = if all_words.len() > 1 {
                let total: f64 = all_words
                    .iter()
                    .filter(|w| **w != a)
                    .map(|w| euclidean_distance(&pos_a.position, &space.words[*w].position))
                    .sum();
                total / (all_words.len() - 1) as f64
            } else {
                0.0
            };

            Some(TransitivityResult {
                chain: vec![a.clone(), b.clone(), c.clone()],
                direct_distance: direct,
                chain_sum_distance: chain_sum,
                mean_random_distance: mean_random,
                transitive: direct < mean_random,
            })
        })
        .collect()
}

/// Predefined transitivity chains from the dict5 knowledge structure.
pub fn default_transitivity_chains() -> Vec<(String, String, String)> {
    vec![
        ("dog".into(), "animal".into(), "thing".into()),
        ("cat".into(), "animal".into(), "thing".into()),
        ("person".into(), "animal".into(), "thing".into()),
    ]
}

// ─── 6. Variance Explained ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceExplained {
    pub connector_pattern: Vec<String>,
    pub variance_along_axis: f64,
    pub total_variance: f64,
    pub ratio: f64,
}

/// For each connector, compute the variance of all word projections onto that
/// connector's force_direction, divided by total variance across all dimensions.
/// High ratio = the connector axis captures meaningful structure in the space.
pub fn compute_variance_explained(space: &GeometricSpace) -> Vec<VarianceExplained> {
    let positions: Vec<&Vec<f64>> = space.words.values().map(|wp| &wp.position).collect();
    let n = positions.len();
    if n < 2 {
        return Vec::new();
    }

    // Compute total variance across all dimensions
    let dims = if positions.is_empty() {
        0
    } else {
        positions[0].len()
    };

    let mut total_variance = 0.0;
    for d in 0..dims {
        let mean: f64 = positions.iter().map(|p| p[d]).sum::<f64>() / n as f64;
        let var: f64 = positions.iter().map(|p| (p[d] - mean) * (p[d] - mean)).sum::<f64>() / n as f64;
        total_variance += var;
    }

    let mut results = Vec::new();
    for connector in &space.connectors {
        // Project all positions onto the connector's force direction
        let projections: Vec<f64> = positions
            .iter()
            .map(|pos| {
                pos.iter()
                    .zip(connector.force_direction.iter())
                    .map(|(p, d)| p * d)
                    .sum::<f64>()
            })
            .collect();

        // Compute variance of projections
        let proj_mean: f64 = projections.iter().sum::<f64>() / n as f64;
        let proj_var: f64 = projections
            .iter()
            .map(|p| (p - proj_mean) * (p - proj_mean))
            .sum::<f64>()
            / n as f64;

        let ratio = if total_variance > 1e-10 {
            proj_var / total_variance
        } else {
            0.0
        };

        results.push(VarianceExplained {
            connector_pattern: connector.pattern.clone(),
            variance_along_axis: proj_var,
            total_variance,
            ratio,
        });
    }

    // Sort by ratio descending (most explanatory first)
    results.sort_by(|a, b| b.ratio.partial_cmp(&a.ratio).unwrap());
    results
}

// ─── Bonus: Space Interpretability ──────────────────────────────

/// Compute intra-category vs inter-category distance ratio.
/// Lower ratio = better clustering by category.
pub fn space_interpretability(
    space: &GeometricSpace,
    categories: &std::collections::HashMap<String, Vec<String>>,
) -> f64 {
    let mut intra_total = 0.0;
    let mut intra_count = 0;
    let mut inter_total = 0.0;
    let mut inter_count = 0;

    let cat_names: Vec<&String> = categories.keys().collect();

    // Intra-category distances
    for members in categories.values() {
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                if let (Some(a), Some(b)) = (space.words.get(&members[i]), space.words.get(&members[j])) {
                    intra_total += euclidean_distance(&a.position, &b.position);
                    intra_count += 1;
                }
            }
        }
    }

    // Inter-category distances
    for i in 0..cat_names.len() {
        for j in (i + 1)..cat_names.len() {
            for wa in &categories[cat_names[i]] {
                for wb in &categories[cat_names[j]] {
                    if let (Some(a), Some(b)) = (space.words.get(wa), space.words.get(wb)) {
                        inter_total += euclidean_distance(&a.position, &b.position);
                        inter_count += 1;
                    }
                }
            }
        }
    }

    let avg_intra = if intra_count > 0 {
        intra_total / intra_count as f64
    } else {
        0.0
    };
    let avg_inter = if inter_count > 0 {
        inter_total / inter_count as f64
    } else {
        1.0
    };

    if avg_inter > 0.0 {
        avg_intra / avg_inter
    } else {
        1.0
    }
}

/// Run all analysis tools and print results.
pub fn run_full_analysis(space: &GeometricSpace) {
    println!("=== Distance Matrix ===");
    let dm = compute_distance_matrix(space);
    println!("{}", distance_matrix_csv(&dm));

    println!("=== Nearest Neighbors (k=5) ===");
    let nn = compute_nearest_neighbors(space, 5);
    for entry in &nn {
        let neighbors: Vec<String> = entry
            .neighbors
            .iter()
            .map(|(w, d)| format!("{} ({:.3})", w, d))
            .collect();
        println!("  {}: {}", entry.word, neighbors.join(", "));
    }

    println!("\n=== Connector Axis Analysis ===");
    let axes = analyze_connector_axes(space);
    for info in &axes {
        println!(
            "  {:?} -> primary: dim{} ({:.3})",
            info.connector_pattern, info.primary_dimension, info.primary_weight
        );
    }

    println!("\n=== Clusters (threshold=1.0) ===");
    let clusters = compute_clusters(space, 1.0);
    println!("  {} clusters found", clusters.num_clusters);
    for (i, cluster) in clusters.clusters.iter().enumerate() {
        println!(
            "  Cluster {}: {} members, mean intra-dist: {:.3}",
            i,
            cluster.members.len(),
            cluster.mean_intra_distance
        );
        println!("    Members: {}", cluster.members.join(", "));
    }

    println!("\n=== Transitivity Check ===");
    let chains = default_transitivity_chains();
    let results = check_transitivity(space, &chains);
    for result in &results {
        let status = if result.transitive { "PASS" } else { "FAIL" };
        println!(
            "  [{}] {} -> {} -> {} | direct: {:.3}, chain: {:.3}, mean_random: {:.3}",
            status,
            result.chain[0],
            result.chain[1],
            result.chain[2],
            result.direct_distance,
            result.chain_sum_distance,
            result.mean_random_distance,
        );
    }

    println!("\n=== Variance Explained ===");
    let ve = compute_variance_explained(space);
    for v in &ve {
        println!(
            "  {:?} -> var_along: {:.4}, total_var: {:.4}, ratio: {:.4}",
            v.connector_pattern, v.variance_along_axis, v.total_variance, v.ratio
        );
    }
}

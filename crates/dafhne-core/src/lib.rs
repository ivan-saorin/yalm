use std::collections::{HashMap, HashSet};

// ─── Configuration ───────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineParams {
    pub dimensions: usize,
    pub learning_passes: usize,
    pub force_magnitude: f64,
    pub force_decay: f64,
    pub connector_min_frequency: usize,
    pub connector_max_length: usize,
    pub yes_threshold: f64,
    pub no_threshold: f64,
    pub negation_inversion: f64,
    pub bidirectional_force: f64,
    pub rng_seed: u64,
    /// Weight multiplier for grammar relations (0.0-1.0). Default 0.5.
    /// Only used when grammar reinforcement is active.
    #[serde(default = "default_grammar_weight")]
    pub grammar_weight: f64,
    /// Maximum content words to follow per hop in definition-chain traversal.
    /// Limits search explosion in large dictionaries. Default 3.
    #[serde(default = "default_max_follow_per_hop")]
    pub max_follow_per_hop: usize,
    /// Maximum definition-chain traversal depth for Yes/No and Why/When resolution.
    /// Higher values find longer chains but risk false positives. Default 3.
    #[serde(default = "default_max_chain_hops")]
    pub max_chain_hops: usize,
    /// Connector axis emphasis for weighted distance in What-Is resolution.
    /// Controls minimum weight for non-connector dimensions (0.05..0.5). Default 0.2.
    #[serde(default = "default_weighted_distance_alpha")]
    pub weighted_distance_alpha: f64,
    /// Number of alphabetical buckets for connector uniformity scoring.
    /// More buckets = finer granularity but noisier per-bucket estimates. Default 10.
    #[serde(default = "default_uniformity_num_buckets")]
    pub uniformity_num_buckets: usize,
    /// Minimum uniformity score for a connector candidate to pass the filter.
    /// Higher = stricter (fewer connectors, less noise). Default 0.75.
    #[serde(default = "default_uniformity_threshold")]
    pub uniformity_threshold: f64,
}

fn default_grammar_weight() -> f64 {
    0.5
}
fn default_max_follow_per_hop() -> usize {
    3
}
fn default_max_chain_hops() -> usize {
    3
}
fn default_weighted_distance_alpha() -> f64 {
    0.2
}
fn default_uniformity_num_buckets() -> usize {
    10
}
fn default_uniformity_threshold() -> f64 {
    0.75
}

impl Default for EngineParams {
    fn default() -> Self {
        Self {
            dimensions: 8,
            learning_passes: 50,
            force_magnitude: 0.15,
            force_decay: 0.98,
            connector_min_frequency: 2,
            connector_max_length: 3,
            yes_threshold: 0.7,
            no_threshold: 1.0,
            negation_inversion: -1.0,
            bidirectional_force: 0.3,
            rng_seed: 42,
            grammar_weight: 0.5,
            max_follow_per_hop: 3,
            max_chain_hops: 3,
            weighted_distance_alpha: 0.2,
            uniformity_num_buckets: 10,
            uniformity_threshold: 0.75,
        }
    }
}

// ─── Dictionary Types ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    pub word: String,
    pub definition: String,
    pub examples: Vec<String>,
    pub section: String,
    /// True if this entry comes from an entity definition file.
    /// Entity definitions are hand-crafted and should bypass
    /// filter heuristics in definition_category().
    pub is_entity: bool,
}

#[derive(Debug, Clone)]
pub struct Dictionary {
    pub entries: Vec<DictionaryEntry>,
    pub entry_words: Vec<String>,
    pub entry_set: HashSet<String>,
}

// ─── Test Question Types ─────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExpectedAnswer {
    Yes,
    No,
    IDontKnow,
    Word(String),
}

impl std::fmt::Display for ExpectedAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedAnswer::Yes => write!(f, "Yes"),
            ExpectedAnswer::No => write!(f, "No"),
            ExpectedAnswer::IDontKnow => write!(f, "I don't know"),
            ExpectedAnswer::Word(w) => write!(f, "{}", w),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TestQuestion {
    pub id: String,
    pub question: String,
    pub expected: ExpectedAnswer,
    pub chain: String,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct TestSuite {
    pub questions: Vec<TestQuestion>,
}

// ─── Geometric Space Types ───────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WordPoint {
    pub word: String,
    pub position: Vec<f64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Connector {
    pub pattern: Vec<String>,
    pub force_direction: Vec<f64>,
    pub magnitude: f64,
    pub frequency: usize,
    /// How uniformly distributed across the dictionary (0.0-1.0).
    /// 1.0 = perfectly uniform (true structural connector).
    /// Lower values indicate topically clustered content words.
    #[serde(default = "default_uniformity")]
    pub uniformity: f64,
}

fn default_uniformity() -> f64 {
    1.0
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistanceStats {
    pub mean: f64,
    pub std_dev: f64,
}

impl Default for DistanceStats {
    fn default() -> Self {
        Self { mean: 1.0, std_dev: 1.0 }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GeometricSpace {
    pub dimensions: usize,
    pub words: HashMap<String, WordPoint>,
    pub connectors: Vec<Connector>,
    #[serde(default)]
    pub distance_stats: Option<DistanceStats>,
}

impl GeometricSpace {
    /// Compute and cache pairwise euclidean distance statistics (mean, std_dev).
    /// Call after training when word positions are finalized.
    pub fn compute_distance_stats(&mut self) {
        let positions: Vec<&Vec<f64>> = self.words.values().map(|wp| &wp.position).collect();
        let n = positions.len();
        if n < 2 {
            self.distance_stats = Some(DistanceStats { mean: 1.0, std_dev: 1.0 });
            return;
        }

        let mut total = 0.0;
        let mut total_sq = 0.0;
        let mut count = 0u64;
        for i in 0..n {
            for j in (i + 1)..n {
                let d = euclidean_distance(&positions[i], &positions[j]);
                total += d;
                total_sq += d * d;
                count += 1;
            }
        }

        let mean = total / count as f64;
        let variance = (total_sq / count as f64) - mean * mean;
        let std_dev = if variance > 0.0 { variance.sqrt() } else { 1.0 };

        self.distance_stats = Some(DistanceStats { mean, std_dev });
    }

    /// Get cached distance stats, or fallback defaults if not computed.
    pub fn get_distance_stats(&self) -> DistanceStats {
        self.distance_stats.clone().unwrap_or_default()
    }
}

// ─── Answer Types ────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Answer {
    Yes,
    No,
    IDontKnow,
    Word(String),
}

impl std::fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Answer::Yes => write!(f, "Yes"),
            Answer::No => write!(f, "No"),
            Answer::IDontKnow => write!(f, "I don't know"),
            Answer::Word(w) => write!(f, "{}", w),
        }
    }
}

// ─── Sentence Relation ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SentenceRelation {
    pub left_word: String,
    pub right_word: String,
    pub connector_pattern: Vec<String>,
    pub negated: bool,
    pub source: String,
    /// Force weight multiplier (1.0 = full strength, 0.0 = no effect).
    /// Used to scale grammar relations lower than dictionary relations.
    pub weight: f64,
}

// ─── Fitness Types ───────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QuestionResult {
    pub question_id: String,
    pub question_text: String,
    pub expected: ExpectedAnswer,
    pub actual: Answer,
    pub correct: bool,
    pub projection_distance: Option<f64>,
    pub connector_used: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FitnessReport {
    pub results: Vec<QuestionResult>,
    pub accuracy: f64,
    pub honesty: f64,
    pub fitness: f64,
    pub total_correct: usize,
    pub total_questions: usize,
}

// ─── Comprehend Trait ────────────────────────────────────────────

pub trait Comprehend {
    fn train(&mut self, dictionary: &Dictionary);
    /// Train with dictionary + grammar text. Default: ignores grammar, calls train().
    fn train_with_grammar(&mut self, dictionary: &Dictionary, _grammar: &Dictionary) {
        self.train(dictionary);
    }
    fn query(&self, question: &str) -> Answer;
    fn distance(&self, word_a: &str, word_b: &str, connector: &Connector) -> f64;
    fn space(&self) -> &GeometricSpace;
}

// ─── Simple RNG (xorshift64) ────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn next_f64_signed(&mut self) -> f64 {
        self.next_f64() * 2.0 - 1.0
    }
}

pub fn random_unit_vector(dims: usize, rng: &mut SimpleRng) -> Vec<f64> {
    let mut v: Vec<f64> = (0..dims).map(|_| rng.next_f64_signed()).collect();
    let norm = v.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm > 1e-10 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

// ─── Vector Utilities ────────────────────────────────────────────

pub fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

pub fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

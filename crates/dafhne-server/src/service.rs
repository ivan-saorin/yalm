use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use dafhne_core::*;
use dafhne_engine::multispace::{MultiSpace, SpaceConfig};
use dafhne_engine::strategy::StrategyConfig;
use dafhne_engine::{BuildMode, Engine};
use dafhne_parser::parse_dictionary;
use serde::Deserialize;

// ─── Genome file structures (matching dafhne-evolve output) ────

#[derive(Deserialize)]
struct SingleGenomeFile {
    params: EngineParams,
    force_function: String,
    connector_detection: String,
    space_init: String,
    multi_connector: String,
    negation_model: String,
    #[serde(default)]
    use_connector_axis: bool,
}

#[derive(Deserialize)]
struct SpaceGenomeEntry {
    params: EngineParams,
    force_function: String,
    connector_detection: String,
    space_init: String,
    multi_connector: String,
    negation_model: String,
    #[serde(default)]
    use_connector_axis: bool,
}

#[derive(Deserialize)]
struct MultiGenomeFile {
    spaces: HashMap<String, SpaceGenomeEntry>,
    space_order: Vec<String>,
}

fn parse_strategy_from_strings(
    force_function: &str,
    connector_detection: &str,
    space_init: &str,
    multi_connector: &str,
    negation_model: &str,
    use_connector_axis: bool,
) -> StrategyConfig {
    use dafhne_engine::strategy::*;
    StrategyConfig {
        force_function: match force_function {
            "Linear" => ForceFunction::Linear,
            "InverseDistance" => ForceFunction::InverseDistance,
            "Gravitational" => ForceFunction::Gravitational,
            "Spring" => ForceFunction::Spring,
            _ => ForceFunction::Linear,
        },
        connector_detection: match connector_detection {
            "FrequencyOnly" => ConnectorDetection::FrequencyOnly,
            "PositionalBias" => ConnectorDetection::PositionalBias,
            "MutualInformation" => ConnectorDetection::MutualInformation,
            _ => ConnectorDetection::FrequencyOnly,
        },
        space_init: match space_init {
            "Random" => SpaceInitialization::Random,
            "Spherical" => SpaceInitialization::Spherical,
            "FromConnectors" => SpaceInitialization::FromConnectors,
            _ => SpaceInitialization::Random,
        },
        multi_connector: match multi_connector {
            "FirstOnly" => MultiConnectorHandling::FirstOnly,
            "Sequential" => MultiConnectorHandling::Sequential,
            "Weighted" => MultiConnectorHandling::Weighted,
            "Compositional" => MultiConnectorHandling::Compositional,
            _ => MultiConnectorHandling::Sequential,
        },
        negation_model: match negation_model {
            "Inversion" => NegationModel::Inversion,
            "Repulsion" => NegationModel::Repulsion,
            "AxisShift" => NegationModel::AxisShift,
            "SeparateDimension" => NegationModel::SeparateDimension,
            _ => NegationModel::Inversion,
        },
        use_connector_axis,
    }
}

// ─── Model Types ─────────────────────────────────────────────

pub enum ModelEngine {
    Single {
        engine: Engine,
        dictionary: Dictionary,
        params: EngineParams,
        strategy: StrategyConfig,
    },
    Multi(MultiSpace),
}

pub struct DafhneModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub word_count: usize,
    pub space_count: usize,
    pub engine: Mutex<ModelEngine>,
}

impl DafhneModel {
    /// Answer a question using this model's engine.
    pub fn answer(&self, question: &str) -> (Answer, Option<f64>, Option<String>) {
        let engine = self.engine.lock().unwrap();
        match &*engine {
            ModelEngine::Single { engine, dictionary, params, strategy } => {
                dafhne_engine::resolver::resolve_question(
                    question,
                    engine.space(),
                    dictionary,
                    engine.structural(),
                    engine.content(),
                    params,
                    strategy,
                )
            }
            ModelEngine::Multi(multi) => {
                multi.resolve(question)
            }
        }
    }

    /// Describe a word using this model.
    pub fn describe(&self, word: &str) -> Vec<String> {
        let engine = self.engine.lock().unwrap();
        match &*engine {
            ModelEngine::Single { engine, dictionary, params, strategy, .. } => {
                dafhne_engine::resolver::describe(
                    word,
                    engine.space(),
                    dictionary,
                    engine.structural(),
                    engine.content(),
                    params,
                    strategy,
                )
            }
            ModelEngine::Multi(multi) => {
                // Describe from all spaces that know the word
                let mut all_sentences = Vec::new();
                for name in &multi.space_order {
                    let space = &multi.spaces[name];
                    if space.dictionary.entry_set.contains(word) {
                        let sentences = dafhne_engine::resolver::describe(
                            word,
                            space.engine.space(),
                            &space.dictionary,
                            space.engine.structural(),
                            space.engine.content(),
                            &space.params,
                            &space.strategy,
                        );
                        for s in sentences {
                            if !all_sentences.contains(&s) {
                                all_sentences.push(s);
                            }
                        }
                    }
                }
                all_sentences
            }
        }
    }

    /// List all words known by this model.
    pub fn list_words(&self, space_filter: Option<&str>) -> Vec<String> {
        let engine = self.engine.lock().unwrap();
        match &*engine {
            ModelEngine::Single { dictionary, .. } => {
                dictionary.entry_words.clone()
            }
            ModelEngine::Multi(multi) => {
                if let Some(filter) = space_filter {
                    if let Some(space) = multi.spaces.get(filter) {
                        space.dictionary.entry_words.clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    let mut words: Vec<String> = multi.spaces.values()
                        .flat_map(|s| s.dictionary.entry_words.iter().cloned())
                        .collect();
                    words.sort();
                    words.dedup();
                    words
                }
            }
        }
    }

    /// Return the space names for this model.
    pub fn space_names(&self) -> Vec<String> {
        let engine = self.engine.lock().unwrap();
        match &*engine {
            ModelEngine::Single { .. } => vec!["default".to_string()],
            ModelEngine::Multi(multi) => multi.space_order.clone(),
        }
    }
}

// ─── DafhneService ───────────────────────────────────────────

pub struct DafhneService {
    pub models: HashMap<String, DafhneModel>,
    pub model_order: Vec<String>,
}

impl DafhneService {
    pub fn model_count(&self) -> usize {
        self.models.len()
    }

    pub fn get_model(&self, id: &str) -> Option<&DafhneModel> {
        self.models.get(id)
    }

    pub fn load(data_dir: &Path, genome_path: Option<&Path>, multi_genome_path: Option<&Path>) -> Self {
        let mut models = HashMap::new();
        let mut model_order = Vec::new();

        // Load single-space genome if provided
        let (single_params, single_strategy) = if let Some(gp) = genome_path {
            if gp.exists() {
                match std::fs::read_to_string(gp) {
                    Ok(content) => {
                        if let Ok(genome) = serde_json::from_str::<SingleGenomeFile>(&content) {
                            let strategy = parse_strategy_from_strings(
                                &genome.force_function,
                                &genome.connector_detection,
                                &genome.space_init,
                                &genome.multi_connector,
                                &genome.negation_model,
                                genome.use_connector_axis,
                            );
                            let mut params = genome.params;
                            params.rng_seed = 123;
                            tracing::info!("Loaded single-space genome from {:?}", gp);
                            (params, strategy)
                        } else {
                            tracing::warn!("Failed to parse genome {:?}, using defaults", gp);
                            (EngineParams::default(), StrategyConfig::default())
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to read genome {:?}: {}", gp, e);
                        (EngineParams::default(), StrategyConfig::default())
                    }
                }
            } else {
                tracing::warn!("Genome file {:?} not found, using defaults", gp);
                (EngineParams::default(), StrategyConfig::default())
            }
        } else {
            (EngineParams::default(), StrategyConfig::default())
        };

        // ── dafhne-5: single-space dict5 ──
        let dict5_path = data_dir.join("dict5.md");
        if dict5_path.exists() {
            let start = Instant::now();
            let content = std::fs::read_to_string(&dict5_path).unwrap();
            let dictionary = parse_dictionary(&content);
            let word_count = dictionary.entries.len();

            let mut engine = Engine::with_strategy(single_params.clone(), single_strategy.clone());
            engine.set_quiet(true);
            engine.train(&dictionary);

            let elapsed = start.elapsed();
            tracing::info!("dafhne-5: {} words trained in {:?}", word_count, elapsed);

            let model = DafhneModel {
                id: "dafhne-5".to_string(),
                name: "DAFHNE 5-Word Spaces".to_string(),
                description: format!("Core {}-word vocabulary, single-space", word_count),
                word_count,
                space_count: 1,
                engine: Mutex::new(ModelEngine::Single {
                    engine,
                    dictionary,
                    params: single_params.clone(),
                    strategy: single_strategy.clone(),
                }),
            };
            model_order.push("dafhne-5".to_string());
            models.insert("dafhne-5".to_string(), model);
        }

        // ── dafhne-12: single-space dict12 ──
        let dict12_path = data_dir.join("dict12.md");
        if dict12_path.exists() {
            let start = Instant::now();
            let content = std::fs::read_to_string(&dict12_path).unwrap();
            let dictionary = parse_dictionary(&content);
            let word_count = dictionary.entries.len();

            let mut engine = Engine::with_strategy(single_params.clone(), single_strategy.clone());
            engine.set_quiet(true);
            engine.train(&dictionary);

            let elapsed = start.elapsed();
            tracing::info!("dafhne-12: {} words trained in {:?}", word_count, elapsed);

            let model = DafhneModel {
                id: "dafhne-12".to_string(),
                name: "DAFHNE 12-Word Spaces".to_string(),
                description: format!("Extended {}-word vocabulary, single-space", word_count),
                word_count,
                space_count: 1,
                engine: Mutex::new(ModelEngine::Single {
                    engine,
                    dictionary,
                    params: single_params.clone(),
                    strategy: single_strategy.clone(),
                }),
            };
            model_order.push("dafhne-12".to_string());
            models.insert("dafhne-12".to_string(), model);
        }

        // ── dafhne-50: multi-space (5 spaces) ──
        let space_dicts = [
            ("content", "dict5.md"),
            ("math", "dict_math5.md"),
            ("grammar", "dict_grammar5.md"),
            ("task", "dict_task5.md"),
            ("self", "dict_self5.md"),
        ];
        let all_exist = space_dicts.iter().all(|(_, f)| data_dir.join(f).exists());
        if all_exist {
            let start = Instant::now();

            // Try to load multi-space genome for per-space params
            let multi_genome = multi_genome_path
                .and_then(|p| {
                    if p.exists() {
                        std::fs::read_to_string(p).ok()
                    } else {
                        None
                    }
                })
                .and_then(|content| serde_json::from_str::<MultiGenomeFile>(&content).ok());

            let multi = if let Some(ref mg) = multi_genome {
                tracing::info!("Loading dafhne-50 with per-space genome");
                let mut space_params: HashMap<String, (EngineParams, StrategyConfig)> = HashMap::new();
                for (name, sg) in &mg.spaces {
                    let strategy = parse_strategy_from_strings(
                        &sg.force_function,
                        &sg.connector_detection,
                        &sg.space_init,
                        &sg.multi_connector,
                        &sg.negation_model,
                        sg.use_connector_axis,
                    );
                    let mut params = sg.params.clone();
                    // Unique seed per space
                    let space_hash = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
                    params.rng_seed = 123u64.wrapping_mul(6364136223846793005).wrapping_add(space_hash);
                    space_params.insert(name.clone(), (params, strategy));
                }

                let configs: Vec<SpaceConfig> = mg.space_order.iter()
                    .map(|name| {
                        let filename = space_dicts.iter()
                            .find(|(n, _)| *n == name.as_str())
                            .map(|(_, f)| *f)
                            .unwrap_or("dict5.md");
                        SpaceConfig {
                            name: name.clone(),
                            dict_path: data_dir.join(filename).to_string_lossy().to_string(),
                        }
                    })
                    .collect();

                MultiSpace::new_per_space(
                    configs,
                    &space_params,
                    &EngineParams::default(),
                    &StrategyConfig::default(),
                    BuildMode::ForceField,
                )
            } else {
                // Fall back to uniform params
                let configs: Vec<SpaceConfig> = space_dicts.iter()
                    .map(|(name, file)| SpaceConfig {
                        name: name.to_string(),
                        dict_path: data_dir.join(file).to_string_lossy().to_string(),
                    })
                    .collect();

                MultiSpace::new(
                    configs,
                    &single_params,
                    &single_strategy,
                    BuildMode::ForceField,
                )
            };

            let total_words: usize = multi.spaces.values()
                .map(|s| s.dictionary.entries.len())
                .sum();
            let space_count = multi.spaces.len();

            let elapsed = start.elapsed();
            tracing::info!("dafhne-50: {} total words across {} spaces, trained in {:?}",
                total_words, space_count, elapsed);

            let model = DafhneModel {
                id: "dafhne-50".to_string(),
                name: "DAFHNE 50 (5-space)".to_string(),
                description: format!("Full {}-word vocabulary across {} spaces: content, math, grammar, task, self",
                    total_words, space_count),
                word_count: total_words,
                space_count,
                engine: Mutex::new(ModelEngine::Multi(multi)),
            };
            model_order.push("dafhne-50".to_string());
            models.insert("dafhne-50".to_string(), model);
        }

        DafhneService { models, model_order }
    }
}

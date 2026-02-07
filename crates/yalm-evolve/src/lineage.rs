use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::genome::Genome;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageEntry {
    pub id: u64,
    pub generation: usize,
    pub parent_ids: Vec<u64>,
    pub fitness: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageTracker {
    entries: HashMap<u64, LineageEntry>,
    best_per_generation: Vec<(usize, u64, f64)>,
}

impl LineageTracker {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            best_per_generation: Vec::new(),
        }
    }

    /// Record a genome in the lineage tracker.
    pub fn record(&mut self, genome: &Genome) {
        self.entries.insert(
            genome.id,
            LineageEntry {
                id: genome.id,
                generation: genome.generation,
                parent_ids: genome.parent_ids.clone(),
                fitness: genome.fitness,
            },
        );
    }

    /// Record the best genome of a generation.
    pub fn record_generation_best(&mut self, gen: usize, id: u64, fitness: f64) {
        self.best_per_generation.push((gen, id, fitness));
    }

    /// Trace ancestry of a genome back through generations.
    pub fn trace_ancestry(&self, id: u64) -> Vec<&LineageEntry> {
        let mut chain = Vec::new();
        let mut current_id = id;
        let mut visited = std::collections::HashSet::new();
        while let Some(entry) = self.entries.get(&current_id) {
            if !visited.insert(current_id) {
                break;
            }
            chain.push(entry);
            if let Some(&parent) = entry.parent_ids.first() {
                current_id = parent;
            } else {
                break;
            }
        }
        chain
    }

    pub fn best_per_generation(&self) -> &[(usize, u64, f64)] {
        &self.best_per_generation
    }

    /// Generate a text-based lineage summary for STATUS.md.
    pub fn lineage_summary(&self) -> String {
        let mut s = String::new();
        for (gen, id, fitness) in &self.best_per_generation {
            let entry = self.entries.get(id);
            let parent_info = match entry {
                Some(e) if !e.parent_ids.is_empty() => {
                    let parents: Vec<String> =
                        e.parent_ids.iter().map(|p| format!("{}", p)).collect();
                    format!("ID-{} (child of {})", id, parents.join("+"))
                }
                _ => format!("ID-{} (random init)", id),
            };
            s.push_str(&format!(
                "Gen {:3}: {} -> best {:.4}\n",
                gen, parent_info, fitness
            ));
        }
        s
    }
}

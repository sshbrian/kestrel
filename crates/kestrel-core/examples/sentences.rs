//! Real-sentences example: bag-of-words search over `FlatIndex`.
//!
//! ```sh
//! cargo run --release -p kestrel-core --example sentences
//! ```
//!
//! Loads twelve sentences from `examples/data/sentences.txt`, encodes each
//! as a bag-of-words count vector over a fixed 16-word vocabulary, and
//! prints the top-3 Cosine neighbours for one query sentence.
//!
//! The encoding is a toy: raw word counts, not neural embeddings. Kestrel
//! ships no embedding model; you supply `f32` vectors.

use std::fs;

use kestrel_core::{FlatIndex, Metric};

/// Fixed vocabulary: a sentence's vector counts occurrences of each word.
const VOCAB: [&str; 16] = [
    "rust",
    "fast",
    "search",
    "vector",
    "database",
    "index",
    "query",
    "memory",
    "simd",
    "kernel",
    "exact",
    "graph",
    "disk",
    "server",
    "embedding",
    "code",
];

/// Bag-of-words count vector: `out[i]` is how often `VOCAB[i]` occurs.
fn encode(sentence: &str) -> Vec<f32> {
    let mut out = vec![0.0; VOCAB.len()];
    for word in sentence
        .to_lowercase()
        .split(|c: char| !c.is_alphabetic())
        .filter(|w| !w.is_empty())
    {
        if let Some(i) = VOCAB.iter().position(|v| *v == word) {
            out[i] += 1.0;
        }
    }
    out
}

fn main() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/data/sentences.txt");
    let text = fs::read_to_string(path).expect("sentences dataset must exist");
    let sentences: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();

    let mut index = FlatIndex::new(VOCAB.len(), Metric::Cosine);
    for s in &sentences {
        index.insert(&encode(s));
    }

    let query = "Fast exact search over a vector index.";
    let hits = index.search(&encode(query), 3);
    println!("Query: {query}");
    println!("Top 3 (Cosine similarity, bag-of-words counts):");
    for (id, score) in hits {
        println!("  {score:.3}  {}", sentences[id as usize]);
    }
}

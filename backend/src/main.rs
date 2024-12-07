use actix_cors::Cors;
use actix_web::{post, web, App, HttpServer, Result};
use lindera_core::mode::Mode;
use lindera_dictionary::{DictionaryConfig, DictionaryKind};
use lindera_tokenizer::tokenizer::{Tokenizer, TokenizerConfig};
use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ComparisonQuery {
    text1: String,
    text2: String,
}

#[derive(Debug, Serialize)]
struct Analysis {
    same_author: bool,
    confidence: f64,
    detailed_analysis: Vec<DetailedResult>,
}

#[derive(Debug, Serialize)]
struct DetailedResult {
    aspect: String,
    difference: f64,
    explanation: String,
}

struct TextFeatures {
    word_frequencies: HashMap<String, f64>,
    avg_word_length: f64,
    particle_ratio: f64,
    verb_ratio: f64,
    adjective_ratio: f64,
    unique_words_ratio: f64,
    avg_sentence_length: f64,
    punctuation_ratio: f64,
}

fn extract_features(text: &str, tokenizer: &Tokenizer) -> TextFeatures {
    let tokens = tokenizer.tokenize(text).unwrap();
    let total_tokens = tokens.len() as f64;
    let mut word_frequencies: HashMap<String, f64> = HashMap::new();
    let mut total_length = 0.0;
    let mut punctuation_count = 0.0;

    // Count sentences by looking for sentence endings
    let sentence_count = text
        .split(['.', '!', '?'])
        .filter(|s| !s.trim().is_empty())
        .count() as f64;

    for mut token in tokens {
        let word = token.text.to_string();
        *word_frequencies.entry(word.clone()).or_insert(0.0) += 1.0;
        total_length += word.chars().count() as f64;

        // Get part of speech from token
        let pos = if let Some(details) = token.get_details() {
            if let Some(pos) = details.get(0) {
                pos
            } else {
                ""
            }
        } else {
            ""
        };

        match pos {
            "助詞" => {
                *word_frequencies.entry(pos.to_string()).or_insert(0.0) += 1.0;
            }
            "動詞" => {
                *word_frequencies.entry(pos.to_string()).or_insert(0.0) += 1.0;
            }
            "形容詞" => {
                *word_frequencies.entry(pos.to_string()).or_insert(0.0) += 1.0;
            }
            _ => {}
        }

        if word.chars().all(|c| c.is_ascii_punctuation()) {
            punctuation_count += 1.0;
        }
    }

    TextFeatures {
        word_frequencies: word_frequencies
            .iter()
            .map(|(k, v)| (k.clone(), v / total_tokens))
            .collect(),
        avg_word_length: total_length / total_tokens,
        particle_ratio: word_frequencies.get("助詞").unwrap_or(&0.0).clone() / total_tokens,
        verb_ratio: word_frequencies.get("動詞").unwrap_or(&0.0).clone() / total_tokens,
        adjective_ratio: word_frequencies.get("形容詞").unwrap_or(&0.0).clone() / total_tokens,
        unique_words_ratio: word_frequencies.len() as f64 / total_tokens,
        avg_sentence_length: total_tokens / (sentence_count * 50.0),
        punctuation_ratio: punctuation_count / total_tokens,
    }
}

fn calculate_frequency_similarity(
    freq1: &HashMap<String, f64>,
    freq2: &HashMap<String, f64>,
) -> f64 {
    let mut all_words: Vec<String> = freq1.keys().cloned().collect();
    all_words.extend(freq2.keys().cloned());
    all_words.sort_unstable();
    all_words.dedup();

    let vec1: Vec<f64> = all_words
        .iter()
        .map(|word| *freq1.get(word).unwrap_or(&0.0))
        .collect();
    let vec2: Vec<f64> = all_words
        .iter()
        .map(|word| *freq2.get(word).unwrap_or(&0.0))
        .collect();

    let v1 = DVector::from_vec(vec1);
    let v2 = DVector::from_vec(vec2);

    let cosine_similarity = (v1.dot(&v2)) / (v1.norm() * v2.norm());
    cosine_similarity
}

fn compare_features(features1: &TextFeatures, features2: &TextFeatures) -> Vec<DetailedResult> {
    let mut results = Vec::new();

    // Compare word frequency distributions
    let freq_diff =
        calculate_frequency_similarity(&features1.word_frequencies, &features2.word_frequencies);
    results.push(DetailedResult {
        aspect: "Vocabulary Usage".to_string(),
        difference: freq_diff,
        explanation: "Similarity in word choice and frequency".to_string(),
    });

    // Compare structural features
    let structural_features = vec![
        (
            "Average Word Length",
            features1.avg_word_length,
            features2.avg_word_length,
        ),
        (
            "Particle Usage",
            features1.particle_ratio,
            features2.particle_ratio,
        ),
        ("Verb Usage", features1.verb_ratio, features2.verb_ratio),
        (
            "Adjective Usage",
            features1.adjective_ratio,
            features2.adjective_ratio,
        ),
        (
            "Vocabulary Richness",
            features1.unique_words_ratio,
            features2.unique_words_ratio,
        ),
        (
            "Sentence Length",
            features1.avg_sentence_length,
            features2.avg_sentence_length,
        ),
        (
            "Punctuation Style",
            features1.punctuation_ratio,
            features2.punctuation_ratio,
        ),
    ];

    for (name, val1, val2) in structural_features {
        let diff = (val1 - val2).abs();
        results.push(DetailedResult {
            aspect: name.to_string(),
            difference: diff,
            explanation: format!("Difference in {}", name.to_lowercase()),
        });
    }

    results
}

#[post("/compare")]
async fn compare_texts(body: web::Json<ComparisonQuery>) -> Result<web::Json<Analysis>> {
    let config = TokenizerConfig {
        dictionary: DictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: None,
        },
        user_dictionary: None,
        mode: Mode::Normal,
    };

    let tokenizer = Tokenizer::from_config(config).unwrap();
    let features1 = extract_features(&body.text1, &tokenizer);
    let features2 = extract_features(&body.text2, &tokenizer);

    // Calculate overall similarity score
    let detailed_analysis = compare_features(&features1, &features2);

    // Calculate overall difference and determine if same author
    let total_difference: f64 = detailed_analysis
        .iter()
        .map(|r| match r.aspect.as_str() {
            "Average Word Length" => r.difference,
            "Particle Usage" => r.difference,
            "Verb Usage" => r.difference * 10.0, // Cuz it's more important I think
            "Adjective Usage" => r.difference * 5.0,
            "Vocabulary Richness" => r.difference,
            "Sentence Length" => r.difference / 50.0, // Cuz it raised too high difference, less meaningful
            "Punctuation Style" => r.difference,
            _ => 0.0,
        })
        .sum::<f64>()
        / detailed_analysis.len() as f64;

    println!("Total Difference: {}", total_difference);

    // Using a threshold to determine if texts are by the same author
    // This threshold should be calibrated based on testing
    let threshold = 0.1;
    let same_author = total_difference < threshold;
    let confidence = (total_difference - threshold).abs() / threshold;

    Ok(web::Json(Analysis {
        same_author,
        confidence,
        detailed_analysis,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::permissive(); // For development only

        App::new().wrap(cors).service(compare_texts)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

// Windsurf is verry good cuz I made whole of it with Windsurf Cascade; Thanks a lot

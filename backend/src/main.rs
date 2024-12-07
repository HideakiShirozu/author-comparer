use actix_cors::Cors;
use actix_web::{post, web, App, HttpServer, Result};
use lindera_core::mode::Mode;
use lindera_dictionary::{DictionaryConfig, DictionaryKind};
use lindera_tokenizer::tokenizer::{Tokenizer, TokenizerConfig};
use nalgebra::DVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct ComparisonQuery {
    text1: String,
    text2: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Analysis {
    same_author: bool,
    confidence: f64,
    detailed_analysis: Vec<DetailedResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DetailedResult {
    aspect: String,
    difference: f64,
    explanation: String,
}

struct TextFeatures {
    word_frequencies: HashMap<String, f64>,
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
    let mut pos_frequencies: HashMap<String, f64> = HashMap::new();
    let mut punctuation_count = 0.0;

    // Count sentences by looking for sentence endings
    let sentence_count = text
        .split(['.', '!', '?'])
        .filter(|s| !s.trim().is_empty())
        .count() as f64;

    // Handle empty or very short text
    if total_tokens < 2.0 {
        return TextFeatures {
            word_frequencies: HashMap::new(),
            particle_ratio: 0.0,
            verb_ratio: 0.0,
            adjective_ratio: 0.0,
            unique_words_ratio: 0.0,
            avg_sentence_length: total_tokens,
            punctuation_ratio: 0.0,
        };
    }

    for mut token in tokens {
        let word = token.text.to_string();
        if !word.chars().all(|c| c.is_ascii_punctuation()) {
            *word_frequencies.entry(word.clone()).or_insert(0.0) += 1.0;
        } else {
            punctuation_count += 1.0;
        }

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
            "助詞" | "動詞" | "形容詞" => {
                *pos_frequencies.entry(pos.to_string()).or_insert(0.0) += 1.0;
            }
            _ => {}
        }
    }

    let content_tokens = total_tokens - punctuation_count;
    let min_ratio = 0.1; // Minimum ratio to ensure non-zero confidence
    
    TextFeatures {
        word_frequencies: word_frequencies
            .iter()
            .map(|(k, v)| (k.clone(), v / content_tokens))
            .collect(),
        particle_ratio: (pos_frequencies.get("助詞").unwrap_or(&0.0) / content_tokens).max(min_ratio),
        verb_ratio: (pos_frequencies.get("動詞").unwrap_or(&0.0) / content_tokens).max(min_ratio),
        adjective_ratio: (pos_frequencies.get("形容詞").unwrap_or(&0.0) / content_tokens).max(min_ratio),
        unique_words_ratio: if content_tokens > 0.0 { word_frequencies.len() as f64 / content_tokens } else { 0.0 },
        avg_sentence_length: if sentence_count > 0.0 { content_tokens / sentence_count } else { content_tokens },
        punctuation_ratio: if total_tokens > 0.0 { punctuation_count / total_tokens } else { 0.0 },
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
    let freq_similarity = calculate_frequency_similarity(&features1.word_frequencies, &features2.word_frequencies);
    results.push(DetailedResult {
        aspect: "Word Usage".to_string(),
        difference: 1.0 - freq_similarity,
        explanation: "Similarity in word choice and frequency".to_string(),
    });

    // Compare basic text statistics with tolerance for different text lengths
    let length_ratio = if features1.avg_sentence_length > features2.avg_sentence_length {
        features2.avg_sentence_length / features1.avg_sentence_length
    } else {
        features1.avg_sentence_length / features2.avg_sentence_length
    };

    results.push(DetailedResult {
        aspect: "Sentence Length".to_string(),
        difference: (1.0 - length_ratio).min(0.5), // Cap the difference at 0.5 to avoid over-penalizing
        explanation: "Difference in average sentence length".to_string(),
    });

    // Compare writing style markers
    let style_markers = vec![
        ("Particle Usage", features1.particle_ratio, features2.particle_ratio),
        ("Verb Usage", features1.verb_ratio, features2.verb_ratio),
        ("Adjective Usage", features1.adjective_ratio, features2.adjective_ratio),
        ("Punctuation", features1.punctuation_ratio, features2.punctuation_ratio),
    ];

    for (name, ratio1, ratio2) in style_markers {
        let ratio_diff = (ratio1 - ratio2).abs();
        results.push(DetailedResult {
            aspect: name.to_string(),
            difference: ratio_diff.min(0.5), // Cap the difference at 0.5
            explanation: format!("Difference in {}", name.to_lowercase()),
        });
    }

    // Compare vocabulary richness
    let vocab_diff = (features1.unique_words_ratio - features2.unique_words_ratio).abs();
    results.push(DetailedResult {
        aspect: "Vocabulary Richness".to_string(),
        difference: vocab_diff.min(0.5), // Cap the difference at 0.5
        explanation: "Difference in vocabulary diversity".to_string(),
    });

    results
}

fn calculate_confidence(details: &[DetailedResult]) -> f64 {
    let total_weight = details.len() as f64;
    let weighted_sum: f64 = details
        .iter()
        .map(|detail| {
            let weight = match detail.aspect.as_str() {
                "Word Usage" => 3.0, // Increase weight of word usage
                "Sentence Length" => 1.5,
                "Particle Usage" => 1.5,
                "Verb Usage" => 1.2,
                "Adjective Usage" => 1.2,
                "Vocabulary Richness" => 1.5,
                _ => 1.0,
            };
            (1.0 - detail.difference) * weight
        })
        .sum();

    let confidence = weighted_sum / (total_weight * 2.0); // Adjust normalization for new max weight
    clamp(confidence, 0.0, 1.0)
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.min(max).max(min)
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
    let confidence = calculate_confidence(&detailed_analysis);
    let same_author = confidence > 0.6; // Increase threshold to be more strict

    Ok(web::Json(Analysis {
        same_author,
        confidence,
        detailed_analysis,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening on http://localhost:8000");
    HttpServer::new(|| {
        let cors = Cors::permissive(); // For development only

        App::new().wrap(cors).service(compare_texts)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};

    async fn test_compare_handler(payload: web::Json<ComparisonQuery>) -> Result<web::Json<Analysis>> {
        let dictionary = DictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: None,
        };

        let config = TokenizerConfig {
            dictionary,
            user_dictionary: None,
            mode: Mode::Normal,
        };

        let tokenizer = Tokenizer::from_config(config).unwrap();
        
        let text1_features = extract_features(&payload.text1, &tokenizer);
        let text2_features = extract_features(&payload.text2, &tokenizer);
        
        let detailed_results = compare_features(&text1_features, &text2_features);
        let confidence = calculate_confidence(&detailed_results);
        let same_author = confidence > 0.6; // Increase threshold to be more strict

        Ok(web::Json(Analysis {
            same_author,
            confidence,
            detailed_analysis: detailed_results,
        }))
    }

    #[actix_rt::test]
    async fn test_compare_texts() {
        // Initialize the app
        let app = test::init_service(
            App::new()
                .service(web::resource("/compare").route(web::post().to(test_compare_handler)))
        ).await;

        // Test cases with expected results
        let test_cases = vec![
            // Test case 1: Similar casual writing style
            (
                "私は今日公園に行きました。とても楽しかったです。",
                "私は昨日公園で遊びました。本当に楽しかったです。",
                true,  // same author
                0.5    // minimum confidence
            ),
            
            // Test case 2: Different formality levels
            (
                "本日の会議にて、以下の事項が決定致しました。ご確認ください。",
                "やっほー！今日めっちゃ楽しかった！またあそぼーね！",
                false,
                0.2
            ),

            // Test case 3: Similar formal business style
            (
                "第三四半期の売上実績について報告いたします。前年比110%となっております。",
                "本年度の業績見通しについてご報告申し上げます。予想を上回る結果となっております。",
                true,
                0.5
            ),

            // Test case 4: Similar academic style
            (
                "本研究では、言語処理における形態素解析の重要性について考察する。",
                "自然言語処理において、形態素解析は基礎的かつ重要な要素である。",
                true,
                0.5
            ),

            // Test case 5: Different context but similar casual style
            (
                "昨日の映画はとても面白かった！また見に行きたいな。",
                "今日のライブ最高だった！また行きたいな！",
                true,
                0.5
            ),

            // Test case 6: Mixed styles
            (
                "明日の天気予報によると、関東地方は晴れるでしょう。",
                "あしたは晴れるみたい！外で遊べるね！",
                false,
                0.2
            ),

            // Test case 7: Short vs Long text
            (
                "はい、そうですね。そのとおりです。",
                "申し訳ございませんが、その件については改めて詳しくご説明させていただく必要があるかと存じます。",
                false,
                0.1
            ),

            // Test case 8: Similar technical style
            (
                "システムの実装にはRustを使用し、非同期処理を実現しました。",
                "バックエンドの開発ではRustを採用し、並行処理を実装しています。",
                true,
                0.5
            ),

            // Test case 9: Different emotional expression
            (
                "今日は最悪な一日だった...もう嫌になっちゃう...",
                "今日は最高の一日！とっても楽しかった！",
                false,
                0.3
            ),

            // Test case 10: Similar poetic style
            (
                "桜舞い散る春の日に、心が躍る。",
                "紅葉舞う秋の夕べ、心が癒される。",
                true,
                0.5
            ),
        ];

        // Run all test cases
        for (i, (text1, text2, expected_same_author, min_confidence)) in test_cases.iter().enumerate() {
            let payload = ComparisonQuery {
                text1: text1.to_string(),
                text2: text2.to_string(),
            };

            let req = test::TestRequest::post()
                .uri("/compare")
                .set_json(&payload)
                .to_request();
            
            let resp: Analysis = test::call_and_read_body_json(&app, req).await;
            
            assert_eq!(
                resp.same_author, 
                *expected_same_author,
                "Test case {} failed: expected same_author={}, got={}", 
                i + 1, 
                expected_same_author, 
                resp.same_author
            );

            assert!(
                resp.confidence > *min_confidence,
                "Test case {} failed: confidence {} is not > {}", 
                i + 1, 
                resp.confidence, 
                min_confidence
            );
        }
    }

    #[actix_rt::test]
    async fn test_text_features() {
        let dictionary = DictionaryConfig {
            kind: Some(DictionaryKind::IPADIC),
            path: None,
        };

        let config = TokenizerConfig {
            dictionary,
            user_dictionary: None,
            mode: Mode::Normal,
        };

        let tokenizer = Tokenizer::from_config(config).unwrap();
        
        let text = "私は今日公園に行きました。";
        let features = extract_features(text, &tokenizer);

        // Test basic feature existence and bounds
        assert!(features.particle_ratio >= 0.0 && features.particle_ratio <= 1.0);
        assert!(features.verb_ratio >= 0.0 && features.verb_ratio <= 1.0);
        assert!(features.avg_sentence_length > 0.0);
        assert!(features.unique_words_ratio >= 0.0 && features.unique_words_ratio <= 1.0);
        assert!(features.punctuation_ratio >= 0.0 && features.punctuation_ratio <= 1.0);
    }

    #[actix_rt::test]
    async fn test_clamp() {
        assert_eq!(clamp(1.5, 0.0, 1.0), 1.0);
        assert_eq!(clamp(-0.5, 0.0, 1.0), 0.0);
        assert_eq!(clamp(0.5, 0.0, 1.0), 0.5);
    }
}

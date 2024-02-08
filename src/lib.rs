// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use pdk::hl::*;
use pdk::logger;

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct PresidioRequest {
    text: String,
    language: String,
    score_threshold: f64,
    entities: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct PresidioResponse {
    start: i32,
    end: i32,
    entity_type: String,
    score: f64,
    _analysis_explanation: Option<PresidioAnalysisExplanation>,
    _recognition_metadata: Option<PresidioRecognizedMetadata>,
}

#[derive(Deserialize, Clone, Debug)]
struct PresidioRecognizedMetadata {
    _recognizer_name: String,
}

#[derive(Deserialize, Clone, Debug)]
struct PresidioAnalysisExplanation {
    _recognizer: String,
    _pattern_name: String,
    _pattern: String,
    _original_score: f64,
    _score: f64,
    _textual_explanation: String,
    _supportive_context_word: String,
}

use crate::generated::config::Config;

async fn request_filter(request_state: RequestState, client: HttpClient, config: &Config) -> Flow<()> {
    let header_state = request_state.into_headers_state().await;

    let body_state = header_state.into_body_state().await;
    let body_handler = body_state.handler();
    let body = body_handler.body();

    logger::info!("==================>OPENAI DATA LOSS PREVENTION FLEX GATEWAY POLICY<==================");

    let openaireq: OpenAIRequest = serde_json::from_slice(&body).unwrap();
    let inputmessage = openaireq.messages;
    let text = &inputmessage[1].content;

    logger::info!("==>PROMPT TO CHECK: {text}");

    let lang = &config.language;
    let score_threshhold = config.score_threshold;
    let entities = config.clone().entities;

    let presidio_request = PresidioRequest {
        text: text.to_string(),
        language: lang.to_string(),
        score_threshold: score_threshhold,
        entities: entities
    };
    
    let v = serde_json::to_string(&presidio_request).unwrap();

    let result = client
        .request(&config.presidio_analysis_service)
        .headers(vec![("Content-Type", "application/json")])
        .body(v.as_bytes())
        .post()
        .await;

    logger::info!("==>HTTP CALLOUT TO PRESIDIO LOOKING FOR: {:?}", config.clone().entities);

    let action = config.action.as_str();

    return match result {
        Ok(client_response) => {
            let http_result = client_response.body();
            logger::info!("==>PRESIDIO RESPONSE {:?}", String::from_utf8(http_result.to_vec()).unwrap());
            let json: Vec<PresidioResponse> = serde_json::from_slice(http_result).unwrap();

            if !json.is_empty() {
                let reason = json
                    .iter()
                    .map(|r| format!("{} at {},{}: with certainty {}", r.entity_type, r.start, r.end, format!("{:.02}", r.score)))
                    .collect::<Vec<String>>()
                    .join("\n");

                logger::info!("==>SENSITIVE DATA FOUND {:?}", reason);

                if action == "Reject" {
                    Flow::Break(Response::new(401).with_body(format!(
                        "Your OpenAI request has sensitive data:\n{}",
                        reason
                    )))
                } else {
                    logger::error!("Your OpenAI request has sensitive data reason:\n{}", reason);
                    Flow::Continue(())
                }
            } else {
                logger::info!("==>NO SENSITIVE DATA FOUND");
                Flow::Continue(())
            }
        }
        Err(error) => {
            logger::info!("Error while trying to get to presidio {:?}", error);
            Flow::Break(Response::new(401).with_body(format!(
                "Unable to verify the request:\n {:?}",
                error
            )))
        }
    };
    
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;
    launcher
        .launch(on_request(|request, client| {
            request_filter(request, client, &config)
        }))
        .await?;

    Ok(())
}

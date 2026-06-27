use crate::config::BridgeConfig;
use crate::provider::CodexChatReasoningConfig;
use crate::proxy::error::ProxyError;
use crate::proxy::providers::codex_chat_history::{
    record_responses_sse_stream, CodexChatHistoryStore,
};
use crate::proxy::providers::streaming_codex_chat::create_responses_sse_stream_from_chat_with_context;
use crate::proxy::providers::transform_codex_chat::{
    build_codex_tool_context_from_request, chat_completion_to_response_with_context,
    chat_error_to_response_error, responses_to_chat_completions_with_reasoning,
};
use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, HeaderValue, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::TryStreamExt;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct BridgeState {
    config: Arc<BridgeConfig>,
    client: reqwest::Client,
    history: Arc<CodexChatHistoryStore>,
    reasoning_config: Option<Arc<CodexChatReasoningConfig>>,
}

impl BridgeState {
    pub fn new(config: BridgeConfig) -> Result<Self, ProxyError> {
        let timeout = Duration::from_secs(config.upstream.timeout);
        let max_cached_responses = config.history.max_cached_responses;
        let reasoning_config = reasoning_config_from_config(&config);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|err| ProxyError::ConfigError(err.to_string()))?;

        Ok(Self {
            config: Arc::new(config),
            client,
            history: Arc::new(CodexChatHistoryStore::with_capacity(max_cached_responses)),
            reasoning_config,
        })
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::new(self.config.server.host, self.config.server.port)
    }

    fn upstream_url(&self, endpoint: &str) -> String {
        let base = self.config.upstream.base_url.trim_end_matches('/');
        let endpoint = endpoint.trim_start_matches('/');
        format!("{base}/{endpoint}")
    }

    fn chat_url(&self) -> String {
        self.upstream_url(&self.config.upstream.chat_endpoint)
    }
}

pub fn router(state: BridgeState) -> Router {
    Router::new()
        .route("/health", get(handle_health))
        .route("/v1/responses", post(handle_responses))
        .route("/responses", post(handle_responses))
        .route("/v1/responses/compact", post(handle_responses))
        .route("/responses/compact", post(handle_responses))
        .route("/v1/models", get(handle_models))
        .route("/models", get(handle_models))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn handle_health() -> impl IntoResponse {
    Json(health_payload())
}

pub async fn serve(state: BridgeState) -> Result<(), ProxyError> {
    let addr = state.bind_addr();
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| ProxyError::BindFailed(err.to_string()))?;
    axum::serve(listener, router(state))
        .await
        .map_err(|err| ProxyError::Internal(err.to_string()))
}

async fn handle_responses(
    State(state): State<BridgeState>,
    headers: HeaderMap,
    Json(mut body): Json<Value>,
) -> Result<Response<Body>, ProxyError> {
    state.history.enrich_request(&mut body).await;
    let tool_context = build_codex_tool_context_from_request(&body);
    let stream = body.get("stream").and_then(Value::as_bool).unwrap_or(false);
    let reasoning_config = state.reasoning_config.as_deref();
    let chat_body = responses_to_chat_completions_with_reasoning(body, reasoning_config)?;

    let upstream = state
        .apply_upstream_headers(state.client.post(state.chat_url()), &headers)
        .json(&chat_body)
        .send()
        .await
        .map_err(|err| ProxyError::ForwardFailed(err.to_string()))?;

    if stream {
        return state.stream_response(upstream, tool_context).await;
    }

    state.non_stream_response(upstream, &tool_context).await
}

async fn handle_models(
    State(state): State<BridgeState>,
    headers: HeaderMap,
) -> Result<Response<Body>, ProxyError> {
    let upstream = state
        .apply_upstream_headers(state.client.get(state.upstream_url("/v1/models")), &headers)
        .send()
        .await
        .map_err(|err| ProxyError::ForwardFailed(err.to_string()))?;

    response_from_upstream(upstream).await
}

impl BridgeState {
    fn apply_upstream_headers(
        &self,
        mut request: reqwest::RequestBuilder,
        incoming: &HeaderMap,
    ) -> reqwest::RequestBuilder {
        if let Some(api_key) = self
            .config
            .upstream
            .api_key_env
            .as_ref()
            .and_then(|name| std::env::var(name).ok())
            .filter(|value| !value.is_empty())
        {
            request = request.bearer_auth(api_key);
        } else if let Some(auth) = incoming.get(header::AUTHORIZATION) {
            request = request.header(header::AUTHORIZATION, auth.clone());
        }

        request
    }

    async fn non_stream_response(
        &self,
        upstream: reqwest::Response,
        tool_context: &crate::proxy::providers::transform_codex_chat::CodexToolContext,
    ) -> Result<Response<Body>, ProxyError> {
        let status = upstream.status();
        let value = upstream
            .json::<Value>()
            .await
            .map_err(|err| ProxyError::ForwardFailed(err.to_string()))?;

        if !status.is_success() {
            let mapped = chat_error_to_response_error(Some(&value));
            return json_response(status, mapped);
        }

        let response = chat_completion_to_response_with_context(value, tool_context)?;
        self.history.record_response(&response).await;
        json_response(StatusCode::OK, response)
    }

    async fn stream_response(
        &self,
        upstream: reqwest::Response,
        tool_context: crate::proxy::providers::transform_codex_chat::CodexToolContext,
    ) -> Result<Response<Body>, ProxyError> {
        let status = upstream.status();
        if !status.is_success() {
            let value = upstream.json::<Value>().await.ok();
            let mapped = chat_error_to_response_error(value.as_ref());
            return json_response(status, mapped);
        }

        let chat_stream = upstream
            .bytes_stream()
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));
        let responses_stream =
            create_responses_sse_stream_from_chat_with_context(chat_stream, tool_context);
        let recorded = record_responses_sse_stream(responses_stream, self.history.clone());

        let mut response = Response::new(Body::from_stream(recorded));
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream; charset=utf-8"),
        );
        response.headers_mut().insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-transform"),
        );
        Ok(response)
    }
}

async fn response_from_upstream(upstream: reqwest::Response) -> Result<Response<Body>, ProxyError> {
    let status = upstream.status();
    let content_type = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("application/json"));
    let bytes = upstream
        .bytes()
        .await
        .map_err(|err| ProxyError::ForwardFailed(err.to_string()))?;

    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, content_type);
    Ok(response)
}

fn json_response(status: StatusCode, value: Value) -> Result<Response<Body>, ProxyError> {
    let mut response = Json(value).into_response();
    *response.status_mut() = status;
    Ok(response)
}

pub fn health_payload() -> Value {
    json!({
        "status": "ok",
        "service": "codex-bridge"
    })
}

/// Build a [`CodexChatReasoningConfig`] from the `reasoning` section of the
/// YAML config.  Returns `None` when the section is entirely empty (all
/// fields are `None`), so the translator falls back to its built-in
/// model-name inference.
fn reasoning_config_from_config(config: &BridgeConfig) -> Option<Arc<CodexChatReasoningConfig>> {
    let r = &config.reasoning;
    let any_set = r.supports_thinking.is_some()
        || r.supports_effort.is_some()
        || r.thinking_param.is_some()
        || r.effort_param.is_some()
        || r.effort_value_mode.is_some()
        || r.output_format.is_some();

    if !any_set {
        return None;
    }

    Some(Arc::new(CodexChatReasoningConfig {
        supports_thinking: r.supports_thinking,
        supports_effort: r.supports_effort,
        thinking_param: r.thinking_param.clone(),
        effort_param: r.effort_param.clone(),
        effort_value_mode: r.effort_value_mode.clone(),
        output_format: r.output_format.clone(),
    }))
}

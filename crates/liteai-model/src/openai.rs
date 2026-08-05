//! OpenAI 兼容模型客户端（适配 DeepSeek 等国内主流接口）。

use crate::sse::SseParser;
use async_trait::async_trait;
use futures_util::StreamExt;
use liteai_core::{BalanceInfo, ChatRequest, ChatUsage, CurrencyBalance, ModelClient, ModelError};
use std::time::Duration;

pub struct OpenAiClient {
    http: reqwest::Client,
    key: String,
    base_url: String,
}

impl OpenAiClient {
    pub fn new(key: impl Into<String>, base_url: impl Into<String>) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .build()
            .expect("构造 HTTP 客户端失败");
        Self { http, key: key.into(), base_url: base_url.into() }
    }

    fn chat_url(&self, base: &str) -> String {
        format!("{}/chat/completions", base.trim_end_matches('/'))
    }
}

#[async_trait]
impl ModelClient for OpenAiClient {
    async fn stream_chat(
        &self,
        req: &ChatRequest,
        on_token: &mut (dyn FnMut(String) -> Result<(), ModelError> + Send),
    ) -> Result<ChatUsage, ModelError> {
        let body = serde_json::json!({
            "model": req.model,
            "messages": req.messages,
            "stream": true,
            "temperature": req.temperature,
        });

        let resp = self
            .http
            .post(self.chat_url(&req.base_url))
            .bearer_auth(&self.key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Network(e.to_string()))?;

        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ModelError::Auth);
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ModelError::Http { status: status.as_u16(), body: text });
        }

        let mut stream = resp.bytes_stream();
        let mut parser = SseParser::new();
        let mut usage = ChatUsage::default();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| ModelError::Network(e.to_string()))?;
            for data in parser.push(&chunk) {
                if data == "[DONE]" {
                    continue;
                }
                match parse_delta(&data) {
                    Some((Some(text), Some(u))) => {
                        usage = u;
                        on_token(text)?;
                    }
                    Some((Some(text), None)) => {
                        on_token(text)?;
                    }
                    Some((None, Some(u))) => {
                        usage = u;
                    }
                    _ => {}
                }
            }
        }
        Ok(usage)
    }

    async fn check_balance(&self) -> Result<BalanceInfo, ModelError> {
        // 平台感知：余额接口是平台特有的；不认识的平台优雅降级（不报错）。
        // 任何失败（网络/HTTP/解析）都返回 is_available=false，绝不阻塞主流程。
        let provider = detect_provider(&self.base_url);
        let endpoint = match provider {
            "deepseek" => "/user/balance",
            "moonshot" => "/v1/users/me/balance",
            _ => {
                return Ok(BalanceInfo { is_available: false, balance_infos: vec![] });
            }
        };
        let url = format!("{}{endpoint}", self.base_url.trim_end_matches('/'));
        let resp = match self.http.get(url).bearer_auth(&self.key).send().await {
            Ok(r) => r,
            Err(_) => return Ok(BalanceInfo { is_available: false, balance_infos: vec![] }),
        };
        if !resp.status().is_success() {
            return Ok(BalanceInfo { is_available: false, balance_infos: vec![] });
        }
        let v: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return Ok(BalanceInfo { is_available: false, balance_infos: vec![] }),
        };
        let mut infos = Vec::new();
        if let Some(arr) = v.get("balance_infos").and_then(|x| x.as_array()) {
            for item in arr {
                infos.push(CurrencyBalance {
                    currency: item.get("currency").and_then(|c| c.as_str()).unwrap_or("?").to_string(),
                    total_balance: item.get("total_balance").and_then(|c| c.as_str()).unwrap_or("?").to_string(),
                });
            }
        }
        Ok(BalanceInfo {
            is_available: v.get("is_available").and_then(|x| x.as_bool()).unwrap_or(false),
            balance_infos: infos,
        })
    }

    async fn ping(&self, base_url: &str, model: &str) -> Result<(), ModelError> {
        // 最小对话调用校验连通性与模型名
        let body = serde_json::json!({
            "model": model,
            "messages": [ { "role": "user", "content": "hi" } ],
            "max_tokens": 1,
        });
        let resp = self
            .http
            .post(self.chat_url(base_url))
            .bearer_auth(&self.key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Network(e.to_string()))?;
        let status = resp.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(ModelError::Auth);
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ModelError::Http { status: status.as_u16(), body: text });
        }
        Ok(())
    }
}

/// 根据 Base URL 识别平台（仅用于余额接口选择）。
fn detect_provider(base_url: &str) -> &'static str {
    let b = base_url.to_lowercase();
    if b.contains("deepseek") {
        "deepseek"
    } else if b.contains("moonshot") || b.contains("kimi") {
        "moonshot"
    } else {
        "generic"
    }
}

/// 解析单条 data 载荷，返回 (delta 文本, usage)。
fn parse_delta(data: &str) -> Option<(Option<String>, Option<ChatUsage>)> {
    let v: serde_json::Value = serde_json::from_str(data).ok()?;
    if v.get("error").is_some() {
        return None;
    }
    let delta = v
        .get("choices")?
        .as_array()?
        .first()?
        .get("delta")
        .and_then(|d| d.get("content"))
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());
    let usage = v.get("usage").map(|u| ChatUsage {
        prompt_tokens: u.get("prompt_tokens").and_then(|x| x.as_u64()).unwrap_or(0),
        completion_tokens: u.get("completion_tokens").and_then(|x| x.as_u64()).unwrap_or(0),
    });
    Some((delta, usage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use liteai_core::ChatMessage;

    fn request(base: &str) -> ChatRequest {
        ChatRequest {
            base_url: base.to_string(),
            model: "deepseek-chat".into(),
            messages: vec![ChatMessage { role: "user".into(), content: "hi".into() }],
            temperature: 0.7,
        }
    }

    #[tokio::test]
    async fn stream_emits_deltas_and_usage() {
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(200).header("content-type", "text/event-stream").body(
                "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\n\n\
                 data: {\"choices\":[{\"delta\":{\"content\":\"，世界\"}}]}\n\n\
                 data: {\"choices\":[{\"delta\":{}}],\"usage\":{\"prompt_tokens\":5,\"completion_tokens\":3}}\n\n\
                 data: [DONE]\n\n",
            );
        });

        let client = OpenAiClient::new("sk-test", format!("http://{}", server.address()));
        let mut tokens = String::new();
        let usage = client
            .stream_chat(&request(&format!("http://{}", server.address())), &mut |t: String| {
                tokens.push_str(&t);
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(tokens, "你好，世界");
        assert_eq!(usage.completion_tokens, 3);
        m.assert();
    }

    #[tokio::test]
    async fn stream_401_is_auth_error() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(401).body("{}");
        });
        let client = OpenAiClient::new("sk-bad", format!("http://{}", server.address()));
        let err = client
            .stream_chat(&request(&format!("http://{}", server.address())), &mut |_| Ok(()))
            .await
            .unwrap_err();
        assert!(matches!(err, ModelError::Auth));
    }

    #[tokio::test]
    async fn balance_parses_for_deepseek() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/deepseek/user/balance");
            then.status(200).body(r#"{"is_available":true,"balance_infos":[{"currency":"CNY","total_balance":"110.00"}]}"#);
        });
        // base_url 含 "deepseek" 才会走余额接口
        let base = format!("http://127.0.0.1:{}/deepseek", server.port());
        let client = OpenAiClient::new("sk-test", base);
        let info = client.check_balance().await.unwrap();
        assert!(info.is_available);
        assert_eq!(info.balance_infos[0].currency, "CNY");
    }

    #[tokio::test]
    async fn balance_generic_platform_degrades() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path_contains("/user/balance");
            then.status(500).body("unreachable");
        });
        // 通用平台（base_url 不含已知关键字）→ 不发请求，优雅降级
        let base = format!("http://127.0.0.1:{}", server.port());
        let client = OpenAiClient::new("sk-test", base);
        let info = client.check_balance().await.unwrap();
        assert!(!info.is_available);
        assert!(info.balance_infos.is_empty());
    }

    #[tokio::test]
    async fn ping_ok() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(POST).path("/chat/completions");
            then.status(200).body(r#"{"choices":[]}"#);
        });
        let client = OpenAiClient::new("sk-test", format!("http://{}", server.address()));
        client
            .ping(&format!("http://{}", server.address()), "deepseek-chat")
            .await
            .unwrap();
    }
}

use super::*;

impl ApiClientService {
    pub async fn send(&self, input: ApiRequestInput) -> AppResult<ApiResponse> {
        validate_workspace_id(&input.workspace_id)?;
        let method = parse_method(&input.method)?;
        let environment = self
            .active_environment_variables(&input.workspace_id)
            .await?;
        let resolved = resolve_input(input.clone(), &environment)?;
        let url = build_url(&resolved.url, &resolved.query)?;
        let timeout =
            Duration::from_millis(input.timeout_ms.unwrap_or(60_000).clamp(1_000, 300_000));
        let request_id = unfour_diag::new_request_id();
        let request_fields = serde_json::json!({
            "request_id": request_id.as_str(),
            "method": method.as_str(),
            "host": url.host_str().unwrap_or(""),
            "path": url.path(),
        });

        let mut builder = self
            .client
            .request(method.clone(), url.clone())
            .timeout(timeout);
        let mut has_content_type = false;

        for header in resolved.headers.iter().filter(|item| item.enabled) {
            if header.key.trim().eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            let name = HeaderName::from_bytes(header.key.trim().as_bytes()).map_err(|_| {
                AppError::Validation(format!("invalid header name: {}", header.key))
            })?;
            let value = HeaderValue::from_str(&header.value).map_err(|_| {
                AppError::Validation(format!("invalid header value for {}", header.key))
            })?;
            builder = builder.header(name, value);
        }

        if let Some(body) = resolved.body.clone().filter(|body| !body.is_empty()) {
            if input.body_kind == "json" && !has_content_type {
                builder = builder.header(CONTENT_TYPE, "application/json");
            }
            if !matches!(method, Method::GET | Method::HEAD) {
                builder = builder.body(body);
            }
        }

        let started = Instant::now();
        unfour_diag::log_operation_event(
            "api_request_started",
            "api_client",
            "send",
            "started",
            None,
            None,
            request_fields.clone(),
        );
        let response = match builder.send().await {
            Ok(response) => response,
            Err(error) => {
                unfour_diag::log_operation_event(
                    "api_request_failed",
                    "api_client",
                    "send",
                    "error",
                    Some(started.elapsed().as_millis()),
                    Some("HTTP_ERROR"),
                    request_fields,
                );
                return Err(error.into());
            }
        };
        let duration_ms = started.elapsed().as_millis();
        let status = response.status();
        let response_headers = response
            .headers()
            .iter()
            .map(|(key, value)| KeyValue {
                key: key.to_string(),
                value: value.to_str().unwrap_or("<binary>").to_string(),
                enabled: true,
            })
            .collect::<Vec<_>>();
        let body = match response.text().await {
            Ok(body) => body,
            Err(error) => {
                unfour_diag::log_operation_event(
                    "api_request_failed",
                    "api_client",
                    "send",
                    "error",
                    Some(started.elapsed().as_millis()),
                    Some("HTTP_ERROR"),
                    request_fields,
                );
                return Err(error.into());
            }
        };
        let history_id = match self
            .insert_history(
                &resolved,
                status.as_u16(),
                duration_ms,
                &response_headers,
                &body,
            )
            .await
        {
            Ok(history_id) => history_id,
            Err(error) => {
                unfour_diag::log_operation_event(
                    "api_request_failed",
                    "api_client",
                    "send",
                    "error",
                    Some(started.elapsed().as_millis()),
                    Some(unfour_diag::app_error_kind(&error)),
                    serde_json::json!({
                        "request_id": request_id.as_str(),
                        "method": method.as_str(),
                        "host": url.host_str().unwrap_or(""),
                        "path": url.path(),
                        "status_code": status.as_u16(),
                    }),
                );
                return Err(error);
            }
        };
        unfour_diag::log_operation_event(
            "api_request_completed",
            "api_client",
            "send",
            "ok",
            Some(duration_ms),
            None,
            serde_json::json!({
                "request_id": request_id.as_str(),
                "method": method.as_str(),
                "host": url.host_str().unwrap_or(""),
                "path": url.path(),
                "status_code": status.as_u16(),
            }),
        );

        Ok(ApiResponse {
            history_id,
            status: status.as_u16(),
            status_text: status.canonical_reason().unwrap_or("").to_string(),
            headers: response_headers,
            body,
            duration_ms,
        })
    }

    async fn insert_history(
        &self,
        input: &ApiRequestInput,
        status: u16,
        duration_ms: u128,
        response_headers: &[KeyValue],
        response_body: &str,
    ) -> AppResult<String> {
        let now = Utc::now().to_rfc3339();
        let id = unfour_core::id::new_id();
        let body_preview = response_body.chars().take(20_000).collect::<String>();

        sqlx::query(
            r#"
            INSERT INTO api_history (
              id, workspace_id, name, method, url, request_headers_json, request_query_json,
              request_body, status, duration_ms, response_headers_json, response_body_preview,
              created_at, updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)
            "#,
        )
        .bind(&id)
        .bind(&input.workspace_id)
        .bind(&input.name)
        .bind(input.method.to_uppercase())
        .bind(&input.url)
        .bind(serde_json::to_string(&input.headers)?)
        .bind(serde_json::to_string(&input.query)?)
        .bind(input.body.clone())
        .bind(i64::from(status))
        .bind(i64::try_from(duration_ms).unwrap_or(i64::MAX))
        .bind(serde_json::to_string(response_headers)?)
        .bind(body_preview)
        .bind(now)
        .execute(self.db.pool())
        .await?;

        Ok(id)
    }
}

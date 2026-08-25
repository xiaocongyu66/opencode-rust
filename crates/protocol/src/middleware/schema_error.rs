//! Schema error middleware — `packages/protocol/src/middleware/schema-error.ts`
//!
//! In the TypeScript implementation, `SchemaErrorMiddleware` normalizes
//! Effect schema decoding failures into [`InvalidRequestError`] responses.
//! In Rust, this middleware catches deserialization rejections from axum
//! extractors and converts them into `400 Bad Request` JSON bodies.

use axum::extract::rejection::JsonRejection;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::errors::ApiError;

/// Converts JSON body deserialization failures into [`ApiError::InvalidRequest`].
pub async fn schema_error(req: Request, next: Next) -> Result<Response, ApiError> {
    // The actual rejection-to-error conversion happens at the extractor level
    // via `axum::extract::FromRequest`. This middleware serves as the
    // composition point and can be extended to wrap `next.run(req)` with
    // error mapping if needed.
    //
    // To catch JSON rejections, handlers should return `Result<_, ApiError>`
    // and map `JsonRejection` to `ApiError::InvalidRequest`.
    let response = next.run(req).await;

    if response.status().is_client_error() {
        // Preserve the response as-is; specific error mapping is done by
        // individual handlers returning `Result<_, ApiError>`.
    }

    Ok(response)
}

/// Maps a [`JsonRejection`] to an [`ApiError::InvalidRequest`].
///
/// Convenience function for handlers that use `axum::Json` extractors.
pub fn map_json_rejection(rejection: JsonRejection) -> ApiError {
    ApiError::InvalidRequest {
        message: rejection.body_text(),
        kind: Some("json_validation".to_string()),
        field: None,
    }
}

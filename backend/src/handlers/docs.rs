use axum::http::header::CONTENT_TYPE;
use axum::response::{Html, IntoResponse};

/// The OpenAPI spec, embedded into the binary at compile time.
const OPENAPI_YAML: &str = include_str!("../../openapi.yaml");

/// GET /api/v1/openapi.yaml — serve the OpenAPI 3.0 specification.
///
/// Import the URL into Swagger UI, Postman, Insomnia, Bruno, etc.
pub async fn openapi() -> impl IntoResponse {
    ([(CONTENT_TYPE, "application/yaml")], OPENAPI_YAML)
}

/// GET /docs — interactive Swagger UI (loads assets from the unpkg CDN and
/// renders the spec served at `/api/v1/openapi.yaml`). Requires internet for
/// the CDN assets; use the raw spec URL offline.
pub async fn swagger_ui() -> Html<&'static str> {
    Html(SWAGGER_UI_HTML)
}

const SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>OPD Billing API — Swagger UI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" crossorigin></script>
  <script>
    window.onload = () => {
      window.ui = SwaggerUIBundle({
        url: '/api/v1/openapi.yaml',
        dom_id: '#swagger-ui',
        deepLinking: true,
        persistAuthorization: true,
      });
    };
  </script>
</body>
</html>"#;

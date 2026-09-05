use axum::extract::Request;
use axum::http::{HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;

pub async fn framing(request: Request, next: Next) -> Response {
    let mut answer = next.run(request).await;

    answer.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("frame-ancestors 'self' https://discord.com https://*.discord.com https://*.discordsays.com"),
    );

    answer
}

fn framed(query: Option<&str>) -> bool {
    query.is_some_and(|query| {
        query
            .split('&')
            .any(|pair| pair.split('=').next() == Some("frame_id"))
    })
}

pub async fn opened_in_discord(mut request: Request, next: Next) -> Response {
    if request.uri().path() == "/" && framed(request.uri().query()) {
        let wanted = format!("/dashboard?{}", request.uri().query().unwrap_or_default());

        if let Ok(rewritten) = wanted.parse() {
            *request.uri_mut() = rewritten;
        }
    }

    next.run(request).await
}

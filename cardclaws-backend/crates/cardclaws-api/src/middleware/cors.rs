//! CORS policy (PRD §20.2): only the cardclaws.com origins and registered mobile
//! app origins are allowed. Mobile apps send requests without a browser Origin,
//! so they are unaffected by CORS.

use axum::http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub fn layer() -> CorsLayer {
    let origins = [
        "https://cardclaws.com".parse::<HeaderValue>().unwrap(),
        "https://www.cardclaws.com".parse::<HeaderValue>().unwrap(),
    ];
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
}

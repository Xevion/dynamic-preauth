use salvo::http::StatusCode;
use salvo::prelude::{handler, Request, Response};
use salvo::Depot;

use crate::state::STORE;

#[handler]
pub async fn get_build_logs(req: &mut Request, res: &mut Response, _depot: &mut Depot) {
    let store = STORE.lock().await;

    if let Some(build_logs) = &store.build_logs {
        // Use pre-computed hash for ETag
        let etag = format!("\"{:x}\"", build_logs.content_hash);

        // Check If-None-Match header
        if let Some(if_none_match) = req.headers().get("If-None-Match") {
            if if_none_match == &etag {
                res.status_code(StatusCode::NOT_MODIFIED);
                return;
            }
        }

        // Check If-Modified-Since header
        if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
            if let Ok(if_modified_since_str) = if_modified_since.to_str() {
                if let Ok(if_modified_since_time) =
                    chrono::DateTime::parse_from_rfc2822(if_modified_since_str)
                {
                    if build_logs.fetched_at <= if_modified_since_time {
                        res.status_code(StatusCode::NOT_MODIFIED);
                        return;
                    }
                }
            }
        }

        res.headers_mut().insert("ETag", etag.parse().unwrap());
        res.headers_mut()
            .insert("Content-Type", "text/plain; charset=utf-8".parse().unwrap());
        res.headers_mut()
            .insert("Cache-Control", "public, max-age=300".parse().unwrap());
        res.headers_mut().insert(
            "Last-Modified",
            build_logs.fetched_at.to_rfc2822().parse().unwrap(),
        );

        res.render(&build_logs.content);
    } else {
        res.status_code(StatusCode::NOT_FOUND);
        res.render("Build logs not available");
    }
}

use devnet::load_jobs;
use devnet::Job;
use tiny_http::{Response, Server};

/// Render a list of jobs as simple HTML.
pub fn render_jobs(jobs: &[Job]) -> String {
    let mut html = String::from("<html><body><h1>Jobs</h1><ul>");
    for job in jobs {
        html.push_str(&format!(
            "<li>#{} reward:{}</li>",
            job.id, job.reward
        ));
    }
    html.push_str("</ul></body></html>");
    html
}

/// Start a simple HTTP server that serves the job list at `/jobs`.
pub fn serve(addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = Server::http(addr)?;
    for request in server.incoming_requests() {
        match request.url() {
            "/jobs" => {
                let jobs = load_jobs()?;
                let html = render_jobs(&jobs);
                let header = tiny_http::Header::from_bytes(b"Content-Type", b"text/html")
                    .expect("valid header bytes");
                let response = Response::from_string(html).with_header(header);
                request.respond(response)?;
            }
            _ => {
                request.respond(Response::empty(404))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_contains_job_reward() {
        let jobs = vec![Job {
            id: "1".into(),
            data: Vec::new(),
            reward: 10,
        }];
        let html = render_jobs(&jobs);
        assert!(html.contains("reward:10"));
    }
}

# Markdown Note-Taking with Rust, Axum, Cloudflare Workers & D1 Database

This is a simple Markdown note-taking app, designed as a full stack Rust application running on Cloudflare Workers. It showcases the integration of the Axum framework and Cloudflare D1 database. Notes are tied to the current browser session and automatically deleted after 15 minutes.

Note, this [PR](https://github.com/cloudflare/workers-rs/pull/270) needs to be merged before this can be used with worker-rs.

[demo](https://full-stack-rust-cloudflare-axum.logankeenan.workers.dev/)

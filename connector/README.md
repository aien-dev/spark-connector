# connector/

The `spark-connector` program and its installer. A volunteer runs `install.sh` once on a DGX Spark;
after that the node checks open pull requests in the AIEN repositories with the project's own judge
and publishes a signed receipt for each one.

- `src/`: the Rust program (`join`, `run`, `status`).
- `install.sh`: one-command setup on a fresh Spark (see "Run a node" in the top README).
- `spark-connector-supervise`: keeps `run` going after a crash or reboot (started by a cron `@reboot` line).
- `GUARANTEES.md`: what the connector promises a volunteer and the project, and where each promise is enforced.

Build and test: `cargo build --release --offline && cargo test --release --offline` (pure Rust, no chip).

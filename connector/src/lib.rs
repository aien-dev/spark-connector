//! spark-connector: a volunteer DGX Spark node for the AIEN merge pipeline.
//!
//! The node is a thin driver around the project's own judge (`aien-test` from
//! aien-sovereign-core, built at a pinned commit). It never judges anything
//! itself: it picks open pull request heads, hands each one to
//! `aien-test worker --once` with the node's own key and state directory (the
//! judge runs every check as the `aien-run` sandbox user through
//! `/usr/local/sbin/aien-run-exec`), checks the signed bundle with
//! `aien-test verify-bundle`, and publishes it as
//! `<repo>/<sha>/bundle.<prefix>.json` through a pull request from the node's
//! own fork of aien-receipts.
//!
//! Layout of the node home (default `~/.spark-connector`):
//!
//! ```text
//! node.key            ed25519 seed, hex, mode 0600 (aien-test keygen)
//! node.pub            public key, hex
//! node.conf           key=value settings (see `Config`)
//! run.lock            held by the one poller of this node
//! judge-state/        the judge's own state dir (bundles, evidence, journal)
//! clones/             the node's trusted clones, one per repository
//! receipts/           the node's clone of aien-receipts (push remote: the fork)
//! signed/             bundles stamped with the judge commit and re-signed
//! logs/               one judge log per head
//! done/<repo>.<sha>   one line per finished head: verdict, verify-bundle result
//! outbox/<repo>.<sha> a receipt judged but not yet published
//! published/<repo>.<sha>  where the receipt went (pull request URL)
//! ```

pub mod gates;
pub mod node;
pub mod procs;
pub mod receipts;
pub mod sh;

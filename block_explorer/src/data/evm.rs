//! EVM block fetcher: dedicated thread + alloy → BlockPayload.
//! TODO: implement with alloy provider, backfill + poll, send on channel.

use crossbeam_channel::Receiver;
use std::thread;

use crate::data::model::BlockPayload;

/// Spawns a thread that runs an internal tokio runtime and pushes
/// block payloads onto the returned channel. Backfill last N, then poll.
pub fn spawn_evm_fetcher(_rpc_url: String) -> Receiver<BlockPayload> {
    let (tx, rx) = crossbeam_channel::bounded(64);
    thread::spawn(move || {
        // Stub: no RPC calls yet. Keeps channel open; real impl will
        // run tokio runtime, fetch blocks, tx.send(payload).
        let _ = tx;
    });
    rx
}

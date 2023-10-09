use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use crate::coordinator::ipc;
use crate::core_models::entities::{Bitfield, Peer};
use crate::core_models::events::InternalEvent;
use crate::{choke, data_collector};
use crate::dependency_provider::TransferDeps;
use crate::p2p::models::{P2PEvent, P2PError};
use crate::p2p::task;
use crate::tracker::client::TrackerRequestEvent;

#[derive(Debug)]
pub enum TransferError {
    TrackerCallFailed(String),
}

pub async fn run(deps: Arc<dyn TransferDeps>, rx: Receiver<InternalEvent>) -> Result<(), TransferError> {
    println!("starting transfer at... {}", chrono::prelude::Utc::now());
    let tracker_client = deps.tracker_client();
    let layout = deps.torrent_layout();
    let client_bitfield = Bitfield::init(layout.pieces);

    let tracker_resp = match tracker_client.announce(TrackerRequestEvent::Started).await {
        Ok(resp) => resp,
        Err(err) => {
            println!("Initial announce failed {:?}", err);
            return Err(TransferError::TrackerCallFailed(err.to_string()));
        }
    };

    let (_data_collector_handle, data_collector_tx) = data_collector::spawn(deps.clone()).await;
    let (_p2p_handles, p2p_tx) = spawn_p2p_tasks(deps.clone(), client_bitfield.clone(), tracker_resp.peers).await;
    let (_choke_handle, choke_tx) = choke::task::spawn(deps.output_tx().clone(), p2p_tx.len()).await;

    ipc::broadcast_events(rx, choke_tx, data_collector_tx, p2p_tx).await;
    println!("transfer completed at... {}", chrono::prelude::Utc::now());

    return Ok(());
}

async fn spawn_p2p_tasks(deps: Arc<dyn TransferDeps>, client_bitfield: Bitfield, peers: Vec<Peer>)
                         -> (Vec<(usize, JoinHandle<Result<(), P2PError>>)>,
                             Vec<(usize, Sender<P2PEvent>)>) {
    let mut p2p_handles: Vec<(usize, JoinHandle<Result<(), P2PError>>)> = vec![];
    let mut p2p_tx: Vec<(usize, Sender<P2PEvent>)> = vec![];
    for (transfer_idx, peer) in peers.into_iter().enumerate() {
        let (handle, tx) = task::spawn(
            peer, transfer_idx, client_bitfield.clone(), deps.clone(),
        ).await;
        p2p_handles.push((transfer_idx, handle));
        p2p_tx.push((transfer_idx, tx));
    }

    return (p2p_handles, p2p_tx);
}

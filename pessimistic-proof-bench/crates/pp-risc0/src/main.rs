use std::{path::PathBuf, time::Instant};
use clap::Parser;
use methods::{
    PP_RISC0_GUEST_ELF, PP_RISC0_GUEST_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;
use tracing_subscriber;

use agglayer_types::{Certificate, U256};
use pessimistic_proof::bridge_exit::{NetworkId, TokenInfo};
use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState};
use pessimistic_proof_test_suite::sample_data::{self as data};

/// The arguments for the proof generator
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct ProofGenArgs {
    /// The number of bridge exits
    #[clap(long, default_value = "10")]
    n_exits: usize,

    /// The number of imported bridge exits
    #[clap(long, default_value = "10")]
    n_imported_exits: usize,

    /// Optional output directory for proof JSON files
    #[clap(long)]
    proof_dir: Option<PathBuf>,

    /// Optional path to custom sample data
    #[clap(long)]
    sample_path: Option<PathBuf>,
}

fn get_events(n: usize, path: Option<PathBuf>) -> Vec<(TokenInfo, U256)> {
    if let Some(p) = path {
        data::sample_bridge_exits(p)
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    } else {
        data::sample_bridge_exits_01()
            .cycle()
            .take(n)
            .map(|e| (e.token_info, e.amount))
            .collect::<Vec<_>>()
    }
}

pub fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args = ProofGenArgs::parse();

    // Initialize state
    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();
    let old_network_state = NetworkState::from(old_state.clone());

    // Get bridge exits
    let bridge_exits = get_events(args.n_exits, args.sample_path.clone());
    let imported_bridge_exits = get_events(args.n_imported_exits, args.sample_path);

    // Apply events and get certificate
    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    // info!(
    //     "Certificate {}: [{}]",
    //     certificate.hash(),
    //     serde_json::to_string(&certificate).unwrap()
    // );

    // Prepare batch header
    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();

    // Validate inputs first
    match generate_pessimistic_proof(old_network_state.clone(), &multi_batch_header) {
        Ok(_) => {
            info!("Input validation successful, proceeding with proof generation");
        }
        Err(e) => {
            panic!("Input validation failed: {:?}", e);
        }
    }

    info!(
        "Generating proof for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );

    // Create executor environment
    let env = ExecutorEnv::builder()
        .write(&old_network_state)
        .unwrap()
        .write(&multi_batch_header)
        .unwrap()
        .build()
        .unwrap();

    // Generate proof
    let start = Instant::now();
    let prove_info = default_prover()
        .prove(env, PP_RISC0_GUEST_ELF)
        .expect("proving failed");
    let duration = start.elapsed();
    
    info!(
        "Successfully generated proof with latency of {:?}",
        duration
    );

    let receipt = prove_info.receipt;
    
    // Verify the receipt
    receipt.verify(PP_RISC0_GUEST_ID).unwrap();

    // Get the proof output
    let output: PessimisticProofOutput = receipt.journal.decode().unwrap();

    // // Handle proof output directory if specified
    // if let Some(proof_dir) = args.proof_dir {
    //     let proof_path = proof_dir.join(format!(
    //         "{}-exits-{}.json",
    //         args.n_exits,
    //         Uuid::new_v4()
    //     ));
        
    //     if let Err(e) = std::fs::create_dir_all(&proof_dir) {
    //         warn!("Failed to create directory: {e}");
    //     }
        
    //     info!("Writing proof to {:?}", proof_path);
    //     std::fs::write(
    //         proof_path,
    //         serde_json::to_string_pretty(&output).unwrap()
    //     ).expect("failed to write proof");
    // } else {
    //     info!("Proof output: {:?}", output);
    // }
}
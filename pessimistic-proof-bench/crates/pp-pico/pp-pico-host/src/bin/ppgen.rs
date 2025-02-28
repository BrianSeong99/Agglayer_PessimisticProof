use std::{path::PathBuf, time::Instant};
use pico_sdk::{client::DefaultProverClient, init_logger};
use agglayer_types::{Certificate, U256};
use clap::Parser;
use pessimistic_proof::bridge_exit::{NetworkId, TokenInfo};
use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState};
use pessimistic_proof_test_suite::sample_data::{self as data};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

/// The arguments for the pp generator.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct PPGenArgs {
    /// The number of bridge exits.
    #[clap(long, default_value = "10")]
    n_exits: usize,

    /// The number of imported bridge exits.
    #[clap(long, default_value = "10")]
    n_imported_exits: usize,

    /// The optional output directory to write the proofs in JSON. If not set,
    /// the proof is simply logged.
    #[clap(long)]
    proof_dir: Option<PathBuf>,

    /// The optional path to the custom sample data.
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
    // Initialize logger
    init_logger();

    let args = PPGenArgs::parse();

    // Load the ELF file
    let elf = std::fs::read("crates/pp-pico/pp-pico-guest/elf/riscv32im-pico-zkvm-elf").expect("Failed to load ELF file");
    info!("Loaded ELF file, length: {}", elf.len());

    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();
    let old_network_state = NetworkState::from(old_state.clone());

    let bridge_exits = get_events(args.n_exits, args.sample_path.clone());
    let imported_bridge_exits = get_events(args.n_imported_exits, args.sample_path);

    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    // info!(
    //     "Certificate {}: [{}]",
    //     certificate.hash(),
    //     serde_json::to_string(&certificate).unwrap()
    // );

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();

    // Validate inputs by running generate_pessimistic_proof first
    match generate_pessimistic_proof(old_network_state.clone(), &multi_batch_header) {
        Ok(_) => {
            info!("Input validation successful, proceeding with proof generation");
        }
        Err(e) => {
            panic!("Input validation failed: {:?}", e);
        }
    }

    info!(
        "Generating the proof for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );

    // Initialize the prover client
    let client = DefaultProverClient::new(&elf);
    let stdin_builder = client.get_stdin_builder();

    // Write inputs to the VM
    stdin_builder.borrow_mut().write(&old_network_state);
    stdin_builder.borrow_mut().write(&multi_batch_header);

    let start = Instant::now();
    let proof = client.prove_fast().expect("proving failed");
    let duration = start.elapsed();
    info!(
        "Successfully generated the proof with a latency of {:?}",
        duration
    );

    // // Get the verification key
    // let vkey = hex::encode(&proof.vk);
    // info!("vkey: {}", vkey);

    // let fixture = PessimisticProofFixture {
    //     certificate,
    //     pp_inputs: PessimisticProofOutput::from_bytes(&proof.pv_stream.unwrap()).into(),
    //     signer: state.get_signer(),
    //     vkey: vkey.clone(),
    //     public_values: format!("0x{}", hex::encode(proof.pv_stream.unwrap())),
    //     proof: format!("0x{}", hex::encode(proof.proof)),
    // };

    // if let Some(proof_dir) = args.proof_dir {
    //     // Save the proof to a json file
    //     let proof_path = proof_dir.join(format!(
    //         "{}-exits-v{}-{}.json",
    //         args.n_exits,
    //         &vkey[..8],
    //         Uuid::new_v4()
    //     ));
    //     if let Err(e) = std::fs::create_dir_all(&proof_dir) {
    //         warn!("Failed to create directory: {e}");
    //     }
    //     info!("Writing the proof to {:?}", proof_path);
    //     std::fs::write(proof_path, serde_json::to_string_pretty(&fixture).unwrap())
    //         .expect("failed to write fixture");
    // } else {
    //     info!("Proof: {:?}", fixture);
    // }
}

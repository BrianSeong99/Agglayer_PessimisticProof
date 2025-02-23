use std::{path::PathBuf, time::Instant};
use openvm_circuit::arch::{instructions::exe::VmExe, VmExecutor};
use openvm_keccak256_circuit::Keccak256Rv32Config;
use openvm_keccak256_transpiler::Keccak256TranspilerExtension;
use openvm_rv32im_circuit::Rv32ImConfig;
use openvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32IoTranspilerExtension, Rv32MTranspilerExtension,
};
use openvm_sdk::StdIn;
use openvm_stark_sdk::p3_baby_bear::BabyBear;
use openvm_transpiler::{
    elf::Elf, openvm_platform::memory::MEM_SIZE, transpiler::Transpiler, FromElf,
};
use openvm_stark_sdk::openvm_stark_backend::p3_field::PrimeField32;

use pessimistic_proof::bridge_exit::TokenInfo;
use pessimistic_proof::PessimisticProofOutput;
use pessimistic_proof_test_suite::sample_data::{self as data};
use pessimistic_proof_core::{generate_pessimistic_proof, NetworkState};
use agglayer_types::U256;
use clap::Parser;
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

    /// The optional output directory to write the proofs in JSON.
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

fn main() {
    let args = PPGenArgs::parse();

    // Load the ELF file
    let elf = Elf::decode(
        include_bytes!("../../../../target/riscv32im-risc0-zkvm-elf/release/program-openvm"),
        MEM_SIZE as u32,
    ).unwrap();
    info!("Loaded ELF file, length: {}", elf.instructions.len());

    let mut state = data::sample_state_00();
    let old_state = state.state_b.clone();
    let old_network_state = NetworkState::from(old_state.clone());

    let bridge_exits = get_events(args.n_exits, args.sample_path.clone());
    let imported_bridge_exits = get_events(args.n_imported_exits, args.sample_path);

    let certificate = state.apply_events(&imported_bridge_exits, &bridge_exits);

    info!(
        "Certificate {}: [{}]",
        certificate.hash(),
        serde_json::to_string(&certificate).unwrap()
    );

    let l1_info_root = certificate.l1_info_root().unwrap().unwrap_or_default();
    let multi_batch_header = old_state
        .make_multi_batch_header(&certificate, state.get_signer(), l1_info_root)
        .unwrap();

    info!(
        "Generating the proof for {} bridge exit(s) and {} imported bridge exit(s)",
        bridge_exits.len(),
        imported_bridge_exits.len()
    );

    // Validate inputs first
    match generate_pessimistic_proof(old_network_state.clone(), &multi_batch_header) {
        Ok(_) => {
            info!("Input validation successful, proceeding with proof generation");
        }
        Err(e) => {
            panic!("Input validation failed: {:?}", e);
        }
    }

    // Create transpiler with extensions
    let mut transpiler = Transpiler::<BabyBear>::default();
    transpiler = transpiler.with_extension(Rv32ITranspilerExtension);
    transpiler = transpiler.with_extension(Rv32MTranspilerExtension);
    transpiler = transpiler.with_extension(Rv32IoTranspilerExtension);
    transpiler = transpiler.with_extension(Keccak256TranspilerExtension);

    // Create VM executable
    let exe = VmExe::from_elf(elf, transpiler).unwrap();

    // Configure and create VM executor
    let mut config = Keccak256Rv32Config::default();
    config.system = config.system.with_continuations();
    let executor = VmExecutor::<BabyBear, _>::new(config);

    // Prepare input data as bytes
    let mut input_data = Vec::new();
    input_data.extend_from_slice(&bincode::serialize(&old_network_state).unwrap());
    input_data.extend_from_slice(&bincode::serialize(&multi_batch_header).unwrap());

    let start = Instant::now();
    let result = executor
        .execute(exe.clone(), StdIn::from_bytes(&input_data))
        .unwrap();
    let duration = start.elapsed();
    info!(
        "Successfully generated the proof with a latency of {:?}",
        duration
    );

    // Get the proof output from stdout
    let pp_output: PessimisticProofOutput = bincode::deserialize(&result.stdout).unwrap();
    
    if let Some(proof_dir) = args.proof_dir {
        let proof_path = proof_dir.join(format!(
            "{}-exits-{}.json",
            args.n_exits,
            Uuid::new_v4()
        ));
        if let Err(e) = std::fs::create_dir_all(&proof_dir) {
            warn!("Failed to create directory: {e}");
        }
        info!("Writing the proof to {:?}", proof_path);
        std::fs::write(proof_path, serde_json::to_string_pretty(&pp_output).unwrap())
            .expect("failed to write proof");
    } else {
        info!("Proof: {:?}", pp_output);
    }
} 
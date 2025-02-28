use std::{path::PathBuf, time::Instant};
use openvm_circuit::arch::{instructions::exe::VmExe, VmExecutor};
use openvm_keccak256_circuit::Keccak256Rv32Config;
use openvm_keccak256_transpiler::Keccak256TranspilerExtension;
use openvm_rv32im_circuit::Rv32ImConfig;
use openvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32IoTranspilerExtension, Rv32MTranspilerExtension,
};
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
use bincode::{serialize, deserialize};

use openvm_sdk::{
    StdIn,
    config::{SdkVmConfig, AppConfig, DEFAULT_APP_LOG_BLOWUP, DEFAULT_LEAF_LOG_BLOWUP},
    prover::AppProver,
    Sdk,
};
use openvm_stark_sdk::config::FriParameters;

use openvm_native_compiler::conversion::CompilerOptions;

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
        include_bytes!("../../../pp-openvm-guest/target/riscv32im-risc0-zkvm-elf/release/pp-openvm-guest"),
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

    // Configure VM
    let mut config = Keccak256Rv32Config::default();
    config.system = config.system.with_continuations();
    
    // Create SDK and app config
    let sdk = Sdk;
    let app_vm_config = SdkVmConfig::builder()
        .system(openvm_sdk::config::SdkSystemConfig { config: config.system })
        .rv32i(Default::default())
        .rv32m(Default::default())
        .io(Default::default())
        .keccak(Default::default())
        .sha256(Default::default())
        .build();
    
    // Define log blowup values
    let app_log_blowup = DEFAULT_APP_LOG_BLOWUP;
    let leaf_log_blowup = DEFAULT_LEAF_LOG_BLOWUP;
    
    // Create AppConfig with all required fields
    let app_config = AppConfig {
        app_fri_params: FriParameters::standard_with_100_bits_conjectured_security(
            app_log_blowup,
        ).into(),
        app_vm_config,
        leaf_fri_params: FriParameters::standard_with_100_bits_conjectured_security(
            leaf_log_blowup,
        ).into(),
        compiler_options: CompilerOptions {
            enable_cycle_tracker: false,
            ..Default::default()
        },
    };

    // Generate keys and commit executable
    let app_pk = sdk.app_keygen(app_config).unwrap();
    let app_committed_exe = sdk.commit_app_exe(app_pk.app_fri_params(), exe).unwrap();

    // Serialize test data
    let mut input_data = serialize(&old_network_state).unwrap();
    let batch_header_bytes = serialize(&multi_batch_header).unwrap();
    input_data.extend(batch_header_bytes);

    // Create stdin from serialized data
    let stdin = StdIn::from_bytes(&input_data);

    // Create prover and generate proof
    let program_name = format!("pessimistic-proof-{}", args.n_exits);
    let app_prover = AppProver::new(app_pk.app_vm_pk.clone(), app_committed_exe)
        .with_program_name(program_name);
    let start = Instant::now();
    let proof = app_prover.generate_app_proof(stdin);
    let duration = start.elapsed();
    
    // Verify the proof
    let app_vk = app_pk.get_app_vk();
    sdk.verify_app_proof(&app_vk, &proof).unwrap();
    info!(
        "Successfully generated and verified the proof with a latency of {:?}",
        duration
    );

    // // Get the proof output from stdout
    // let pp_output: PessimisticProofOutput = bincode::deserialize(&result.unwrap()).unwrap();
    // if let Some(proof_dir) = args.proof_dir {
    //     let proof_path = proof_dir.join(format!(
    //         "{}-exits-{}.json",
    //         args.n_exits,
    //         Uuid::new_v4()
    //     ));
    //     if let Err(e) = std::fs::create_dir_all(&proof_dir) {
    //         warn!("Failed to create directory: {e}");
    //     }
    //     info!("Writing the proof to {:?}", proof_path);
    //     std::fs::write(proof_path, serde_json::to_string_pretty(&pp_output).unwrap())
    //         .expect("failed to write proof");
    // } else {
    //     info!("Proof: {:?}", pp_output);
    // }
}

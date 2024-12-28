use bincode::config::Options;
use eyre::Result;
pub use pessimistic_proof::{LocalNetworkState, PessimisticProofOutput};

use std::{fs, sync::Arc};
use openvm::platform::memory::MEM_SIZE;
use openvm_stark_sdk::{
    config::{
        baby_bear_poseidon2::{BabyBearPoseidon2Config},
        FriParameters,
    },
    p3_baby_bear::BabyBear,
};
use openvm_sdk::{
    config::{AppConfig, SdkVmConfig},
    prover::{
        AppProver,
        vm::ContinuationVmProof
    },
    keygen::AppVerifyingKey,
    Sdk, StdIn
};
use openvm_transpiler::elf::Elf;

use crate::PESSIMISTIC_PROOF_ELF;

pub type Hasher = pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
pub type Digest = <Hasher as pessimistic_proof::local_exit_tree::hasher::Hasher>::Digest;
pub type MultiBatchHeader = pessimistic_proof::multi_batch_header::MultiBatchHeader<Hasher>;

pub struct ProofOutput {}

pub(crate) type SC = BabyBearPoseidon2Config;

/// A convenient interface to run the pessimistic proof ELF bytecode.
pub struct Runner {
    sdk: Sdk,
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

impl Runner {
    /// Create a new pessimistic proof client.
    pub fn new() -> Self {
        let sdk = Sdk;
        Self { sdk }
    }

    // /// Create a new pessimistic proof client from a custom generic client.
    // pub fn from_client(client: sp1_sdk::ProverClient) -> Self {
    //     Self { client }
    // }

    /// Convert inputs to stdin.
    pub fn prepare_stdin(state: &LocalNetworkState, batch_header: &MultiBatchHeader) -> StdIn {
        let mut stdin = StdIn::default();
        stdin.write(state);
        stdin.write(batch_header);
        stdin
    }

    // /// Extract outputs from the committed public values.
    // pub fn extract_output(public_vals: Vec<BabyBear>) -> PessimisticProofOutput {
    //     PessimisticProofOutput::bincode_options()
    //         .deserialize(public_vals.as_slice())
    //         .expect("deser")
    // }

    /// Execute the ELF with given inputs.
    pub fn execute(
        &self,
        state: &LocalNetworkState,
        batch_header: &MultiBatchHeader,
    ) -> Result<bool> {
        let stdin = Self::prepare_stdin(state, batch_header);

        let vm_config = SdkVmConfig::builder()
            .system(Default::default())
            .rv32i(Default::default())
            .rv32m(Default::default())
            .io(Default::default())
            .bigint(Default::default())
            .native(Default::default())
            .keccak(Default::default())
            .build();

        println!("here");

        let elf_bytes = fs::read("target/riscv32im-risc0-zkvm-elf/release/pessimistic-proof-openvm")?;
        let elf = Elf::decode(&elf_bytes, MEM_SIZE as u32)?;

        println!("here1");

        // 3. Transpile the ELF into a VmExe
        let exe = self.sdk.transpile(elf, vm_config.transpiler())?;

        println!("here2");

        let output = self.sdk.execute(exe.clone(), vm_config.clone(), stdin.clone())?;

        println!("here3");

        println!("public values output: {:?}", output);

        Ok(true)

        // let pp_output = Self::extract_output(output);

        // Ok(pp_output)
    }

    // pub fn get_vkey(&self) -> AppVerifyingKey {
    //     let (_pk, vk) = self.sdk.setup(PESSIMISTIC_PROOF_ELF);
    //     vk
    // }

    /// Generate one plonk proof.
    pub fn generate_proof(
        &self,
        state: &LocalNetworkState,
        batch_header: &MultiBatchHeader,
    ) -> Result<(
        ContinuationVmProof<SC>,
        AppVerifyingKey,
        // PessimisticProofOutput,
    )> {
        // 2b. Load the ELF from a file
        let elf_bytes = fs::read("../../pessimistic-proof-program/elf/riscv32im-openvm-zkvm-elf")?;
        let elf = Elf::decode(&elf_bytes, MEM_SIZE as u32)?;
        
        let vm_config = SdkVmConfig::builder()
            .system(Default::default())
            .rv32i(Default::default())
            .rv32m(Default::default())
            .io(Default::default())
            .build();

        // 3. Transpile the ELF into a VmExe
        let exe = self.sdk.transpile(elf, vm_config.transpiler())?;

        let app_log_blowup = 2;
        let app_fri_params = FriParameters::standard_with_100_bits_conjectured_security(app_log_blowup);
        let app_config = AppConfig::new(app_fri_params, vm_config);

        // 7. Commit the exe
        let app_committed_exe = self.sdk.commit_app_exe(app_fri_params, exe)?;

        // 8. Generate an AppProvingKey
        let app_pk = Arc::new(self.sdk.app_keygen(app_config)?);

        let stdin = Self::prepare_stdin(state, batch_header);

        // // 9a. Generate a proof
        // let proof = self.sdk.generate_app_proof(app_pk.clone(), app_committed_exe.clone(), stdin.clone())?;
        // 9b. Generate a proof with an AppProver with custom fields
        let app_prover = AppProver::new(app_pk.app_vm_pk.clone(), app_committed_exe.clone())
            .with_program_name("pessimistic_proof");
        let proof = app_prover.generate_app_proof(stdin.clone());
        // let output = Self::extract_output(proof.user_public_values.public_values.clone());

        let app_vk = app_pk.get_vk();

        // Ok((proof, app_vk, output))
        Ok((proof, app_vk))
    }
}

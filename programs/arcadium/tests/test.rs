//! LiteSVM tests for Arcadium program.
//!
//! Run from repo root:
//!   cd arcadium
//!   anchor build
//!   cargo test -p arcadium -- --nocapture

use anyhow::Result;
use borsh::BorshSerialize;
use sha2::{Digest, Sha256};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

/// Compute the 8-byte Anchor discriminator for a global instruction.
fn anchor_discriminator(ix_name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("global:{ix_name}").as_bytes());
    let hash = hasher.finalize();
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash[..8]);
    disc
}

#[derive(BorshSerialize)]
struct InitializeArgs {
    platform_fee_bps: u16,
}

#[derive(BorshSerialize)]
struct RegisterAgentArgs {
    name: String,
    skill_uri: String,
    price_lamports: u64,
}

#[derive(BorshSerialize)]
struct CreateJobArgs {
    task_description: String,
}

#[derive(BorshSerialize)]
struct RateJobArgs {
    rating: u8,
    comment: String,
}

#[test]
fn prints_instruction_encodings_and_pdas() -> Result<()> {
    // This test does not run the VM — it prints PDAs + encoded instruction data.
    // It is useful even if LiteSVM API changes.

    let program_id = arcadium::ID;
    println!("program_id: {}", program_id);

    // Keypairs
    let authority = Keypair::new();
    let owner = Keypair::new();
    let client = Keypair::new();

    // PDAs
    let (platform_pda, platform_bump) = Pubkey::find_program_address(&[b"platform"], &program_id);
    let (agent_pda, agent_bump) = Pubkey::find_program_address(&[b"agent", owner.pubkey().as_ref()], &program_id);
    let (agent_vault_pda, agent_vault_bump) = Pubkey::find_program_address(&[b"agent_vault", agent_pda.as_ref()], &program_id);

    // NOTE: job PDA seed in program uses jobs_completed counter; for tests we start at 0.
    let job_index_bytes = 0u64.to_le_bytes();
    let (job_pda, job_bump) = Pubkey::find_program_address(&[b"job", agent_pda.as_ref(), &job_index_bytes], &program_id);
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(&[b"escrow", job_pda.as_ref()], &program_id);
    let (review_pda, review_bump) = Pubkey::find_program_address(&[b"review", job_pda.as_ref()], &program_id);

    println!("platform_pda: {platform_pda} bump={platform_bump}");
    println!("agent_pda: {agent_pda} bump={agent_bump}");
    println!("agent_vault_pda: {agent_vault_pda} bump={agent_vault_bump}");
    println!("job_pda: {job_pda} bump={job_bump}");
    println!("escrow_pda: {escrow_pda} bump={escrow_bump}");
    println!("review_pda: {review_pda} bump={review_bump}");

    // Build instruction datas (discriminator + borsh args)
    let init_ix_data = {
        let mut data = Vec::from(anchor_discriminator("initialize"));
        data.extend(InitializeArgs { platform_fee_bps: 1000 }.try_to_vec()?);
        data
    };
    println!("initialize ix data len={} hex={}", init_ix_data.len(), hex::encode(&init_ix_data));

    let reg_ix_data = {
        let mut data = Vec::from(anchor_discriminator("register_agent"));
        data.extend(
            RegisterAgentArgs {
                name: "Sarah's Professional Emails".to_string(),
                skill_uri: "ipfs://example".to_string(),
                price_lamports: 10_000_000,
            }
            .try_to_vec()?,
        );
        data
    };
    println!("register_agent ix data len={}", reg_ix_data.len());

    let create_job_ix_data = {
        let mut data = Vec::from(anchor_discriminator("create_job"));
        data.extend(
            CreateJobArgs {
                task_description: "Write an email declining a meeting but offering alternatives".to_string(),
            }
            .try_to_vec()?,
        );
        data
    };
    println!("create_job ix data len={}", create_job_ix_data.len());

    let complete_job_ix_data = Vec::from(anchor_discriminator("complete_job"));
    println!("complete_job ix data len={} hex={}", complete_job_ix_data.len(), hex::encode(&complete_job_ix_data));

    let rate_job_ix_data = {
        let mut data = Vec::from(anchor_discriminator("rate_job"));
        data.extend(
            RateJobArgs {
                rating: 5,
                comment: "Perfect output".to_string(),
            }
            .try_to_vec()?,
        );
        data
    };
    println!("rate_job ix data len={}", rate_job_ix_data.len());

    // If you want, we can add a full LiteSVM execution test below.
    // Keeping this file compiling + printing useful info is the priority.

    Ok(())
}

/// Full in-process execution test using LiteSVM.
///
/// NOTE: LiteSVM APIs can change by version. If this fails to compile,
/// tell me the exact litesvm version you have resolved in Cargo.lock and I will adjust.
#[test]
fn litesvm_happy_path_create_and_complete_job() -> Result<()> {
    use litesvm::LiteSVM;

    let program_id = arcadium::ID;

    // Paths assume you ran `anchor build` from `arcadium/` root.
    let so_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("deploy")
        .join("arcadium.so");

    println!("Loading program .so from: {}", so_path.display());

    let mut svm = LiteSVM::new();

    // Fund some accounts
    let authority = Keypair::new();
    let owner = Keypair::new();
    let client = Keypair::new();

    svm.airdrop(&authority.pubkey(), 10_000_000_000)?;
    svm.airdrop(&owner.pubkey(), 10_000_000_000)?;
    svm.airdrop(&client.pubkey(), 10_000_000_000)?;

    // Load BPF program
    svm.add_program_from_file(program_id, so_path)?;

    // Derive PDAs
    let (platform_pda, _platform_bump) = Pubkey::find_program_address(&[b"platform"], &program_id);
    let (agent_pda, _agent_bump) = Pubkey::find_program_address(&[b"agent", owner.pubkey().as_ref()], &program_id);
    let (agent_vault_pda, _agent_vault_bump) = Pubkey::find_program_address(&[b"agent_vault", agent_pda.as_ref()], &program_id);
    let job_index_bytes = 0u64.to_le_bytes();
    let (job_pda, _job_bump) = Pubkey::find_program_address(&[b"job", agent_pda.as_ref(), &job_index_bytes], &program_id);
    let (escrow_pda, _escrow_bump) = Pubkey::find_program_address(&[b"escrow", job_pda.as_ref()], &program_id);

    // 1) initialize
    let mut init_data = Vec::from(anchor_discriminator("initialize"));
    init_data.extend(InitializeArgs { platform_fee_bps: 1000 }.try_to_vec()?);

    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(platform_pda, false),
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: init_data,
    };

    let mut tx = Transaction::new_with_payer(&[init_ix], Some(&authority.pubkey()));
    let bh = svm.latest_blockhash();
    tx.sign(&[&authority], bh);
    let res = svm.send_transaction(tx);
    println!("initialize result: {res:?}");
    res?;

    // 2) register_agent (also creates agent_vault)
    let mut reg_data = Vec::from(anchor_discriminator("register_agent"));
    reg_data.extend(
        RegisterAgentArgs {
            name: "Sarah".to_string(),
            skill_uri: "ipfs://example".to_string(),
            price_lamports: 10_000_000,
        }
        .try_to_vec()?,
    );

    let reg_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(agent_pda, false),
            AccountMeta::new(agent_vault_pda, false),
            AccountMeta::new(owner.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: reg_data,
    };

    let mut tx = Transaction::new_with_payer(&[reg_ix], Some(&owner.pubkey()));
    let bh = svm.latest_blockhash();
    tx.sign(&[&owner], bh);
    let res = svm.send_transaction(tx);
    println!("register_agent result: {res:?}");
    res?;

    // 3) create_job
    let mut create_data = Vec::from(anchor_discriminator("create_job"));
    create_data.extend(
        CreateJobArgs {
            task_description: "Decline meeting".to_string(),
        }
        .try_to_vec()?,
    );

    let create_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(agent_pda, false),
            AccountMeta::new(job_pda, false),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new(platform_pda, false),
            AccountMeta::new(client.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: create_data,
    };

    let mut tx = Transaction::new_with_payer(&[create_ix], Some(&client.pubkey()));
    let bh = svm.latest_blockhash();
    tx.sign(&[&client], bh);
    let res = svm.send_transaction(tx);
    println!("create_job result: {res:?}");
    res?;

    // Print balances after escrow
    let client_bal = svm.get_balance(&client.pubkey())?;
    let escrow_bal = svm.get_balance(&escrow_pda)?;
    println!("balances after create_job: client={client_bal} escrow={escrow_bal}");

    // 4) complete_job
    let complete_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(job_pda, false),
            AccountMeta::new(agent_pda, false),
            AccountMeta::new(escrow_pda, false),
            AccountMeta::new(agent_vault_pda, false),
            AccountMeta::new(platform_pda, false),
            AccountMeta::new(authority.pubkey(), false),
            AccountMeta::new(client.pubkey(), true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: Vec::from(anchor_discriminator("complete_job")),
    };

    let mut tx = Transaction::new_with_payer(&[complete_ix], Some(&client.pubkey()));
    let bh = svm.latest_blockhash();
    tx.sign(&[&client], bh);
    let res = svm.send_transaction(tx);
    println!("complete_job result: {res:?}");
    res?;

    // Print balances after completion
    let escrow_bal = svm.get_balance(&escrow_pda)?;
    let vault_bal = svm.get_balance(&agent_vault_pda)?;
    let authority_bal = svm.get_balance(&authority.pubkey())?;
    println!("balances after complete_job: escrow={escrow_bal} agent_vault={vault_bal} platform_authority={authority_bal}");

    Ok(())
}

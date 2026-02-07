# Arcadium Solana Program

## Overview

This Anchor program implements the core escrow and payment logic for Arcadium marketplace.

## Program Structure

### Instructions

1. **`initialize`** - One-time platform setup
   - Sets platform authority and fee (10% = 1000 bps)
   - Creates platform account

2. **`register_agent`** - Register a new agent
   - Creates agent PDA with name, skill URI, price
   - Agent owner can later update or deactivate

3. **`create_job`** - Client hires agent
   - Creates job account
   - Transfers payment from client to escrow PDA
   - Increments platform job counter

4. **`complete_job`** - Client accepts work
   - Validates job status
   - Splits payment: 90% to agent vault, 10% to platform
   - Updates agent stats (jobs completed, total earned)
   - Closes escrow

5. **`rate_job`** - Client rates completed job
   - Creates review account
   - Updates agent's rating (sum and count)

6. **`withdraw_earnings`** - Agent withdraws from vault
   - Transfers from agent vault PDA to owner wallet
   - Can withdraw partial or full amount

### Accounts

- **Platform** - Global platform config (authority, fee, stats)
- **Agent** - Agent profile (owner, name, skill URI, price, stats)
- **Job** - Individual job (agent, client, task, status, timestamps)
- **Review** - Rating for completed job (1-5 stars + comment)

### PDAs

- `platform` → `["platform"]`
- `agent` → `["agent", owner_pubkey]`
- `job` → `["job", agent_pubkey, job_index]`
- `escrow` → `["escrow", job_pubkey]`
- `agent_vault` → `["agent_vault", agent_pubkey]`
- `review` → `["review", job_pubkey]`

## Build Instructions

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI (v1.18.18)
sh -c "$(curl -sSfL https://release.solana.com/v1.18.18/install)"

# Install Anchor CLI (v0.30.1)
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
```

### Build

```bash
cd arcadium
anchor build
```

### Test

```bash
# Run tests (requires local validator)
anchor test

# Or test on devnet
anchor test --skip-local-validator
```

### Deploy

```bash
# Generate new program keypair (first time only)
anchor keys list
solana-keygen new -o target/deploy/arcadium-keypair.json

# Get program ID and update Anchor.toml + lib.rs declare_id!()
anchor keys list

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet
anchor deploy --provider.cluster mainnet
```

## Example Usage

### 1. Initialize Platform

```typescript
await program.methods
  .initialize(1000) // 10% fee
  .accounts({
    platform: platformPDA,
    authority: authority.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([authority])
  .rpc();
```

### 2. Register Agent

```typescript
await program.methods
  .registerAgent(
    "Sarah's Professional Emails",
    "ipfs://QmSkillFile123...",
    10_000_000 // 0.01 SOL
  )
  .accounts({
    agent: agentPDA,
    owner: owner.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([owner])
  .rpc();
```

### 3. Create Job (Client Hires Agent)

```typescript
await program.methods
  .createJob("Write email declining meeting but offering alternative")
  .accounts({
    agent: agentPDA,
    job: jobPDA,
    escrow: escrowPDA,
    platform: platformPDA,
    client: client.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([client])
  .rpc();
```

### 4. Complete Job (Client Accepts)

```typescript
await program.methods
  .completeJob()
  .accounts({
    job: jobPDA,
    agent: agentPDA,
    escrow: escrowPDA,
    agentVault: agentVaultPDA,
    platform: platformPDA,
    platformAuthority: platformAuthority.publicKey,
    client: client.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([client])
  .rpc();
```

### 5. Rate Job

```typescript
await program.methods
  .rateJob(5, "Perfect email! Exactly what I needed.")
  .accounts({
    job: jobPDA,
    agent: agentPDA,
    review: reviewPDA,
    client: client.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([client])
  .rpc();
```

### 6. Withdraw Earnings

```typescript
await program.methods
  .withdrawEarnings(new BN(50_000_000)) // 0.05 SOL
  .accounts({
    agent: agentPDA,
    agentVault: agentVaultPDA,
    owner: owner.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([owner])
  .rpc();
```

## Security Considerations

### Implemented

✅ PDA-based escrow (trustless, no admin control over funds)  
✅ Payment split enforced on-chain (90/10)  
✅ Client must sign to accept and release payment  
✅ Agent vault isolated per agent  
✅ Rate limiting via account rent  

### For Production

⚠️ Add dispute resolution (timeout, admin arbitration)  
⚠️ Add agent deactivation logic  
⚠️ Add job cancellation (before completion)  
⚠️ Consider slashing for bad agents  
⚠️ Add comprehensive unit tests  
⚠️ Professional audit before mainnet launch  

## MVP Notes

For the hackathon demo:
- We'll initialize with 10% platform fee (1000 bps)
- Register one agent: "Sarah's Email Writer"
- Show full job flow: create → complete → rate
- Deploy to devnet first, then mainnet for submission

Program is ~350 lines, focused on core escrow mechanics. Ready to build!

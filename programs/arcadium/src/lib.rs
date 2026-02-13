use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

declare_id!("GNZJTKQDSJdDLLxuNYKMwV4qHL8fdxsABzfqmMyzhEHJ");

#[program]
pub mod arcadium {
    use super::*;

    /// Initialize the platform (one-time setup)
    pub fn initialize(ctx: Context<Initialize>, platform_fee_bps: u16) -> Result<()> {
        require!(platform_fee_bps <= 10000, ErrorCode::InvalidFeeBps);
        
        let platform = &mut ctx.accounts.platform;
        platform.authority = ctx.accounts.authority.key();
        platform.platform_fee_bps = platform_fee_bps;
        platform.total_jobs = 0;
        platform.total_volume = 0;
        
        Ok(())
    }

    /// Register an agent
    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        name: String,
        skill_uri: String,
        price_lamports: u64,
    ) -> Result<()> {
        require!(name.len() <= 50, ErrorCode::NameTooLong);
        require!(skill_uri.len() <= 200, ErrorCode::UriTooLong);
        require!(price_lamports > 0, ErrorCode::InvalidPrice);

        let agent = &mut ctx.accounts.agent;
        agent.owner = ctx.accounts.owner.key();
        agent.name = name;
        agent.skill_uri = skill_uri;
        agent.price_lamports = price_lamports;
        agent.jobs_completed = 0;
        agent.total_earned = 0;
        agent.rating_sum = 0;
        agent.rating_count = 0;
        agent.is_active = true;
        agent.bump = ctx.bumps.agent;

        Ok(())
    }

    /// Create a job (client pays, funds go to escrow)
    pub fn create_job(
        ctx: Context<CreateJob>,
        task_description: String,
    ) -> Result<()> {
        require!(task_description.len() <= 500, ErrorCode::TaskTooLong);

        let agent = &ctx.accounts.agent;
        require!(agent.is_active, ErrorCode::AgentNotActive);

        let job = &mut ctx.accounts.job;
        job.agent = agent.key();
        job.client = ctx.accounts.client.key();
        job.task_description = task_description;
        job.price_lamports = agent.price_lamports;
        job.status = JobStatus::Created;
        job.created_at = Clock::get()?.unix_timestamp;
        job.bump = ctx.bumps.job;

        // Transfer payment from client to escrow PDA
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.client.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
            },
        );
        transfer(cpi_context, agent.price_lamports)?;

        // Increment platform job counter
        let platform = &mut ctx.accounts.platform;
        platform.total_jobs += 1;

        Ok(())
    }

    /// Complete a job (agent delivers work, client accepts, payment releases)
    pub fn complete_job(ctx: Context<CompleteJob>) -> Result<()> {
        let job = &mut ctx.accounts.job;
        require!(job.status == JobStatus::Created, ErrorCode::InvalidJobStatus);

        let _agent = &ctx.accounts.agent;
        let platform = &ctx.accounts.platform;

        // Calculate splits: 90% to agent, 10% to platform
        let total_amount = job.price_lamports;
        let platform_fee = (total_amount as u128)
            .checked_mul(platform.platform_fee_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;
        let agent_amount = total_amount.checked_sub(platform_fee).unwrap();

        // Transfer from escrow to agent vault
        // (bind pubkeys so the references outlive this statement)
        let job_key = job.key();
        let escrow_seeds = &[
            b"escrow",
            job_key.as_ref(),
            &[job.bump],
        ];
        let signer_seeds = &[&escrow_seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.agent_vault.to_account_info(),
            },
            signer_seeds,
        );
        transfer(cpi_context, agent_amount)?;

        // Transfer platform fee
        let cpi_context_fee = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.platform_authority.to_account_info(),
            },
            signer_seeds,
        );
        transfer(cpi_context_fee, platform_fee)?;

        // Update job status
        job.status = JobStatus::Completed;
        job.completed_at = Some(Clock::get()?.unix_timestamp);

        // Update agent stats
        let agent_mut = &mut ctx.accounts.agent;
        agent_mut.jobs_completed += 1;
        agent_mut.total_earned += agent_amount;

        // Update platform stats
        let platform_mut = &mut ctx.accounts.platform;
        platform_mut.total_volume += total_amount;

        Ok(())
    }

    /// Submit a rating for a completed job
    pub fn rate_job(ctx: Context<RateJob>, rating: u8, comment: String) -> Result<()> {
        require!(rating >= 1 && rating <= 5, ErrorCode::InvalidRating);
        require!(comment.len() <= 200, ErrorCode::CommentTooLong);

        let job = &ctx.accounts.job;
        require!(job.status == JobStatus::Completed, ErrorCode::JobNotCompleted);

        let review = &mut ctx.accounts.review;
        review.job = job.key();
        review.agent = job.agent;
        review.client = ctx.accounts.client.key();
        review.rating = rating;
        review.comment = comment;
        review.created_at = Clock::get()?.unix_timestamp;

        // Update agent rating
        let agent = &mut ctx.accounts.agent;
        agent.rating_sum += rating as u64;
        agent.rating_count += 1;

        Ok(())
    }

    /// Withdraw earnings from agent vault
    pub fn withdraw_earnings(ctx: Context<WithdrawEarnings>, amount: u64) -> Result<()> {
        let agent = &ctx.accounts.agent;
        let vault_balance = ctx.accounts.agent_vault.lamports();
        
        require!(amount <= vault_balance, ErrorCode::InsufficientFunds);

        // (bind pubkey so the reference outlives this statement)
        let agent_key = agent.key();
        let vault_seeds = &[
            b"agent_vault",
            agent_key.as_ref(),
            &[ctx.bumps.agent_vault],
        ];
        let signer_seeds = &[&vault_seeds[..]];

        let cpi_context = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.agent_vault.to_account_info(),
                to: ctx.accounts.owner.to_account_info(),
            },
            signer_seeds,
        );
        transfer(cpi_context, amount)?;

        Ok(())
    }
}

// ============================================================================
// Contexts
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Platform::INIT_SPACE,
        seeds = [b"platform"],
        bump
    )]
    pub platform: Account<'info, Platform>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Agent::INIT_SPACE,
        seeds = [b"agent", owner.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateJob<'info> {
    #[account(mut)]
    pub agent: Account<'info, Agent>,
    
    #[account(
        init,
        payer = client,
        space = 8 + Job::INIT_SPACE,
        seeds = [b"job", agent.key().as_ref(), &agent.jobs_completed.to_le_bytes()],
        bump
    )]
    pub job: Account<'info, Job>,
    
    /// CHECK: PDA for escrow
    #[account(
        mut,
        seeds = [b"escrow", job.key().as_ref()],
        bump
    )]
    pub escrow: AccountInfo<'info>,
    
    #[account(mut)]
    pub platform: Account<'info, Platform>,
    
    #[account(mut)]
    pub client: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CompleteJob<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    
    #[account(
        mut,
        seeds = [b"agent", agent.owner.as_ref()],
        bump = agent.bump
    )]
    pub agent: Account<'info, Agent>,
    
    /// CHECK: PDA for escrow
    #[account(
        mut,
        seeds = [b"escrow", job.key().as_ref()],
        bump = job.bump
    )]
    pub escrow: AccountInfo<'info>,
    
    /// CHECK: PDA for agent vault
    #[account(
        mut,
        seeds = [b"agent_vault", agent.key().as_ref()],
        bump
    )]
    pub agent_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub platform: Account<'info, Platform>,
    
    /// CHECK: Platform authority receives fees
    #[account(mut)]
    pub platform_authority: AccountInfo<'info>,
    
    pub client: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RateJob<'info> {
    #[account(mut)]
    pub job: Account<'info, Job>,
    
    #[account(mut)]
    pub agent: Account<'info, Agent>,
    
    #[account(
        init,
        payer = client,
        space = 8 + Review::INIT_SPACE,
        seeds = [b"review", job.key().as_ref()],
        bump
    )]
    pub review: Account<'info, Review>,
    
    #[account(mut)]
    pub client: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawEarnings<'info> {
    #[account(
        seeds = [b"agent", owner.key().as_ref()],
        bump = agent.bump,
        has_one = owner
    )]
    pub agent: Account<'info, Agent>,
    
    /// CHECK: PDA for agent vault
    #[account(
        mut,
        seeds = [b"agent_vault", agent.key().as_ref()],
        bump
    )]
    pub agent_vault: AccountInfo<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

// ============================================================================
// Accounts
// ============================================================================

#[account]
#[derive(InitSpace)]
pub struct Platform {
    pub authority: Pubkey,
    pub platform_fee_bps: u16,  // Basis points (1000 = 10%)
    pub total_jobs: u64,
    pub total_volume: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Agent {
    pub owner: Pubkey,
    #[max_len(50)]
    pub name: String,
    #[max_len(200)]
    pub skill_uri: String,
    pub price_lamports: u64,
    pub jobs_completed: u64,
    pub total_earned: u64,
    pub rating_sum: u64,
    pub rating_count: u64,
    pub is_active: bool,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Job {
    pub agent: Pubkey,
    pub client: Pubkey,
    #[max_len(500)]
    pub task_description: String,
    pub price_lamports: u64,
    pub status: JobStatus,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Review {
    pub job: Pubkey,
    pub agent: Pubkey,
    pub client: Pubkey,
    pub rating: u8,
    #[max_len(200)]
    pub comment: String,
    pub created_at: i64,
}

// ============================================================================
// Enums & Errors
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum JobStatus {
    Created,
    Completed,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid fee basis points (must be <= 10000)")]
    InvalidFeeBps,
    #[msg("Name too long (max 50 chars)")]
    NameTooLong,
    #[msg("URI too long (max 200 chars)")]
    UriTooLong,
    #[msg("Task description too long (max 500 chars)")]
    TaskTooLong,
    #[msg("Comment too long (max 200 chars)")]
    CommentTooLong,
    #[msg("Invalid price (must be > 0)")]
    InvalidPrice,
    #[msg("Agent is not active")]
    AgentNotActive,
    #[msg("Invalid job status")]
    InvalidJobStatus,
    #[msg("Job not completed")]
    JobNotCompleted,
    #[msg("Invalid rating (must be 1-5)")]
    InvalidRating,
    #[msg("Insufficient funds")]
    InsufficientFunds,
}

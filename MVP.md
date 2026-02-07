# Arcadium MVP - Hackathon Build Plan

## Core Value Demo

Demonstrate: "Expert creates Skill → Agent loads it → Client hires agent → Work delivered → Payment flows"

---

## MVP Scope (5 Days)

### What We Build

#### 1. Marketplace Frontend (Day 1-2)
- Browse agents (card view: name, specialty, price, rating, jobs completed)
- Agent detail page (description, sample outputs, reviews)
- Hire flow (describe task → pay SOL → wait → get result)
- Simple search/filter

**Tech:** Next.js + React, Wallet connection (Phantom/Solflare)

#### 2. Agent Registration (Day 1)
- Register agent form (name, description, webhook URL, price)
- Create agent vault (PDA for escrow)
- Upload skill file (stored on IPFS or S3 for MVP)
- Agent gets unique ID

**Tech:** Next.js API routes, Anchor program for vault

#### 3. Escrow Smart Contract (Day 2-3)
- `create_job` - Client pays, creates job, funds go to escrow
- `complete_job` - Agent delivers work, client accepts, payment releases (90/10 split)
- `dispute_job` - Client disputes, admin reviews, refund logic

**Tech:** Anchor (Solana program)

#### 4. Job Execution System (Day 3)
- Client submits job → Job created on-chain → Backend calls agent webhook
- Agent returns result → Stored in database
- Client reviews → Accepts or disputes

**Tech:** Node.js backend, webhook calls, simple job queue

#### 5. Rating System (Day 4)
- On-chain reviews (stored in program account)
- Average rating calculation
- Review history on agent page

#### 6. Demo Agent (Day 4-5)
- Build one working example agent: "Sarah's Professional Emails"
- Deploy it with a real webhook
- Pre-load with skill examples
- Use for demo video

---

## What We Skip (For Now)

- ❌ IPFS for skills (use S3/DB for MVP)
- ❌ Complex training UI (manual skill upload)
- ❌ Admin dispute resolution UI (hardcode for demo)
- ❌ Withdrawal UI (agents can withdraw via CLI)
- ❌ Advanced search/filters
- ❌ Chat between client and agent
- ❌ Skill versioning
- ❌ Multi-skill agents

---

## Tech Stack

### Frontend
- **Framework:** Next.js 14 (App Router)
- **Wallet:** `@solana/wallet-adapter` + Phantom
- **UI:** Tailwind CSS + shadcn/ui
- **RPC:** Helius (get API key via their agent portal)

### Smart Contract
- **Framework:** Anchor
- **Language:** Rust
- **Testing:** LiteSVM for fast unit tests

### Backend
- **Runtime:** Node.js
- **Framework:** Express or Next.js API routes
- **Database:** PostgreSQL (job queue, results, metadata)
- **Storage:** S3 or local filesystem (skills)

### Deployment
- **Frontend:** Vercel
- **Backend:** Railway or Render
- **Smart Contract:** Solana Devnet (then mainnet before submission)

---

## 5-Day Build Timeline

### Day 1 (Today) - Setup & Registration
- [x] Register for hackathon ✅
- [x] Set up AgentWallet ✅
- [ ] Create GitHub repo
- [ ] Scaffold Next.js project
- [ ] Set up Anchor project
- [ ] Build agent registration form
- [ ] Deploy placeholder contract to devnet

### Day 2 - Escrow Contract + Marketplace UI
- [ ] Write `create_job` instruction
- [ ] Write `complete_job` instruction
- [ ] Write `dispute_job` instruction
- [ ] Deploy and test contract
- [ ] Build marketplace browse page
- [ ] Build agent detail page

### Day 3 - Job Execution
- [ ] Build job submission flow (frontend)
- [ ] Backend webhook handler
- [ ] Job queue system
- [ ] Result delivery UI
- [ ] Accept/dispute flow

### Day 4 - Demo Agent + Reviews
- [ ] Build "Sarah's Email Agent"
- [ ] Create skill file with examples
- [ ] Deploy agent with webhook
- [ ] Implement on-chain reviews
- [ ] Rating display on agent cards

### Day 5 - Polish + Demo
- [ ] Test full flow end-to-end
- [ ] Fix bugs
- [ ] Record demo video (3-5 min)
- [ ] Write project description
- [ ] Deploy to mainnet
- [ ] Submit project

---

## Demo Flow (What Judges Will See)

1. **Browse Marketplace**
   - See 3-5 agents listed
   - Sarah's Professional Emails, Alex's Cold Outreach, etc.

2. **Hire Sarah's Agent**
   - Click "Sarah's Professional Emails"
   - See her rating (5 stars, 47 jobs)
   - Read sample output
   - Click "Hire Agent" (0.01 SOL)

3. **Submit Job**
   - Task: "Write email declining a meeting but offering alternative times"
   - Connect Phantom wallet
   - Pay 0.01 SOL (goes to escrow)
   - Job created on-chain

4. **Agent Works**
   - Backend calls Sarah's agent webhook
   - Agent loads skill, generates email
   - Returns result in ~10 seconds

5. **Review Result**
   - Client sees email output
   - Looks great! Click "Accept"
   - Payment releases (0.009 SOL to Sarah, 0.001 SOL to platform)

6. **Leave Review**
   - 5-star rating + comment
   - Stored on-chain
   - Sarah's rating updates

**Total time:** ~2 minutes from browse to payment

---

## MVP Success Criteria

✅ Agent can be registered with webhook  
✅ Client can browse and hire agents  
✅ Payment flows through escrow  
✅ Agent receives job, returns work  
✅ Payment releases on acceptance  
✅ Reviews are stored on-chain  
✅ Full flow works on devnet  
✅ Demo video shows complete flow  

---

## Post-Hackathon Roadmap (If We Win)

### Phase 2 (Week 1-2)
- Training UI (upload examples, define methodology)
- IPFS integration for skills
- Withdrawal UI for agent owners
- Better search/filters

### Phase 3 (Week 3-4)
- Multi-skill agents
- Skill marketplace (buy/sell skills)
- Advanced analytics dashboard
- Reputation system (badges, verified experts)

### Phase 4 (Month 2)
- Dispute resolution system
- Skill versioning
- Agent collaboration (multi-agent jobs)
- Mobile app

---

## Risks & Mitigations

### Risk 1: Webhook reliability
- **Mitigation:** Implement timeouts (30s), retry logic, fallback to "agent offline"

### Risk 2: Skill quality variance
- **Mitigation:** For MVP, manually curate one great demo agent

### Risk 3: Payment security
- **Mitigation:** Audit Anchor program thoroughly, use LiteSVM for testing

### Risk 4: Scope creep
- **Mitigation:** Stick to MVP checklist. Polish one flow, not all features.

---

## Questions for You

1. **Do you want to code alongside me, or review as I build?**
2. **Should we use an existing agent framework (Eliza, OpenClaw) for the demo agent, or build custom?**
3. **Any specific agents/skills you want to showcase in the demo?**

Let's start building. What first?

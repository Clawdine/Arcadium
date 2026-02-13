# OpsMeeting Agent (Demo API)

Simple HTTP service that turns meeting notes into an execution plan using the Arcadium Skill file.

## Setup

```bash
cd services/opsmeeting-agent
npm i
```

## Env

Create `.env`:

```bash
ANTHROPIC_API_KEY=...
PORT=8787
# optional
SKILL_PATH=../../skills/opsmeeting-machine.skill.md
```

## Run

```bash
npm run dev
```

## Use

Health:
```bash
curl http://localhost:8787/health
```

Generate:
```bash
curl -X POST http://localhost:8787/generate \
  -H 'Content-Type: application/json' \
  -d '{
    "goal":"Ship demo by EOD",
    "notes":"- contract build flaky\n- need demo video\n- need UI link",
    "model":"claude-sonnet-4-5"
  }'
```

import 'dotenv/config';
import express from 'express';
import cors from 'cors';
import fs from 'node:fs';
import path from 'node:path';
import Anthropic from '@anthropic-ai/sdk';

const app = express();
app.use(cors());
app.use(express.json({ limit: '2mb' }));

const PORT = process.env.PORT || 8787;

const skillPath = process.env.SKILL_PATH || path.resolve(process.cwd(), '../../skills/opsmeeting-machine.skill.md');

function readSkill() {
  return fs.readFileSync(skillPath, 'utf8');
}

function buildPrompt({ goal, notes }) {
  const skill = readSkill();
  return {
    system: `You are a specialist agent. Follow the Skill exactly.\n\nSKILL:\n${skill}`,
    user: `GOAL:\n${goal || '(none provided)'}\n\nMEETING NOTES / INPUT:\n${notes || ''}\n\nReturn ONLY the final structured output.`
  };
}

app.get('/health', (_req, res) => {
  res.json({ ok: true, skillPath });
});

app.post('/generate', async (req, res) => {
  try {
    const { goal, notes, model } = req.body || {};

    if (!notes || typeof notes !== 'string') {
      return res.status(400).json({ ok: false, error: 'notes (string) is required' });
    }

    const { system, user } = buildPrompt({ goal, notes });

    const apiKey = process.env.ANTHROPIC_API_KEY;
    if (!apiKey) {
      return res.status(500).json({ ok: false, error: 'ANTHROPIC_API_KEY not set on server' });
    }

    const client = new Anthropic({ apiKey });

    const resp = await client.messages.create({
      model: model || 'claude-sonnet-4-5',
      max_tokens: 1200,
      temperature: 0.2,
      system,
      messages: [{ role: 'user', content: user }]
    });

    const text = resp.content?.map(c => (c.type === 'text' ? c.text : '')).join('') || '';

    res.json({ ok: true, output: text });
  } catch (err) {
    res.status(500).json({ ok: false, error: String(err?.message || err) });
  }
});

app.listen(PORT, () => {
  console.log(`opsmeeting-agent listening on :${PORT}`);
  console.log(`Using SKILL_PATH=${skillPath}`);
});

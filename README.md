# WhisperFlow Alternative (That Doesn't Suck)

## NO API KEYS REQUIRED. EVERYTHING RUNS LOCALLY.

**Mac & Windows.** A Whisper Flow alternative. Hold a key (or click), talk, release, and your words become text. On your machine. No "cloud," no "AI platform," no "we're totally not reading your stuff."

---

## What Even Is This?

A menubar app that uses OpenAI's Whisper **locally** to transcribe your voice. Hold a shortcut (e.g. Right Option) or toggle with a combo, speak, then get text typed wherever your cursor is. Your audio and text stay on your computer. Wild concept, we know.

Built with **Tauri 2** (Rust + HTML/JS). Tiny binary, low memory, native performance. Works on **macOS (Apple Silicon & Intel)** and **Windows (x64)**.

---

## Why Would You Need Whisper Flow Now?

Reddit's been buzzing about Whisper Flow collecting screenshots and data. *Seriously.* That's your private data. Meetings, notes, rants—why would you want any of that shipped off to someone else's servers? We don't. So we built this instead: **faster, cheaper, and better.** It runs on your machine, uses your CPU, and we don't see a single byte.

---

## The Comparison Table You Didn't Ask For

| | **WhisperFlow Alternative (this thing)** | **Whisper Flow** |
|---|---|---|
| **Monthly fee** | We do not charge monthly. $0. Forever. | You know the drill. |
| **API keys** | None. Zero. Nada. Runs 100% offline. | Needs API keys or cloud access. |
| **Open source** | Yes. Read the code. Change it. | Nope. |
| **Your data** | We do not collect any of your data. It never leaves your machine. | Screenshots, transcripts, who knows what. Reddit said so. |
| **Runs** | Works locally. Your machine, your rules. | Their cloud, their rules. |
| **Speed** | Faster (local inference, no round-trip). | Slower (upload, wait, download). |
| **Platform** | Mac (Apple Silicon & Intel) & Windows. | Mac only. |
| **Vibe** | Faster. Cheaper. Better. | Slower. Pricier. And that privacy thing. |

So again: **why do you need Whisper Flow now?** This is literally the best option if you care about privacy and not paying every month.

---

## Releases

Find the build in the [Releases](../../releases) section. Download, run, done.

- **macOS**: `.dmg` for Apple Silicon (arm64) and Intel (x64)
- **Windows**: `.exe` installer (NSIS) and `.msi`

---

## Building from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (18+)
- [Rust](https://rustup.rs/) (stable)
- [Tauri CLI](https://tauri.app/): `npm install`

### Build
```bash
npm run tauri build
```

The built app will be in `src-tauri/target/release/bundle/`.

---

Contributions are welcome. **Even AI-generated contributions.** Just make sure whatever you send actually works. We'll review it, and merging might take a while—or it might not merge at all. No hard feelings; we're picky because we like things that run.

---

*Your voice. Your data. Your machine. No subscriptions, no API keys, no telemetry, no funny business.*

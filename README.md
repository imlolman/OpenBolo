# OpenBolo

**Open source WhisperFlow alternative.** No API keys required; everything runs locally.

**Mac & Windows.** Hold a key (or click), talk, release, and your words become text. On your machine. No cloud, no AI platform gatekeeping, no third party reading your audio.

---

## What Even Is This?

A menubar app that uses OpenAI's Whisper **locally** to transcribe your voice. Hold a shortcut (e.g. Right Option) or toggle with a combo, speak, then get text typed wherever your cursor is. Your audio and text stay on your computer.

Built with **Tauri 2** (Rust + HTML/JS). Tiny binary, low memory, native performance. Works on **macOS (Apple Silicon & Intel)** and **Windows (x64)**.

---

## Why OpenBolo vs Whisper Flow?

If you want local, open source dictation without sending voice or metadata to someone else's servers, OpenBolo is built for that.

---

## The Comparison Table You Didn't Ask For

| | **OpenBolo (this thing)** | **Whisper Flow** |
|---|---|---|
| **Monthly fee** | We do not charge monthly. $0. Forever. | You know the drill. |
| **API keys** | None. Zero. Nada. Runs 100% offline. | Needs API keys or cloud access. |
| **Open source** | Yes. Read the code. Change it. | Nope. |
| **Your data** | We do not collect any of your data. It never leaves your machine. | Cloud-dependent; review their policy. |
| **Runs** | Works locally. Your machine, your rules. | Their cloud, their rules. |
| **Speed** | Faster (local inference, no round-trip). | Slower (upload, wait, download). |
| **Platform** | Mac (Apple Silicon & Intel) & Windows. | Mac only. |
| **Vibe** | Faster. Cheaper. Better. | Subscription model; network bound. |

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

Contributions are welcome. **Even AI-generated contributions.** Just make sure whatever you send actually works. We'll review it, and merging might take a while, or it might not merge at all. No hard feelings; we're picky because we like things that run.

---

*Your voice. Your data. Your machine. No subscriptions, no API keys, no telemetry, no funny business.*

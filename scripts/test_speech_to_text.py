#!/usr/bin/env python3
"""One-off test: load Whisper model, record a few seconds, transcribe. Verifies model + pipeline."""
import sys
import time

sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent.parent))

import numpy as np
import sounddevice as sd

from wisperflow.config import SAMPLE_RATE
from wisperflow.transcriber import get_device, load_model, transcribe


def main():
    print("1) Device...")
    device = get_device()
    print(f"   -> {device}")

    print("2) Loading model (base.en)...")
    t0 = time.time()
    model = load_model(device)
    print(f"   -> {time.time() - t0:.1f}s")

    print("3) Recording 3s (speak now)...")
    duration_s = 3
    audio = sd.rec(
        int(duration_s * SAMPLE_RATE),
        samplerate=SAMPLE_RATE,
        channels=1,
        dtype="float32",
    )
    sd.wait()
    audio = audio.flatten()
    print(f"   -> {len(audio) / SAMPLE_RATE:.1f}s captured")

    print("4) Transcribing...")
    t0 = time.time()
    text = transcribe(model, audio, device)
    print(f"   -> {time.time() - t0:.1f}s")
    print(f"   -> result: {repr(text)}")

    print("\nOK: model load + record + transcribe pipeline works.")


if __name__ == "__main__":
    main()

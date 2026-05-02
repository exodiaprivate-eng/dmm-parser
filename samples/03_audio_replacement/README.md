# Sample 03 — Audio replacement

Swaps a single English voice clip with a custom WEM. Demonstrates the
WEM asset target and Crimson's localized voice path convention.

## Files

- `mod.field.json` — manifest with one WEM asset target
- `assets/voice/macduff_attack01.wem` — drop your replacement WEM here

## Building

1. Author the WEM in Wwise (free for non-commercial use). Export with
   the same format as the vanilla clip (`classify_wem` shows
   `WwiseVorbis` or `WaveformatExtensible` on the original).
2. Place at `assets/voice/macduff_attack01.wem`.
3. Validate + pack:

```sh
python -m dmm_parser.tools.validate 03_audio_replacement/mod.field.json --assets 03_audio_replacement/assets
python -m dmm_parser.tools.pack    03_audio_replacement/mod.field.json --assets 03_audio_replacement/assets
```

## What this teaches

- `asset_type: "wem"` for per-clip audio
- vpath under `0006/sound/windows/<lang>/...` for localized voice
- BNK soundbank's DIDX entry must agree with the new WEM size — if
  you're swapping clips wholesale, also include the parent BNK as a
  second asset target.

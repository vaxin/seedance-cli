---
name: seedream-cli
description: 'Use the seedream CLI to generate and edit images with Seedream models (4.0 / 5.0 / 5.0 Pro) on Volcengine Ark. Covers text-to-image, image-to-image, local editing, multi-image fusion (up to 10 refs), sequential image groups, size/format control, and config. Use when generating an image, editing or compositing images, rendering text into images, or building image pipelines from the terminal.'
license: MIT
user-invocable: true
user-invokable: true
tags: ["cli", "terminal", "image", "seedream", "ark"]
metadata: {"version": "0.2.0", "updated": "2026-08-20", "parent": "seedance-cli"}
---

# seedream-cli

Command-line interface for **Seedream** image generation on Volcengine Ark.

Binary name: `seedream`. Ships alongside `seedance` in the same repo/npm package. Written in Rust. Config stored at `~/.config/seedream/config.toml` (separate from seedance's). Uses the **synchronous** `/api/v3/images/generations` endpoint — no task polling, no task DB.

Shares the `ARK_API_KEY` environment variable with the seedance CLI.

---

## Quick Start

```bash
# 1. Configure API key (env var takes precedence)
export ARK_API_KEY=your-key

# 2. Text-to-image
seedream generate "A cozy ramen shop at midnight, neon reflections on wet asphalt"

# 3. Image editing / restyle
seedream generate "Change the background to a snowy mountain at dawn" -i photo.jpg

# 4. Multi-image fusion (up to 10 refs)
seedream generate "Blend these two characters into one scene" -i hero.png -i landscape.jpg -o fusion.jpg

# 5. Sequential image group (2-4 images)
seedream generate "The same tree across four seasons" --sequential 4 --size 2048x2048
```

---

## Commands

### `seedream generate`

The only generation command — the prompt plus reference images determines the mode (T2I / I2I / editing / fusion).

```
seedream generate <PROMPT> [OPTIONS]
```

**PROMPT** can be a literal string or `@file.txt` to read from a file.

#### Options

| Flag                 | Short | Default | Description                                             |
| -------------------- | ----- | ------- | ------------------------------------------------------- |
| `--model`            | `-m`  | `standard` | `standard`/`4.0`, `5.0`, `pro` — or a raw Ark model ID |
| `--image`            | `-i`  |         | Reference image (repeatable, max 10). URL, local path, or `asset://` |
| `--size`             |       |         | Output size: `1024x1024`, `2048x2048`, `1K`, `2K`, `4K`, `adaptive` |
| `--output`           | `-o`  |         | Output file path (default: `seedream-<timestamp>.<ext>` in cwd) |
| `--sequential`       |       |         | Generate a group of 2–4 related images (Seedream 4.0+)  |
| `--seed`             |       |         | Random seed for reproducibility                         |
| `--watermark`        |       | `false` | Add watermark                                           |
| `--response-format`  |       | `url`   | API response format: `url` or `b64_json`                |
| `--quiet`            | `-q`  | `false` | Only print output file paths (script-friendly)          |
| `--json`             |       | `false` | Print raw API response (also saves files)               |

### `seedream config`

```
seedream config show               # display all config
seedream config get <KEY>          # read one key
seedream config set <KEY> <VALUE>  # write one key
```

Keys: `api_key`, `base_url`, `model`, `resolution`, `ratio`, `duration`, `output_dir`.
Config file: `~/.config/seedream/config.toml`. Env var `ARK_API_KEY` overrides `api_key`.

---

## Models

| Alias          | Model ID                         | Notes                                             |
| -------------- | -------------------------------- | ------------------------------------------------- |
| `standard`/`4.0` | `doubao-seedream-4-0-250828`   | 4K output, 10-image fusion, sequential groups     |
| `5.0`          | `doubao-seedream-5-0-260128`     | 5.0 preview — 2K/4K, better composition          |
| `pro`          | `doubao-seedream-5-0-pro`        | 5.0 Pro — interactive editing, layer separation, 14+ language text rendering. ⚠️ ID unverified — check the Ark console, or pass a raw ID |

Any other string is passed through as a raw Ark model ID.

---

## Modes

### Text-to-Image (T2I)

```bash
seedream generate "A vintage travel poster of Mount Fuji, bold typography"
```

### Image-to-Image / Editing (I2I)

One reference image + an editing instruction. Works best with explicit, surgical instructions:

```bash
seedream generate "Remove the power lines from the sky; keep everything else identical" -i street.jpg
seedream generate "Turn the sweater from red to forest green, preserve fabric texture" -i portrait.png
```

### Multi-Image Fusion

2–10 reference images combined into one output:

```bash
seedream generate "Put the person from image 1 into the environment of image 2, golden hour lighting" \
  -i person.png -i environment.jpg
```

### Sequential Image Groups (4.0+)

```bash
seedream generate "A detective story: discovery, investigation, chase, resolution" --sequential 4
```

Generates 2–4 images from one prompt (the model writes per-image variation prompts). Output files are numbered `seedream-<ts>-00.jpg`, `-01.jpg`, ...

---

## Local File Handling

- Reference images given as local paths are inlined as base64 data URIs when ≤ **10MB**.
- Larger files are automatically uploaded to TOS (requires `TOS_ACCESS_KEY`, `TOS_SECRET_KEY`, `TOS_BUCKET` env vars).
- URLs (`https://...`) and `asset://` refs pass through untouched.

## Output

- Default filename: `seedream-YYYYMMDD-HHMMSS.<ext>` in the cwd (or next to the first reference image).
- The extension is sniffed from the image bytes — Seedream typically returns **JPEG**, so files are saved as `.jpg`.
- Use `-o path` to control the output path; with `--sequential`/multi-image output, files get an `-NN` index before the extension.
- `--quiet` prints only the saved paths (one per line) for scripting.

---

## Prompt Tips

- **In-image text**: quote exact strings in the prompt — `bold title "SEEDREAM", subtitle "EXPLORE THE WILD"`. 5.0 Pro renders 14+ languages.
- **Editing**: describe the *change* and add "keep everything else identical" to avoid unintended drift.
- **Fusion**: refer to references by ordinal — "the person from image 1", "the background of image 2".
- **Size**: `1K`/`2K`/`4K` are tier shortcuts; explicit `WIDTHxHEIGHT` for exact control; `adaptive` (4.0+) matches the reference image's aspect.

---

## Errors

| Error | Cause / Fix |
| --- | --- |
| `API key not configured` | Set `ARK_API_KEY` or run `seedream config set api_key <KEY>` |
| `max 10 reference images` | Fusion is capped at 10 references |
| `local image ... over the 10MB inline base64 limit` | Set `TOS_*` env vars for auto-upload, or pass a URL |
| `--sequential must be between 2 and 4` | Group size range is 2–4 |
| `--response-format must be url or b64_json` | Typo in the flag value |
| HTTP 429 | Rate limited — retry after a backoff |
| HTTP 404 model not found | The `pro` alias ID may differ on Ark — use the console's exact model ID |

---

## Related Skills

- `seedance-cli` — video generation CLI from the same repo (Seedance 2.0/2.5).

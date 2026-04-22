---

## name: seedance-cli

description: 'Use the seedance CLI to generate, track, download, and manage Seedance 2.0 AI videos from the terminal. Covers all subcommands (generate, status, download, list, config), flag reference, workflow patterns, CI/CD integration, and error handling. Use when generating a video, checking task status, downloading results, configuring defaults, or building automated pipelines with Seedance 2.0.'
license: MIT
user-invocable: true
user-invokable: true
tags: ["cli", "terminal", "automation", "pipeline", "seedance-20"]
metadata: {"version": "1.0.0", "updated": "2026-04-22", "parent": "seedance-20"}

# seedance-cli

Command-line interface for **Seedance 2.0** video generation on Volcengine Ark.

Binary name: `seedance`. Written in Rust. Config stored at `~/.config/seedance/config.toml`. Task history in local SQLite at `~/.config/seedance/tasks.db`.

---

## Quick Start

```bash
# 1. Configure API key (interactive)
seedance config init

# 2. Generate a video (text-to-video)
seedance generate "A cat sits on a windowsill watching rain fall outside"

# 3. Generate and wait for result
seedance generate "Sunset over mountain lake, slow drone pullback" --wait

# 4. Generate with full control
seedance generate "Product bottle on marble surface, slow orbit" \
  --model standard --duration 8 --ratio 16:9 --resolution 1080p --wait
```

---

## Installation

```bash
# From source
cargo install --path .

# Verify
seedance --version
```

---

## Environment


| Variable      | Purpose                                |
| ------------- | -------------------------------------- |
| `ARK_API_KEY` | API key (overrides config file if set) |


Priority: `ARK_API_KEY` env var > `config.toml` api_key.

---

## Commands

### `seedance generate` — Create a Video

The core command. Supports T2V, I2V, V2V, and R2V workflows.

```
seedance generate <PROMPT> [OPTIONS]
```

**PROMPT** can be a literal string or `@file.txt` to read from a file.

#### Generation Options


| Flag                  | Short | Default    | Description                                           |
| --------------------- | ----- | ---------- | ----------------------------------------------------- |
| `--model`             | `-m`  | `standard` | Model variant: `standard` or `fast`                   |
| `--duration`          | `-d`  | `5`        | Duration in seconds (4–15)                            |
| `--ratio`             | `-r`  | `16:9`     | Aspect ratio (`16:9` `9:16` `4:3` `3:4` `21:9` `1:1`) |
| `--resolution`        |       | `1080p`    | Resolution (`480p` `720p` `1080p` `2K`)               |
| `--seed`              |       |            | Random seed for reproducibility                       |
| `--watermark`         |       | `false`    | Add watermark                                         |
| `--audio-gen`         |       | `false`    | Enable native audio generation                        |
| `--return-last-frame` |       | `false`    | Return the last frame as an image                     |
| `--callback`          |       |            | Webhook callback URL                                  |


#### Material Inputs (I2V / V2V / R2V)


| Flag            | Short | Repeatable | Max | Description                         |
| --------------- | ----- | ---------- | --- | ----------------------------------- |
| `--image`       | `-i`  | Yes        | 9   | Image reference (URL or local path) |
| `--video`       | `-v`  | Yes        | 3   | Video reference (URL or local path) |
| `--audio`       | `-a`  | Yes        | 3   | Audio reference (URL or local path) |
| `--first-frame` |       | No         | 1   | First frame image                   |
| `--last-frame`  |       | No         | 1   | Last frame image                    |


Total files across all types: max 12 (Rule of 12).

Local files are automatically converted to base64 data URIs. Supported formats: JPG, PNG, WEBP (images), MP4, MOV (video), MP3 (audio).

#### Wait & Output Options


| Flag              | Short | Default | Description                                     |
| ----------------- | ----- | ------- | ----------------------------------------------- |
| `--wait`          | `-w`  | `false` | Wait for completion and auto-download           |
| `--output`        | `-o`  |         | Output file path (default: `seedance_<id>.mp4`) |
| `--timeout`       |       | `300`   | Max wait seconds (with `--wait`)                |
| `--poll-interval` |       | `10`    | Poll interval seconds                           |
| `--strict`        |       | `false` | Exit code 2 on timeout (for CI)                 |


#### Output Format Options


| Flag      | Short | Description                                       |
| --------- | ----- | ------------------------------------------------- |
| `--quiet` | `-q`  | Quiet mode: only print task_id or final file path |
| `--json`  |       | JSON output                                       |


---

### `seedance status` — Check Task Status

```
seedance status <TASK_ID> [OPTIONS]
```


| Flag        | Default | Description                      |
| ----------- | ------- | -------------------------------- |
| `--json`    | `false` | JSON output                      |
| `--wait`    | `false` | Poll until task completes        |
| `--timeout` | `300`   | Max wait seconds (with `--wait`) |


---

### `seedance download` — Download Completed Video

```
seedance download <TASK_ID> [OPTIONS]
```


| Flag       | Short | Description                                     |
| ---------- | ----- | ----------------------------------------------- |
| `--output` | `-o`  | Output file path (default: `seedance_<id>.mp4`) |


---

### `seedance list` — View Task History

```
seedance list [OPTIONS]
```


| Flag       | Short | Default | Description      |
| ---------- | ----- | ------- | ---------------- |
| `--limit`  | `-n`  | `20`    | Number of tasks  |
| `--status` |       |         | Filter by status |
| `--json`   |       | `false` | JSON output      |


---

### `seedance config` — Manage Configuration

```bash
seedance config init          # Interactive setup wizard
seedance config show          # Display all settings
seedance config get <KEY>     # Get a single value
seedance config set <KEY> <VALUE>  # Set a single value
```

Available keys: `api_key`, `base_url`, `model`, `resolution`, `ratio`, `duration`, `output_dir`.

---

## Workflow Patterns

### Text-to-Video (T2V)

```bash
seedance generate "A glass perfume bottle on white marble. Smooth 360-degree orbit camera. Soft studio key light." \
  --duration 8 --ratio 16:9 --wait
```

### Image-to-Video (I2V)

```bash
seedance generate "Character walks forward slowly, wind blows hair" \
  --first-frame character_pose.png --duration 5 --wait
```

### Reference-to-Video (R2V) with Multiple Images

```bash
seedance generate "Character A throws a punch at Character B" \
  --image charA.png --image charB.png --duration 8 --wait
```

### Video-to-Video (V2V) with Style Transfer

```bash
seedance generate "Match the camera movement and editing style from reference" \
  --video reference_clip.mp4 --image character.png --duration 10 --wait
```

### Audio-Driven Generation

```bash
seedance generate "Character speaks directly to camera, medium close-up, locked" \
  --audio dialogue.mp3 --image character.png --audio-gen --duration 8 --wait
```

### Music Beat-Sync

```bash
seedance generate "Scene images transition on musical downbeats, energetic pacing" \
  --image scene1.jpg --image scene2.jpg --image scene3.jpg \
  --audio music_track.mp3 --audio-gen --duration 10 --wait
```

### Prompt from File

```bash
# Write a complex prompt in a text file
echo '古风仙侠场景，白衣剑客凌空而起，剑光如虹划过夜空，远山云海翻涌。升格慢动作，仰拍视角。' > prompt.txt

seedance generate @prompt.txt --duration 10 --ratio 16:9 --wait
```

### Batch Generation with Seed Variation

```bash
for seed in 1000 1001 1002 1003 1004; do
  seedance generate "Sunset over ocean, golden light, slow pan right" \
    --seed $seed --duration 5 -q
done
```

### Fire-and-Forget + Later Download

```bash
# Submit task (returns immediately)
TASK_ID=$(seedance generate "A dramatic scene" -q)

# ... do other work ...

# Check status later
seedance status $TASK_ID

# Download when ready
seedance download $TASK_ID -o final_output.mp4
```

---

## CI / Script Integration

Use `--quiet`, `--json`, and `--strict` for automation:

```bash
# CI pipeline: generate, wait, fail on timeout
TASK_ID=$(seedance generate "Product ad scene" --wait --strict --timeout 600 -q)

# JSON output for parsing
seedance status $TASK_ID --json | jq '.status'

# List recent failures
seedance list --status failed --json | jq '.[].task_id'
```

---

## Task States


| Status      | Terminal | Description                   |
| ----------- | -------- | ----------------------------- |
| `submitted` | No       | Task received by server       |
| `queued`    | No       | Waiting in generation queue   |
| `running`   | No       | Video is being generated      |
| `succeeded` | Yes      | Generation complete           |
| `failed`    | Yes      | Generation failed             |
| `expired`   | Yes      | Task result expired on server |
| `cancelled` | Yes      | Task was cancelled            |


---

## Exit Codes


| Code | Meaning                                |
| ---- | -------------------------------------- |
| `0`  | Success                                |
| `1`  | Error (API failure, invalid args, etc) |
| `2`  | Timeout (only with `--strict`)         |


---

## Agent Gotchas

1. **Always `--wait` for one-shot workflows.** Without it, `generate` returns immediately with just a task_id. The video isn't ready yet.
2. **Use `-q` for scripting.** Quiet mode outputs only the task_id (on generate) or file path (on download), nothing else.
3. **Local files auto-upload.** Pass local paths to `--image`, `--video`, `--audio` — the CLI handles base64 encoding. No manual upload step needed.
4. `**@file.txt` for long prompts.** Chinese prompts with special characters are easier to manage in a file.
5. `**--strict` for CI.** Without it, timeout is a soft warning (exit 0). With it, timeout exits 2 so your pipeline catches it.
6. **Seed for reproducibility.** Same seed + same prompt + same model = same output. Use for A/B testing and iteration.
7. **Config cascade:** `ARK_API_KEY` env var overrides `config.toml`. Useful for CI where you set secrets via env.
8. **Rule of 12:** Total files (images + videos + audios + first_frame + last_frame) cannot exceed 12. CLI validates this before sending.
9. **Duration range is 4–15 seconds.** CLI rejects values outside this range.
10. **Audio must be MP3.** Other formats fail silently on the server side. Convert before passing to `--audio`.


---
name: seedance-pipeline
description: 'Build automated video generation pipelines with the seedance CLI, ComfyUI nodes, and post-processing chains covering upscale, frame interpolation, color grade, composite, and metadata cleanup. Use when building automated video pipelines, connecting Seedance to external tools, or finishing and delivering a generated video clip.'
license: MIT
user-invocable: true
user-invokable: true
tags: ["pipeline", "comfyui", "cli", "automation", "openclaw", "antigravity", "gemini-cli", "codex", "cursor", "windsurf", "opencode"]
metadata: {"version": "6.0.0", "updated": "2026-04-22", "openclaw": {"emoji": "🔗", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "parent": "seedance-20", "antigravity": {"emoji": "🔗", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "gemini-cli": {"emoji": "🔗", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "firebase": {"emoji": "🔗", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "author": "Emily (@iamemily2050)", "repository": "https://github.com/Emily2040/seedance-2.0"}
---

# seedance-pipeline

CLI-driven pipelines, ComfyUI integration, and post-processing for Seedance 2.0.

---

## CLI-First Pipeline

The `seedance` CLI replaces direct API calls. All generation, polling, and downloading are handled through the CLI binary. See [skill:seedance-cli] for full command reference.

### Single-Shot Pipeline (Generate → Wait → Download)

```bash
seedance generate "Glass perfume bottle on white marble. Slow 360° orbit. Soft studio light." \
  --model standard --duration 8 --ratio 16:9 --resolution 1080p \
  --wait --output product_shot.mp4
```

### Fire-and-Forget Pipeline

```bash
# Submit without waiting
TASK_ID=$(seedance generate "Sunset mountain lake, drone pullback" -q)

# Check status later
seedance status $TASK_ID

# Download when done
seedance download $TASK_ID -o sunset_lake.mp4
```

### Batch Pipeline (Multiple Variants)

```bash
for seed in 1000 1001 1002 1003 1004; do
  seedance generate "Urban street at night, neon reflections, handheld tracking" \
    --seed $seed --duration 8 -q
done

# Review all tasks
seedance list -n 5 --json
```

### I2V Pipeline (Image-to-Video)

```bash
seedance generate "Character walks forward, wind blows coat" \
  --first-frame hero_pose.png \
  --duration 8 --wait --output hero_walk.mp4
```

### Multi-Reference Pipeline (R2V)

```bash
seedance generate "Character A throws a punch, Character B blocks with crossed arms" \
  --image charA.png --image charB.png \
  --video fight_reference.mp4 \
  --duration 10 --wait --output fight_scene.mp4
```

### Audio-Driven Pipeline

```bash
seedance generate "Character speaks to camera, medium close-up, locked" \
  --image character.png --audio dialogue.mp3 \
  --audio-gen --duration 8 --wait --output dialogue_clip.mp4
```

### CI / Automation Pipeline

```bash
# Strict mode: exit 2 on timeout (for CI failure detection)
seedance generate "Product ad" \
  --wait --strict --timeout 600 --json \
  --output artifact.mp4

# Parse JSON output
seedance status $TASK_ID --json | jq '.status'
seedance list --status failed --json | jq '.[].task_id'
```

---

## Platform Access


| Surface  | Endpoint / App                 | Notes                 |
| -------- | ------------------------------ | --------------------- |
| **CLI**  | `seedance` binary              | Full pipeline control |
| Web      | jimeng.jianying.com (Dreamina) | 4–15 s, up to 1080p   |
| Mobile   | CapCut / Jianying · Xiaoyunque | 5–10 s                |
| Consumer | Doubao app                     | Standard web limits   |


## File Budget ("Rule of 12")


| Type            | Max count | Max size each             | Format           |
| --------------- | --------- | ------------------------- | ---------------- |
| Image           | 9         | 30 MB                     | JPG · PNG · WEBP |
| Video           | 3         | **combined ≤ 15 s total** | MP4 · MOV        |
| Audio           | 3         | total ≤ 15 s              | MP3              |
| **Total files** | **12**    | —                         | —                |


The CLI validates the Rule of 12 before sending. Local files are automatically encoded to base64.

---

## ComfyUI Node Workflow

```
[Load Image / Load Video] → [Seedance2 Sampler]
      ↓                           ↓
[CLIP Text Encode]          [Prompt Compiler]
      └────────────────────────→ ↓
                         [Video Output Node]
                                 ↓
                      [Frame Interpolation]
                                 ↓
                         [Upscale Node]
                                 ↓
                       [Color Grade Node]
                                 ↓
                        [Export / Mux Audio]
```

Key node parameters: `duration`, `aspect_ratio`, `resolution`, `seed`, `motion_strength`.

---

## Post-Processing Chain

### 1 · Upscale

- Tool: Topaz Video AI · Real-ESRGAN · ffmpeg `scale=iw*2:ih*2`
- Target: 720p → 1080p (standard) · 1080p → 2K (premium)

### 2 · Frame Interpolation

- Tool: RIFE v4.x · DAIN
- Standard: 24 fps → 60 fps (smooth motion)
- Fight / fast action: 24 fps → 120 fps

### 3 · Color Grade

- Tools: DaVinci Resolve · FFmpeg LUT
- Workflow: normalize exposure → apply LUT → mask-lift shadows → finalize.
- LUT slots: Rec.709 (web) · Log-C (archive).

### 4 · Audio Mux

- Merge generated stereo audio with video: `ffmpeg -i video.mp4 -i audio.mp3 -c:v copy -c:a aac -shortest out.mp4`

### 5 · Metadata Clean

- Strip generation metadata before distribution: `exiftool -all= output.mp4`
- Rename: `{project}_{shot}_{take}_{date}.mp4`

### 6 · Composite (optional)

- Layer generated clips in After Effects / DaVinci Fusion.
- Match color temperature across cuts before export.

---

## End-to-End Shell Pipeline Example

```bash
#!/bin/bash
set -euo pipefail

PROMPT="Glass tower, 60 floors. Drone approach from south-east. Golden hour."
OUTPUT_RAW="raw_tower.mp4"
OUTPUT_FINAL="tower_final.mp4"

# Step 1: Generate with CLI
seedance generate "$PROMPT" \
  --model standard --duration 10 --ratio 16:9 --resolution 1080p \
  --wait --strict --timeout 600 --output "$OUTPUT_RAW"

# Step 2: Upscale
ffmpeg -i "$OUTPUT_RAW" -vf "scale=iw*2:ih*2" -c:v libx264 -crf 18 upscaled.mp4

# Step 3: Frame interpolation (RIFE)
# rife -i upscaled.mp4 -o interpolated.mp4 -n 60

# Step 4: Color grade
# ffmpeg -i interpolated.mp4 -vf lut3d=grade.cube -c:v libx264 graded.mp4

# Step 5: Metadata clean
exiftool -all= -overwrite_original "$OUTPUT_FINAL"

echo "Done: $OUTPUT_FINAL"
```

---

## Output Specs


| Use case      | Resolution | FPS | Container  | Audio           |
| ------------- | ---------- | --- | ---------- | --------------- |
| Web / social  | 1080p      | 30  | MP4 H.264  | AAC 192k stereo |
| Film festival | 2K         | 24  | MOV ProRes | PCM 48kHz       |
| Archive       | 2K         | 24  | MKV H.265  | FLAC stereo     |


---

## Routing

For CLI command reference → [skill:seedance-cli]
For prompt issues → [skill:seedance-prompt]
For camera/storyboard → [skill:seedance-camera]
For QA / errors → [skill:seedance-troubleshoot]

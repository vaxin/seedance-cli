---
name: seedance-recipes
description: 'Apply genre recipe templates to Seedance 2.0 — product ads, fight scenes, brand films, mood pieces, dialogue clips, one-take journeys, music videos, novel adaptations, architecture walkthroughs, and action transfers. Use when you need a ready-made prompt structure for a known genre or format.'
license: MIT
user-invocable: true
user-invokable: true
tags: ["recipes", "templates", "genre", "openclaw", "antigravity", "gemini-cli", "codex", "cursor"]
metadata: {"version": "6.0.0", "updated": "2026-04-22", "openclaw": {"emoji": "📖", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "parent": "seedance-20", "antigravity": {"emoji": "📖", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "gemini-cli": {"emoji": "📖", "homepage": "https://github.com/Emily2040/seedance-2.0"}, "author": "Emily (@iamemily2050)", "repository": "https://github.com/Emily2040/seedance-2.0"}
---

# seedance-recipes · Genre Templates (v5.0)

This skill provides production-ready prompt templates for the seven most common Seedance 2.0 content categories. For a full breakdown of each genre, see [ref:genre-guides].

---

## 1. Product / E-commerce

**Goal:** Clean, focused, high-fidelity product showcases.

```
// 360° Orbit Template
A [product] on a [surface]. Smooth 360-degree orbit camera. Soft studio key light, rim highlight. Macro depth-of-field. 4K pristine detail. No text.
```

**Example:**

```bash
seedance generate "A glass perfume bottle on a white marble surface. Smooth 360-degree orbit camera. Soft studio key light, rim highlight. Macro depth-of-field. No text." \
  --duration 8 --ratio 16:9 --wait
```

---

## 2. Lifestyle / Social Media

**Goal:** Relatable, authentic-feeling moments for platforms like TikTok and Instagram.

```
// Morning Routine Template
A [person] [action] in a [location]. Handheld phone perspective, slight sway. They [emotion]. [Aesthetic].
```

**Example:**

```bash
seedance generate "A young woman makes coffee in a sunlit kitchen. Handheld phone perspective, slight sway. She smiles as she sips the coffee. Warm, cozy aesthetic." \
  --duration 5 --ratio 9:16 --wait
```

---

## 3. Drama / Narrative

**Goal:** Cinematic storytelling with consistent characters and emotional weight.

```
// Suspense Corridor Template
@Image1 character reference. A [character] cautiously [action] down a [scene]. They [action]. The camera follows closely behind them, handheld. The only sound is [sound].
```

**Example:**

```bash
seedance generate "Character reference @Image1. A man cautiously walks down a dark, narrow corridor. He holds a flashlight, its beam cutting through the darkness. The camera follows closely behind him, handheld. The only sound is his footsteps and nervous breathing." \
  --image character.png --duration 10 --audio-gen --wait
```

---

## 4. Music Video

**Goal:** Visually dynamic clips that sync with an audio track.

```
// Beat Sync Template
@Image1 artist, @Audio1 track. The artist performs in a [location]. Quick cuts sync to the drum beat at [BPM] BPM. [Camera moves] on the chorus. [Style].
```

**Example:**

```bash
seedance generate "Artist @Image1 performs with @Audio1 track. Neon-lit warehouse. Quick cuts sync to the drum beat at 120 BPM. Whip pans and snap zooms on the chorus." \
  --image artist.png --audio track.mp3 --audio-gen --duration 10 --wait
```

---

## 5. Landscape / Travel

**Goal:** Sweeping, beautiful shots of natural and urban environments.

```
// Drone Aerial Template
Sweeping aerial drone shot over [location] at [time of day]. [Weather condition]. The camera slowly [camera move] to reveal the vast scale. [Style].
```

**Example:**

```bash
seedance generate "Sweeping aerial drone shot over the Scottish Highlands at dawn. Mist hangs low in the valleys. The camera slowly pulls back to reveal the vast scale of the mountains." \
  --duration 10 --ratio 16:9 --resolution 1080p --wait
```

---

## 6. Commercial / Brand

**Goal:** Polished, high-end visuals that evoke a specific brand feeling.

```
// Car Ad Template
A sleek [color] [vehicle type] drives on a [road type] at [time of day]. The camera tracks alongside the car, [camera position]. Lens flare from the sun. The car looks [brand feeling].
```

**Example:**

```bash
seedance generate "A sleek black sports car drives on a winding mountain road at sunset. The camera tracks alongside the car, low to the ground. Lens flare from the sun." \
  --duration 8 --ratio 16:9 --wait
```

---

## 7. Anime / Artistic

**Goal:** Achieve specific, non-photorealistic visual styles.

```
// Cel-Shaded Action Template
An anime [character type] with [features], in the style of [studio/artist]. They [action]. Dynamic [camera angle]. Bold outlines, flat color fills, high-contrast shading. [Energy].
```

**Example:**

```bash
seedance generate "An anime warrior with white hair. He draws a glowing blue sword. Dynamic low-angle shot. Bold outlines, flat color fills, high-contrast shading. Explosive energy." \
  --duration 8 --ratio 16:9 --wait
```

---

*Maintained by [Emily (@iamemily2050)](https://github.com/Emily2040)*

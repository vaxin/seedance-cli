# seedream

CLI for **Seedream** image generation on Volcengine Ark — text-to-image, image-to-image editing, and multi-image fusion (up to 10 references), powered by Seedream 4.0 / 5.0.

Same binary as the [`seedance`](https://www.npmjs.com/package/seedance) video CLI, published as a standalone package for `npm install -g seedream`.

## Install

```bash
npm install -g seedream
```

## Quick start

```bash
# Configure (env var takes precedence over config file)
export ARK_API_KEY=your-key

# Text-to-image
seedream generate "A cozy ramen shop at midnight, neon reflections on wet asphalt"

# Image editing
seedream generate "Change the background to a snowy mountain at dawn" -i photo.jpg

# Multi-image fusion (up to 10 references)
seedream generate "Put the person from image 1 into the environment of image 2" \
  -i person.png -i environment.jpg -o fusion.jpg

# Sequential image group (2-4 images)
seedream generate "The same tree across four seasons" --sequential 4 --size 2048x2048
```

## Options

| Flag | Description |
| --- | --- |
| `--model` / `-m` | `standard`/`4.0`, `5.0`, `pro` — or a raw Ark model ID |
| `--image` / `-i` | Reference image (repeatable, max 10): URL or local path |
| `--size` | `1024x1024`, `2K`, `4K`, `adaptive` |
| `--output` / `-o` | Output file path |
| `--sequential` | Generate a group of 2–4 related images |
| `--seed` | Random seed |
| `--watermark` | Add watermark |
| `--quiet` / `-q` | Print only output file paths |
| `--json` | Print the raw API response |

Full documentation: [github.com/vaxin/seedance-cli](https://github.com/vaxin/seedance-cli)

## License

MIT

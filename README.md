# seedance

Seedance 2.0 AI 视频生成命令行工具，基于火山引擎 Ark 平台。

支持 Text-to-Video、Image-to-Video、Video-to-Video、Reference-to-Video 四种生成模式。

## 安装

```bash
npm install -g seedance
```

为 AI Agent（Claude Code / Cursor / Codex 等）安装 Skill：

```bash
npx skills add vaxin/seedance-cli
```

或从源码构建：

```bash
cargo install --path .
```

安装后验证：

```bash
seedance --version
```

## 快速开始

```bash
# 1. 配置 API Key（交互式）
seedance config init

# 2. 文生视频
seedance generate "夕阳下的湖面，金色光芒铺满水面，镜头缓缓后拉" --wait

# 3. 图生视频
seedance generate "角色缓步走来，风吹动发丝" \
  --first-frame character.png --wait

# 4. 完整参数控制
seedance generate "产品瓶身置于大理石台面，缓慢环绕拍摄" \
  --model standard --duration 8 --ratio 16:9 --resolution 1080p --wait
```

## 命令一览

### `seedance generate` — 创建视频

```
seedance generate <PROMPT> [OPTIONS]
```

`PROMPT` 可以是字符串，也可以用 `@file.txt` 从文件读取。

**生成参数**

| 参数 | 短写 | 默认值 | 说明 |
|---|---|---|---|
| `--model` | `-m` | `standard` | 模型：`standard` / `fast` |
| `--duration` | `-d` | `5` | 时长 4–15 秒 |
| `--ratio` | `-r` | `16:9` | 画面比例：`16:9` `9:16` `4:3` `3:4` `21:9` `1:1` |
| `--resolution` | | `1080p` | 分辨率：`480p` `720p` `1080p` `2K` |
| `--seed` | | | 随机种子，用于复现 |
| `--watermark` | | `false` | 添加水印 |
| `--audio-gen` | | `false` | 启用原生音频生成 |
| `--return-last-frame` | | `false` | 返回最后一帧图片 |
| `--callback` | | | Webhook 回调地址 |

**素材输入（I2V / V2V / R2V）**

| 参数 | 短写 | 可重复 | 上限 | 说明 |
|---|---|---|---|---|
| `--image` | `-i` | 是 | 9 | 图片参考（URL 或本地路径） |
| `--video` | `-v` | 是 | 3 | 视频参考 |
| `--audio` | `-a` | 是 | 3 | 音频参考 |
| `--first-frame` | | 否 | 1 | 首帧图片 |
| `--last-frame` | | 否 | 1 | 尾帧图片 |

所有素材总数不超过 12 个（Rule of 12）。本地文件自动 base64 编码上传。

**等待与输出**

| 参数 | 短写 | 默认值 | 说明 |
|---|---|---|---|
| `--wait` | `-w` | `false` | 等待完成并自动下载 |
| `--output` | `-o` | | 输出路径（默认 `seedance_<id>.mp4`） |
| `--timeout` | | `300` | 最大等待秒数 |
| `--poll-interval` | | `10` | 轮询间隔秒数 |
| `--strict` | | `false` | 超时返回 exit code 2（CI 用） |
| `--quiet` | `-q` | | 静默模式：仅输出 task_id 或文件路径 |
| `--json` | | | JSON 格式输出 |

### `seedance status` — 查询任务状态

```bash
seedance status <TASK_ID> [--json] [--wait] [--timeout 300]
```

### `seedance download` — 下载视频

```bash
seedance download <TASK_ID> [-o output.mp4]
```

### `seedance list` — 查看任务历史

```bash
seedance list [-n 20] [--status failed] [--json]
```

### `seedance config` — 管理配置

```bash
seedance config init              # 交互式配置向导
seedance config show              # 显示所有配置
seedance config get <KEY>         # 获取单项
seedance config set <KEY> <VALUE> # 设置单项
```

可配置项：`api_key`、`base_url`、`model`、`resolution`、`ratio`、`duration`、`output_dir`。

## 环境变量

| 变量 | 说明 |
|---|---|
| `ARK_API_KEY` | API Key，优先级高于 config.toml |

## 使用示例

**R2V：多角色参考生成**

```bash
seedance generate "角色A向角色B挥拳出击" \
  --image charA.png --image charB.png --duration 8 --wait
```

**V2V：风格迁移**

```bash
seedance generate "匹配参考视频的运镜和剪辑风格" \
  --video reference.mp4 --image character.png --duration 10 --wait
```

**音频驱动 + 口型同步**

```bash
seedance generate "角色面对镜头讲话，中近景，锁定机位" \
  --audio dialogue.mp3 --image character.png --audio-gen --duration 8 --wait
```

**从文件读取长提示词**

```bash
echo '古风仙侠场景，白衣剑客凌空而起，剑光如虹划过夜空，远山云海翻涌。升格慢动作，仰拍视角。' > prompt.txt
seedance generate @prompt.txt --duration 10 --ratio 16:9 --wait
```

**批量生成（种子变体）**

```bash
for seed in 1000 1001 1002 1003 1004; do
  seedance generate "海边日落，金色光芒，镜头缓缓右移" --seed $seed -d 5 -q
done
```

**异步提交 + 稍后下载**

```bash
TASK_ID=$(seedance generate "一段戏剧性场景" -q)
# ... 做其他事情 ...
seedance status $TASK_ID
seedance download $TASK_ID -o final.mp4
```

**CI/CD 集成**

```bash
TASK_ID=$(seedance generate "产品广告场景" --wait --strict --timeout 600 -q)
seedance status $TASK_ID --json | jq '.status'
seedance list --status failed --json | jq '.[].task_id'
```

## 任务状态

| 状态 | 终态 | 说明 |
|---|---|---|
| `submitted` | 否 | 已提交 |
| `queued` | 否 | 排队中 |
| `running` | 否 | 生成中 |
| `succeeded` | 是 | 生成完成 |
| `failed` | 是 | 生成失败 |
| `expired` | 是 | 结果已过期 |
| `cancelled` | 是 | 已取消 |

## 退出码

| 退出码 | 含义 |
|---|---|
| `0` | 成功 |
| `1` | 错误（API 失败、参数错误等） |
| `2` | 超时（仅 `--strict` 模式） |

## 项目结构

```
seedance-cli/
├── src/
│   ├── main.rs            # 入口 + 命令路由
│   ├── cli/               # 各子命令实现
│   │   ├── generate.rs
│   │   ├── status.rs
│   │   ├── download.rs
│   │   ├── list.rs
│   │   └── config.rs
│   ├── client/            # Ark API 客户端
│   ├── config/            # 配置文件管理
│   ├── core/              # 核心逻辑（上传 / 轮询 / 下载）
│   ├── store/             # SQLite 任务历史
│   └── ui/                # 终端输出 & 进度条
├── scripts/
│   ├── install.js         # npm postinstall（下载二进制）
│   └── run.js             # npm bin shim
├── .github/workflows/
│   └── release.yml        # CI: 多平台编译 + npm 发布
├── Cargo.toml
└── package.json
```

## 支持平台

| 平台 | 架构 |
|---|---|
| macOS | x86_64, aarch64 (Apple Silicon) |
| Linux | x86_64, aarch64 |
| Windows | x86_64, aarch64 |

## 发版

```bash
# 1. 同步 package.json 和 Cargo.toml 版本号
# 2. 创建并推送 tag
bash scripts/tag-release.sh
# 3. GitHub Actions 自动编译所有平台 + 发布到 npm
```

## License

MIT

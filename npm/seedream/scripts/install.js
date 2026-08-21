// Installer for the standalone `seedream` npm package.
// Downloads the shared release archive (which bundles both `seedance` and
// `seedream` since v0.2.0) and extracts only the `seedream` binary.
const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const VERSION = require("../package.json").version;
const REPO = "vaxin/seedance-cli";
// The archive is named after the repo's primary package (seedance).
const ARCHIVE_PREFIX = "seedance";
const NAME = "seedream";

// The seedream binary only ships in v0.2.0+ archives.
const MIN_VERSION = "0.2.0";

const PLATFORM_MAP = {
  darwin: "apple-darwin",
  linux: "unknown-linux-gnu",
  win32: "pc-windows-msvc",
};

const ARCH_MAP = {
  x64: "x86_64",
  arm64: "aarch64",
};

function cmpVersion(a, b) {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    if (pa[i] !== pb[i]) return (pa[i] || 0) - (pb[i] || 0);
  }
  return 0;
}

if (cmpVersion(VERSION, MIN_VERSION) < 0) {
  console.error(
    `seedream requires release >= v${MIN_VERSION}; package version is ${VERSION}`
  );
  process.exit(1);
}

const platform = PLATFORM_MAP[process.platform];
const arch = ARCH_MAP[process.arch];

if (!platform || !arch) {
  console.error(
    `Unsupported platform: ${process.platform}-${process.arch}`
  );
  process.exit(1);
}

const isWindows = process.platform === "win32";
const target = `${arch}-${platform}`;
const ext = isWindows ? ".zip" : ".tar.gz";
const archiveName = `${ARCHIVE_PREFIX}-v${VERSION}-${target}${ext}`;
const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;
const binDir = path.join(__dirname, "..", "bin");
const binaryName = NAME + (isWindows ? ".exe" : "");
const dest = path.join(binDir, binaryName);

if (fs.existsSync(dest)) {
  console.log(`${NAME} v${VERSION} already installed`);
  process.exit(0);
}

fs.mkdirSync(binDir, { recursive: true });

function download(url, destPath) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith("https") ? https : require("http");
    client
      .get(url, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          return download(res.headers.location, destPath).then(
            resolve,
            reject
          );
        }
        if (res.statusCode !== 200) {
          return reject(
            new Error(`Download failed with status ${res.statusCode}: ${url}`)
          );
        }
        const file = fs.createWriteStream(destPath);
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
        file.on("error", reject);
      })
      .on("error", reject);
  });
}

async function install() {
  const tmpDir = fs.mkdtempSync(path.join(require("os").tmpdir(), "seedream-"));
  try {
    const archivePath = path.join(tmpDir, archiveName);

    console.log(`Downloading ${NAME} v${VERSION} for ${target}...`);
    await download(url, archivePath);

    if (isWindows) {
      execSync(
        `powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}'"`,
        { stdio: "ignore" }
      );
    } else {
      execSync(`tar -xzf "${archivePath}" -C "${tmpDir}"`, {
        stdio: "ignore",
      });
    }

    const extracted = path.join(tmpDir, binaryName);
    if (fs.existsSync(extracted)) {
      fs.copyFileSync(extracted, dest);
    } else {
      const files = fs.readdirSync(tmpDir);
      throw new Error(
        `seedream binary not found in release archive (contents: ${files.join(", ")}). ` +
          `seedream ships in v${MIN_VERSION}+ archives — make sure the GitHub release exists.`
      );
    }

    fs.chmodSync(dest, 0o755);
    console.log(`${NAME} v${VERSION} installed successfully`);
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

install().catch((err) => {
  console.error(`Failed to install ${NAME}:`, err.message);
  console.error(
    `You can install manually from: https://github.com/${REPO}/releases`
  );
  process.exit(1);
});

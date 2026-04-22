const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");
const os = require("os");

const VERSION = require("../package.json").version;
const REPO = "vaxin/seedance-cli";
const NAME = "seedance";

const PLATFORM_MAP = {
  darwin: "apple-darwin",
  linux: "unknown-linux-gnu",
  win32: "pc-windows-msvc",
};

const ARCH_MAP = {
  x64: "x86_64",
  arm64: "aarch64",
};

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
const archiveName = `${NAME}-v${VERSION}-${target}${ext}`;
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
        file.on("finish", () => {
          file.close();
          resolve();
        });
      })
      .on("error", reject);
  });
}

async function install() {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), `${NAME}-`));
  const archivePath = path.join(tmpDir, archiveName);

  try {
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

    const extractedBinary = path.join(tmpDir, binaryName);

    if (!fs.existsSync(extractedBinary)) {
      const files = fs.readdirSync(tmpDir);
      const found = files.find((f) => f === binaryName || f.startsWith(NAME));
      if (found) {
        fs.copyFileSync(path.join(tmpDir, found), dest);
      } else {
        const subDirs = files.filter((f) =>
          fs.statSync(path.join(tmpDir, f)).isDirectory()
        );
        let copied = false;
        for (const dir of subDirs) {
          const nested = path.join(tmpDir, dir, binaryName);
          if (fs.existsSync(nested)) {
            fs.copyFileSync(nested, dest);
            copied = true;
            break;
          }
        }
        if (!copied) {
          throw new Error(
            `Binary not found in archive. Contents: ${files.join(", ")}`
          );
        }
      }
    } else {
      fs.copyFileSync(extractedBinary, dest);
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

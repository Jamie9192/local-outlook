#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
if ! xcrun --show-sdk-path >/dev/null 2>&1; then
  xcode-select --install
  echo 'Finish installing Apple Command Line Tools, then run this script again.'
  exit 1
fi
if [ ! -x "$HOME/.cargo/bin/rustup" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
export PATH="$HOME/.cargo/bin:$PATH"
if ! command -v node >/dev/null 2>&1; then
  if command -v brew >/dev/null 2>&1; then brew install node@24; export PATH="$(brew --prefix node@24)/bin:$PATH";
  else echo 'Install Node.js 24 LTS, then run this script again.'; exit 1; fi
fi
node -e 'const [major,minor]=process.versions.node.split(".").map(Number); if (major < 22 || (major === 22 && minor < 12)) { console.error("Node.js 22.12 or newer is required; Node 24 LTS is recommended."); process.exit(1) }'
rustup component add rustfmt clippy
rustup target add aarch64-apple-darwin x86_64-apple-darwin
npm ci
cargo fetch --manifest-path src-tauri/Cargo.toml --locked
echo 'Ready. Run npm run desktop -- dev to open Local Outlook.'

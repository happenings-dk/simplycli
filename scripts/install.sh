#!/usr/bin/env bash
set -euo pipefail

REPO_URL="${SIMPLYCLI_REPO_URL:-https://github.com/happenings-dk/simplycli.git}"
RAW_BASE="${SIMPLYCLI_RAW_BASE:-https://raw.githubusercontent.com/happenings-dk/simplycli/main}"
BIN_NAME="simply"
SKILL_NAME="simply-cli"
INSTALL_SKILLS=1
INSTALL_BINARY=1
FORCE=1

usage() {
  cat <<'EOF'
Install the Simply CLI and optional Claude/Codex skills.

Usage:
  install.sh [options]

Options:
  --no-skills       Install only the `simply` binary.
  --skills-only     Install only the Claude/Codex skill files.
  --no-force        Do not pass --force to cargo install.
  --repo-url URL    Git repository used for cargo install.
  --raw-base URL    Raw file base URL used for skill installation.
  -h, --help        Show this help.

Environment:
  SIMPLYCLI_REPO_URL   Override the Git repository URL.
  SIMPLYCLI_RAW_BASE   Override the raw GitHub content base URL.
EOF
}

log() {
  printf '%s\n' "==> $*"
}

die() {
  printf '%s\n' "error: $*" >&2
  exit 1
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --no-skills)
      INSTALL_SKILLS=0
      ;;
    --skills-only)
      INSTALL_BINARY=0
      INSTALL_SKILLS=1
      ;;
    --no-force)
      FORCE=0
      ;;
    --repo-url)
      [ "$#" -ge 2 ] || die "--repo-url requires a value"
      REPO_URL="$2"
      shift
      ;;
    --raw-base)
      [ "$#" -ge 2 ] || die "--raw-base requires a value"
      RAW_BASE="$2"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
  shift
done

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

download() {
  local url="$1"
  local dest="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
  else
    die "curl or wget is required to install skills"
  fi
}

install_binary() {
  need_cmd cargo

  local args=(install --git "$REPO_URL" --bin "$BIN_NAME" --locked)
  if [ "$FORCE" -eq 1 ]; then
    args+=(--force)
  fi

  log "Installing $BIN_NAME from $REPO_URL"
  cargo "${args[@]}"

  if command -v "$BIN_NAME" >/dev/null 2>&1; then
    log "Installed $(command -v "$BIN_NAME")"
  else
    log "Installed $BIN_NAME, but it is not on PATH. Cargo usually installs to ~/.cargo/bin."
  fi
}

install_skill_file() {
  local root="$1"
  local label="$2"
  local dest_dir="$root/$SKILL_NAME"
  local dest="$dest_dir/SKILL.md"

  mkdir -p "$dest_dir"
  download "$RAW_BASE/skills/$SKILL_NAME/SKILL.md" "$dest"
  log "Installed $label skill: $dest"
}

install_skills() {
  log "Installing Claude/Codex skills"
  install_skill_file "${HOME:?HOME is not set}/.claude/skills" "Claude Code"
  install_skill_file "${HOME:?HOME is not set}/.codex/skills" "Codex"
}

if [ "$INSTALL_BINARY" -eq 1 ]; then
  install_binary
fi

if [ "$INSTALL_SKILLS" -eq 1 ]; then
  install_skills
fi

log "Done"

#!/usr/bin/env bash
# fix_config_imports.sh
# Usage: ./fix_config_imports.sh [repo-root]
# Example: ./fix_config_imports.sh .
set -euo pipefail

ROOT="${1:-.}"
PID="$$"

# List of files the user provided
files=(
  "src/commands/auth.rs"
  "src/commands/clone.rs"
  "src/commands/config.rs"
  "src/commands/delete.rs"
  "src/commands/fork.rs"
  "src/commands/info.rs"
  "src/commands/list.rs"
  "src/commands/nodes.rs"
  "src/commands/pin.rs"
  "src/commands/popular.rs"
  "src/commands/pull.rs"
  "src/commands/push.rs"
  "src/commands/search.rs"
  "src/commands/star.rs"
  "src/commands/stats.rs"
  "src/commands/tag.rs"
  "src/commands/tags.rs"
  "src/commands/trending.rs"
  "src/main.rs"
)

# sanity: ensure we operate relative to root
cd "$ROOT"

did_any=0

for f in "${files[@]}"; do
  if [ ! -f "$f" ]; then
    printf "SKIP: %s (not found)\n" "$f"
    continue
  fi

  bak="${f}.bak.${PID}"
  cp -- "$f" "$bak"

  # Perform ordered replacements using perl for robust regex
  perl -0777 -pe '
    # 1) exact use stmt: use crate::config::Config; -> use crate::config::AppConfig;
    s{\buse\s+crate::config::Config\s*;}{"use crate::config::AppConfig;"}g;

    # 2) config::Config -> config::AppConfig (covers use lists like use crate::{api, config::Config, git};)
    s{\bconfig::Config\b}{config::AppConfig}g;

    # 3) config::Config::load -> config::AppConfig::load (just in case)
    s{\bconfig::Config::load\b}{config::AppConfig::load}g;

    # 4) Config::load(...) -> AppConfig::load(...)
    s{\bConfig::load\b}{AppConfig::load}g;

    # 5) Config::config_path -> AppConfig::config_path
    s{\bConfig::config_path\b}{AppConfig::config_path}g;

    # 6) Generic type/name replacements (word-boundary) last to avoid partial double-replacement earlier
    s{\bConfig\b}{AppConfig}g;
  ' -i -- "$f"

  # If file changed, show unified diff between backup and new file
  if ! cmp -s -- "$f" "$bak"; then
    did_any=1
    printf "\n=== CHANGED: %s ===\n" "$f"
    if command -v diff >/dev/null 2>&1; then
      diff -u -- "$bak" "$f" || true
    else
      printf "(diff tool not available to show changes)\n"
    fi
  else
    printf "NO-CHANGE: %s\n" "$f"
    rm -f -- "$bak"
  fi
done

if [ "$did_any" -eq 1 ]; then
  printf "\nDone. Backups of modified files are saved with suffix .bak.%s\n" "$PID"
  printf "If you like, run 'git add -A && git commit -m \"rename Config -> AppConfig imports and types\"'\n"
else
  printf "\nNo files were modified.\n"
fi

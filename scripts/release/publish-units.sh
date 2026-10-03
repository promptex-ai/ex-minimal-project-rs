#!/usr/bin/env bash
# Publish released package units to crates.io, at every stage of a train. publish.yml calls it with
# CARGO_REGISTRY_TOKEN set to the short-lived token rust-lang/crates-io-auth-action exchanged for the
# job's OIDC token; release-please.yml dispatches publish.yml with the released paths and the train's
# stage.
#
# The registry follows from the file the unit carries (unit_registry). Every unit of this repo is a
# crate (Cargo.toml), and publish.yml installs only the Rust toolchain, so a unit with any other file
# stops the run. The version is SemVer as is; cargo skips pre-releases unless the requirement names
# one. Each crate depends only on promptex-rs from crates.io, not on the other crate, so the two
# publish in any order. No Cargo.lock is tracked, so cargo publish resolves the declared ranges.
#
# Every unit's version must belong to <stage> (version_stage in scripts/lib/release-stages.sh), so a
# dispatch with the wrong stage stops. A path must be an entry of the manifest, which lists exactly
# this repo's two crates, so nothing else is published. A version that is already on crates.io is
# skipped, so a re-run is safe.
#
# usage: publish-units.sh <alpha|beta|rc|ga> <path>...
#   DRY_RUN=1  run cargo publish --dry-run: the crate is packaged and verified, nothing is uploaded
set -euo pipefail
source "$(dirname "$0")/../lib/common.sh"
cd "$REPO_ROOT"
usage() { die "usage: publish-units.sh <alpha|beta|rc|ga> <path>..."; }

toml_get() { # toml_get <file> <table> <key>
  python3 -c 'import sys, tomllib; print(tomllib.load(open(sys.argv[1], "rb"))[sys.argv[2]][sys.argv[3]])' "$@"
}

summary() { [[ -z "${GITHUB_STEP_SUMMARY:-}" ]] || printf '%s\n' "- $*" >> "$GITHUB_STEP_SUMMARY"; }

# The unit's own file must carry the manifest version: release-please writes both in one commit, and
# a broken updater would otherwise publish a version the manifest never released.
check_file_version() { # check_file_version <path> <file> <version in file> <manifest version>
  [[ "$3" == "$4" ]] || die "$1：manifest 是 $4，但 $2 是 ${3:-（空）}；release-please 沒有更新到這個檔"
}

publish_crates() { # publish_crates <path> <version>
  local p="$1" ver="$2" name status
  name="$(toml_get "$p/Cargo.toml" package name)"
  check_file_version "$p" "$p/Cargo.toml" "$(toml_get "$p/Cargo.toml" package version)" "$ver"
  status="$(curl -sS -o /dev/null -w '%{http_code}' -A 'ex-minimal-project-rs publish (github.com/promptex-ai/ex-minimal-project-rs)' \
    "https://crates.io/api/v1/crates/${name}/${ver}")"
  case "$status" in
    200)
      info "crates.io：$name $ver 已發布，略過"
      summary "crates.io \`$name $ver\` 已在註冊中心，略過"
      return 0 ;;
    404) ;;
    *) die "查詢 crates.io 的 ${name} ${ver} 回 HTTP ${status}，無法判斷是否已發布" ;;
  esac
  log "crates.io：${name} ${ver}"
  if (( DRY_RUN )); then
    cargo publish --dry-run --manifest-path "$p/Cargo.toml"
    summary "crates.io \`$name $ver\`（dry-run）"
  else
    cargo publish --manifest-path "$p/Cargo.toml"
    summary "crates.io \`$name $ver\`"
  fi
}

# publish.yml runs at the tag release-please.yml dispatched it with: the tag of the first path, which is
# <component>/v<version> (include-component-in-tag, tag-separator "/"). Any other ref, such as a branch
# or an older tag, would publish a version the ref does not carry, so stop. A local run is not checked.
check_dispatch_ref() { # check_dispatch_ref <first path>
  [[ "${GITHUB_ACTIONS:-}" == true ]] || return 0
  local p="${1%/}" comp ver want
  comp="$(jq -r --arg p "$p" '.packages[$p].component // empty' "$CONFIG")"
  ver="$(manifest_version "$p")"
  [[ -n "$comp" && -n "$ver" ]] || die "$p 在 ${CONFIG} 沒有 component，或在 ${MANIFEST} 沒有版號，無法核對發布的 tag"
  want="refs/tags/$comp/v$ver"
  [[ "${GITHUB_REF:-}" == "$want" ]] || die "發布必須從 tag ${want#refs/tags/} 執行，但 GITHUB_REF 是「${GITHUB_REF:-（空）}」"
}

[[ $# -ge 2 ]] || usage
stage="$1"; shift
check_dispatch_ref "$1"
is_stage "$stage" || die "階段「${stage}」不是 ${RELEASE_STAGES[*]} 之一"
for p in "$@"; do
  p="${p%/}"
  ver="$(manifest_version "$p")"
  [[ -n "$ver" ]] || die "$p 不在 ${MANIFEST}，不是本倉庫發布的套件"
  vs="$(version_stage "$ver")" || die "$p 的版號 $ver 不是 X.Y.Z 或 X.Y.Z-<alpha|beta|rc>.N，無法決定發布階段"
  [[ "$vs" == "$stage" ]] || die "$p 的版號 $ver 屬於 ${vs}，但這次發布的階段是 ${stage}"
  reg="$(unit_registry "$p")" || die "$p 沒有 Cargo.toml，不知道要發布到哪個註冊中心"
  [[ "$reg" == crates ]] || die "$p 的單元檔對應 ${reg}，本倉庫的 publish.yml 只發布到 crates.io"
  publish_crates "$p" "$ver"
done

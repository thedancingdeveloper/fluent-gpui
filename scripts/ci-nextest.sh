#!/usr/bin/env bash
set -euo pipefail

# Merge queues and ordinary pushes are the authoritative full-workspace run.
if [[ "${GITHUB_EVENT_NAME:-}" != "pull_request" ]]; then
    exec cargo nextest run --workspace
fi

base_ref="origin/${GITHUB_BASE_REF:?GITHUB_BASE_REF is required for pull_request runs}"
git fetch --no-tags origin "${GITHUB_BASE_REF}"
mapfile -t changed < <(git diff --name-only "${base_ref}...HEAD")

# A vendor change can affect any workspace crate, so it opts out of scoping.
for path in "${changed[@]}"; do
    if [[ "$path" == vendor/* ]]; then
        echo "vendor change detected; running the full workspace"
        exec cargo nextest run --workspace
    fi
done

packages=()
declare -A seen=()
for path in "${changed[@]}"; do
    if [[ "$path" =~ ^crates/(fluent-[^/]+)/ ]]; then
        package="${BASH_REMATCH[1]}"
        if [[ -z "${seen[$package]:-}" ]]; then
            packages+=("$package")
            seen[$package]=1
        fi
    fi
done

if ((${#packages[@]} == 0)); then
    echo "no fluent-* crate changed; skipping package tests"
    exit 0
fi

printf 'testing changed packages:'
printf ' %s' "${packages[@]}"
printf '\n'
args=()
for package in "${packages[@]}"; do
    args+=(-p "$package")
done
exec cargo nextest run "${args[@]}"

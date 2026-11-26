# Releasing

A release is a Git tag `vX.Y.Z+zed.<sha>` on `main`, plus a GitHub Release whose
notes are copied from `CHANGELOG.md`. `<sha>` is the vendored Zed commit
(`vendor/zed/UPSTREAM.md`), at least 7 and at most 40 hex characters; we use the
first 12, for example `v3.2.0+zed.ba63a022c9a2`.

Crate versions are bumped once, in the release change, never in feature changes.
The tag version, every published `crates/fluent-*/Cargo.toml` version and the
`zed-rev` metadata must agree (`scripts/check-zed-rev.sh` checks this).

## 1. Prepare the release change

1. Check `main` is green and nothing relevant is still in flight.
2. Decide the version from the `## Unreleased (next release: X.Y.Z)` section:
   breaking changes mean a major version, additions a minor version, fixes only
   a patch version.
3. In `CHANGELOG.md`, rename the heading to `## X.Y.Z - YYYY-MM-DD` (release
   date). Drop the `(next release: ...)` text and the how-to-add comment.
   Read the section once as an outside reader: plain language, no internal ids.
4. Set the version in every published `crates/fluent-*/Cargo.toml` (crates with
   `publish = false` are skipped by the tag check) and refresh `Cargo.lock`.
5. Run `scripts/check-changelog.sh`. It fails on a missing `## Unreleased`
   heading, so add the fresh Unreleased section in the same change
   (step 1 of the post-release checklist) or run the script before renaming.
6. Land that change on `main` through the normal process.

## 2. Dry run

`.github/workflows/release.yml` ("Release dry run") runs:

- on pushes of tags matching `v*+zed.*`,
- on pull requests that touch `release.yml`, `scripts/check-zed-rev.sh` or
  `scripts/release-notes.sh`,
- nightly at 03:23 UTC, and
- on manual dispatch (`workflow_dispatch`).

Jobs:

1. **Validate release inputs.** `scripts/check-zed-rev.sh` checks that every
   `crates/fluent-*` `zed-rev` equals the commit in `vendor/zed/UPSTREAM.md`.
   On a tag run it also checks the tag matches
   `v<major>.<minor>.<patch>+zed.<7-40 hex>`, that the sha is a prefix of the
   vendored commit, and that each published crate version equals the tag
   version. Then `scripts/release-notes.sh` writes the text under
   `## Unreleased` into `release-notes.md` and shows it in the job summary.
2. **Build.** `cargo build --workspace --release --locked` on `ubuntu-latest`
   and `windows-latest`.
3. **Draft GitHub release**, only on a tag run, after the build:
   `gh release create "$RELEASE_TAG" --draft --verify-tag --title "$RELEASE_TAG" --notes-file release-notes.md`.

Run it before tagging: start it by hand with
`gh workflow run release.yml --ref main`, or open a pull request that
touches the workflow. A tag must never be the first run of the release build.

Note: `scripts/release-notes.sh` reads the `## Unreleased` section only. For a
tagged release, the draft is made from the section named `Unreleased` in the
tagged tree. If you renamed it to the version in step 1, put the finished text
in the GitHub Release yourself (step 4) and discard the draft body, or tag
before the rename. Keep the script and this note in step.

## 3. Tag

```sh
git switch main && git pull --ff-only
sha=$(sed -n 's/^- zed commit: `\([0-9a-f]\{12\}\)[0-9a-f]*`$/\1/p' vendor/zed/UPSTREAM.md)
scripts/check-zed-rev.sh "vX.Y.Z+zed.$sha"
git tag -a "vX.Y.Z+zed.$sha" -m "vX.Y.Z"
git push origin "vX.Y.Z+zed.$sha"
```

The tag push starts the workflow above and creates a draft release.

## 4. GitHub Release

Copy the finished section from the changelog (without its heading) as the
release body:

```sh
v=X.Y.Z; tag="v$v+zed.$sha"
awk -v h="## $v - " 'index($0,h)==1{f=1;next} f&&/^## /{exit} f' CHANGELOG.md > release-notes.md
gh release edit "$tag" --notes-file release-notes.md --draft=false   # when the workflow drafted it
# or, if no draft exists:
gh release create "$tag" --verify-tag --title "$tag" --notes-file release-notes.md
```

Read `release-notes.md` before publishing. Do not write notes anywhere but the
changelog; the changelog is the single source.

## 5. After the release

- [ ] Add a new `## Unreleased (next release: X.Y.Z)` section above the release,
      with the how-to-add comment, and `scripts/check-changelog.sh` passes.
- [ ] The README names the new tag (status line and consumer snippet).
- [ ] Refresh the README screenshots (`scripts/refresh-readme-screenshots.sh`)
      if the look changed, and check the gallery pages.
- [ ] Open the GitHub Release page and check the notes render.
- [ ] Tell consumers to pin the new tag.

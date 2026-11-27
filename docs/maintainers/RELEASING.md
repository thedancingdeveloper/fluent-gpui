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
- on pull requests that touch `release.yml`, `scripts/check-zed-rev.sh`,
  `scripts/release-notes.sh` or `scripts/test-release-notes.sh`,
- nightly at 03:23 UTC, and
- on manual dispatch (`workflow_dispatch`).

Jobs:

1. **Validate release inputs.** `scripts/check-zed-rev.sh` checks that every
   `crates/fluent-*` `zed-rev` equals the commit in `vendor/zed/UPSTREAM.md`.
   On a tag run it also checks the tag matches
   `v<major>.<minor>.<patch>+zed.<7-40 hex>`, that the sha is a prefix of the
   vendored commit, and that each published crate version equals the tag
   version. Then `scripts/test-release-notes.sh` runs, and
   `scripts/release-notes.sh` writes the changelog text for the tag's version
   (the section `## X.Y.Z - date`, with the leading `v` and the `+zed.<sha>`
   suffix stripped) into `release-notes.md` and shows it in the job summary.
   A dry run (no tag) uses the `## Unreleased` section instead.
2. **Build.** `cargo build --workspace --release --locked` on `ubuntu-latest`
   and `windows-latest`.
3. **Draft GitHub release**, only on a tag run, after the build:
   `gh release create "$RELEASE_TAG" --draft --verify-tag --title "$RELEASE_TAG" --notes-file release-notes.md`.

Run it before tagging: start it by hand with
`gh workflow run release.yml --ref main`, or open a pull request that
touches the workflow. A tag must never be the first run of the release build.

The release body comes from the tag's own section of `CHANGELOG.md`, so
`## Unreleased` must already be renamed and dated (step 1.3) in the tagged
tree. If the section for the tag's version is missing or empty, the validate
job fails.

## 3. Tag

```sh
git switch main && git pull --ff-only
sha=$(sed -n 's/^- zed commit: `\([0-9a-f]\{12\}\)[0-9a-f]*`$/\1/p' vendor/zed/UPSTREAM.md)
scripts/check-zed-rev.sh "vX.Y.Z+zed.$sha"
git tag -a "vX.Y.Z+zed.$sha" -m "vX.Y.Z"
git push origin "vX.Y.Z+zed.$sha"
```

The tag push starts the workflow above and creates a draft release whose body
is the tag's changelog section.

## 4. GitHub Release

The workflow drafts the release with the tag's changelog section as its body.
Read the draft, check the notes render, then publish it:

```sh
gh release edit "vX.Y.Z+zed.$sha" --draft=false
```

Do not write notes anywhere but the changelog; the changelog is the single
source. To fix the notes, correct `CHANGELOG.md` and re-cut the draft.

## 5. After the release

- [ ] Add a new `## Unreleased (next release: X.Y.Z)` section above the release,
      with the how-to-add comment, and `scripts/check-changelog.sh` passes.
- [ ] The README names the new tag (status line and consumer snippet).
- [ ] Refresh the README screenshots (`scripts/refresh-readme-screenshots.sh`)
      if the look changed, and check the gallery pages.
- [ ] Open the GitHub Release page and check the notes render.
- [ ] Tell consumers to pin the new tag.

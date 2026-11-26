# How changes land: graft

fluent-gpui lands through graft, like the estate's other projects. GitHub
`main` stays the source of truth. Changes reach it as batch pull requests
opened from a graft integration line.

## Where things are

| Item | Value |
|---|---|
| Graft server | `graft-dev` (estate graft server, port 17888 on the tailnet) |
| Graft repository | `fluent-gpui` |
| Followed upstream | `https://github.com/thedancingdeveloper/fluent-gpui.git`, branch `main` → line `main` |
| Integration line | `backlog-1` (workers land here) |
| Mirror service | `graft-dev-mirror-fluent-gpui` (`graft-mirror follow`, metrics port 17899) |
| Mirror config | graft repo `deploy/mirror-fluent-gpui.toml`, registered in `deploy/mirrors.tsv` |

## Tokens

Credentials are files on the graft-dev host or in a session's own directory.
They never go in this repository.

- **Worker sessions** each get their own agent token, bound to the session,
  with scopes `repo:fluent-gpui:read`, `write`, `drafts` and
  `land:backlog-1`. The `drafts` scope lets the pre-land check run. Oversight
  mints it with
  `graft-backlog-oversight/bin/mint-worker fluent-gpui <ses_…> <engine-uuid>`.
- **The mirror** uses its own graft token (`/etc/graft/mirror-token-fluent-gpui`)
  and the shared GitHub token file (`/etc/graft/github-token`). It pushes
  `graft/batch-*` branches and opens the batch pull requests.

## The loop

1. A worker runs `g clone` and `g sync`, makes one small change, and runs
   `g land` on `backlog-1`. Then it stops. It doesn't push branches or open
   pull requests.
2. Oversight runs `graft-mirror pr-batch backlog-1 --base main`. That opens a
   single batch pull request, with one commit per land.
3. The batch is reviewed under the workspace delivery policy and merged with
   `land-pr --method rebase`.
4. `follow` imports the new `main`, and the batch is settled.

Each land arrives on `main` as its own commit. It carries a `Graft-Session`
trailer and the configured author identity.

## GitHub `main` settings

These match rdpapp's settings:

- Ruleset "main history protection": deleting `main` and force-pushing to it
  are blocked.
- Ruleset "main required checks": the status check `test` (the Linux job in
  `.github/workflows/ci.yml`) is required. It will be replaced by a single
  `ci` check covering Windows and Linux once the CI guard work (WI-2725) adds
  it.
- Rebase is the only merge method allowed in the repository settings, so
  each land keeps its own commit.
- There is no pull-request-required rule, as on rdpapp. Batches still reach
  `main` only as pull requests opened by `pr-batch` and merged by `land-pr`.

See the workspace rules `graft-only.md` and `delivery-process.md`
("Landing") for batch size and cadence.

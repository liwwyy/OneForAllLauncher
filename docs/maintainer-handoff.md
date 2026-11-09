# Maintainer handoff: the 2.10.0 rewrite

Snapshot: 2026-11-09. Read [AGENTS.md](../AGENTS.md) before making changes.
The [implementation plan](implementation-plan.md) records the agreed design;
the [validation record](validation.md) distinguishes automated checks from owner
testing. Update this handoff when publication or acceptance status changes.

## Current checkout

- Work is on `rewrite/2.10.0`, based on stable upstream `oneclient-2.10.0`, commit
  `3968fb95f98569de41ec341ef0da2ed63802cfc4`. The launcher version is **2.10.0**.
- The fork retains offline, Microsoft, Ely.by and custom authentication,
  same-name accounts across providers, the optional VanillaHUD preview workaround,
  offline launching and private local skins. Both Java hooks build from source.
- Fresh setup defaults to OneClient's active game library. Existing fork users
  retain their previous location. Accounts/settings remain in the private fork
  folder; instances, game files, caches and managed Java use the selected library.
- The community installation guide is
  [oneclient-modpack-alternative-guide.md](oneclient-modpack-alternative-guide.md).
  Windows SmartScreen screenshots are included in
  [installation troubleshooting](installation-troubleshooting.md). macOS has text
  instructions; no Mac hardware test has been reported.
- `AGENTS.md`, the implementation/validation documents and Java-agent sources are
  tracked. Generated build outputs, icon catalogs and Python caches are ignored.
  Recovery bundles and publication metadata are kept outside the repository.

## Owner confirmations

The owner reports that shared OneClient instances/data and Launch Offline work,
local skins work on Ornithe 1.8.9 and modern versions, and the OneConfig HUD opens
with an Ely.by account. The preview-workaround setting used in that HUD test was
not specified. Keep the toggle until the actual agent-disabled game test is
confirmed; authlib-injector remains separate and necessary for Ely.by/custom
authentication.

Earlier automated build, Rust and Java-agent matrix results are recorded in
[validation.md](validation.md). Documentation maintenance does not rerun those
application tests. The owner tests UI/game behavior manually; do not take
screenshots or perform automated visual comparisons without an explicit request.

## Authorized publication handoff

The owner explicitly authorized pushing the rewrite and automatically creating
the release, and requested that the agent **not wait for GitHub builds**. The
owner will verify CI results and report failures. This publication instruction
supersedes the earlier request to hold the push and wait for acceptance checks.
Do not describe pending CI, CodeQL or package tests as passed.

Before the authorized replacement, remote `main` was
`71699a140a820538335f67924961973c501eec41` and the latest release was
`oneforall-2.9.0`. Check GitHub for current publication/build status.

1. Review the final commit series and diff against `oneclient-2.10.0`, confirm the
   owner's publication instructions, and record the exact final tip. A verified
   pre-rewrite bundle and archived branch are described in
   [the recovery notes](validation.md#recoverable-history-replacement).
2. Replace only remote `main` with the agreed rewrite using an explicit
   `--force-with-lease` for the expected old SHA above. Stop if the remote has
   moved. Preserve upstream history, existing release tags/assets and the updater
   signing identity. Refresh local publication metadata after further commits.
3. Managed CodeQL scans the exact rewritten commit on push. All four languages
   use the Extended suite. The initial scan covered the old published `main` and
   reported 260 untriaged alerts; it does not cover this rewrite. Review the new
   findings when they become available; the owner is handling result verification
   for this automatic publication. See [CodeQL configuration](security-verification.md#codeql-configuration).
4. Complete outstanding package/upgrade acceptance checks from
   [validation.md](validation.md#manual-acceptance-before-release), preserving
   existing user data, private account settings and side-by-side installation.
   Record platform testing limits explicitly.
5. The [release workflow](../.github/workflows/oneclient_release.yml) starts on
   pushes to `main` and **publishes automatically after successful builds/signing**.
   The target is `oneforall-2.10.0`; automatic pushes skip existing release tags.
   Manual dispatch remains available. The owner requested no wait for these jobs.
   Its updater signatures require the existing repository signing secrets.
   The separate [nightly workflow](../.github/workflows/oneclient_nightly.yml)
   provides unsigned test artifacts without publishing a release.

The [sync badge workflow](../.github/workflows/upstream_sync.yml) runs hourly,
on main pushes and on manual dispatch. Its JSON lives on the separate `badges`
branch, so badge refreshes do not add source commits or trigger releases. It
checks ancestry of the latest stable `oneclient-*` release, excluding drafts,
prereleases and other package tags. Unavailable checks display that state.

[Return to the README](../README.md#maintenance).

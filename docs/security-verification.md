# Reviewing the fork and verifying releases

No scanner, clean antivirus result, signature or badge proves that a program
contains no malicious behavior. Useful evidence combines independent source
review, automated checks and a verifiable link between source and release files.
This page records free options researched on 2026-11-09; it is a recommendation,
not a claim that these scans or an independent audit have already passed.

## Recommended free approach

1. **GitHub CodeQL code scanning.** This is GitHub's own static-analysis engine,
   available for public repositories. Its supported languages include this
   project's Rust, Java and Python, plus GitHub Actions workflows. Enable it under
   the repository's security/code-scanning settings, select the supported languages
   and review the results for the exact release commit. It finds vulnerability
   patterns and coding errors; it does not certify the author's intent or fully
   model dynamically generated Java-agent bytecode.
   [Availability and language support](https://docs.github.com/en/code-security/concepts/code-scanning/codeql/codeql-code-scanning),
   [Rust/Java build modes](https://docs.github.com/en/code-security/concepts/code-scanning/codeql/codeql-for-compiled-languages).
2. **Independent review of the fork's changes.** Publish a small commit series
   against the pinned upstream release. Ask reviewers to inspect authentication,
   token handling, outbound requests, update verification, native/Java hooks and
   build/download scripts. Include dependencies and workflows in the review.
   Running tests helps validate behavior; human review is still needed to assess
   suspicious logic and whether the intended behavior is acceptable.
3. **Public CI builds and GitHub artifact attestations.** Build releases from the
   reviewed commit in a public workflow, retain build logs and checksums, and add
   attestations for the final distributed files. GitHub provides attestations for
   public repositories on its free plan. Users can verify the artifact with
   `gh attestation verify FILE --repo liwwyy/OneForAllLauncher` once attestations
   are actually published. This establishes build provenance, not that the source
   is harmless. Check that the attested source commit and release workflow match
   the reviewed release. [Official instructions](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).
4. **OpenSSF Scorecard as an additional public report.** Its official GitHub Action
   is free for public repositories and checks practices such as code review,
   dependency pinning and release security. Treat each finding as something to
   inspect, rather than treating the aggregate score as a malware verdict.
   [Official Action](https://github.com/ossf/scorecard-action),
   [what Scorecard measures](https://github.com/ossf/scorecard/blob/main/docs/faq.md).

CodeQL is the best first automated check for this public GitHub fork. The stronger
trust story is **reviewed source + visible test/scan results + CI-built, attested
release files**. This can use free public-repository features and standard hosted
runners; it does not require a paid scanner.

## What the current checkout demonstrates

- The Java hooks are built from tracked source, with pinned ASM downloads and
  licenses included in the generated artifacts. The offline hook is gated to a
  standalone offline account with a selected PNG.
- The updater verifies packages with the fork's existing signing identity.
  Updater signatures establish authenticity/integrity; they are separate from
  Windows/macOS application signing and from a source security review.
- Tests cover account/provider boundaries, private settings, cache-only launches
  and local skin transport. See [validation notes](validation.md) for actual runs
  and the remaining manual checks.
- CodeQL default setup was reported as **not configured** by GitHub during this
  review. No security setting, workflow or release was changed for this research,
  and no new CodeQL result or build attestation is claimed here.

## Reproducible builds

An independently reproducible build lets another party recreate identical
artifacts from the same source, instructions and environment. It can strengthen
the source-to-binary link, but identical malicious source still produces malicious
binaries. [Definition](https://reproducible-builds.org/docs/definition/).

The Java archive builder normalizes archive metadata. Full launcher/installer
reproducibility across Linux, Windows and macOS has not been demonstrated. Do not
advertise the entire release as reproducible until independent rebuilds match.

[Return to the README](../README.md)

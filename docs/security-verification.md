# Reviewing the fork and verifying releases

No scanner, clean antivirus result, signature or badge proves that a program
contains no malicious behavior. Useful evidence combines independent source
review, automated checks and a verifiable link between source and release files.
This page records free options researched on 2026-11-09 and the CodeQL setup
enabled at the maintainer's request. Recommendations are separate from completed
checks; no independent audit or build attestation is claimed.

## Recommended free approach

1. **GitHub CodeQL code scanning.** This is GitHub's own static-analysis engine,
   available for public repositories. Its supported languages include this
   project's Rust, Java and Python, plus GitHub Actions workflows. The fork uses
   managed default setup; review its results for the exact release commit before
   publishing. It finds vulnerability patterns and coding errors; it does not
   certify the author's intent or fully model dynamically generated Java-agent
   bytecode.
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

## CodeQL configuration

GitHub's managed **default setup** was enabled on 2026-11-09, using standard
GitHub-hosted runners and the **Extended** query suite. It detects Rust,
Java/Kotlin, Python and GitHub Actions. The threat model includes remote and local
sources where supported. Default setup uses no-build analysis for Rust and Java,
so these scans do not replace compilation or the agent's runtime tests.
[Default setup](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/configure-code-scanning/configure-code-scanning),
[configuration options](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/edit-default-setup).

The [initial setup run](https://github.com/liwwyy/OneForAllLauncher/actions/runs/42659907902)
completed successfully for all four languages on published `main` at
`71699a140a820538335f67924961973c501eec41`. It reported **260 open alerts**
(245 Rust, 3 Java, 11 Python and 1 Actions). These findings remain untriaged;
a successful analysis job does not mean the code has no vulnerabilities.
That run predates the `rewrite/2.10.0` changes and does not cover them. A push of
the rewrite to `main` triggers its own analysis. Check the [code-scanning results](https://github.com/liwwyy/OneForAllLauncher/security/code-scanning)
and the analyzed commit before citing a result for a release.

GitHub manages the analysis workflow; no `.github/workflows/codeql.yml` is needed.
Subsequent pushes and pull requests targeting the default/protected branches
trigger analysis subject to GitHub's default-setup rules. GitHub reports a weekly
schedule configured; its documentation says recurring scans start after a push or
pull-request scan. Keep this configuration enabled when publishing the rewrite
and inspect the new commit's results.
[Triggers and scheduling](https://docs.github.com/en/code-security/concepts/code-scanning/setup-types).

## Reproducible builds

An independently reproducible build lets another party recreate identical
artifacts from the same source, instructions and environment. It can strengthen
the source-to-binary link, but identical malicious source still produces malicious
binaries. [Definition](https://reproducible-builds.org/docs/definition/).

The Java archive builder normalizes archive metadata. Full launcher/installer
reproducibility across Linux, Windows and macOS has not been demonstrated. Do not
advertise the entire release as reproducible until independent rebuilds match.

[Return to the README](../README.md)

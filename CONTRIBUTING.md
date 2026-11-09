# Contributing to OneForAllLauncher

OneForAllLauncher is a fork of OneClient by Polyfrost, licensed under GPL-3.0-only.
Preserve LICENSE, ATTRIBUTION.md and original copyright notices.

## Scope

This fork only adds or modifies features directly associated with:

- Standalone offline accounts.
- Ely.by authentication support.
- Custom authentication server support.

Contributions must directly support one of these account features. Keep
modifications to OneLauncher minimal so the fork remains easy to maintain,
review and update. Necessary maintenance must support this scope without
introducing additional features. Propose unrelated features, general launcher
improvements and UI redesigns to upstream OneLauncher.

Reuse upstream implementations, Freya components, styling, layout and interaction
patterns. Avoid broad refactors, mechanical renames and unrelated formatting.
Keep upstream instance import/export and sharing rather than adding fork-specific
exporters. Preserve existing fork data, account formats, installation identities
and updater compatibility.

The local skin hook is restricted to launches with an actual offline
account (`AccountKind::Offline`). It must not load for Microsoft, Ely.by or custom
accounts, including when those accounts are launched without internet or with
the explicit offline launch mode.

## Changes and validation

Follow [README.md](README.md) for building, testing and packaging, and
[AGENTS.md](AGENTS.md) for repository conventions and compatibility requirements.
The application behavior and acceptance checks are in
[the implementation plan](docs/implementation-plan.md); planned features are not
yet available in released builds.

In a pull request, explain which supported account feature the change serves,
why the upstream implementation needs to differ, and how the behavior was
verified. Keep commits focused and separate fork-specific behavior from upstream
maintenance. Include targeted tests for changed behavior and upgrade compatibility.
Run relevant build, formatting, lint and test checks; documentation-only changes
need link and diff checks rather than an application build.

Do not take screenshots or perform automated visual comparisons unless the owner
explicitly requests them. The owner manually tests UI changes. Account creation,
onboarding, startup, side-by-side installation and packaged upgrades must be
verified as relevant before publishing a release.

## Upstream updates and reports

The fork follows stable OneLauncher/OneClient releases. It will usually be
updated within a day of an upstream stable release, after validation.

Submit issues and pull requests to this fork, rather than to Polyfrost for fork-specific behavior.

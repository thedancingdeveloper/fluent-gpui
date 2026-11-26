# Security policy

## Reporting a vulnerability

Please report vulnerabilities privately. Do not open a public issue.

Use GitHub's **Report a vulnerability** button on the
[Security tab](https://github.com/thedancingdeveloper/fluent-gpui/security/advisories/new)
of `thedancingdeveloper/fluent-gpui`. This opens a private security advisory
that only the maintainers can see.

Please include:

- the affected crate and version or tag;
- your operating system and, if relevant, GPU and driver;
- steps to reproduce, or a minimal example;
- what you expected and what happened, and the impact you see.

We aim to acknowledge reports promptly and will keep you updated in the
advisory. We do not promise specific response times.

## Supported versions

Only the latest release line receives security fixes.

## Scope

In scope: the `fluent-*` crates in this repository.

The code under `vendor/zed` is a vendored copy of Zed's gpui. Please report
vulnerabilities in upstream gpui to [Zed](https://github.com/zed-industries/zed)
directly. If a problem comes from one of our local patches (listed in
`vendor/zed/PATCHES.md`), report it here.

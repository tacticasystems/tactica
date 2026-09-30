# Tactica

The milsim unit management platform.

See [the unit API map](docs/api.md) for implemented read endpoints, access rules,
and the next endpoint batches.

## Build

```
mise run build    # build the runtime components
mise run test     # run all tests
mise run check    # run all CI steps
```

`check` needs Docker to run the [`testcontainers`](https://testcontainers.com/)
fixtures.

CI is in GitHub Actions, in `.github/workflows/`.

## Releases

[Release Please](https://github.com/googleapis/release-please) maintains separate
release PRs for the Rust workspace and the web app on `main`. Merging a release PR
creates a GitHub release and a component tag, such as `rust-v0.1.1` or `web-v0.1.1`.

Use [Conventional Commits](https://www.conventionalcommits.org/), including in
squash merge titles: `fix(rust): ...`, `feat(web): ...`, and `feat(rust)!: ...`
for breaking changes. Changed file paths determine which component releases;
web-only changes do not bump Rust. While versions are below 1.0, features and
fixes bump the patch version, and breaking changes bump the minor version.

The Rust crates share one release version, recorded in `version.txt`. Release
Please updates every crate's `Cargo.toml`, the local package entries in
`Cargo.lock`, and the root `CHANGELOG.md`. This uses the `simple` strategy with
TOML extra files because the native Rust strategy requires a root `[package]`
section, which this virtual workspace does not have. The web release updates
`web/package.json` and `web/CHANGELOG.md`; pnpm's lockfile does not contain the
app's own version. Changelogs are created with the first release PR. The initial
version baseline for both components is 0.1.0 in `.release-please-manifest.json`.
With no existing release tags, the first changelog includes existing eligible
commits. After that, only commits since each component's previous release count.

Enable **Allow GitHub Actions to create and approve pull requests** in the
repository's **Settings > Actions > General**. To run CI automatically on release
PRs, add a repository Actions secret named `RELEASE_PLEASE_TOKEN`, using a
fine-grained personal access token with repository **Contents**,
**Issues**, and **Pull requests** read/write permissions. Without that secret,
the workflow uses `GITHUB_TOKEN`, whose PR and release events do not trigger other
workflows. In that case, close and reopen the generated PR as a user to trigger
CI before merging it.

This workflow creates GitHub releases. Container publishing continues through
the existing CI workflows on pushes to `main`.

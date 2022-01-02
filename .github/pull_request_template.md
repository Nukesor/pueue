Thanks for sending a pull request!
Here's some info on how development works in this project:

The `main` branch is the version of the last release. \
`patch` level PRs, for example bugs or critical library updates, should be merged into `main` directly. \
A new `patch` release is usually published shortly after the PR is accepted.

New features should branch of the `development` branch. \

## Checklist

- [ ] I picked the correct source and target branch.
- [ ] I included a new entry to the `CHANGELOG.md`.
- [ ] I checked `cargo clippy` and `cargo fmt`. The CI will fail otherwise anyway.
- [ ] (If applicable) I added tests for this feature or adjusted existing tests.
- [ ] (If applicable) I checked if anything in the wiki needs to be changed.

Thanks for sending a pull request!
Here's some info on how development works in this project:

The `main` branch is the version of the last release. \
`patch` level PRs, for example bugs or critical library updates, should be merged into `main` directly. \
A new `patch` release is usually published shortly after the PR is accepted.

New features should branch of the `development` branch. \
`development` usually works with the `main` branch of the `pueue-lib` repository. \
That way we can work with it without having to release new versions all the time.

If your issue also requires a PR to `pueue-lib` as well, please use your `pueue-lib` development branch as a dependency.
```
pueue-lib = { git = "https://github.com/YourName/pueue-lib", branch = "your_branch" }
```
Once the MR in the `pueue-lib` repository is merged, just change the dependency back to the original `pueue-lib/main` branch.

## Checklist

- [ ] I picked the correct source and target branch.
- [ ] I included a new entry to the `CHANGELOG.md`.
- [ ] I checked `cargo clippy` and `cargo fmt`. The CI will fail otherwise anyway.
- [ ] (If applicable) I added tests for this feature or adjusted existing tests.
- [ ] (If applicable) I adjusted the wiki according to the new changes.

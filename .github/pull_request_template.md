Thanks for sending a pull request!
Here's some info on how development works in this repository:

The `main` branch represents the source code of the last release. \
`patch` level PRs, for instance bugs or critical library updates, should be merged into `main` directly. \
A new `patch` release will be published soon after the change being merged.

New features should branch of the `development` branch. \
`development` usually works with the `main` branch of the `pueue-lib` repository. \
That way we can work with it without having to release new versions all the time.

If your issue also requires a PR to `pueue-lib` as well, just use your development branch for the time being
```
pueue-lib = { git = "https://github.com/YourName/pueue-lib", branch = "your_branch" }
```
Once the MR in the `pueue-lib` repository is merged, just change the dependency back to the original `pueue-lib` `main` branch.

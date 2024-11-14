# Contributing

Thank you for considering to contribute to `metapac`, all contributions are
welcome!

There are no hard rules for this project but there are some general
guidelines to avoid problems:

- Open github issues for any features/bugs/issues you want to raise, but
  try to check the existing issues to avoid duplicates.
- Try to comment on an issue before attempting any pull requests to avoid
  multiple people working on the same feature at the same time which would
  result in wasted effort.
- Discussion is always welcome on issues since there are usually multiple
  ways to fix/implement every bug/feature and discussion is the best and
  easiest way to select the best solution.

## Release Process

In order to release a new version of `metapac`:

- update the `CHANGELOG.md` file with the changes
- run `cargo release x.y.z --execute`
- update the AUR packages (`metapac` and `metapac-bin`):
  - update the version numbers in the `PKGBUILD` files
  - run `updpkgsums`
  - run `makepkg --printsrcinfo > .SRCINFO`
  - discard the tar file though made when running `updpkgsums`
  - commit and push the changes

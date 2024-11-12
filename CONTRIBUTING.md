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

In order to release a new version of pacdef:

- Update the `CHANGELOG.md` file with the changes
- Run `cargo release x.y.z --execute`
- In the AUR package repos, run `updpkgsums` and `makepkg
--printsrcinfo > .SRCINFO`, then commit and push the changes. (Discard
  the tar file though from `updpkgsums` though)

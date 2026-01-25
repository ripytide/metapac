# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

‼️ This is a breaking release. All users will need to fix their group
files to move to the new format. See the migration guide below. ‼️

This is big release, adding the (hopefully) final ingredient in a
declarative meta package manager, repo management. Because what good is
being able to declare a list of packages if the names of those packages are
ambiguous due to coming from a bunch of different unmentioned repos.

This release fixes that by introducing a second declaratively managed
entity, a repo. Lots of package managers support multiple and even
arbitrary repos, but for this release the driving backend was `dnf` and
specifically issue #176. Thanks to @komapro too for making the case so well
too.

Hopefully, in future releases `metapac` will support repo management for
more backends.

And just as luck would have it, `toml` `v1.1.0` was released only a month
ago <https://github.com/toml-lang/toml/issues/928> which added support for
newlines and trailing commas in inline tables which makes group files a lot
more flexible.

### Migration Guide

The array of packages gets moved under the `packages` property of the
backend and the `package` property of long-form packages is renamed to
`name` so it can be shared with the new repos.

#### Old group file format

```toml
arch = [
  "package1",
  { package = "package2" },
]
```

#### New group file format

```toml
arch = {
  packages = [
    "package1",
    { name = "package2" },
  ]
}
```

### Added

- `metapac` now supports `.toml` files using [`toml
  v1.1.0`](https://toml.io/en/v1.1.0) (#188).

### Changed

- ‼️ Breaking Change ‼️ The group file format has changed to allow both
  repos and packages to be added for the same backend (discussion in #176,
  implemented in #188).
- ❗ Breaking Change ❗ Invalid properties at the top level of a group file
  are now no longer a soft warning but a hard error (#188).
- ❗ Breaking Change ❗ The `flatpak` has been completely overhauled and a
  new `installation:name` format is used for package/repo names since each
  installation is independent. The `flatpak.systemwide` config has been
  removed in favor of the more explicit installation on each package (#200
  and #201)

### Removed

- ‼️ Breaking Change ‼️ The `metapac add`, `metapac remove`, `metapac
  install`, and `metapac uninstall` subcommands have been removed (#198).
  This is up for discussion though, #197 has been opened for this
  purpose and contains further reasoning.

## [0.8.0] - 2026-01-11

### Changed

- ‼️ Breaking Change ‼️ All short-forms and single letter sub-command
  aliases have been removed. This is due to my preference for explicitness
  and that as the cli api surface has grown with more sub-commands it is no
  longer easy to remember which letter goes with which sub-command (#187).

  As an example `metapac s` is no longer valid, use the explicit `metapac
  sync` instead. The same goes for options so instead of `metapac -n
  hostname` use `metapac --hostname hostname`.
- ❗ Breaking Change ❗ the `dnf` backend has been massively simplified and
  now has no package options. Previously there was a `repo` and `user`
  options which didn't make any sense. The backend now behaves much
  more similarly to the other backends (#185).

### Fixed

- A lot of the complicated group file parsing and validation has been moved
  into only those sub-commands that require that rather than being done for
  all sub-commands which should result is a gargantuan speedup for those
  sub-commands that didn't and don't need the group files such as `metapac
  upgrade-all` and `metapac backends` (#187).
- Fixed running non-`.exe` executables on windows such as `.ps1` and `.cmd`
  files which aren't normally executable by rust's `Command` by wrapping
  all commands executed on windows in `cmd /C` which enables them.
  (reported in #184, fixed in #186)

## [0.7.2] - 2026-01-03

### Added

- Added the `zypper` backend (#181), thanks @teoc98!

## [0.7.1] - 2025-12-29

### Fixed

- Issue with the `pipx` backend using a wrong command when updating all
  packages (#177), thanks @Deuchnord!
- Issue with `xbps` backend managing all installed packages instead of only
  those that were installed manually by the user (#179), thanks @teoc98!

## [0.7.0] - 2025-12-13

### Added

- New backend: `mise` (requested in #127, implemented in #166), thanks
  @Mikel-Landa!
- New backend: `mas` (Mac App Store) (#166), thanks @jtbrough!

### Changed

- ❗ Breaking Change ❗ the `hostname_enabled_backends_enabled` config has
  been removed and now the `enabled_backends` will be merged with any
  matching entries in the `hostname_enabled_backends` table rather than being
  mutually exclusive. (#163)

### Fixed

- Issue with the `cargo` `binstall` option not tracking installed packages
  has been fixed by switching from using the `.crates2.json` file to using
  the `.crates.toml` file (#167), thanks @Mikel-Landa!

## [0.6.4] - 2025-11-10

### Added

- Added a `binstall` option to the `cargo` backend to allow using
  `cargo-binstall` to install packages instead of `cargo install` (#165),
  thanks @Mikel-Landa!

## [0.6.3] - 2025-10-28

### Added

- Added more logging for which commands get run and allowed debug logging to be
  enabled via an environment variable.

### Fixed

- The `metpac update-all` commands for the `winget` backend have been fixed
  so that it now actually tries to update all packages (#162), thanks
  @davidlindenbaum!.

## [0.6.2] - 2025-10-11

### Added

- The `brew` backend and packages now have a `quarantine` option. (#156)

### Fixed

- The `brew` backend can now properly list both casks and formulae
  (reported in #154, fixed in #156)
- The `arch` backend now no longer complains that installed AUR packages are
  invalid when using `pacman`. (reported in #153, fixed in #155)

## [0.6.1] - 2025-09-29

### Changed

- Duplicate packages are now back to being a warning rather than a hard
  error after it was excellently pointed out that there is a great use-case
  for them. (reported in #149, fixed in #151)

## [0.6.0] - 2025-09-27

### Added

- New configs: `hostname_enabled_backends_enabled` and
  `hostname_enabled_backends`. They allow the option of specifying the
  enabled backends per hostname. This is very handy if you use multiple
  machines with different operating systems and so require different
  backends. (#146)
- New 🪝Hooks🪝: `before_sync` and `after_sync`. Which are always run when
  using the `metapac sync` command regardless of whether the package is
  already installed or not unlike `before_install` and `after_install`.
  Only runs for enabled backends. (#136)
- Options and hooks are now printed when `metapac` outputs packages. Mainly
  useful for the `metapac unmanaged` command but is also relevant for the
  `metapac clean` and `metapac sync` commands.
- Package validation step added for some backends to ensure that listed
  packages are valid before starting any of the commands. This ensures that
  common errors like using an implicit package id for backends like `scoop`
  or `winget` are caught before they turn into undesired uninstalls. (#145)

### Changed

- ‼️ Breaking Change ‼️ `arch` group files now no longer accept package
  groups. See the [`arch`](https://github.com/ripytide/metapac#arch)
  section part of the `README.md` for why the functionality was removed,
  and what to do instead. (#133)
- ❗ Breaking Change ❗ Enabled backends will now hard error if you have
  invalid packages in your group files whereas before this was just a
  warning. Not all backends can tell which packages are valid or invalid
  yet, so only the `arch` backend does this so far. But I'm planning on
  adding this validation to at least the `scoop` backend at some point and
  possibly more. (#133)
- ❗ Breaking Change ❗ The `flatpak` backend now only lists applications
  and not runtimes since runtimes are only ever dependencies to
  applications and so are intrinsically implicit packages. (#134)
- ❗ Breaking Change ❗ Duplicate packages in group files is now a hard
  error rather than a warning. (#136)

### Fixed

- Cancelling a `metapac install` or `metapac uninstall` before installing
  the packages now does not change your group files. Reported in #130,
  fixed in #135.

## [0.5.0] - 2025-08-09

This release marks the final big part of `metapac` that has now been
implemented: updating packages! There are four core mechanisms that make up
a package manager:

- install a package
- uninstall a package
- list installed packages
- update a package

`metapac` now implements all of these key mechanisms for all our
supported backends!

> [!WARNING]
> There are very high chances that the update implementations
> for most of the backends don't actually work yet. This is simply because
> I don't have the time or energy required to fully test every different
> backend's update commands, especially since lots of the package managers
> only work on a specific linux distro!
>
> Therefore, I've tried my best for each backend and I am going to
> crowdsource the rest of the testing to the users. So as usual, if a
> backend is not behaving as you would expect, please file a bug report.
> Ideally, you can also try to find the issue with the commands so we can
> fix them.

### Added

- New subcommand `metapac update`! (#113)
- New subcommand `metapac update-all`! (#113)
- New `--locked` option for the `cargo` backend, including a new config:
  `cargo_default_locked` (#104), thanks @DominicD!.

### Changed

- ‼️ Breaking Change ‼️ Backend specific configs have been moved into
  sections for each backend and `flatpak_default_systemwide` has been
  renamed to `flatpak.systemwide` (#117), thanks @steven-omaha! For
  example:

  ```toml
  # instead of this
  arch_package_manager = "paru"
  flatpak_default_systemwide = false
  vscode_variant = "code"

  # do this
  [arch]
  package_manager = "paru"
  [flatpak]
  # notice this is now just systemwide and not default_systemwide
  systemwide = false
  [vscode]
  variant = "code"
  ```

- ‼️ Breaking Change ‼️ The `clean-cache` command and the new `update`
  command now do 3 different things depending on what you pass to the
  `--backends` option if anything:
  - if no backends are passed then the enabled_backend config is used
  - if "all" is passed by itself then all backends are used
  - otherwise the list will be parsed as a list of backends to be used

### Fixed

- For commands that take a list of packages as an option, the `--packages`
  option is now required and has a `num_args=1..` so not passing at least
  one package is a clap error. (#113)

## [0.4.1] - 2025-08-02

### Added

- Added the `scoop` backend (#101), thanks @jsfr!

### Fixed

- Fixed a bug with packages being added twice when the backend didn't yet
  exist in a group file with the `metapac add` command. Fixed in #99,
  thanks @DominicD!
- Fixed a bug when running `metapac sync --no_confirm` with the `arch`
  backend. Used the incorrect `--no_confirm` option rather than
  `--noconfirm`. Fixed in #105.

## [0.4.0] - 2025-07-27

Another big release 🚢, with no breaking changes this time 🎉🎉. In this
release we welcome the javascript ecosystem of package managers into
`metapac`. With an exception for no `deno` support yet since it doesn't yet
support listing global packages (see
<https://github.com/denoland/deno/discussions/28230>)

### Added

- Added the `bun` backend (#95)
- Added the `npm` backend (#95)
- Added the `pnpm` backend (#95)
- Added the `yarn` backend (#95)
- Added a wishlist section to the documentation for listing package
  managers we might want to one day support
- Added relevant XKCDs to the readme since it feels like a right of passage
  and to not show them would be worse

## [0.3.2] - 2025-07-17

### Fixed

- Fixed a bug with packages not being set to be explicitly installed if
  they were previously installed as a dependency (reported in #93,
  introduced by #87, fixed in #94).

## [0.3.1] - 2025-07-13

### Removed

- I managed to make it about 30 minutes after releasing `v0.3.0` before
  realizing that the `before_uninstall` and `after_uninstall` hooks were
  completely useless since to uninstall a package your remove it from you
  group files in which case there will be no hooks to execute! These hooks
  have now been removed (#92).

## [0.3.0] - 2025-07-13

This is the biggest release `metapac` has had in quite a while including
two breaking changes that will require changes to most people's config and
group files. As such I've bumped the minor version number. And so begins
the `0.3.x` journey for `metapac` :ship: and farewell `0.2.x` :wave:

### Added

- 🪝Hooks🪝: Per-package custom commands: `before_install`,
  `after_install`, `before_uninstall` and `after_uninstall` have been
  added. This allows the automatic enabling of `systemd` services for arch
  packages by using the `after_install` hook. See the `README.md` for
  examples (#91).

### Changed

- ‼️ Breaking Change ‼️ the config `disabled_backends` has now been
  inverted and renamed to `enabled_backends` to prevent everything being
  uninstalled when adding new backends (#90).
- ‼️ Breaking Change ‼️ the toml format for group files has been changed,
  options have been moved under a key called `options` in order to make way
  for the new per-package `hooks` feature (#91).
  - For example, instead of `{ package = "metapac", backend_relevant_option
= "yes"}` you should change this to `{ package = "metapac", options = {
 backend_relevant_option = "yes" } }`

### Fixed

- `metapac` will now throw errors if it encounters unexpected keys when
  deserializing config and group files. This prevents accidentally
  misspelling a config option and then being confused by `metapac` not
  respecting it (#91).
- A warning is now given to the user if duplicate packages are found in the
  same group file as well as across multiple group files (#91).
- A warning is now also given if duplicate packages are passed on the
  command line with the add/remove/install/uninstall commands (#91).

## [0.2.14] - 2025-06-04

### Fixed

- The `pipx` backend could would fail to uninstall multiple packages. This
  has been fixed in (#88), thanks @arutonee1!

## [0.2.13] - 2025-06-03

### Fixed

- The `arch` backend would sometimes attempt to reinstall already installed
  packages. This has been fixed in (#87), thanks @arutonee1!

## [0.2.12] - 2025-05-21

### Added

- The `pamac` arch package manager as another option for the `arch` backend (#84).

## [0.2.11] - 2025-05-21

### Added

- The `snap` backend now accepts a per-package `confinement` option (#82)
  thanks @curetheopensource!

### Removed

- The release notes on GitHub will no longer contain release artifacts as
  `cargo-dist` seems to have become unmaintained and has various issues at
  the moment.
- This also means that the `metapac-bin` AUR package will receive no more
  updates as it relies upon having a built artifact hosted by GitHub. The
  non-binary `metapac` AUR package will still receive updates though.

## [0.2.10] - 2025-03-22

### Added

- New backend: `vscode`, comes with a new config option: `vscode_variant`
  for selecting between VSCode and VSCodium (#78)!

## [0.2.9] - 2025-03-16

### Added

- New backend: `uv` (#75)!

### Changed

- All commands that previously took a `--package` argument now take a
  `--packages` argument instead to allow passing multiple packages instead
  of just one at a time.

### Removed

- The `rustup` backend has been removed since `rustup` components are more
  complex to manage and so don't fit with the rest of the backends very
  nicely. (#76)

## [0.2.8] - 2025-03-09

### Added

- New commands (#71):
  - `metapac remove` which is the opposite of `metapac add`
  - `metapac install` which does `metapac add` plus immediately installs
    the requested package
  - `metapac uninstall` which does the opposite of `metapac install`

### Removed

- The `metapac review` command has been removed since it hasn't had an
  implementation and has been replaced by editing the output of the
  `metapac unmanaged` command using a text editor as outlined in the usage
  section of the `README.md`.

## [0.2.7] - 2025-02-08

### Fixed

- `flatpak` backend now doesn't skip the first listed package (#65) thanks
  @latin-1!
- `dnf` backend now queries installed packages using the correct format
  (it was missing a newline between packages) (#66) thanks @latin-t!

### Changed

- There is quite a large code refactor for each of the backends in this
  release, there shouldn't be any functional differences from the last
  version but there is a higher likelihood for new bugs to be introduced.

## [0.2.6] - 2024-12-23

### Added

- A new per-package `systemwide` setting for `flatpak` packages. (#62)

### Changed

- The `flatpak_systemwide` has been renamed to
  `flatpak_default_systemwide` to allow for a new `systemwide`
  per-package setting for `flatpak` packages (#62)

### Removed

- The `optional_deps` options on `arch` packages has been removed
  since it not a feature of the `arch` backend package managers and
  was handled by `metapac`, in the interest of simplicity this odd bit
  of logic has been removed (this also it makes the
  code nicer). Instead if you have multiple packages
  which you want installed only if another package is installed
  consider using a comment and whitespace to separate them
  visually in your group files so that it is obvious when reading or
  modifying them that they are linked. You could even separate the
  packages out into another group file and include or uninclude the
  entire group via symlinking or the `hostname_groups` config feature. (#62)

### Fixed

- The `flatpak` backend no longer mistakenly uses `sudo` when removing packages. (#57)

## [0.2.5] - 2024-11-24

### Added

- New backend: `snap` (#54) thanks @watzon!

## [0.2.4] - 2024-11-21

### Added

- New subcommand `metapac backends`! (#50)

  A new subcommand `metapac backends` has been added which shows you which
  backends `metapac` can find on your system and also their version
  numbers!

- `flatpak` packages now support a `remote` config value to allow you to
  specify which remote you want to install each package from (#53)

### Fixed

- `metapac` now gives out a hefty warning when you have arch packages in
  your group files which don't match real arch packages in the `arch` package
  repositories (#52). Here is an example of the warning:

  ```
  WARN  metapac::backends::arch > arch package "mesa-vdpau" was not found as an available package and so was ignored (you can test
  if the package exists via `pacman -Si "mesa-vdpau"` or similar command using your chosen AUR helper)

  it may be due to one of the following issues:
    - the package name has a typo as written in your group files
    - the package is a virtual package (https://wiki.archlinux.org/title/Pacman#Virtual_packages)
      and so is ambiguous. You can run `pacman -Ss "mesa-vdpau"` to list non-virtual packages which
      which provide the virtual package
    - the package was removed from the repositories
    - the package was renamed to a different name
    - the local package database is out of date and so doesn't yet contain the package:
      update it with `sudo pacman -Sy` or similar command using your chosen AUR helper
  ```

### Documentation

- Added release process to `CONTRIBUTING.md`

## [0.2.3] - 2024-11-14

### Fixed

- Fixed `winget` commands not working (#49)
- Fixed `metapac unmanaged` output backend names in lowercase (#49)

## [0.2.2] - 2024-11-10

### Added

- Added the `winget` Package Manager (#44)
- Added the `brew` Package Manager (#41)
- Added new test to de-duplicate the codebase by pulling the example config and
  group files directly from the README.md

### Fixed

- Fixed build errors and commands not being found on Windows (#44)
- Fixed the optional dependencies install option in `arch` packages being
  ignored (#39)
- Fixed `flatpak` package runtimes not being detected (#40)

### Documentation

- Improved the config and group file location documentation (#44, #45)

## [0.2.1] - 2024-10-29

### Documentation

- Update `cargo install` command to the README.md
- Added AUR build install command to README.md
- Rewrote `CONTRIBUTING.md` (#36)

### Added

- Added `pikaur` as another optional `arch` backend

### Fixed

- Fixed Install Options in group files being ignored (#30)

## [0.2.0] - 2024-10-20

### Build

- Adjust build automation

## [0.1.0] - 2024-10-20

### Added

- Initial release

# metapac

multi-backend declarative package manager

`metapac` allows you to maintain a consistent set of packages across
multiple machines. It also makes setting up a new system with your
preferred packages from your preferred package managers much easier.

## Obligatory XKCDs

[<img src="https://imgs.xkcd.com/comics/standards_2x.png" title="How Standards Proliferate" height="300"/>](https://xkcd.com/927/)
[<img src="https://imgs.xkcd.com/comics/universal_install_script_2x.png" title="Universal Install Script" height="300"/>](https://xkcd.com/1654/)

## Installation

### With Cargo

```shell
cargo install metapac
```

### With Arch User Repository

```shell
paru -S metapac
```

## Meta

`metapac` is a meta package manager, that means it does not directly
implement the functionality to install packages on your system, instead it
provides a standardised interface for installing packages from other
package managers. See the [Supported Backends](#supported-backends) section
for a list of the currently supported backend package managers.

## Declarative

`metapac` is also a declarative package manager, that means that you
declare in `.toml` group files the packages you would like installed on
your system and then run one of the `metapac` commands which read these
group files and then operate on your system to do some function such as
install packages in your group files that are not present on your system
yet (`metapac sync`), or remove packages present on your system but not in
your group files (`metapac clean`).

The group files are then stored with your other system configuration files
and so can be tracked with version control.

## Usage

### Enable backends

By default all backends are disabled. Enable the backends you want
`metapac` to manage in the config file. See the [`Config`](#config) section
for more details.

### Migrating a default system into `metapac`

Run `metapac unmanaged` and save the output into a group file in
`metapac`'s `groups/` folder, see the [`Group Files`](#group-files)
section for the exact location of this folder on your operating system.

For example, on linux this would mean:

```console
mkdir -p ~/.config/metapac/groups
metapac unmanaged > ~/.config/metapac/groups/all.toml
```

Now `metapac` won't try to remove any of your explicitly installed packages
when you run `metapac clean`.

> [!CAUTION]
> If you run `metapac clean` without first configuring your group files
> with the packages you want installed then `metapac` will attempt to
> remove all of your packages from your enabled backends.
>
> `metapac clean` will always show you which packages it intends to remove
> and ask for confirmation, so make sure to double check that the expected
> packages are being removed before confirming.

### Adding a new package

1. Edit your group files with a text editor to add the package to an
   existing group file or create a new group file and add the package to
   it. See the [`Group Files`](#group-files) section for the group file
   syntax
2. Run the `metapac add` command, see `metapac add --help` for arguments
3. Run the `metapac install` command, see `metapac install --help` for
   arguments

After the first two options you will then need to run `metapac sync` for
the newly added package to be installed, whereas for `metapac install` it
also installs the package while adding it to a group file.

> [!TIP]
> The first option is recommended since then you can group or organize the
> order of packages in your group files in a way that is meaningful to you
> and even add comments using the `toml` format.

### Removing a package

Do the opposite of [`Adding a new package`](#adding-a-new-package). The
opposite of `metapac add` is `metapac remove`, the opposite of `metapac
install` is `metapac uninstall` and the opposite of `metapac sync` is
`metapac clean`.

### Hooks

Hooks are commands that you can add per-package in your group files. They
get run by `metapac` at various stages in some of `metapac`'s commands.

One of the main use-case is to allow you to declaratively maintain your
enabled `systemd` services alongside each package in your group files. See
the [`Group Files`](#group-files) section for some examples.

- `before_install`: Run before a package is installed. Only applies to the
  `metapac sync` command.
- `after_install`: Run after a package is installed. Only applies to the
  `metapac sync` command.
- `before_sync`: Run before installing any packages, regardless of whether
  the package is already installed or not. Only applies to the `metapac
  sync` command.
- `after_sync`: Run after installing all packages, regardless of whether
  the package was already installed or not. Only applies to the `metapac
  sync` command.

### Advanced usage

For more advanced usage read through the remaining sections, especially the
[`Config`](#config) section. You can also run `metapac --help` to get a
list of all of the available commands.

## Supported Backends

At the moment, these are the supported backends. Pull requests and issues
for additional backends are always welcome!

| Backend               |
| --------------------- |
| [`apt`](#apt)         |
| [`arch`](#arch)       |
| [`brew`](#brew)       |
| [`bun`](#bun)         |
| [`cargo`](#cargo)     |
| [`dnf`](#dnf)         |
| [`flatpak`](#flatpak) |
| [`npm`](#npm)         |
| [`pipx`](#pipx)       |
| [`pnpm`](#pnpm)       |
| [`scoop`](#scoop)     |
| [`snap`](#snap)       |
| [`uv`](#uv)           |
| [`vscode`](#vscode)   |
| [`winget`](#winget)   |
| [`xbps`](#xbps)       |
| [`yarn`](#yarn)       |

### apt

### arch

#### Package Groups

Arch has two special types of packages called meta packages and package
groups. (See
<https://wiki.archlinux.org/title/Meta_package_and_package_group>).
`metapac` only supports meta packages in group files since they are "real"
packages whereas groups are not "real". This is because meta packages are
normal PKGBUILD files with no content of themselves but which have several
dependencies, whereas package groups are special cases that don't have a
corresponding PKGBUILD file. For example, running `pacman -Si nerd-fonts`
returns "error: package 'nerd-fonts' was not found".

If you still want the behavior of a meta package you have two options.

Firstly, consider creating your own meta package with the same packages as
the group. Consider also publishing this package to the AUR so other users
can also benefit from it. Convention has it that meta packages end in
`-meta`, for example, the meta package version of `nerd-fonts` might be
called `nerd-fonts-meta` (Although `nerd-fonts-meta` does not yet exist at
the time of writing, 2025-09-03).

Alternatively, you could create a new group file using the packages from
the package group, which you can get from the command: `pacman -Sgq
<group_name>`.

### brew

### bun

### cargo

### dnf

### flatpak

### npm

If on linux you might need to first run `npm config set prefix ~/.local`.

### pipx

### pnpm

You might need to first run `pnpm setup`.

### scoop

`scoop` doesn't differentiate between implicit and explicit packages.
Therefore, you will need to list all packages and their dependencies in
your group files. See
<https://github.com/ScoopInstaller/Scoop/issues/4276>.

### snap

### uv

### vscode

### winget

### xbps

### yarn

## Config

```toml
# metapac's config.toml file (like this one) should be placed in the following location
# dependent on the operating system as specified in the `dirs` crate:
# | Platform | Value                                                 | Example                                                      |
# | -------- | ----------------------------------------------------- | ------------------------------------------------------------ |
# | Linux    | $XDG_CONFIG_HOME or $HOME/.config/metapac/config.toml | /home/alice/.config/metapac/config.toml                      |
# | macOS    | $HOME/Library/Application Support/metapac/config.toml | /Users/Alice/Library/Application Support/metapac/config.toml |
# | Windows  | {FOLDERID_RoamingAppData}\metapac\config.toml         | C:\Users\Alice\AppData\Roaming\metapac\config.toml           |

# If this is `false` then the enabled backends will be found by using the value of
# the `enabled_backends` config.
# If this is `true` then the enabled backends will be found by using the
# [hostname_enabled_backends] config table.
#
# See the README.md or run `metapac backends` for the list of backend names.
# Default: false
hostname_enabled_backends_enabled = false

# Backends to enable. Subject to `hostname_enabled_backends_enabled`.
# Default: []
enabled_backends = ["arch", "cargo"]

# Backends to enable per hostname. Subject to `hostname_enabled_backends_enabled`.
# Default: None
[hostname_enabled_backends]
pc = ["winget", "cargo"]
laptop = ["arch", "cargo"]
server = ["apt"]

# If this is `false` all toml files recursively found in the groups folder
# will be used as group files.
# If this is `true` then the [hostname_groups] config table will be used to
# decide which group files to use per hostname.
# Default: false
hostname_groups_enabled = false

# Which group files will be used per hostname. Subject to `hostname_groups_enabled`.
# Relative paths are relative to the groups folder.
# Default: None
[hostname_groups]
pc = ["relative_group", "/etc/absolute_group"]
laptop = ["relative_group"]
server = ["relative_group"]

[arch]
# Since pacman, pamac, paru, pikaur and yay all operate on the same package database
# they are mutually exclusive and so you must pick which one you want
# metapac to use.
# Must be one of: ["pacman", "pamac", "paru", "pikaur", "yay"]
# Default: "pacman"
package_manager = "paru"

[cargo]
# Whether to default to installing cargo packages with the --locked option.
# Default: false
locked = false

[flatpak]
# Whether to default to installing flatpak packages systemwide or for the
# current user. This setting can be overridden on a per-package basis.
# Default: true
systemwide = true

[vscode]
# Since VSCode and VSCodium both operate on the same package database
# they are mutually exclusive and so you must pick which one you want
# metapac to use.
# Must be one of: ["code", "codium"]
# Default: "code"
variant = "code"
```

## Group Files

```toml
# metapac's group files (like this one) should be placed in the following location
# dependent on the operating system as specified in the `dirs` crate:
# | Platform | Value                                     | Example                                                  |
# | -------- | ----------------------------------------- | -------------------------------------------------------- |
# | Linux    | $XDG_CONFIG_HOME or $HOME/.config/groups/ | /home/alice/.config/metapac/groups/                      |
# | macOS    | $HOME/Library/Application Support/groups/ | /Users/Alice/Library/Application Support/metapac/groups/ |
# | Windows  | {FOLDERID_RoamingAppData}\groups\         | C:\Users\Alice\AppData\Roaming\metapac\groups\           |
#
# The packages for each backend in group files can come in two formats, short-form
# and long-form:
#
# short-form syntax is simply a string of the name of the package.
#
# long-form syntax is a table which contains several fields which can
# optionally be set to specify install options on a per-package basis.
# The "package" field in the table specifies the name of the package.
#
# For example, the following two packages are equivalent:
# arch = [
#  "metapac",
#  { package = "metapac" }
# ]

apt = ["package1", { package = "package2" }]
arch = [
  "package1",
  { package = "package2" },
  { package = "syncthing", hooks = { after_sync = [
    "sudo",
    "systemctl",
    "enable",
    "--now",
    "syncthing@ripytide",
  ] } },
  { package = "openssh", hooks = { after_sync = [
    "sudo",
    "systemctl",
    "enable",
    "--now",
    "sshd",
  ] } },
  { package = "fastfetch", hooks = { before_install = [
    "echo",
    "before_install",
  ], after_install = [
    "echo",
    "after_install",
  ], before_sync = [
    "echo",
    "before_sync",
  ], after_sync = [
    "echo",
    "after_sync",
  ] } },
]
brew = ["package1", { package = "package2" }]
bun = ["package1", { package = "package2" }]
cargo = [
  "package1",
  { package = "package2", options = { git = "https://github.com/ripytide/metapac", all_features = true, no_default_features = false, features = [
    "feature1",
  ], locked = true } },
]
dnf = [
  "package1",
  { package = "package2", options = { repo = "/etc/yum.repos.d/fedora_extras.repo" } },
]
flatpak = [
  "package1",
  { package = "package2", options = { remote = "flathub", systemwide = false } },
]
npm = ["package1", { package = "package2" }]
pipx = ["package1", { package = "package2" }]
pnpm = ["package1", { package = "package2" }]
scoop = ["main/metapac1", { package = "main/package2" }]
snap = [
  "package1",
  { package = "package2" },
  { package = "package3", options = { confinement = "strict" } },
  { package = "package4", options = { confinement = "classic" } },
  { package = "package5", options = { confinement = "dangerous" } },
  { package = "package6", options = { confinement = "devmode" } },
  { package = "package7", options = { confinement = "jailmode" } },
]
uv = ["package1", { package = "package2", options = { python = "3.11" } }]
vscode = ["package1", { package = "package2" }]
winget = ["ripytide.package1", { package = "ripytide.package2" }]
xbps = ["package1", { package = "package2" }]
yarn = ["package1", { package = "package2" }]
```

## Wishlist

Here is a list of package managers we would like to support along with any
reasons why we can't yet if any. Feel free to add to this list if you know
of any other package managers we should be aware of.

- [`cgwin`](https://cygwin.com/): no attempt made yet
- [`choco`](https://github.com/chocolatey/choco): no attempt made yet
- [`deno`](https://github.com/denoland/deno): can't list installed global
  packages <https://github.com/denoland/deno/discussions/28230>
- [`guix`](https://codeberg.org/guix/guix): no attempt made yet
- [`nix`](https://github.com/NixOS/nix): no attempt made yet
- [`pip`](https://pypi.org/project/pip/): we support `pipx` instead which
  only allows you to install cli programs which makes sense for a global
  package manager
- [`pkg`](https://github.com/freebsd/pkg): no attempt made yet
- [`ports`](https://github.com/openbsd/ports): no attempt made yet
- [`pkgsrc`](https://github.com/NetBSD/pkgsrc): no attempt made yet
- [`sdk`](https://github.com/sdkman/sdkman-cli): can't list installed
  packages <https://github.com/sdkman/sdkman-cli/issues/466>. The project
  is being rewritten in rust with the intention to implement the command in
  the new version <https://github.com/sdkman/sdkman-cli-native>, also see
  <https://github.com/ripytide/metapac/issues/86>
- [`yum`](https://github.com/rpm-software-management/yum): project
  deprecated in favor of `dnf`
- [`mise`](https://github.com/jdx/mise): no attempt made yet

## Similar Projects

- [decman](https://github.com/kiviktnm/decman): written in python,
  archlinux specific, supports installing dotfiles
- [declaro](https://github.com/mantinhas/declaro): written in shell script,
  currently provides support for `apt`, `dnf`, `pacman`, `paru` and `yay`
  but is extensible
- [pacdef](https://github.com/steven-omaha/pacdef): written in rust, custom
  file format, unmaintained, supported `pacman`, `apt`, `dnf`, `flatpak`,
  `pip`, `cargo`, `rustup` and `xbps`

## Credits

This project was forked from <https://github.com/steven-omaha/pacdef> so
credits to the author(s) of that project for all their prior work.

# metapac

multi-backend declarative package manager

`metapac` allows you to maintain a consistent set of packages across
multiple machines. It also makes setting up a new system with your
preferred packages from your preferred package managers much easier.

## Obligatory XKCDs

[<img src="https://imgs.xkcd.com/comics/standards_2x.png" title="How Standards Proliferate" height="300"/>](https://xkcd.com/927/)
[<img src="https://imgs.xkcd.com/comics/universal_install_script_2x.png" title="Universal Install Script" height="300"/>](https://xkcd.com/1654/)

## Installation

### Cargo

```shell
cargo install metapac
```

### Arch User Repository

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
`metapac` to manage by adding them to the `enabled_backends` config in
`metapac`'s config file. See the [`Config`](#config) section for more
details.

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
> remove all of your packages from your `enabled_backends`.
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

There are two hooks that allow you to run a custom command before and after
`metapac` installs your package: `before_install` and `after_install`.
These hooks are configured on a per-package basis. One use-case for `arch`
packages is to enable a package's `systemd` service after it is installed.
This means you can declaratively maintain your enabled `systemd` services.
See the [`Group Files`](#group-files) section for some examples.

> [!WARNING]
> A package's hooks are only run when the package is installed by
> `metapac`. If a package is already installed then running `metapac sync`
> will not run the hooks since it will not have to install the package. You
> can uninstall the package and then re-install it using `metapac sync`
> ensure the hooks are run.

### Advanced usage

For more advanced usage read through the remaining sections, especially the
[`Config`](#config) section. You can also run `metapac --help` to get a
list of all of the available commands.

## Supported Backends

At the moment, these are the supported backends. Pull requests and issues
for additional backends are always welcome!

| Backend   | Notes                                                                                                                                                                                                       |
| --------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `arch`    | see the `arch_package_manager` config                                                                                                                                                                       |
| `apt`     |                                                                                                                                                                                                             |
| `brew`    |                                                                                                                                                                                                             |
| `bun`     |                                                                                                                                                                                                             |
| `cargo`   |                                                                                                                                                                                                             |
| `dnf`     |                                                                                                                                                                                                             |
| `flatpak` |                                                                                                                                                                                                             |
| `npm`     | if on linux you might need to first run `npm config set prefix ~/.local`                                                                                                                                    |
| `pipx`    |                                                                                                                                                                                                             |
| `pnpm`    | you might need to first run `pnpm setup`                                                                                                                                                                    |
| `scoop`   | doesn't differentiate between implicit and explicit packages, you will need to list all packages and their dependencies in your group files (see <https://github.com/ScoopInstaller/Scoop/issues/4276>) |
| `snap`    |                                                                                                                                                                                                             |
| `uv`      |                                                                                                                                                                                                             |
| `vscode`  | see the `vscode_variant` config                                                                                                                                                                             |
| `winget`  |                                                                                                                                                                                                             |
| `xbps`    |                                                                                                                                                                                                             |
| `yarn`    |                                                                                                                                                                                                             |

## Config

```toml
# metapac's config.toml file (like this one) should be placed in the following location
# dependent on the operating system as specified in the `dirs` crate:
# | Platform | Value                                                 | Example                                                      |
# | -------- | ----------------------------------------------------- | ------------------------------------------------------------ |
# | Linux    | $XDG_CONFIG_HOME or $HOME/.config/metapac/config.toml | /home/alice/.config/metapac/config.toml                      |
# | macOS    | $HOME/Library/Application Support/metapac/config.toml | /Users/Alice/Library/Application Support/metapac/config.toml |
# | Windows  | {FOLDERID_RoamingAppData}\metapac\config.toml         | C:\Users\Alice\AppData\Roaming\metapac\config.toml           |

# Backends to enable for most of metapac's behavior. See the README.md or
# run `metapac backends` for the list of backend names
# Default: []
enabled_backends = ["arch"]

# Since pacman, pamac, paru, pikaur and yay all operate on the same package database
# they are mutually exclusive and so you must pick which one you want
# metapac to use.
# Must be one of: ["pacman", "pamac", "paru", "pikaur", "yay"]
# Default: "pacman"
arch_package_manager = "paru"

# Since VSCode and VSCodium both operate on the same package database
# they are mutually exclusive and so you must pick which one you want
# metapac to use.
# Must be one of: ["code", "codium"]
# Default: "code"
vscode_variant = "code"

# Whether to default to installing flatpak packages systemwide or for the
# current user. This setting can be overridden on a per-package basis.
# Default: true
flatpak_default_systemwide = true

# Whether to use the [hostname_groups] config table to decide which
# group files to use or to use all files in the groups folder.
# Default: false
hostname_groups_enabled = true

# Which group files apply for which hostnames
# paths starting without a / are relative to the groups folder
# Default: None
[hostname_groups]
pc = ["example_group"]
laptop = ["example_group"]
server = ["example_group"]

# Whether to default to installing cargo packages with the --locked option.
# Default: false
cargo_default_locked = true
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

arch = [
 "metapac",
 { package = "metapac" },
 { package = "syncthing", hooks = { after_install = [
    "sudo",
    "systemctl",
    "enable",
    "--now",
    "syncthing@ripytide",
 ] } },
 { package = "openssh", hooks = { after_install = [
    "sudo",
    "systemctl",
    "enable",
    "--now",
    "sshd",
 ] } },
]
apt = [
 "metapac",
 { package = "metapac" }
]
brew = [
 "metapac",
 { package = "metapac" }
]
bun = [
 "metapac",
 { package = "metapac" }
]
cargo = [
 "metapac",
 # see cargo docs for info on the options
 { package = "metapac", options = { git = "https://github.com/ripytide/metapac", all_features = true, no_default_features = false, features = [ "feature1", ], locked = true } },
]
dnf = [
 "metapac",
 # see dnf docs for more info on these options
 { package = "metapac", options = { repo = "/etc/yum.repos.d/fedora_extras.repo" } },
]
flatpak = [
 "metapac",
 { package = "metapac", options = { remote = "flathub", systemwide = false } },
]
npm = [
 "metapac",
 { package = "metapac" }
]
pipx = [
 "metapac",
 { package = "metapac" },
]
pnpm = [
 "metapac",
 { package = "metapac" },
]
scoop = [
 "main/metapac",
 { package = "main/metapac" },
]
snap = [
 "metapac",
 { package = "metapac" },
 # see https://snapcraft.io/docs/snap-confinement for more info on confinement
 { package = "metapac", options = { confinement = "strict" } },
 { package = "metapac", options = { confinement = "classic" } },
 { package = "metapac", options = { confinement = "dangerous" } },
 { package = "metapac", options = { confinement = "devmode" } },
 { package = "metapac", options = { confinement = "jailmode" } },
]
uv = [
 "metapac",
 { package = "metapac" },
]
vscode = [
 "metapac",
 { package = "metapac" },
]
winget = [
 "ripytide.metapac",
 { package = "ripytide.metapac" },
]
xbps = [
 "metapac",
 { package = "metapac" },
]
yarn = [
 "metapac",
 { package = "metapac" },
]
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

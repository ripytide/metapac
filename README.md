# metapac

multi-backend declarative package manager

`metapac` allows you maintain consistent packages among multiple machines
and make setting up a new system with your preferred packages from your
preferred package managers much easier.

## Installation

### Cargo

```shell
cargo install metapac
```

### Arch User Repository

```shell
paru -S metapac
```

```shell
paru -S metapac-bin
```

## Meta

`metapac` is a meta package manager, that means it does not directly
implement the functionality to install packages on your system, instead it
provides a standardised interface for installing packages from other
package managers. See the [Supported Backends](#supported-backends) section
for a list of the currently supported backend package managers.

## Declarative

`metapac` is a declarative package manager, that means that you declare in
`.toml` group files the packages you would like installed on your system
and then run one of the `metapac` commands which read these group files and
then operate on your system to do some function such as install packages in
your group files that are not present on your system yet (`metapac sync`),
or remove packages present on your system but not in your group files
(`metapac clean`).

The group files are then stored with your other system configuration files
and so can be tracked with version control.

## Supported Backends

At the moment, these are the supported backends. Pull Requests for adding
support for additional backends are welcome!

| Backend                        | Group Name  | Notes                                 |
| ------------------------------ | ----------- | ------------------------------------- |
| `pacman`/`paru`/`pikaur`/`yay` | `[arch]`    | see the `arch_package_manager` config |
| `apt`                          | `[apt]`     |                                       |
| `brew`                         | `[brew]`    |                                       |
| `cargo`                        | `[cargo]`   |                                       |
| `dnf`                          | `[dnf]`     |                                       |
| `flatpak`                      | `[flatpak]` |                                       |
| `pipx`                         | `[pipx]`    |                                       |
| `rustup`                       | `[rustup]`  |                                       |
| `snap`                         | `[snap]`    |                                       |
| `winget`                       | `[winget]`  |                                       |
| `xbps`                         | `[xbps]`    |                                       |

## Config

```toml
# metapac's config.toml file (like this one) should be placed in the following location
# dependent on the operating system as specified in the `dirs` crate:
# | Platform | Value                                                 | Example                                                      |
# | -------- | ----------------------------------------------------- | ------------------------------------------------------------ |
# | Linux    | $XDG_CONFIG_HOME or $HOME/.config/metapac/config.toml | /home/alice/.config/metapac/config.toml                      |
# | macOS    | $HOME/Library/Application Support/metapac/config.toml | /Users/Alice/Library/Application Support/metapac/config.toml |
# | Windows  | {FOLDERID_RoamingAppData}\metapac\config.toml         | C:\Users\Alice\AppData\Roaming\metapac\config.toml           |

# To decide which group files are relevant for the current machine
# metapac uses the machine's hostname in the hostname_groups table in
# the config file to get a list of group file names.

# Since pacman, paru, pikaur and yay all operate on the same package database
# they are mutually exclusive and so you must pick which one you want
# metapac to use.
# Must be one of: ["pacman", "paru", "pikaur", "yay"]
# Default: "pacman"
arch_package_manager = "paru"

# Whether to default to installing flatpak packages systemwide or for the current user.
# This setting can be overridden on a per-package basis using { systemwide = false|true }.
# Default: true
flatpak_default_systemwide = true

# Backends to disable from all metapac behavior. See the README.md for
# the list of backend names
# Default: []
disabled_backends = ["apt"]

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
 # optional_deps: additional packages to install with this package, short-form syntax only
 { package = "metapac",  optional_deps = ["git"] }
]
apt = [
 "metapac",
 { package = "metapac" }
]
brew = [
 "metapac",
 { package = "metapac" }
]
cargo = [
 "metapac",
 # see cargo docs for info on the options
 { package = "metapac", git = "https://github.com/ripytide/metapac", all_features = true, no_default_features = false, features = [ "feature1", ] },
]
dnf = [
 "metapac",
 # see dnf docs for more info on these options
 { package = "metapac", repo = "/etc/yum.repos.d/fedora_extras.repo" },
]
flatpak = [
 "metapac",
 { package = "metapac", remote = "flathub" }
]
pipx = [
 "metapac",
 { package = "metapac" }
]
rustup = [
 "stable",
 # components: extra non-default components to install with this toolchain
 { package = "stable", components = ["rust-analyzer"] }
]
snap = [
 "metapac",
 { package = "metapac" }
]
winget = [
 "metapac",
 { package = "metapac" }
]
xbps = [
 "metapac",
 { package = "metapac" }
]
```

# Credits

This project was forked from <https://github.com/steven-omaha/pacdef> so
credits to the author(s) of that project for all their prior work.

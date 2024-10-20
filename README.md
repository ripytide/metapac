# metapac

multi-backend declarative package manager

## Installation

### Cargo

```bash
cargo install metapac
```

### AUR

Coming soon.

## Use-case

`metapac` allows the user to have consistent packages among multiple
Linux machines and different backends by managing packages in group
files. The idea is that (1) any package in the group files ("managed
packages") will be installed explicitly, and (2) explicitly installed
packages _not_ found in any of the group files ("unmanaged packages")
will be removed. The group files are maintained outside of `metapac` by
any VCS, like git.

If you work with multiple Linux machines and have asked yourself "_Why
do I have the program that I use every day on my other machine not
installed here?_", then `metapac` is the tool for you.

## Multi-Backend

`metapac` is a sort of meta package manager in that it does not
directly possess the functionality to install packages on your system,
instead it provides a single standardised interface for a whole bunch
of other non-meta package managers. See the [Supported
Backends](#supported-backends) section for a list of the currently
supported backend package managers.

## Declarative

`metapac` is a declarative package manager, that means that you declare
in `.toml` group files the packages you would like installed on your
system and then run one of the `metapac` commands which read these
group files and then operate on your system to do some function such
as install packages in your group files that are not present on your
system yet (`metapac sync`), or remove packages present on your system
but not in your group files (`metapac clean`).

## Supported Backends

At the moment, supported backends are the following. Pull Requests for
additional backends are welcome!

| Backend               | Group Name  | Notes                                 |
| --------------------- | ----------- | ------------------------------------- |
| `pacman`/`paru`/`yay` | `[arch]`    | see the `arch_package_manager` config |
| `apt`                 | `[apt]`     |                                       |
| `dnf`                 | `[dnf]`     |                                       |
| `flatpak`             | `[flatpak]` |                                       |
| `pipx`                | `[pipx]`    |                                       |
| `cargo`               | `[cargo]`   |                                       |
| `rustup`              | `[rustup]`  |                                       |
| `xbps`                | `[xbps]`    |                                       |

## Config

```toml
# The metapac config.toml file is expected in the
# XDG_CONFIG_HOME/metapac directory (usually ~/.config/metapac/config.toml)
# unless using the --config-dir cli option.

# To decide which group files are relevant for the current machine
# metapac uses the machine's hostname in the hostname_groups table in
# the config file to get a list of group file names.

# Since pacman, yay and paru all operate on the same package database
# they are mutually exclusive and so you must pick which one you want
# metapac to use.
# Must be one of: ["pacman", "paru", "yay"]
# Default: "pacman"
arch_package_manager = "paru"

# Whether to install flatpak packages systemwide or for the current user.
# Default: true
flatpak_systemwide = true

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
# Group files (like this one) should be placed in the
# XDG_CONFIG_HOME/metapac directory (usually ~/.config/metapac/config.toml)
# unless using the --config-dir cli option.
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
# 	"metapac",
# 	{ package = "metapac" }
# ]

arch = [
	"metapac",
	# optional_deps: additional packages to install with this package, short-form syntax only
	{ package = "metapac",  optional_deps = ["git"] }
]
cargo = [
	"metapac",
	# see cargo docs for info on the options
	{ package = "metapac", git = "https://github.com/ripytide/metapac", all_features = true, no_default_features = false, features = [ "feature1", ] },
]
pipx = [
	"metapac",
	{ package = "metapac" }
]
apt = [
	"metapac",
	{ package = "metapac" }
]
xbps = [
	"metapac",
	{ package = "metapac" }
]
flatpak = [
	"metapac",
	{ package = "metapac" }
]
dnf = [
	"metapac",
	# see dnf docs for more info on these options
	{ package = "metapac", repo = "/etc/yum.repos.d/fedora_extras.repo" },
]
rustup = [
	"stable",
	# components: extra non-default components to install with this toolchain
	{ package = "stable", components = ["rust-analyzer"] }
]
```

# Credits

This project was forked from <https://github.com/steven-omaha/pacdef> so
credits to the author(s) of that project for all their prior work.

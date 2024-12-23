# metapac

multi-backend declarative package manager

`metapac` allows you to maintain a consistent set of packages across multiple machines. It also makes setting up a new system with your preferred packages from your preferred package managers much easier.

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

`metapac` is also a declarative package manager, that means that you
declare in `.toml` group files the packages you would like installed on
your system and then run one of the `metapac` commands which read these
group files and then operate on your system to do some function such as
install packages in your group files that are not present on your system
yet (`metapac sync`), or remove packages present on your system but not in
your group files (`metapac clean`).

The group files are then stored with your other system configuration files
and so can be tracked with version control.

## Getting Started

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
> remove all of your packages.
>
> `metapac clean` will always show you which packages it intends to remove
> and ask for confirmation, so make sure to double check that the expected
> packages are being removed before confirming.

### Adding a package to your group files

1. Edit your group files with a text editor to add the package to an
   existing group file or create a new group file and add the package to
   it. See the [`Group Files`](#group-files) section for the group file
   syntax
2. Run the `metapac add` command, see `metapac add --help` for arguments

After adding the package to your group files you can then run `metapac
sync` to install the newly added package.

> [!TIP]
> The first option is recommended since then you can group or organize the
> order of packages in your group files in a way that is meaningful to you
> and even add comments in the toml.

### Removing a package from your group files

Edit the group file which contains the package you want to remove and
delete the package from in the group file. You could even delete the entire
group file but that would remove all packages that are in the group file.

After removing the package from your group files you can then run `metapac
sync` to install the newly added package.

> [!TIP]
> There is not yet a equivalent to the `metapac add` command for removing
> packages from your group files, but even if there were a `metapac remove`
> command, using a text editor would still be the recommended method for the
> same reasons as for adding a package.

### Advanced usage

For more advanced usage read through the remaining sections, especially
the [`Config`](#config) section.

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
 { package = "metapac" }
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
 { package = "metapac", remote = "flathub", systemwide = false }
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

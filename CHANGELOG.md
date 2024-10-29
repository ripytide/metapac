# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2024-10-29

### 📚 Documentation

- Update install command (#34)
- Add author metadata
- Add AUR build commands
- Re-write CONTRIBUTING.md (#36)

### Build

- Add pkgbuild
- Update pkgbuilds
- Remove pkgbuilds into a their own repos

## [0.2.0] - 2024-10-20

### ⚙️ Miscellaneous Tasks

- Release metapac version 0.2.0

## [0.1.0] - 2024-10-20

### 🚀 Features

- *(review)* Print enumerated groups with leading spaces
- Implement --noconfirm
- *(groups)* Allow nested group dirs
- Warn about missing group only when relevant
- *(export)* CLI, data structures
- *(export)* Core logic
- *(export)* Second move method
- *(export)* Docstrings, man, README, switches
- *(export)* Zsh completion
- *(grouping)* Check duplicate packages in section
- *(pipx support)* Add support for pipx
- *(pipx support)* Fully supports pipx features
- *(python)* Pipx support
- *(review)* Add 'apply' reply
- *(rustup)* Add rustup as a backend
- *(rustup)* Add rustup as a module
- *(rustup)* Install packages
- *(rustup)* Remove packages
- *(rustup)* Add rustup backend
- *(rust)* Support $CARGO_HOME
- *(void)* Add void Linux backend
- *(fedora)* Add fedora as a backend
- *(fedora)* Use dnf repoquery to query packages
- *(backend)* Add fedora / dnf
- *(groups)* Add group file parsing

### 🐛 Bug Fixes

- *(review)* As dependency using wrong binary
- *(args)* Package subcommands order
- Build when git not installed
- Make descriptions consistent
- Privilege escalation for debian (#25)
- Pin libc version
- Don't overwrite config file
- Remove dbg
- Arg parsing
- Remove dbg statement
- *(group)* 'not a symlink' warning
- *(rust)* Handle missing crates file
- *(grouping)* Non-canonical ordering
- *(grouping)* Non-canonical ordering
- *(README)* Typo
- *(python)* Panic on empty pip_binary in config
- Minor typo in review text
- *(review)* Typo
- *(debian)* Build against rust-apt-0.7.0
- *(rustup)* Remove standalone components
- *(rustup)* Fix individual component uninstall
- *(cmd)* Show full command on error
- *(review)* Apply skips remaining backends
- *(rustup)* Finding installed components
- *(group)* Symlink warning for package operation
- *(debian)* Remove unused import
- *(fedora)* Package install, update and query
- *(fedora)* Fix linter warnings
- *(fedora)* AS_DEPENDENCY and package_info
- *(man)* Toml syntax
- *(core)* Track unmanaged packages once
- *(backend)* Docstring
- Docstring lint
- *(apt)* Temporarily remove apt as a backend
- *(arch)* Add arch to list of backends
- Fix errors

### 🚜 Refactor

- *(Backend)* Trait bound for PackageId
- *(run_cmd_for_stdout)* Trait bound
- *(packages)* Convert to macro only
- *(groups)* Filter iter instead of continue

### 📚 Documentation

- Complete Backend trait
- Add msrv
- Add panics section
- *(config)* Update `Config::load`
- Add docstrings to two methods
- Delete 'completion' subcommand
- *(README)* Add aliases
- Update todo, reminder in Config to update readme
- *(README)* Disable backends
- *(README)* Subcommands alphabetical order
- *(README)* Update for 1.0 release
- *(README)* Add references, links
- *(ui)* Add docstrings
- Fix spelling
- *(path)* Add docstrings
- *(search)* Add docstrings
- *(grouping)* Add docstrings
- *(README)* Add flatpack config value
- *(README)* Update config documentation
- *(README)* Describe `groups import`
- Extend 'get_args' docstring
- *(core)* Docstrings
- *(group)* Fix docstring
- *(man)* Add manpage template
- *(man)* Add pacdef.yaml.5
- *(man)* Add pacdef.8 content
- *(man)* Finalize pacdef.8
- *(core)* Remove outdated docstring
- *(README)* Link to topgrade
- Add release checklist
- *(core)* Docstrings
- *(core)* Docstrings
- Update release checklist
- Update release checklist
- *(core)* Add panics section
- *(README)* MSRV 1.70.0
- Minimum supported apt version
- Remove invalid link
- Bump MSRV to 1.74
- *(README)* Rustup
- *(issue-template)* Add bug report form
- *(issue-template)* Reproduction, formatting
- *(issue-template)* Multiline value
- *(issue-template)* Version shell render
- *(issue,bug)* Pacdef config file
- *(template)* Backend request
- *(template,backend)* Implement checkbox
- *(template)* Feature request
- *(template,bug)* Config yaml
- *(README)* AUR binary version
- *(env)* Should_print_debug_info
- *(README)* Update install section
- *(fedora)* Switches and show_package_info
- *(README)* Add fedora backend
- *(man)* Add pip_binary config value
- Fix toml syntax

### 🎨 Styling

- *(rustup)* Add empty lines
- *(fedora)* Comments and empty lines
- *(fedora)* Implicit types and module consts
- *(packages)* Rename variables

### 🧪 Testing

- Add rstest crate, argument parsing tests
- *(args)* Add negative test cases

### ⚙️ Miscellaneous Tasks

- Release
- Release
- Release
- Release
- Release
- Release
- Update lockfile
- Bump dependencies
- Release
- *(bacon)* Add bacon config
- Update Cargo.lock
- Release
- Reformat a docstring
- Setup github check workflow
- Try triggering github action
- Install git
- Add workflow dispatch trigger
- Rename workflow
- Add badge
- Release
- Release
- Activate for devel branch
- Bump lockfile
- Release
- Run tests
- Update checkout to v3
- Update bacon config
- Fix docs.rs
- Release
- Release
- *(cliff)* Add config
- Bump dependencies
- Release
- Release
- Release
- Update subcrate repository urls
- Bump man pages
- *(cliff)* Update config
- *(cliff)* Update config
- *(release)* Update lockfile
- *(release)* Bump man pages
- Release
- Bump dependencies
- Bump dependencies
- *(release)* Update lockfile
- *(release)* Bump man pages
- Release
- *(release)* Update lockfile
- *(release)* Bump man pages
- Release
- *(release)* Update lockfile
- *(release)* Bump man pages
- Release
- *(release)* Update lockfile
- *(release)* Bump man pages
- Release
- *(release)* Bump dependencies, update lockfile
- *(release)* Bump man pages
- Release
- Fail on any clippy warning
- Build, clippy for all features
- Enable tests for all features
- Build binary on release
- *(README)* Update README with rustup
- *(README)* Update README with Rustup details
- *(check)* Keep going after failure
- *(release)* Update lockfile
- *(release)* Bump man pages
- Release
- *(release)* Add zsh completion
- *(release)* Update lockfile
- *(release)* Bump dependencies
- *(release)* Bump man pages
- Release
- *(release)* Fix adding zsh completion

### Build

- Set msrv in Cargo.toml
- Update build script
- Update build script

### Refact

- *(backend)* Remove dead code
- *(backend)* Get_group_packages_map
- *(python)* Replace unwrap
- *(backend)* Use sort_unstable
- *(regex)* Disable default features
- *(review)* Handle upper-case input
- *(main)* Extract load_default_config
- *(main)* Major update message
- *(review)* User intention query
- *(backend)* Make macro crate-public
- Overhaul arg parsing
- *(core)* Arg destructuring
- *(core)* Remove stale lint config
- *(core)* Simplify valid group name check
- *(main)* Create_empty_config_file
- Unncessary wraps
- Unncessary wraps
- Replace match with if let else
- Use clone instead to_owned
- *(core)* Manual let else
- *(core)* Remove get_group_file_paths...
- *(ui)* Infallible conversion for u8 to char
- *(package.rs)* Expose repo field publicly
- *(rustup)* Fetch installed toolchains and components
- *(rustup)* Refector install_packages
- *(rustup)* Use anyhow::Error
- *(rustup)* Remove clippy warnings
- *(rustup)* Change methods to functions
- *(rustup)* Refactor component installation
- *(rustup)* Apply clippy suggestions
- *(packaging)* Remove unused function
- *(rustup)* Use RepoType instead of strings
- *(rustup)* Use bail instead of panic
- *(rustup)* Use match statements and bail
- *(rustup)* Make_dependency panic
- *(rustup)* Get_all_installed_packages
- *(rustup)* Remove unused mut
- *(rustup)* Add todo
- *(rustup)* Rework most of the code
- *(rustup)* Add run_external_command
- *(rustup)* Use run_external_command
- *(rustup)* Install packages
- *(env)* Should_print_debug_info
- *(rustup)* Introduce modules
- *(rustup)* Getting switches per repotype
- *(rustup)* RustupPackage::from_pacdef_packages
- *(backend)* Add todo
- *(backend)* Cleanup
- *(cmd)* Dont return exitstatus
- Use Result instead of ExitStatus
- Add todo
- Todo_per_backend
- *(fedora)* Changes in Backend trait
- *(fedora)* Package creation from output
- *(fedora)* Move fetch flags to constants
- *(fedora)* Remove core::panic
- Virtual manifest
- *(backend)* Static dispatch
- *(core,cli)* Declarative CLI approach
- *(config)* [**breaking**] Use toml instead of yaml
- *(logging)* Use 'log' logging facade
- *(backend,grouping)* Overhaul

<!-- generated by git-cliff -->

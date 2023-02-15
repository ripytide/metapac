use std::collections::HashSet;
use std::fmt::Display;
use std::fs::{read_to_string, File};
use std::hash::Hash;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::{Package, Section};

/// Representation of a group file, composed of a name (file name from which it
/// was read), the sections in the file, and the absolute path of the original
/// file.
#[derive(Debug)]
pub struct Group {
    pub(crate) name: String,
    pub(crate) sections: HashSet<Section>,
    pub(crate) path: PathBuf,
}

impl Group {
    /// Load all group files from the pacdef group dir.
    ///
    /// This method will print a warning if
    /// - there are no files under `group_dir`, or
    /// - `warn_not_symlinks` is true and a group file is not a symlink.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the files under `group_dir` cannot
    /// be accessed.
    pub fn load(group_dir: &Path, warn_not_symlinks: bool) -> Result<HashSet<Self>> {
        let mut result = HashSet::new();

        if !group_dir.is_dir() {
            // we only need to create the innermost dir. The rest was already created from when
            // we loaded the config
            std::fs::create_dir(group_dir)
                .with_context(|| format!("creating group dir {}", group_dir.to_string_lossy()))?;
        }

        for entry in group_dir.read_dir().context("reading group dir")? {
            let file = entry.context("getting group file")?;
            let path = file.path();

            if warn_not_symlinks && !path.is_symlink() {
                eprintln!(
                    "WARNING: group file {} is not a symlink",
                    path.to_string_lossy()
                );
            }

            let group =
                Self::try_from(&path).with_context(|| format!("reading group file {path:?}"))?;

            result.insert(group);
        }

        if result.is_empty() {
            eprintln!("WARNING: no group files found");
        }

        Ok(result)
    }
}

impl PartialOrd for Group {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.name.partial_cmp(&other.name) {
            Some(core::cmp::Ordering::Equal) => None,
            ord => ord,
        }
    }
}

impl Ord for Group {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Hash for Group {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Group {
    fn assert_receiver_is_total_eq(&self) {}
}

impl Group {
    fn try_from<P>(p: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = p.as_ref();
        let content = read_to_string(path).context("reading file content")?;
        let name = path
            .file_name()
            .context("getting file name")?
            .to_string_lossy()
            .to_string();

        let mut lines = content.lines().peekable();
        let mut sections = HashSet::new();

        while lines.peek().is_some() {
            let result = Section::try_from_lines(&mut lines).context("reading section");
            match result {
                Ok(section) => {
                    sections.insert(section);
                }
                Err(e) => {
                    let err = e.root_cause();
                    eprintln!("WARNING: could not process a section under group '{name}': {err}");
                }
            }
        }

        if sections.is_empty() {
            eprintln!("WARNING: no sections found in group '{name}'");
        }

        let path = path.into();

        Ok(Self {
            name,
            sections,
            path,
        })
    }

    pub(crate) fn save_packages(&self, section_header: &str, packages: &[Package]) -> Result<()> {
        let mut content = read_to_string(&self.path)
            .with_context(|| format!("reading existing file contents from {:?}", &self.path))?;

        if content.contains(section_header) {
            write_packages_to_existing_section(&mut content, section_header, packages)
                .context("existing section")?;
        } else {
            add_new_section_with_packages(&mut content, section_header, packages);
        }

        let mut file = File::create(&self.path)
            .with_context(|| format!("creating descriptor to output file {:?}", &self.path))?;

        write!(file, "{content}").with_context(|| format!("writing file {:?}", &self.path))
    }
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sections: Vec<_> = self.sections.iter().collect();
        sections.sort_unstable();

        let mut iter = sections.into_iter().peekable();

        while let Some(section) = iter.next() {
            section.fmt(f)?;
            if iter.peek().is_some() {
                f.write_str("\n\n")?;
            }
        }
        Ok(())
    }
}

fn write_packages_to_existing_section(
    group_file_content: &mut String,
    section_header: &str,
    packages: &[Package],
) -> Result<()> {
    let idx_of_first_package_line_in_section =
        find_first_package_line_in_section(group_file_content, section_header)?;

    let after = group_file_content.split_off(idx_of_first_package_line_in_section);

    for p in packages {
        group_file_content.push_str(&format!("{p}\n"));
    }

    group_file_content.push_str(&after);
    Ok(())
}

fn find_first_package_line_in_section(
    group_file_content: &str,
    section_header: &str,
) -> Result<usize> {
    let section_start = group_file_content
        .find(section_header)
        .context("finding first package after section header")?;

    let distance_to_next_newline = group_file_content[section_start..]
        .find('\n')
        .context("getting next newline")?;

    Ok(section_start + distance_to_next_newline + 1) // + 1 to be after the newline
}

fn add_new_section_with_packages(
    group_file_content: &mut String,
    section_header: &str,
    packages: &[Package],
) {
    group_file_content.push('\n');
    group_file_content.push_str(section_header);
    group_file_content.push('\n');
    for p in packages {
        group_file_content.push_str(&format!("{p}\n"));
    }
}

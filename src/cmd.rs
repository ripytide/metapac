use std::process::{Command, Stdio};

use color_eyre::{Result, eyre::eyre};
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub enum Perms {
    Sudo,
    Same,
}

pub fn run_command_for_stdout<I, S>(args: I, perms: Perms, hide_stderr: bool) -> Result<String>
where
    S: Into<String>,
    I: IntoIterator<Item = S>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect::<Vec<_>>();

    if args.is_empty() {
        return Err(eyre!("cannot run an empty command"));
    }

    let use_sudo = use_sudo(perms)?;
    let args = Some("sudo".to_string())
        .filter(|_| use_sudo)
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();

    let (first_arg, remaining_args) = args.split_first().unwrap();

    let mut command = Command::new(first_arg);
    let output = command
        .args(remaining_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(if !hide_stderr {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(String::from_utf8(output.stdout)?),
        Ok(output) => Err(eyre!(
            "command failed: {:?}, exit_status_code: {:?}",
            args.into_iter().join(" "),
            output.status
        )),
        Err(err) => Err(eyre!(
            "command failed: {:?}, error: {:?}",
            args.into_iter().join(" "),
            err
        )),
    }
}

pub fn run_command<I, S>(args: I, perms: Perms) -> Result<()>
where
    S: Into<String>,
    I: IntoIterator<Item = S>,
{
    let args: Vec<String> = args.into_iter().map(Into::into).collect::<Vec<_>>();

    if args.is_empty() {
        return Err(eyre!("cannot run an empty command"));
    }

    let use_sudo = use_sudo(perms)?;
    let args = Some("sudo".to_string())
        .filter(|_| use_sudo)
        .into_iter()
        .chain(args)
        .collect::<Vec<_>>();

    let (first_arg, remaining_args) = args.split_first().unwrap();

    let mut command = Command::new(first_arg);
    let status = command
        .args(remaining_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(eyre!(
            "command failed: {:?}, exit_status_code: {:?}",
            args.into_iter().join(" "),
            status
        )),
        Err(err) => Err(eyre!(
            "command failed: {:?}, error: {:?}",
            args.into_iter().join(" "),
            err
        )),
    }
}

fn use_sudo(perms: Perms) -> Result<bool> {
    #[cfg(unix)]
    match perms {
        Perms::Same => Ok(false),
        Perms::Sudo => Ok(unsafe { libc::geteuid() } != 0),
    }
    #[cfg(windows)]
    match perms {
        Perms::Same => Ok(false),
        Perms::Sudo => Err(eyre!(
            "sudo for privilege escalation is not supported on windows"
        )),
    }
}

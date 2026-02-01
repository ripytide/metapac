use std::{
    collections::VecDeque,
    process::{Command, Stdio},
    time::Instant,
};

use color_eyre::{Result, eyre::eyre};
use itertools::Itertools;

#[derive(Debug, Clone, Copy)]
pub enum Perms {
    Sudo,
    Same,
}

#[derive(Debug, Clone, Copy)]
pub enum StdErr {
    Show,
    Hide,
}

pub fn run_command_for_stdout<I, S>(args: I, perms: Perms, stderr: StdErr) -> Result<String>
where
    S: Into<String>,
    I: IntoIterator<Item = S>,
{
    let args: VecDeque<String> = args.into_iter().map(Into::into).collect();

    if args.is_empty() {
        return Err(eyre!("cannot run an empty command"));
    }

    let args = get_args(args, perms)?;
    let args = args.into_iter().collect::<Vec<_>>();

    let (first_arg, remaining_args) = args.split_first().unwrap();

    let mut command = Command::new(first_arg);

    log::debug!("running command: {command:?} with args: {remaining_args:?}");

    let start = Instant::now();

    let output = command
        .args(remaining_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(if matches!(stderr, StdErr::Show) {
            Stdio::inherit()
        } else {
            Stdio::null()
        })
        .output();

    log::trace!("command took {:.2} seconds", start.elapsed().as_secs_f64());

    match output {
        Ok(output) if output.status.success() => {
            log::trace!("command succeeded, status: {}", output.status);
            Ok(String::from_utf8(output.stdout)?)
        }
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
    let args: VecDeque<String> = args.into_iter().map(Into::into).collect();

    if args.is_empty() {
        return Err(eyre!("cannot run an empty command"));
    }

    let args = get_args(args, perms)?;
    let args = args.into_iter().collect::<Vec<_>>();

    let (first_arg, remaining_args) = args.split_first().unwrap();

    let mut command = Command::new(first_arg);

    log::debug!("running command: {command:?} with args: {remaining_args:?}");

    let start = Instant::now();

    let status = command
        .args(remaining_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    log::trace!("command took {:.2} seconds", start.elapsed().as_secs_f64());

    match status {
        Ok(status) if status.success() => {
            log::trace!("command succeeded, status: {status}");
            Ok(())
        }
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

#[allow(clippy::unnecessary_wraps)]
fn get_args(mut args: VecDeque<String>, perms: Perms) -> Result<VecDeque<String>> {
    #[cfg(unix)]
    match perms {
        Perms::Same => Ok(args),
        Perms::Sudo => {
            if unsafe { libc::geteuid() } != 0 {
                args.push_front("sudo".to_string());
            }
            Ok(args)
        }
    }
    #[cfg(windows)]
    match perms {
        // to enable .pw1 and .cmd files being executed such as npm.ps1, (see #184)
        Perms::Same => {
            args.push_front("/C".to_string());
            args.push_front("cmd".to_string());
            Ok(args)
        }
        Perms::Sudo => {
            return Err(eyre!(
                "sudo for privilege escalation is not supported on windows"
            ));
        }
    }
}

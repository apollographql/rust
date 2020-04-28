use atlas::errors::{ExitCode, Fallible};
use atlas::style;
use log::info;
use structopt::StructOpt;

use crate::commands::Command;

#[derive(StructOpt)]
pub struct Setup {}

impl Command for Setup {
    fn run(self) -> Fallible<ExitCode> {
        os::setup_environment()?;

        info!(
            "{} Setup complete. Open a new terminal to start using the Apollo CLI!",
            style::ROCKET
        );

        Ok(ExitCode::Success)
    }
}

#[cfg(unix)]
mod os {
    use std::env;
    use std::fs::File;
    use std::io::{self, BufRead, BufReader, Write};
    use std::path::Path;

    use atlas::errors::{ErrorDetails, Fallible};
    use atlas::layout::apollo_home_bin;
    use log::{debug, warn};

    const PROFILES: [&str; 5] = [
        ".profile",
        ".bash_profile",
        ".bashrc",
        ".zshrc",
        ".config/fish/config.fish",
    ];

    pub fn setup_environment() -> Fallible<()> {
        let user_home_dir = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;

        debug!("Searching for profiles to update");
        let env_profile = env::var("PROFILE");

        let found_profile = PROFILES
            .iter()
            .chain(&env_profile.as_ref().map(String::as_str))
            .fold(false, |prev, path| {
                let profile = user_home_dir.join(path);
                match read_profile_without_apollo(&profile) {
                    Some(contents) => {
                        debug!("Profile script found: {}", profile.display());

                        let write_profile = match profile.extension() {
                            Some(ext) if ext == "fish" => write_profile_fish,
                            _ => write_profile_sh,
                        };

                        match write_profile(&profile, contents) {
                            Ok(()) => {
                                debug!("Wrote $PATH addition into {}", profile.display());
                                true
                            }
                            Err(err) => {
                                warn!(
                                    "Found profile script, but could not modify it: {}",
                                    profile.display()
                                );
                                debug!("Profile modification error: {}", err);
                                prev
                            }
                        }
                    }
                    None => {
                        debug!("Profile script not found: {}", profile.display());
                        prev
                    }
                }
            });

        if found_profile {
            Ok(())
        } else {
            Err(ErrorDetails::NoShellProfile {
                env_profile: String::new(),
                bin_dir: apollo_home_bin()?.to_owned(),
            }
            .into())
        }
    }

    fn read_profile_without_apollo(path: &Path) -> Option<String> {
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        reader
            .lines()
            .filter(|line_result| match line_result {
                Ok(line) if !line.contains(".apollo") => true,
                Ok(_) => false,
                Err(_) => true,
            })
            .collect::<io::Result<Vec<String>>>()
            .map(|lines| lines.join("\n"))
            .ok()
    }

    fn write_profile_sh(path: &Path, contents: String) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(
            file,
            "{}\nexport PATH=\"$HOME/.apollo/bin:$PATH\"\n",
            contents,
        )
    }

    fn write_profile_fish(path: &Path, contents: String) -> io::Result<()> {
        let mut file = File::create(path)?;
        write!(
            file,
            "{}\nset -gx PATH \"$HOME/.apollo/bin\" $PATH\n",
            contents,
        )
    }
}

#[cfg(windows)]
mod os {
    use std::process::Command;

    use atlas::errors::{ErrorDetails, Fallible};
    use atlas::layout::apollo_home_bin;
    use log::debug;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    pub fn setup_environment() -> Fallible<()> {
        let bin_dir = apollo_home_bin().to_string_lossy().to_string();
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey("Environment")
            .with_context(|_| ErrorDetails::ReadUserPathError)?;
        let path: String = env
            .get_value("Path")
            .with_context(|_| ErrorDetails::ReadUserPathError)?;

        if !path.contains(&bin_dir) {
            // Use `setx` command to edit the user Path environment variable
            let mut command = Command::new("setx");
            command.arg("Path");
            command.arg(format!("{};{}", shim_dir, path));

            debug!("Modifying User Path with command: {:?}", command);
            let output = command
                .output()
                .with_context(|_| ErrorDetails::WriteUserPathError)?;

            if !output.status.success() {
                debug!("[setx stderr]\n{}", String::from_utf8_lossy(&output.stderr));
                debug!("[setx stdout]\n{}", String::from_utf8_lossy(&output.stdout));
                return Err(ErrorDetails::WriteUserPathError.into());
            }
        }

        Ok(())
    }
}

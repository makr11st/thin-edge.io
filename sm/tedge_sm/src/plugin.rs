use crate::{
    error::SoftwareError,
    software::*,
};
use std::{
    iter::Iterator,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

pub trait Plugin {
    fn prepare(&self) -> Result<(), SoftwareError>;
    fn install(&self, module: &SoftwareModule) -> Result<(), SoftwareError>;
    fn remove(&self, module: &SoftwareModule) -> Result<(), SoftwareError>;
    fn finalize(&self) -> Result<(), SoftwareError>;
    fn list(&self) -> Result<Vec<SoftwareModule>, SoftwareError>;
    fn version(&self, module: &SoftwareModule) -> Result<Option<String>, SoftwareError>;

    fn apply(&self, update: &SoftwareModuleUpdate) -> Result<(), SoftwareError> {
        match update {
            SoftwareModuleUpdate::Install { module} => self.install(&module),
            SoftwareModuleUpdate::Remove { module} => self.remove(&module),
        }
    }

    fn apply_all(&self, updates: &Vec<SoftwareModuleUpdate>) -> Vec<SoftwareModuleUpdateResult> {
        let mut failed_updates = Vec::new();
        self.prepare();
        for update in updates.iter() {
            if let Err(error) = self.apply(update) {
                let () = failed_updates.push(SoftwareModuleUpdateResult {
                    update: update.clone(),
                    error: Some(error),
                });
            };
        }
        failed_updates
    }
}

#[derive(Debug)]
pub struct ExternalPluginCommand {
    pub name: SoftwareType,
    pub path: PathBuf,
}

impl ExternalPluginCommand {
    pub fn new(name: impl Into<SoftwareType>, path: impl Into<PathBuf>) -> ExternalPluginCommand {
        ExternalPluginCommand {
            name: name.into(),
            path: path.into(),
        }
    }

    pub fn command(
        &self,
        action: &str,
        maybe_module: Option<&SoftwareModule>,
    ) -> Result<Command, SoftwareError> {
        let mut command = Command::new(&self.path);
        command.arg(action);

        if let Some(module) = maybe_module {
            // self.check_module_type(module)?;
            command.arg(&module.name);
            if let Some(ref version) = module.version {
                command.arg(version);
            }
        }

        command
            .current_dir("/tmp")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        Ok(command)
    }

    pub fn execute(&self, mut command: Command) -> Result<Output, SoftwareError> {
        let output = command.output().map_err(|err| self.plugin_error(err))?;
        Ok(output)
    }

    pub fn content(&self, bytes: Vec<u8>) -> Result<String, SoftwareError> {
        String::from_utf8(bytes).map_err(|err| self.plugin_error(err))
    }

    pub fn plugin_error(&self, err: impl std::fmt::Display) -> SoftwareError {
        SoftwareError::Plugin {
            software_type: self.name.clone(),
            reason: format!("{}", err),
        }
    }
}

const PREPARE: &str = "prepare";
const INSTALL: &str = "install";
const REMOVE: &str = "remove";
const FINALIZE: &str = "finalize";
const LIST: &str = "list";
const VERSION: &str = "version";

impl Plugin for ExternalPluginCommand {
    fn prepare(&self) -> Result<(), SoftwareError> {
        let command = self.command(PREPARE, None)?;
        let output = self.execute(command)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SoftwareError::Prepare {
                reason: self.content(output.stderr)?,
            })
        }
    }

    fn install(&self, module: &SoftwareModule) -> Result<(), SoftwareError> {
        let command = self.command(INSTALL, Some(module))?;
        let output = self.execute(command)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SoftwareError::Install {
                module: module.clone(),
                reason: self.content(output.stderr)?,
            })
        }
    }

    fn remove(&self, module: &SoftwareModule) -> Result<(), SoftwareError> {
        let command = self.command(REMOVE, Some(module))?;
        let output = self.execute(command)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SoftwareError::Uninstall {
                module: module.clone(),
                reason: self.content(output.stderr)?,
            })
        }
    }

    fn finalize(&self) -> Result<(), SoftwareError> {
        let command = self.command(FINALIZE, None)?;
        let output = self.execute(command)?;

        if output.status.success() {
            Ok(())
        } else {
            Err(SoftwareError::Finalize {
                reason: self.content(output.stderr)?,
            })
        }
    }

    fn list(&self) -> Result<Vec<SoftwareModule>, SoftwareError> {
        let command = self.command(LIST, None)?;
        let output = self.execute(command)?;

        if output.status.success() {
            let mut software_list = Vec::new();
            let mystr = output.stdout;

            mystr
                .split(|n: &u8| n.is_ascii_whitespace())
                .filter(|split| !split.is_empty())
                .for_each(|split: &[u8]| {
                    let software_json_line = std::str::from_utf8(split).unwrap();
                    let software_module =
                        serde_json::from_str::<SoftwareModule>(software_json_line).unwrap();
                    software_list.push(software_module);
                });

            dbg!(&software_list);
            Ok(software_list)
        } else {
            Err(SoftwareError::Plugin {
                software_type: self.name.clone(),
                reason: self.content(output.stderr)?,
            })
        }
    }

    fn version(&self, module: &SoftwareModule) -> Result<Option<String>, SoftwareError> {
        let command = self.command(VERSION, Some(module))?;
        let output = self.execute(command)?;

        if output.status.success() {
            let version = String::from(self.content(output.stdout)?.trim());
            if version.is_empty() {
                Ok(None)
            } else {
                Ok(Some(version))
            }
        } else {
            Err(SoftwareError::Plugin {
                software_type: self.name.clone(),
                reason: self.content(output.stderr)?,
            })
        }
    }
}

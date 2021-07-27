use crate::{
    message::{
        self, ListSoftwareListResponseList, SoftwareListModule, SoftwareListResponseList,
        SoftwareModulesUpdateRequest, SoftwareOperationResultStatus, SoftwareRequestResponse,
        SoftwareRequestUpdateAction, SoftwareRequestUpdateList,
    },
    plugin::*,
    software::*,
};
use std::{collections::HashMap, fs, io, path::PathBuf};
use tedge_users::{UserManager, ROOT_USER};

/// The main responsibility of a `Plugins` implementation is to retrieve the appropriate plugin for a given software module.
pub trait Plugins {
    type Plugin;

    /// Return the plugin to be used by default when installing a software module, if any.
    fn default(&self) -> Option<&Self::Plugin>;

    /// Return the plugin declared with the given name, if any.
    fn by_software_type(&self, software_type: &str) -> Option<&Self::Plugin>;

    /// Return the plugin associated with the file extension of the module name, if any.
    fn by_file_extension(&self, module_name: &str) -> Option<&Self::Plugin>;

    fn plugin(&self, software_type: &str) -> Result<&Self::Plugin, SoftwareError> {
        let module_plugin = self.by_software_type(software_type).ok_or_else(|| {
            SoftwareError::UnknownSoftwareType {
                software_type: software_type.into(),
            }
        })?;

        Ok(module_plugin)
    }
}

// type PluginName = String;
#[derive(Debug)]
pub struct ExternalPlugins {
    plugin_dir: PathBuf,
    plugin_map: HashMap<String, ExternalPluginCommand>,
}

impl Plugins for ExternalPlugins {
    type Plugin = ExternalPluginCommand;

    fn default(&self) -> Option<&Self::Plugin> {
        self.by_software_type("default")
    }

    fn by_software_type(&self, software_type: &str) -> Option<&Self::Plugin> {
        self.plugin_map.get(software_type)
    }

    fn by_file_extension(&self, module_name: &str) -> Option<&Self::Plugin> {
        if let Some(dot) = module_name.rfind('.') {
            let (_, extension) = module_name.split_at(dot + 1);
            self.by_software_type(extension)
        } else {
            self.default()
        }
    }
}

impl ExternalPlugins {
    pub fn open(plugin_dir: impl Into<PathBuf>) -> io::Result<ExternalPlugins> {
        let mut plugins = ExternalPlugins {
            plugin_dir: plugin_dir.into(),
            plugin_map: HashMap::new(),
        };
        let () = plugins.load()?;
        Ok(plugins)
    }

    pub fn load(&mut self) -> io::Result<()> {
        self.plugin_map.clear();
        for maybe_entry in fs::read_dir(&self.plugin_dir)? {
            let entry = maybe_entry?;
            let path = entry.path();
            if path.is_file() {
                // TODO check the file is exec

                if let Some(file_name) = path.file_name() {
                    if let Some(plugin_name) = file_name.to_str() {
                        let plugin = ExternalPluginCommand::new(plugin_name, &path);
                        self.plugin_map.insert(plugin_name.into(), plugin);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn empty(&self) -> bool {
        self.plugin_map.is_empty()
    }

    pub fn list(&self) -> Result<ListSoftwareListResponseList, SoftwareError> {
        let mut complete_software_list = Vec::new();
        for software_type in self.plugin_map.keys() {
            let plugin_software_list = self.plugin(&software_type)?.list()?;
            complete_software_list.push(plugin_software_list);
        }
        Ok(complete_software_list)
    }

    pub fn process(
        &self,
        request: &message::SoftwareRequestUpdate,
    ) -> Result<SoftwareRequestResponse, SoftwareError> {
        // TODO move this in the aglet _user_guard = self.user_manager.become_user(ROOT_USER)?;

        let mut response = SoftwareRequestResponse {
            id: request.id,
            status: SoftwareOperationResultStatus::Failed,
            reason: None,
            current_software_list: None,
            failures: None,
        };

        let mut failures = Vec::new();

        for software_list_type in request.update_list {
            let plugin = self
                .by_software_type(&software_list_type.plugin_type)
                .unwrap();

            // What to do if prepare fails?
            // What should be in failures list?
            if let Err(e) = plugin.prepare() {
                response.reason = Some(format!("Failed prepare stage: {}", e));

                continue;
            };

            let failures_modules = self.install_or_remove(&software_list_type.list, plugin);

            let () = plugin.finalize()?;

            failures.push(failures_modules);
        }

        Ok(response)
    }

    fn install_or_remove(
        &self,
        software_list_type: &Vec<SoftwareModulesUpdateRequest>,
        plugin: &ExternalPluginCommand,
    ) -> SoftwareListResponseList {
        let mut failures_modules = Vec::new();

        for module in software_list_type.into_iter() {
            let status = match module.action {
                SoftwareRequestUpdateAction::Install => plugin.install(&module),
                SoftwareRequestUpdateAction::Remove => plugin.remove(&module),
            };

            if let Err(_) = status {
                let mut error = module.clone();
                error.reason = Some("Action failed".into());
                let () = failures_modules.push(error);
            };
        }
    }
}
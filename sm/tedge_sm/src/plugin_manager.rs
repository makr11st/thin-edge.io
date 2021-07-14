use crate::message::SoftwareRequestUpdateAction;
use crate::message::SoftwareRequestUpdateModule;
use crate::message::SoftwareRequestUpdateStatus;
use crate::plugin::*;

use crate::software::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::ops::Deref;
use std::path::PathBuf;

/// The main responsibility of a `Plugins` implementation is to retrieve the appropriate plugin for a given software module.
///
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
        dbg!();
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
        dbg!(&self.plugin_map);

        Ok(())
    }

    pub fn empty(&self) -> bool {
        self.plugin_map.is_empty()
    }

    fn prepare(&self) -> Result<(), SoftwareError> {
        todo!()
    }

    pub fn list(&self) -> Result<SoftwareListHashStore, SoftwareError> {
        let mut complete_software_list = SoftwareListHashStore::default();

        for software_type in self.plugin_map.keys() {
            let mut plugin_software_list = self.plugin(&software_type)?.list()?;
            match complete_software_list
                .module_type
                .entry(software_type.clone())
            {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().append(&mut plugin_software_list);
                }
                Entry::Vacant(entry) => {
                    entry.insert(plugin_software_list);
                }
            }
            // complete_software_list.append(&mut plugin_software_list);
        }
        Ok(complete_software_list)
    }

    fn install(&self, module: &SoftwareRequestUpdateModule) -> Result<(), SoftwareError> {
        todo!()
        // self.plugin(module.software_type.as_str())?.install(module)
    }

    fn remove(&self, module: &SoftwareRequestUpdateModule) -> Result<(), SoftwareError> {
        todo!()
        // self.plugin(module.software_type.as_str())?.remove(module)
    }

    fn finalize(&self) -> Result<(), SoftwareError> {
        todo!()
    }

    // fn list(&self) -> Result<SoftwareList, SoftwareError> {
    //     let mut complete_software_list = SoftwareList::new();
    //     for software_type in self.plugin_map.keys() {
    //         let mut plugin_software_list = self.plugin(&software_type)?.list()?;
    //         complete_software_list.append(&mut plugin_software_list);
    //     }
    //     dbg!();
    //     Ok(complete_software_list)
    // }

    fn version(&self, module: &SoftwareModule) -> Result<Option<String>, SoftwareError> {
        todo!()
        // self.plugin(module.software_type.as_str())?.version(module)
    }

    // pub fn apply(&self, update: &SoftwareUpdate) -> SoftwareUpdateStatus {
    //     let result = match update {
    //         SoftwareUpdate::Install { module } => self.install(&module),
    //         SoftwareUpdate::UnInstall { module } => self.remove(&module),
    //     };

    //     SoftwareUpdateStatus::new(update, result)
    // }
    pub fn apply(&self, update: &SoftwareRequestUpdateModule) -> SoftwareRequestUpdateStatus {
        let result = match update.action {
            SoftwareRequestUpdateAction::Install => self.install(&update),
            SoftwareRequestUpdateAction::Remove => self.remove(&update),
        };

        SoftwareRequestUpdateStatus::new(update, result)
    }
}

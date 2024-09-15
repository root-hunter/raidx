use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum ErrorRConfigs {
    Io(std::io::Error),
    SerdeJson(serde_json::Error)
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct RConfigSynchronizer {
    pub timeout: usize
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct RConfigWatcher {
    
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RConfigDatabase {
    pub path: String    
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RConfigNode {
    pub host: String,
    pub port: usize,
    pub ssl: bool
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RConfig {
    pub folder_path: String,
    pub server: RConfigNode,
    pub synchronizer: RConfigSynchronizer,
    pub watcher: RConfigWatcher,
    pub database: RConfigDatabase,
    pub nodes: Vec<RConfigNode>
}

impl RConfig {
    pub fn get_default(folder_path: String) -> RConfig {
        return RConfig{
            folder_path: folder_path,
            server: RConfigNode { host: "0.0.0.0".to_string(), port: 4000, ssl: false },
            synchronizer: RConfigSynchronizer { timeout: 2 },
            watcher: RConfigWatcher {  },
            database: RConfigDatabase{
                path: "/home/roothunter/Dev/raidx/config/raidx.database.db".to_string()
            },
            nodes: Vec::new()
          };
    }

    pub fn load_from_file(path: &std::path::Path) -> Result<RConfig, ErrorRConfigs> {
        let file = std::fs::read(path);

        if file.is_ok() {
            let file = file.unwrap();
            let configs = serde_json::from_slice::<RConfig>(file.as_slice());

            if configs.is_ok() {
                return Ok(configs.unwrap());
            } else {
                return Err(ErrorRConfigs::SerdeJson(configs.unwrap_err()));
            }
        } else {
            return Err(ErrorRConfigs::Io(file.unwrap_err()));
        }
    }

    pub fn load_or_create_default(configs_path: String, folder_path: String) -> Result<RConfig, ErrorRConfigs> {
        let config_file = std::path::Path::new(configs_path.as_str());
        
        return match config_file.exists() {
            true => {
                let result: Result<RConfig, ErrorRConfigs> = RConfig::load_from_file(&config_file);
    
                if result.is_ok() {
                    Ok(result.unwrap())
                } else {
                    Err(result.unwrap_err())
                }
            },
            false => {
                let default_configs = RConfig::get_default(folder_path);
                let result = default_configs.dump_to_file(configs_path);
    
                if result.is_ok() {
                    Ok(default_configs)
                } else {
                    Err(result.unwrap_err())
                }
            }
        };
    }

    pub fn dump_to_file(&self, path: String) -> Result<(), ErrorRConfigs> {
        let path = std::path::Path::new(path.as_str());

        let content = serde_json::to_string_pretty::<RConfig>(&self);

        if content.is_ok() {
            let file: Result<(), std::io::Error> = std::fs::write(path, content.unwrap());
            
            if file.is_ok() {
                return Ok(());
            } else {
                return Err(ErrorRConfigs::Io(file.unwrap_err()));
            }
        } else {
            return Err(ErrorRConfigs::SerdeJson(content.unwrap_err()));
        }
    }

    pub fn get_info(self) -> String {
        return serde_json::to_string_pretty(&self).unwrap();
    }
}
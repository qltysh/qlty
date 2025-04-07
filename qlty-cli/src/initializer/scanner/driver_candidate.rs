use qlty_config::config::{DriverDef, PluginDef};

#[derive(Debug, Default, Clone)]
pub struct DriverCandidate {
    pub key: String,
    pub version: String,
    pub name: String,
    pub driver: DriverDef,
}

impl DriverCandidate {
    pub fn build_driver_candidates(plugin_def: &PluginDef) -> Vec<DriverCandidate> {
        let mut driver_candidates = vec![];

        for (driver_name, driver) in plugin_def.drivers.iter() {
            if driver.version.is_empty() {
                driver_candidates.push(DriverCandidate {
                    key: driver_name.clone(),
                    name: driver_name.clone(),
                    driver: driver.clone(),
                    version: Self::get_version(driver),
                });
            } else {
                for driver_version in &driver.version {
                    let version = Self::get_version(driver_version);

                    driver_candidates.push(DriverCandidate {
                        key: format!("{}@{}", driver_name, version),
                        name: driver_name.clone(),
                        version,
                        driver: driver_version.clone(),
                    })
                }
            }
        }

        driver_candidates
    }

    fn get_version(driver: &DriverDef) -> String {
        if let Some(driver_known_good_version) = &driver.known_good_version {
            driver_known_good_version.clone()
        } else {
            "known_good".to_string()
        }
    }
}

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::utils::HashMap;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct RootConfigs
{
    inner: HashMap<String, HashMap<String, toml::value::Value>>,
}

impl RootConfigs
{
    #[cfg(not(target_family = "wasm"))]
    pub fn read(&mut self, root_path: PathBuf, mut config_files: Vec<PathBuf>) -> Result<(), String>
    {
        for path in config_files.drain(..) {
            let context = path
                .as_path()
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or_else(|| format!("failed reading config file \"{path:?}\"; failed extracting file name"))?;
            let ctx_map: &mut HashMap<String, toml::value::Value> = self.inner.entry(context.into()).or_default();
            let data = std::fs::read_to_string(root_path.join(&path))
                .map_err(|e| format!("failed reading config file \"{path:?}\"; io err: {e:?}"))?;
            let vals = data
                .parse::<toml::Table>()
                .map_err(|e| format!("failed parsing config file \"{path:?}\"; parse err: {e:?}"))?;
            for (key, value) in vals.iter() {
                if let Some(prev_value) = ctx_map.get(key) {
                    tracing::debug!("overwriting config {context}::{}; prev={prev_value:?}, new={value:?}", key.as_str());
                }
                ctx_map.insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    #[cfg(target_family = "wasm")]
    pub async fn read(&mut self, root_path: PathBuf, mut config_files: Vec<PathBuf>) -> Result<(), String>
    {
        let reader = HttpWasmAssetReader::new(root_path);
        for path in config_files.drain(..) {
            let context = path
                .as_path()
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or_else(|| {
                    format!("failed reading config file with path \"{path:?}\"; failed extracting file name")
                })?;
            let ctx_map: &mut HashMap<String, toml::value::Value> = self.inner.entry(context.into()).or_default();
            let mut reader = reader
                .read(path.as_path())
                .await
                .map_err(|e| format!("failed reading config file \"{path:?}\"; io err: {e:?}"))?;
            let mut raw_data = Vec::default();
            reader
                .read_to_end(&mut raw_data)
                .await
                .map_err(|e| format!("failed reading config file \"{path:?}\"; reader err: {e:?}"))?;
            let data = String::from_utf8(raw_data)
                .map_err(|e| format!("failed reading config file \"{path:?}\"; utf8 conversion err: {e:?}"))?;
            let vals = data
                .parse::<toml::Table>()
                .map_err(|e| format!("failed parsing config file \"{path:?}\"; parse err: {e:?}"))?;
            for (key, value) in vals.iter() {
                if let Some(prev_value) = ctx_map.get(key) {
                    tracing::debug!("overwriting config {context}::{}; prev={prev_value:?}, new={value:?}", key.as_str());
                }
                ctx_map.insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    pub fn get(&self, context: &str, key: &str) -> Result<&toml::value::Value, String>
    {
        let ctx_map = self
            .inner
            .get(context)
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; context unknown"))?;
        ctx_map
            .get(key)
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; key unknown"))
    }

    pub fn get_integer<T: TryFrom<i64>>(&self, context: &str, key: &str) -> Result<T, String>
    where
        <T as TryFrom<i64>>::Error: std::fmt::Debug,
    {
        let val = self
            .get(context, key)?
            .as_integer()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not integer"))?;
        T::try_from(val)
            .map_err(|e| format!("config lookup failed for {context}::{key}; failed converting int: {e:?}"))
    }

    pub fn get_float(&self, context: &str, key: &str) -> Result<f64, String>
    {
        self.get(context, key)?
            .as_float()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not float"))
    }

    pub fn get_bool(&self, context: &str, key: &str) -> Result<bool, String>
    {
        self.get(context, key)?
            .as_bool()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not bool"))
    }

    pub fn get_str(&self, context: &str, key: &str) -> Result<&str, String>
    {
        self.get(context, key)?
            .as_str()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not str"))
    }

    pub fn get_datetime(&self, context: &str, key: &str) -> Result<&toml::value::Datetime, String>
    {
        self.get(context, key)?
            .as_datetime()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not datetime"))
    }

    pub fn get_array(&self, context: &str, key: &str) -> Result<&Vec<toml::value::Value>, String>
    {
        self.get(context, key)?
            .as_array()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not array"))
    }

    pub fn get_table(&self, context: &str, key: &str) -> Result<&toml::value::Table, String>
    {
        self.get(context, key)?
            .as_table()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not table"))
    }
}

//-------------------------------------------------------------------------------------------------------------------

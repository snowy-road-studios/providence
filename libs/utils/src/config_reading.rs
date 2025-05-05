use std::path::PathBuf;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use serde::Deserialize;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource)]
pub struct ConfigDirectories
{
    pub main_dir: PathBuf,
    pub override_dir: PathBuf,
}

//-------------------------------------------------------------------------------------------------------------------

/// Configs extracted from the filesystem.
///
/// Every config sub-directory must contain a `manifest.toml` file with a `FILE = [ .. ]` entry containing the
/// names of all other files in the directory.
#[derive(Default, Debug)]
pub struct RootConfigs
{
    /// [filename : [config key : config value]]
    inner: HashMap<String, HashMap<String, toml::value::Value>>,
}

impl RootConfigs
{
    pub fn get_type<T: for<'de> Deserialize<'de>>(&self, context: &str, key: &str) -> Result<T, String>
    {
        let type_name = std::any::type_name::<T>();
        Ok(T::deserialize(self.get_value(context, key)?.clone()).map_err(|err| {
            format!("config lookup failed for {context}::{key}; could not deserialize into {type_name}: {err:?}")
        })?)
    }

    pub fn get_value(&self, context: &str, key: &str) -> Result<&toml::value::Value, String>
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
            .get_value(context, key)?
            .as_integer()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not integer"))?;
        T::try_from(val)
            .map_err(|e| format!("config lookup failed for {context}::{key}; failed converting int: {e:?}"))
    }

    pub fn get_float(&self, context: &str, key: &str) -> Result<f64, String>
    {
        self.get_value(context, key)?
            .as_float()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not float"))
    }

    pub fn get_bool(&self, context: &str, key: &str) -> Result<bool, String>
    {
        self.get_value(context, key)?
            .as_bool()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not bool"))
    }

    pub fn get_str(&self, context: &str, key: &str) -> Result<&str, String>
    {
        self.get_value(context, key)?
            .as_str()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not str"))
    }

    pub fn get_datetime(&self, context: &str, key: &str) -> Result<&toml::value::Datetime, String>
    {
        self.get_value(context, key)?
            .as_datetime()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not datetime"))
    }

    pub fn get_array(&self, context: &str, key: &str) -> Result<&Vec<toml::value::Value>, String>
    {
        self.get_value(context, key)?
            .as_array()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not array"))
    }

    pub fn get_table(&self, context: &str, key: &str) -> Result<&toml::value::Table, String>
    {
        self.get_value(context, key)?
            .as_table()
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; value not table"))
    }
}

#[cfg(not(target_family = "wasm"))]
impl RootConfigs
{
    pub fn new<T: Into<PathBuf> + Clone>(main_dir: &PathBuf, sub_dirs: impl AsRef<[T]>) -> Result<Self, String>
    {
        let files = Self::read_manifests(main_dir.clone(), sub_dirs)?;
        let mut configs = Self::default();
        configs.read(main_dir.clone(), files, false)?;

        Ok(configs)
    }

    pub fn new_with_overrides<T: Into<PathBuf> + Clone>(
        main_dir: &PathBuf,
        override_dir: &PathBuf,
        sub_dirs: impl AsRef<[T]>,
    ) -> Result<Self, String>
    {
        let files = Self::read_manifests(main_dir.clone(), sub_dirs)?;
        let mut configs = Self::default();
        configs.read(main_dir.clone(), files.clone(), false)?;
        configs.read(override_dir.clone(), files, true)?;

        Ok(configs)
    }

    pub fn read_manifests<T: Into<PathBuf> + Clone>(
        main_dir: PathBuf,
        sub_dirs: impl AsRef<[T]>,
    ) -> Result<Vec<PathBuf>, String>
    {
        let mut config_files = vec![];

        for sub_dir in sub_dirs.as_ref().iter() {
            let sub_dir: PathBuf = sub_dir.clone().into();
            let path = sub_dir.join("manifest.toml");

            let mut configs = Self::default();
            configs.read(main_dir.clone(), vec![path], false)?;

            let files = configs.get_value("manifest", "FILES")?;
            match files {
                toml::value::Value::Array(vals) => {
                    for val in vals.iter() {
                        match val {
                            toml::value::Value::String(file) => {
                                config_files.push(sub_dir.join(file));
                            }
                            _ => (),
                        }
                    }
                }
                _ => {
                    return Err(
                        format!("failed reading manifest section of {main_dir:?}/{sub_dir:?}; missing \
                    `FILES = [ .. ]` section"),
                    )
                }
            }
        }

        Ok(config_files)
    }

    fn read(
        &mut self,
        root_path: PathBuf,
        mut files: Vec<PathBuf>,
        allow_missing_files: bool,
    ) -> Result<(), String>
    {
        for path in files.drain(..) {
            let context = path
                .as_path()
                .file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| {
                    let (filename, _) = f.split_once(".toml")?;
                    Some(filename)
                })
                .ok_or_else(|| format!("failed reading config file \"{path:?}\"; failed extracting file name"))?;
            let ctx_map: &mut HashMap<String, toml::value::Value> = self.inner.entry(context.into()).or_default();
            let maybe_data = std::fs::read_to_string(root_path.join(&path));
            let data = match allow_missing_files {
                true => match maybe_data {
                    Ok(data) => data,
                    _ => continue,
                },
                false => maybe_data.map_err(|e| {
                    format!("failed reading config file {:?}; io err: {e:?}", root_path.join(&path))
                })?,
            };
            let vals = data.parse::<toml::Table>().map_err(|e| {
                format!("failed parsing config file {:?}; parse err: {e:?}", root_path.join(&path))
            })?;
            for (key, value) in vals.iter() {
                if let Some(prev_value) = ctx_map.get(key) {
                    tracing::debug!("overwriting config {context}::{}; prev={prev_value:?}, new={value:?}", key.as_str());
                }
                ctx_map.insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }
}

#[cfg(target_family = "wasm")]
impl RootConfigs
{
    pub async fn new<T: Into<PathBuf> + Clone>(
        main_dir: &PathBuf,
        sub_dirs: impl AsRef<[T]>,
    ) -> Result<Self, String>
    {
        let files = Self::read_manifests(main_dir.clone(), sub_dirs).await?;
        let mut configs = Self::default();
        configs.read(main_dir.clone(), files, false).await?;

        Ok(configs)
    }

    pub async fn new_with_overrides<T: Into<PathBuf> + Clone>(
        main_dir: &PathBuf,
        override_dir: &PathBuf,
        sub_dirs: impl AsRef<[T]>,
    ) -> Result<Self, String>
    {
        let files = Self::read_manifests(main_dir.clone(), sub_dirs).await?;
        let mut configs = Self::default();
        configs.read(main_dir.clone(), files.clone(), false).await?;
        configs.read(override_dir.clone(), files, true).await?;

        Ok(configs)
    }

    pub async fn read_manifests<T: Into<PathBuf> + Clone>(
        main_dir: PathBuf,
        sub_dirs: impl AsRef<[T]>,
    ) -> Result<Vec<PathBuf>, String>
    {
        let mut config_files = vec![];

        for sub_dir in sub_dirs.as_ref().iter() {
            let sub_dir: PathBuf = sub_dir.clone().into();
            let path = sub_dir.join("manifest.toml");

            let mut configs = Self::default();
            configs.read(main_dir.clone(), vec![path], false).await?;

            let files = configs.get_value("manifest", "FILES")?;
            match files {
                toml::value::Value::Array(vals) => {
                    for val in vals.iter() {
                        match val {
                            toml::value::Value::String(file) => {
                                config_files.push(sub_dir.join(file));
                            }
                            _ => (),
                        }
                    }
                }
                _ => {
                    return Err(
                        format!("failed reading manifest section of {main_dir:?}/{sub_dir:?}; missing \
                    `FILES = [ .. ]` section"),
                    )
                }
            }
        }

        Ok(config_files)
    }

    #[cfg(target_family = "wasm")]
    async fn read(
        &mut self,
        root_path: PathBuf,
        mut config_files: Vec<PathBuf>,
        allow_missing_files: bool,
    ) -> Result<(), String>
    {
        use bevy::asset::io::{AssetReader, Reader};

        let reader = bevy::asset::io::wasm::HttpWasmAssetReader::new(root_path);
        for path in config_files.drain(..) {
            let context = path
                .as_path()
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or_else(|| {
                    format!("failed reading config file with path \"{path:?}\"; failed extracting file name")
                })?;
            let ctx_map: &mut HashMap<String, toml::value::Value> = self.inner.entry(context.into()).or_default();
            let maybe_reader = reader.read(path.as_path()).await;
            let mut reader = match allow_missing_files {
                true => match maybe_reader {
                    Ok(reader) => reader,
                    _ => continue,
                },
                false => {
                    maybe_reader.map_err(|e| format!("failed reading config file \"{path:?}\"; io err: {e:?}"))?
                }
            };
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
}

//-------------------------------------------------------------------------------------------------------------------

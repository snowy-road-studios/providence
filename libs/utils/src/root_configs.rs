use std::path::PathBuf;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use cobweb_asset_format::prelude::*;
use serde::Deserialize;

//-------------------------------------------------------------------------------------------------------------------

fn try_parse(data: &str, root_path: &PathBuf, path: &PathBuf) -> Result<CobValue, String>
{
    let (Some(val), _, _) = CobValue::try_parse(
        CobFill::default(),
        Span::new_extra(
            data,
            CobLocationMetadata {
                file: root_path.join(&path).to_str().ok_or_else(|| {
                    format!("failed parsing config file {:?}; failed converting path to string",
                        root_path.join(path))
                })?,
            },
        ),
    )
    .map_err(|e| format!("failed parsing config file {:?}; parse err: {e:?}", root_path.join(path)))?
    else {
        return Err(
            format!("failed parsing config file {:?}; could not convert file to CobValue",
            root_path.join(path)),
        );
    };

    Ok(val)
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(not(target_family = "wasm"))]
fn read_file(
    root_path: &PathBuf,
    path: &PathBuf,
    allow_missing_files: bool,
) -> Result<Option<(String, CobValue)>, String>
{
    let context = path
        .as_path()
        .file_name()
        .and_then(|f| f.to_str())
        .and_then(|f| {
            let (filename, _) = f.split_once(".rawcob")?;
            Some(filename)
        })
        .ok_or_else(|| {
            format!("failed reading config file \"{:?}\"; failed extracting file name",
            root_path.join(&path))
        })?;
    let maybe_data = std::fs::read_to_string(root_path.join(&path));
    let data = match allow_missing_files {
        true => match maybe_data {
            Ok(data) => data,
            _ => return Ok(None),
        },
        false => maybe_data
            .map_err(|e| format!("failed reading config file {:?}; io err: {e:?}", root_path.join(&path)))?,
    };
    let val = try_parse(data.as_str(), root_path, &path)?;

    Ok(Some((context.into(), val)))
}

//-------------------------------------------------------------------------------------------------------------------

#[cfg(target_family = "wasm")]
async fn read_file(
    root_path: &PathBuf,
    path: PathBuf,
    allow_missing_files: bool,
) -> Result<Option<(String, CobValue)>, String>
{
    let context = path
        .as_path()
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| {
            format!("failed reading config file with path \"{:?}\"; failed extracting file name",
                root_path.join(&path))
        })?;
    let ctx_map: &mut HashMap<String, CobValue> = self.inner.entry(context.into()).or_default();
    let maybe_reader = reader.read(path.as_path()).await;
    let mut reader = match allow_missing_files {
        true => match maybe_reader {
            Ok(reader) => reader,
            _ => return Ok(None),
        },
        false => maybe_reader.map_err(|e| {
            format!("failed reading config file \"{:?}\"; io err: {e:?}",
                root_path.join(&path))
        })?,
    };
    let mut raw_data = Vec::default();
    reader.read_to_end(&mut raw_data).await.map_err(|e| {
        format!("failed reading config file \"{:?}\"; reader err: {e:?}",
            root_path.join(&path))
    })?;
    let data = String::from_utf8(raw_data).map_err(|e| {
        format!("failed reading config file \"{:?}\"; utf8 conversion err: {e:?}",
            root_path.join(&path))
    })?;
    let val = try_parse(data.as_str(), root_path, &path)?;

    Ok(Some((context.int(), val)))
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Default, Debug)]
enum RootConfigEntry
{
    #[default]
    Empty,
    Config(HashMap<String, CobValue>),
    Type(CobValue),
}

impl RootConfigEntry
{
    fn try_insert(
        &mut self,
        root_path: &PathBuf,
        path: PathBuf,
        context: &str,
        file: CobValue,
        config_type: RootConfigType,
    ) -> Result<(), String>
    {
        match config_type {
            RootConfigType::Config => {
                if matches!(self, Self::Empty) {
                    *self = Self::Config(HashMap::default());
                }

                let ctx_map = match self {
                    Self::Empty => unreachable!(),
                    Self::Config(ctx_map) => ctx_map,
                    Self::Type(_) => {
                        return Err(
                            format!("tried inserting file {:?} as config variant 'Config' to root config entry \
                            but the entry was previously set to 'Type'", root_path.join(&path)),
                        );
                    }
                };

                let CobValue::Map(map) = file else {
                    return Err(
                        format!("failed reading config file {:?}; value is not a map", root_path.join(&path)),
                    );
                };

                for entry in map.entries.iter() {
                    let CobMapEntry::KeyValue(kv) = entry else {
                        return Err(format!("failed reading config file {:?}; value is not key-value map",
                            root_path.join(&path)));
                    };
                    let CobMapKey::Value(CobValue::String(key)) = &kv.key else {
                        return Err(format!("failed reading config file {:?}; found non-string key in map",
                            root_path.join(&path)));
                    };

                    if let Some(prev_value) = ctx_map.get(key.as_str()) {
                        tracing::debug!("overwriting config {context}::{}; prev={prev_value:?}, new={:?}", key.as_str(), kv.value);
                    }
                    ctx_map.insert(key.as_str().into(), kv.value.clone());
                }
            }
            RootConfigType::Type => match self {
                Self::Empty => {
                    *self = Self::Type(file);
                }
                Self::Config(_) => {
                    return Err(
                        format!("tried inserting file {:?} as config variant 'Type' to root config entry \
                            but the entry was previously set to 'Config'", root_path.join(&path)),
                    );
                }
                Self::Type(prev) => {
                    tracing::debug!("overwriting root config entry for file {:?} of variant 'Type'",
                            root_path.join(&path));
                    *prev = file;
                }
            },
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Deserialize, Copy, Clone)]
pub enum RootConfigType
{
    /// The config file should be a map of strings to values.
    Config,
    /// The entire config file will be deserialized as one type.
    Type,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct RootConfigManifestEntry(pub String, pub RootConfigType);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Deserialize, Deref)]
pub struct RootConfigManifest(pub Vec<RootConfigManifestEntry>);

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
/// Every config sub-directory must contain a `manifest.rawcob` file with an array of `RootConfigManifestEntry`.
/// Each entry includes another file in the directory as a string, and specified if it is a Config or Type file.
#[derive(Default, Debug)]
pub struct RootConfigs
{
    /// [filename : [config key : config value]]
    inner: HashMap<String, RootConfigEntry>,
}

impl RootConfigs
{
    /// Treats the file as a map between string keys and values, looks up the key, and deserializes the value
    /// found.
    pub fn get_type<T: for<'de> Deserialize<'de>>(&self, context: &str, key: &str) -> Result<T, String>
    {
        let type_name = std::any::type_name::<T>();
        Ok(T::deserialize(self.get_value(context, key)?).map_err(|err| {
            format!("config lookup failed for {context}::{key}; could not deserialize into {type_name}: {err:?}")
        })?)
    }

    /// Treats the entire file as one value and deserializes it.
    pub fn get_type_from_file<T: for<'de> Deserialize<'de>>(&self, context: &str) -> Result<T, String>
    {
        let type_name = std::any::type_name::<T>();
        Ok(T::deserialize(self.get_value_from_file(context)?).map_err(|err| {
            format!("config lookup failed for {context}; could not deserialize into {type_name}: {err:?}")
        })?)
    }

    /// Treats the file as a map between string keys and values, and looks up the key.
    pub fn get_value(&self, context: &str, key: &str) -> Result<&CobValue, String>
    {
        let entry = self
            .inner
            .get(context)
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; context unknown"))?;
        let RootConfigEntry::Config(ctx_map) = entry else {
            return Err(
                format!("config lookup failed for {context}::{key}; config at context is config variant 'Type' \
                not 'Config'"),
            );
        };
        ctx_map
            .get(key)
            .ok_or_else(|| format!("config lookup failed for {context}::{key}; key unknown"))
    }

    /// Treats the entire file as one value and returns it.
    pub fn get_value_from_file(&self, context: &str) -> Result<&CobValue, String>
    {
        let entry = self
            .inner
            .get(context)
            .ok_or_else(|| format!("config file lookup failed for {context}; context unknown"))?;
        let RootConfigEntry::Type(val) = entry else {
            return Err(
                format!("config lookup failed for {context}; config at context is config variant 'Config' \
                not 'Type'"),
            );
        };
        Ok(val)
    }

    pub fn get_integer<T: TryFrom<i64>>(&self, context: &str, key: &str) -> Result<T, String>
    where
        <T as TryFrom<i64>>::Error: std::fmt::Debug,
    {
        match self.get_value(context, key)? {
            CobValue::Number(number) => {
                let num = number.number.as_i128().ok_or_else(|| {
                    format!("config lookup failed for {context}::{key}; value is number but not i128")
                })?;
                T::try_from(num as i64).map_err(|e| {
                    format!("config lookup failed for {context}::{key}; failed converting int: {e:?}")
                })
            }
            _ => Err(format!("config lookup failed for {context}::{key}; value is not integer")),
        }
    }

    pub fn get_float(&self, context: &str, key: &str) -> Result<f64, String>
    {
        match self.get_value(context, key)? {
            CobValue::Number(number) => number
                .number
                .as_f64()
                .ok_or_else(|| format!("config lookup failed for {context}::{key}; value is number but not f64")),
            _ => Err(format!("config lookup failed for {context}::{key}; value not float")),
        }
    }

    pub fn get_bool(&self, context: &str, key: &str) -> Result<bool, String>
    {
        match self.get_value(context, key)? {
            CobValue::Bool(val) => Ok(val.value),
            _ => Err(format!("config lookup failed for {context}::{key}; value not bool")),
        }
    }

    pub fn get_str(&self, context: &str, key: &str) -> Result<&str, String>
    {
        match self.get_value(context, key)? {
            CobValue::String(val) => Ok(val.as_str()),
            _ => Err(format!("config lookup failed for {context}::{key}; value not string")),
        }
    }

    fn add_value(
        &mut self,
        root_path: &PathBuf,
        path: PathBuf,
        context: String,
        val: CobValue,
        config_type: RootConfigType,
    ) -> Result<(), String>
    {
        let entry = self.inner.entry(context.clone()).or_default();
        entry.try_insert(root_path, path, context.as_str(), val, config_type)?;

        Ok(())
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
    ) -> Result<Vec<(PathBuf, RootConfigType)>, String>
    {
        let mut config_files = vec![];

        for sub_dir in sub_dirs.as_ref().iter() {
            let sub_dir: PathBuf = sub_dir.clone().into();
            let path = sub_dir.join("manifest.rawcob");

            let Some((_, file_val)) = read_file(&main_dir, &path, false)? else { continue };
            let manifest = RootConfigManifest::deserialize(&file_val).map_err(|err| {
                format!("failed reading manifest section of {main_dir:?}/{sub_dir:?}; failed deserializing into \
                    RootConfigManifest: {err:?}")
            })?;

            for RootConfigManifestEntry(import, config_type) in manifest.iter() {
                config_files.push((sub_dir.join(import.as_str()), *config_type));
            }
        }

        Ok(config_files)
    }

    fn read(
        &mut self,
        root_path: PathBuf,
        mut files: Vec<(PathBuf, RootConfigType)>,
        allow_missing_files: bool,
    ) -> Result<(), String>
    {
        for (path, config_type) in files.drain(..) {
            let Some((context, val)) = read_file(&root_path, &path, allow_missing_files)? else { continue };
            self.add_value(&root_path, path, context, val, config_type)?;
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
    ) -> Result<Vec<(PathBuf, RootConfigType)>, String>
    {
        let mut config_files = vec![];

        for sub_dir in sub_dirs.as_ref().iter() {
            let sub_dir: PathBuf = sub_dir.clone().into();
            let path = sub_dir.join("manifest.rawcob");

            let Some((_, file_val)) = read_file(&main_dir, &path, false).await? else { continue };
            let manifest = RootConfigManifest::deserialize(&file_val).map_err(|err| {
                format!("failed reading manifest section of {main_dir:?}/{sub_dir:?}; failed deserializing into \
                    RootConfigManifest: {err:?}")
            })?;

            for RootConfigManifest(import, config_type) in manifest.iter() {
                config_files.push((sub_dir.join(import.as_str()), *config_type));
            }
        }

        Ok(config_files)
    }

    async fn read(
        &mut self,
        root_path: PathBuf,
        mut files: Vec<(PathBuf, RootConfigType)>,
        allow_missing_files: bool,
    ) -> Result<(), String>
    {
        use bevy::asset::io::{AssetReader, Reader};

        let reader = bevy::asset::io::wasm::HttpWasmAssetReader::new(&root_path);
        for (path, config_type) in files.drain(..) {
            let Some((context, val)) = read_file(&root_path, &path, allow_missing_files).await? else { continue };
            self.add_value(&root_path, path, context, val, config_type)?;
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use bevy::asset::{AssetLoadFailedEvent, AssetPath};
use bevy::prelude::*;
use bevy_aseprite_ultra::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use fluent_langneg::{negotiate_languages, LanguageIdentifier, NegotiationStrategy};
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

fn load_localized_aseprites(
    In(loaded): In<Vec<LocalizedAsepriteFile>>,
    mut c: Commands,
    asset_server: Res<AssetServer>,
    mut aseprites: ResMut<AsepriteMap>,
    manifest: Res<LocalizationManifest>,
)
{
    aseprites.insert_localized(loaded, &asset_server, &manifest, &mut c);
}

//-------------------------------------------------------------------------------------------------------------------

fn load_aseprites(In(loaded): In<Vec<String>>, asset_server: Res<AssetServer>, mut aseprites: ResMut<AsepriteMap>)
{
    for path in loaded {
        aseprites.insert(&path, &asset_server);
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_new_lang_list(
    asset_server: Res<AssetServer>,
    manifest: Res<LocalizationManifest>,
    mut aseprites: ResMut<AsepriteMap>,
)
{
    aseprites.negotiate_languages(&manifest, &asset_server);
}

//-------------------------------------------------------------------------------------------------------------------

fn check_loaded_aseprites(
    mut c: Commands,
    mut errors: EventReader<AssetLoadFailedEvent<Aseprite>>,
    mut events: EventReader<AssetEvent<Aseprite>>,
    mut aseprites: ResMut<AsepriteMap>,
)
{
    for error in errors.read() {
        let AssetLoadFailedEvent { id, .. } = error;
        aseprites.remove_pending(id);
    }

    for event in events.read() {
        let AssetEvent::Added { id } = event else { continue };
        aseprites.remove_pending(id);
    }

    aseprites.try_emit_load_event(&mut c);
}

//-------------------------------------------------------------------------------------------------------------------

/// System that runs when the app needs to replace existing aseprites with updated localized aseprites.
fn relocalize_aseprites(
    aseprites: Res<AsepriteMap>,
    mut animations: Query<&mut AseSpriteAnimation>,
    mut slices: Query<&mut AseSpriteSlice>,
    mut ui_animations: Query<&mut AseUiAnimation>,
    mut ui_slices: Query<&mut AseUiSlice>,
)
{
    for mut handle in animations
        .iter_mut()
        .map(|c| &mut c.into_inner().aseprite)
        .chain(slices.iter_mut().map(|c| &mut c.into_inner().aseprite))
        .chain(
            ui_animations
                .iter_mut()
                .map(|c| &mut c.into_inner().aseprite),
        )
        .chain(ui_slices.iter_mut().map(|c| &mut c.into_inner().aseprite))
    {
        aseprites.localize_aseprite(&mut handle);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when [`AsepriteMap`] has updated and become fully loaded *after* a
/// [`LoadLocalizedAsepriteFiles`] instance was applied.
///
/// This event is *not* emitted when aseprites are reloaded due to language renegotiation. Listen for the
/// [`RelocalizeApp`] event instead.
pub struct AsepriteMapLoaded;

//-------------------------------------------------------------------------------------------------------------------

/// Resource that stores handles to loaded aseprite files and manages aseprite localization.
///
/// Requested aseprite handles will be automatically localized based on the currently negotiated languages in
/// [`LocalizationManifest`]. If negotiated languages change, then all aseprite handles in the app will be
/// automatically re-localized if they have fallbacks for the new language list.
///
/// We assume that all localization fallbacks are globally unique. A fallback should be used as a fallback exactly
/// once and never used as a 'main' aseprite.
///
/// Aseprites are automatically loaded and unloaded when languages are changed, so that only aseprites that might
/// be needed are kept in memory.
#[derive(Resource, Default)]
pub struct AsepriteMap
{
    /// Indicates the current pending aseprites came from `LoadAsepriteFiles` and `LoadLocalizedAsepriteFiles`
    /// entries, rather than from negotiating languages.
    ///
    /// This is used to emit `AsepriteMapLoaded` events accurately.
    waiting_for_load: bool,
    /// aseprites currently loading.
    pending: HashSet<AssetId<Aseprite>>,
    /// Localization fallbacks.
    /// - Strings in this map are 'full asset paths' that can be used to load aseprites.
    /// [ main aseprite path : (main aseprite path, [ lang id, fallback aseprite path ]) ]
    localization_map: HashMap<Arc<str>, (AssetPath<'static>, HashMap<LanguageIdentifier, AssetPath<'static>>)>,
    /// Used when replacing aseprites on language change. Includes main aseprite AssetPaths in case newly-loaded
    /// aseprite mappings introduce a new localization so existing main aseprite handles need to be replaced.
    /// [ aseprite path : main aseprite path ]
    localized_aseprites_id_helper: HashMap<AssetPath<'static>, Arc<str>>,
    /// Contains handles for aseprites that should be displayed for each 'main aseprite path' based on
    /// currently negotiated languages.
    /// [ main aseprite path : aseprite handle ]
    localized_aseprites: HashMap<Arc<str>, Handle<Aseprite>>,
    /// Aseprites stored permanently.
    cached_aseprites: HashMap<Arc<str>, Handle<Aseprite>>,
}

impl AsepriteMap
{
    /// Checks if the map has any aseprites waiting to load.
    pub fn is_loading(&self) -> bool
    {
        !self.pending.is_empty()
    }

    fn try_add_pending(
        handle: &Handle<Aseprite>,
        asset_server: &AssetServer,
        pending: &mut HashSet<AssetId<Aseprite>>,
    )
    {
        match asset_server.load_state(handle) {
            bevy::asset::LoadState::Loaded => (),
            _ => {
                pending.insert(handle.id());
            }
        }
    }

    fn try_emit_load_event(&mut self, c: &mut Commands)
    {
        // Note/todo: This waits for both localized and non-localized aseprites to load, even though the loaded
        // event is used for localization.
        if self.is_loading() {
            return;
        }
        if !self.waiting_for_load {
            return;
        }

        self.waiting_for_load = false;
        c.react().broadcast(AsepriteMapLoaded);
    }

    fn negotiate_languages(&mut self, manifest: &LocalizationManifest, asset_server: &AssetServer) -> bool
    {
        // Skip negotiation of there are no negotiated languages yet.
        // - This avoids spuriously loading assets that will be replaced once the language list is known.
        let app_negotiated = manifest.negotiated();
        if app_negotiated.len() == 0 {
            return false;
        }

        // We remove `localized_aseprites` because we assume it might be stale (e.g. if we are negotiating because
        // LoadAsepriteFiles was hot-reloaded).
        let prev_localized_aseprites = std::mem::take(&mut self.localized_aseprites);
        self.localized_aseprites
            .reserve(self.localization_map.len());

        let mut langs_buffer = Vec::default();

        self.localization_map
            .iter()
            .for_each(|(main_path, (main_asset_path, fallbacks))| {
                // Collect fallback langs for this aseprite.
                langs_buffer.clear();
                langs_buffer.extend(fallbacks.keys());

                // Negotiate the language we should use, then look up its asset path.
                // - Note: `negotiated_languages` may allocate multiple times, but we don't think this is a huge
                //   issue since it's unlikely users will localize a *lot* of aseprites. It *could* be an issue if
                //   a user loads aseprites from many LoadAsepriteFiles commands, causing this loop to run many
                //   times.
                let asset_path =
                    negotiate_languages(&langs_buffer, app_negotiated, None, NegotiationStrategy::Lookup)
                        .get(0)
                        .map(|lang| {
                            fallbacks
                                .get(lang)
                                .expect("negotiation should only return fallback langs")
                        })
                        .unwrap_or(main_asset_path);

                // Look up or load the handle currently associated with the main aseprite.
                // - If we found a the handle but it doesn't match the language we want, then load the aseprite
                //   fresh.
                let handle = prev_localized_aseprites
                    .get(main_path)
                    .or_else(|| self.cached_aseprites.get(main_path))
                    .filter(|handle| {
                        // Filter based on if the handle has a path that equals the target path.
                        handle.path().filter(|path| *path == asset_path).is_some()
                    })
                    .cloned()
                    .unwrap_or_else(|| {
                        let handle = asset_server.load(asset_path.clone());
                        Self::try_add_pending(&handle, asset_server, &mut self.pending);
                        handle
                    });

                // Now save the localized aseprite.
                self.localized_aseprites.insert(main_path.clone(), handle);
            });

        // Note: old aseprites that are no longer needed will be released when `prev_localized_aseprites` is
        // dropped.

        true
    }

    fn remove_pending(&mut self, id: &AssetId<Aseprite>)
    {
        let _ = self.pending.remove(id);
    }

    /// Adds an aseprite that should be cached.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the aseprite to be loaded.
    pub fn insert(&mut self, path: impl AsRef<str>, asset_server: &AssetServer)
    {
        let path = path.as_ref();

        // Check if the aseprite is cached already.
        if self.cached_aseprites.contains_key(path) {
            tracing::warn!("ignoring duplicate insert for aseprite {}", path);
            return;
        }

        // Check if the aseprite is a localized aseprite.
        let asset_path = match AssetPath::try_parse(path) {
            Ok(asset_path) => asset_path,
            Err(err) => {
                tracing::error!("failed parsing aseprite path {:?} on insert to AsepriteMap: {:?}", path, err);
                return;
            }
        };
        if let Some((key, handle)) = self
            .localized_aseprites
            .get_key_value(path)
            .filter(|(_, handle)| {
                *handle
                    .path()
                    .expect("handles in localized_aseprites should have paths")
                    == asset_path
            })
        {
            self.cached_aseprites.insert(key.clone(), handle.clone());
            return;
        }

        // Add a new cached aseprite.
        let handle = asset_server.load(asset_path);
        Self::try_add_pending(&handle, asset_server, &mut self.pending);
        self.cached_aseprites.insert(Arc::from(path), handle);
    }

    /// Adds a new set of [`LocalizedAsepriteFiles`](`LocalizedAsepriteFile`).
    ///
    /// Will automatically renegotiate languages and emit [`AsepriteMapLoaded`] if appropriate.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for new aseprites to be loaded.
    pub fn insert_localized(
        &mut self,
        mut loaded: Vec<LocalizedAsepriteFile>,
        asset_server: &AssetServer,
        manifest: &LocalizationManifest,
        c: &mut Commands,
    )
    {
        for mut loaded in loaded.drain(..) {
            let main_path = Arc::<str>::from(loaded.aseprite.as_str());

            let (main_asset_path, fallbacks) = self
                .localization_map
                .entry(main_path.clone())
                .or_insert_with(|| {
                    let main_asset_path = match AssetPath::try_parse(&main_path) {
                        Ok(asset_path) => asset_path.clone_owned(),
                        Err(err) => {
                            tracing::error!("failed parsing aseprite path {:?} on insert loaded to AsepriteMap: {:?}",
                                main_path, err);
                            AssetPath::<'static>::default()
                        }
                    };
                    (main_asset_path, HashMap::default())
                });

            // Add helper entry for main aseprite.
            self.localized_aseprites_id_helper
                .insert(main_asset_path.clone(), main_path.clone());

            // Save fallbacks.
            #[cfg(not(feature = "dev"))]
            if fallbacks.len() > 0 {
                // This is feature-gated by hot_reload to avoid spam when hot reloading large lists.
                tracing::warn!("overwritting aseprite fallbacks for main aseprite {:?}; main aseprites should \
                    only appear in one LoadAsepriteFiles command per app", main_path);
            }

            fallbacks.clear();
            fallbacks.reserve(loaded.fallbacks.len());

            for LocalizedAsepriteFileFallback { lang, aseprite } in loaded.fallbacks.drain(..) {
                // Save fallback.
                let lang_id = match LanguageIdentifier::from_str(lang.as_str()) {
                    Ok(lang_id) => lang_id,
                    Err(err) => {
                        tracing::error!("failed parsing target language id  {:?} for aseprite fallback {:?} for \
                            aseprite {:?}: {:?}", lang, aseprite, main_path, err);
                        continue;
                    }
                };
                let fallback_asset_path = match AssetPath::try_parse(aseprite.as_str()) {
                    Ok(asset_path) => asset_path.clone_owned(),
                    Err(err) => {
                        tracing::error!("failed parsing fallback aseprite path {:?} for {:?} on insert loaded to \
                            AsepriteMap: {:?}", aseprite, main_path, err);
                        continue;
                    }
                };

                if let Some(prev) = fallbacks.insert(lang_id, fallback_asset_path.clone()) {
                    tracing::warn!("overwriting aseprite fallback {:?} for aseprite {:?} for lang {:?}",
                        prev, main_path, lang);
                }

                // Save fallback to helper.
                self.localized_aseprites_id_helper
                    .insert(fallback_asset_path, main_path.clone());
            }

            // Note: we populate `localized_aseprites` in `Self::negotiate_languages`.
        }

        // Load aseprites as needed.
        if self.negotiate_languages(manifest, asset_server) {
            self.waiting_for_load = true;
            self.try_emit_load_event(c);
        }
    }

    /// Updates an aseprite handle with the correct localized handle.
    ///
    /// Does nothing if the handle is already correctly localized or if there are no localization fallbacks
    /// associated with the aseprite.
    pub fn localize_aseprite(&self, handle: &mut Handle<Aseprite>)
    {
        let Some(path) = handle.path().cloned() else {
            tracing::debug!("failed localizing aseprite handle that doesn't have a path");
            return;
        };

        if let Some(localized_handle) = self
            .localized_aseprites_id_helper
            .get(&path)
            .and_then(|main_path| self.localized_aseprites.get(main_path))
        {
            *handle = localized_handle.clone();
        } else {
            tracing::debug!("failed localizing aseprite handle with {:?} that doesn't have a localization entry", path);
        }
    }

    /// Gets an aseprite handle for the given path.
    ///
    /// If the given path has a localization fallback for the current [`LocalizationManifest::negotiated`]
    /// languages, then the handle for that fallback will be returned.
    ///
    /// Returns a default handle if the aseprite was not pre-inserted via [`Self::insert`] or
    /// [`Self::insert_localized`].
    pub fn get(&self, path: impl AsRef<str>) -> Handle<Aseprite>
    {
        let path = path.as_ref();

        self.localized_aseprites
            .get(path)
            .or_else(|| self.cached_aseprites.get(path))
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!("failed getting aseprite {} that was not loaded to AsepriteMap", path);
                Default::default()
            })
    }

    /// Gets an aseprite handle for the given path, or loads and caches the aseprite if it's unknown.
    ///
    /// If the given path has a localization fallback for the current [`LocalizationManifest::negotiated`]
    /// languages, then the handle for that fallback will be returned.
    ///
    /// Note that if this is called in state [`LoadState::Loading`], then [`LoadState::Done`] will wait
    /// for the aseprite to be loaded.
    pub fn get_or_load(&mut self, path: impl AsRef<str>, asset_server: &AssetServer) -> Handle<Aseprite>
    {
        let path = path.as_ref();

        // Looks up the aseprite, otherwise loads it fresh.
        self.localized_aseprites
            .get(path)
            .or_else(|| self.cached_aseprites.get(path))
            .cloned()
            .unwrap_or_else(|| {
                let handle = asset_server.load(String::from(path));
                Self::try_add_pending(&handle, asset_server, &mut self.pending);
                self.cached_aseprites
                    .insert(Arc::from(path), handle.clone());
                handle
            })
    }
}

impl AssetLoadProgress for AsepriteMap
{
    fn pending_assets(&self) -> usize
    {
        self.pending.len()
    }

    fn total_assets(&self) -> usize
    {
        // This may double-count some aseprites.
        self.localized_aseprites.len() + self.cached_aseprites.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Contains information for an aseprite fallback.
///
/// See [`LocalizedAsepriteFile`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedAsepriteFileFallback
{
    /// The language id for the fallback.
    pub lang: String,
    /// The path to the asset.
    pub aseprite: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// See [`LoadAsepriteFiles`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalizedAsepriteFile
{
    /// Path to the asset.
    pub aseprite: String,
    /// Fallback aseprites for specific languages.
    ///
    /// Add fallbacks if `self.aseprite` cannot be used for all languages. Any reference to `self.aseprite` will
    /// be automatically localized to the right fallback if you use [`AsepriteMap::get`].
    #[reflect(default)]
    pub fallbacks: Vec<LocalizedAsepriteFileFallback>,
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering localized aseprite assets that need to be pre-loaded.
///
/// The loaded aseprites can be accessed via [`AsepriteMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadLocalizedAsepriteFiles(pub Vec<LocalizedAsepriteFile>);

impl Command for LoadLocalizedAsepriteFiles
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_localized_aseprites);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Loadable command for registering aseprite assets that need to be pre-loaded.
///
/// The loaded aseprites can be accessed via [`AsepriteMap`].
#[derive(Reflect, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadAsepriteFiles(pub Vec<String>);

impl Command for LoadAsepriteFiles
{
    fn apply(self, world: &mut World)
    {
        world.syscall(self.0, load_aseprites);
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Requires `AsepriteUltraPlugin`.
pub struct AsepriteLoadPlugin;

impl Plugin for AsepriteLoadPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<AsepriteMap>()
            .register_asset_tracker::<AsepriteMap>()
            .register_command_type::<LoadAsepriteFiles>()
            .register_command_type::<LoadLocalizedAsepriteFiles>()
            .add_reactor(broadcast::<LanguagesNegotiated>(), handle_new_lang_list)
            .add_reactor(
                (broadcast::<AsepriteMapLoaded>(), broadcast::<RelocalizeApp>()),
                relocalize_aseprites,
            )
            .add_systems(PreUpdate, check_loaded_aseprites.in_set(LoadProgressSet::Prepare));
    }
}

//-------------------------------------------------------------------------------------------------------------------

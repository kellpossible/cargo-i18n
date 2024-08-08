use std::borrow::Cow;

use rust_embed::RustEmbed;

use crate::I18nEmbedError;

/// A trait to handle the retrieval of localization assets.
pub trait I18nAssets {
    /// Get localization asset files that correspond to the specified `file_path`. Returns an empty
    /// [`Vec`] if the asset does not exist, or unable to obtain the asset due to a non-critical
    /// error.
    fn get_files(&self, file_path: &str) -> Vec<Cow<'_, [u8]>>;
    /// Get an iterator over the file paths of the localization assets. There may be duplicates
    /// where multiple files exist for the same file path.
    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String> + '_>;
    /// A method to allow users of this trait to subscribe to change events, and reload assets when
    /// they have changed. The subscription will be cancelled when the returned [`Watcher`] is
    /// dropped.
    ///
    /// **NOTE**: The implementation of this method is optional, don't rely on it functioning for all
    /// implementations.
    fn subscribe_changed(
        &self,
        #[allow(unused_variables)] changed: std::sync::Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher + Send + Sync + 'static>, I18nEmbedError> {
        Ok(Box::new(()))
    }
}

impl Watcher for () {}

impl<T> I18nAssets for T
where
    T: RustEmbed,
{
    fn get_files(&self, file_path: &str) -> Vec<Cow<'_, [u8]>> {
        Self::get(file_path)
            .map(|file| file.data)
            .into_iter()
            .collect()
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(Self::iter().map(|filename| filename.to_string()))
    }

    #[allow(unused_variables)]
    fn subscribe_changed(
        &self,
        changed: std::sync::Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher + Send + Sync + 'static>, I18nEmbedError> {
        Ok(Box::new(()))
    }
}

/// A wrapper for [`rust_embed::RustEmbed`] that supports notifications when files have changed on
/// the file system. A wrapper is required to provide `base_dir` as this is unavailable in the type
/// derived by the [`rust_embed::RustEmbed`] macro.
///
/// ⚠️ *This type requires the following crate features to be activated: `autoreload`.*
#[cfg(feature = "autoreload")]
#[derive(Debug)]
pub struct RustEmbedNotifyAssets<T: rust_embed::RustEmbed> {
    base_dir: std::path::PathBuf,
    embed: core::marker::PhantomData<T>,
}

#[cfg(feature = "autoreload")]
impl<T: rust_embed::RustEmbed> RustEmbedNotifyAssets<T> {
    /// Construct a new [`RustEmbedNotifyAssets`].
    pub fn new(base_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            embed: core::marker::PhantomData,
        }
    }
}

#[cfg(feature = "autoreload")]
impl<T> I18nAssets for RustEmbedNotifyAssets<T>
where
    T: RustEmbed,
{
    fn get_files(&self, file_path: &str) -> Vec<Cow<'_, [u8]>> {
        T::get(file_path)
            .map(|file| file.data)
            .into_iter()
            .collect()
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(T::iter().map(|filename| filename.to_string()))
    }

    fn subscribe_changed(
        &self,
        changed: std::sync::Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher + Send + Sync + 'static>, I18nEmbedError> {
        let base_dir = &self.base_dir;
        if base_dir.is_dir() {
            log::debug!("Watching for changed files in {:?}", self.base_dir);
            notify_watcher(base_dir, changed).map_err(Into::into)
        } else {
            log::debug!("base_dir {base_dir:?} does not yet exist, unable to watch for changes");
            Ok(Box::new(()))
        }
    }
}

/// An [I18nAssets] implementation which pulls assets from the OS
/// file system.
#[cfg(feature = "filesystem-assets")]
#[derive(Debug)]
pub struct FileSystemAssets {
    base_dir: std::path::PathBuf,
    #[cfg(feature = "autoreload")]
    notify_changes_enabled: bool,
}

#[cfg(feature = "filesystem-assets")]
impl FileSystemAssets {
    /// Create a new `FileSystemAssets` instance, all files will be
    /// read from within the specified base directory.
    pub fn try_new<P: Into<std::path::PathBuf>>(base_dir: P) -> Result<Self, I18nEmbedError> {
        let base_dir = base_dir.into();

        if !base_dir.exists() {
            return Err(I18nEmbedError::DirectoryDoesNotExist(base_dir));
        }

        if !base_dir.is_dir() {
            return Err(I18nEmbedError::PathIsNotDirectory(base_dir));
        }

        Ok(Self {
            base_dir,
            #[cfg(feature = "autoreload")]
            notify_changes_enabled: false,
        })
    }

    /// Enable the notification of changes in the [`I18nAssets`] implementation.
    #[cfg(feature = "autoreload")]
    pub fn notify_changes_enabled(mut self, enabled: bool) -> Self {
        self.notify_changes_enabled = enabled;
        self
    }
}

/// An error that occurs during notification of changes when the `autoreload feature is enabled.`
///
/// ⚠️ *This type requires the following crate features to be activated: `filesystem-assets`.*
#[cfg(feature = "autoreload")]
#[derive(Debug)]
pub struct NotifyError(notify::Error);

#[cfg(feature = "autoreload")]
impl From<notify::Error> for NotifyError {
    fn from(value: notify::Error) -> Self {
        Self(value)
    }
}

#[cfg(feature = "autoreload")]
impl From<notify::Error> for I18nEmbedError {
    fn from(value: notify::Error) -> Self {
        Self::Notify(value.into())
    }
}

#[cfg(feature = "autoreload")]
impl std::fmt::Display for NotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "autoreload")]
impl std::error::Error for NotifyError {}

#[cfg(feature = "autoreload")]
fn notify_watcher(
    base_dir: &std::path::Path,
    changed: std::sync::Arc<dyn Fn() + Send + Sync + 'static>,
) -> notify::Result<Box<dyn Watcher + Send + Sync + 'static>> {
    let mut watcher = notify::recommended_watcher(move |event_result| {
        let event: notify::Event = match event_result {
            Ok(event) => event,
            Err(error) => {
                log::error!("{error}");
                return;
            }
        };
        match event.kind {
            notify::EventKind::Any
            | notify::EventKind::Create(_)
            | notify::EventKind::Modify(_)
            | notify::EventKind::Remove(_)
            | notify::EventKind::Other => changed(),
            _ => {}
        }
    })?;

    notify::Watcher::watch(&mut watcher, base_dir, notify::RecursiveMode::Recursive)?;

    Ok(Box::new(watcher))
}

/// An entity that watches for changes to localization resources.
///
/// NOTE: Currently we rely in the implicit [`Drop`] implementation to remove file system watches,
/// in the future ther may be new methods added to this trait.
pub trait Watcher {}

#[cfg(feature = "autoreload")]
impl Watcher for notify::RecommendedWatcher {}

#[cfg(feature = "filesystem-assets")]
impl I18nAssets for FileSystemAssets {
    fn get_files(&self, file_path: &str) -> Vec<Cow<'_, [u8]>> {
        let full_path = self.base_dir.join(file_path);

        if !(full_path.is_file() && full_path.exists()) {
            return Vec::new();
        }

        match std::fs::read(full_path) {
            Ok(contents) => vec![Cow::from(contents)],
            Err(e) => {
                log::error!(
                    target: "i18n_embed::assets", 
                    "Unexpected error while reading localization asset file: {}", 
                    e);
                Vec::new()
            }
        }
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(
            walkdir::WalkDir::new(&self.base_dir)
                .into_iter()
                .filter_map(|f| match f {
                    Ok(f) => {
                        if f.file_type().is_file() {
                            match f.file_name().to_str() {
                                Some(filename) => Some(filename.to_string()),
                                None => {
                                    log::error!(
                                target: "i18n_embed::assets", 
                                "Filename {:?} is not valid UTF-8.", 
                                f.file_name());
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                    Err(err) => {
                        log::error!(
                    target: "i18n_embed::assets", 
                    "Unexpected error while gathering localization asset filenames: {}", 
                    err);
                        None
                    }
                }),
        )
    }

    /// See [`FileSystemAssets::notify_changes_enabled`] to enable this implementation.
    /// ⚠️ *This method requires the following crate features to be activated: `autoreload`.*
    #[cfg(feature = "autoreload")]
    fn subscribe_changed(
        &self,
        changed: std::sync::Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher + Send + Sync + 'static>, I18nEmbedError> {
        if self.notify_changes_enabled {
            notify_watcher(&self.base_dir, changed).map_err(Into::into)
        } else {
            Ok(Box::new(()))
        }
    }
}

/// A way to multiplex implmentations of [`I18nAssets`].
pub struct AssetsMultiplexor {
    /// Assets that are multiplexed, ordered from most to least priority.
    assets: Vec<Box<dyn I18nAssets + Send + Sync + 'static>>,
}

impl std::fmt::Debug for AssetsMultiplexor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetsMultiplexor")
            .field(
                "assets",
                &self.assets.iter().map(|_| "<ASSET>").collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl AssetsMultiplexor {
    /// Construct a new [`AssetsMultiplexor`]. `assets` are specified in order of priority of
    /// processing for the [`crate::LanguageLoader`].
    pub fn new(
        assets: impl IntoIterator<Item = Box<dyn I18nAssets + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            assets: assets.into_iter().collect(),
        }
    }
}

#[allow(dead_code)] // We rely on the Drop implementation of the Watcher to remove the file system watch.
struct Watchers(Vec<Box<dyn Watcher + Send + Sync + 'static>>);

impl Watcher for Watchers {}

impl I18nAssets for AssetsMultiplexor {
    fn get_files(&self, file_path: &str) -> Vec<Cow<'_, [u8]>> {
        self.assets
            .iter()
            .flat_map(|assets| assets.get_files(file_path))
            .collect()
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.assets
                .iter()
                .flat_map(|assets| assets.filenames_iter()),
        )
    }

    fn subscribe_changed(
        &self,
        changed: std::sync::Arc<dyn Fn() + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher + Send + Sync + 'static>, I18nEmbedError> {
        let watchers: Vec<_> = self
            .assets
            .iter()
            .map(|assets| assets.subscribe_changed(changed.clone()))
            .collect::<Result<_, _>>()?;
        Ok(Box::new(Watchers(watchers)))
    }
}

use std::{borrow::Cow, marker::PhantomData};

use rust_embed::RustEmbed;

use crate::I18nEmbedError;

/// A trait to handle the retrieval of localization assets.
pub trait I18nAssets {
    /// Get a localization asset (returns `None` if the asset does not
    /// exist, or unable to obtain the asset due to a non-critical
    /// error).
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>>;
    /// Get an iterator over the filenames of the localization assets.
    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>>;
    /// A method to allow users of this trait to subscribe to change events, and reload assets when
    /// they have changed. The subscription will be cancelled when the returned [`Watcher`] is
    /// dropped.
    fn subscribe_changed(
        &self,
        changed: Box<dyn Fn() -> () + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher>, I18nEmbedError>;
}

impl Watcher for () {}

impl<T> I18nAssets for T
where
    T: RustEmbed,
{
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>> {
        Self::get(file_path).map(|file| file.data)
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(Self::iter().map(|filename| filename.to_string()))
    }

    #[allow(unused_variables)]
    fn subscribe_changed(
        &self,
        changed: Box<dyn Fn() -> () + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher>, I18nEmbedError> {
        Ok(Box::new(()))
    }
}

#[cfg(feature = "rust-embed")]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RustEmbedAssetsError {
    #[error(transparent)]
    Notify(#[from] notify::Error),
}

pub struct RustEmbedNotifyAssets<T: rust_embed::RustEmbed> {
    base_dir: std::path::PathBuf,
    embed: PhantomData<T>,
}

impl<T: rust_embed::RustEmbed> RustEmbedNotifyAssets<T> {
    pub fn new(base_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            embed: PhantomData,
        }
    }
}

impl<T> I18nAssets for RustEmbedNotifyAssets<T>
where
    T: RustEmbed,
{
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>> {
        T::get(file_path).map(|file| file.data)
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(T::iter().map(|filename| filename.to_string()))
    }

    fn subscribe_changed(
        &self,
        changed: Box<dyn Fn() -> () + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher>, I18nEmbedError> {
        log::debug!("Watching for changed files in {:?}", self.base_dir);
        notify_watcher(&self.base_dir, changed)
            .map_err(Into::into)
    }
}

/// An [I18nAssets] implementation which pulls assets from the OS
/// file system.
#[cfg(feature = "filesystem-assets")]
#[derive(Debug)]
pub struct FileSystemAssets {
    base_dir: std::path::PathBuf,
}

#[cfg(feature = "filesystem-assets")]
impl FileSystemAssets {
    /// Create a new `FileSystemAssets` instance, all files will be
    /// read from within the specified base directory. Will panic if
    /// the specified `base_dir` does not exist, or is not a valid
    /// directory.
    pub fn new<P: Into<std::path::PathBuf>>(base_dir: P) -> Self {
        let base_dir = base_dir.into();

        if !base_dir.exists() {
            panic!("specified `base_dir` ({:?}) does not exist", base_dir);
        }

        if !base_dir.is_dir() {
            panic!("specified `base_dir` ({:?}) is not a directory", base_dir);
        }

        Self { base_dir }
    }
}

#[cfg(feature = "filesystem-assets")]
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FileSystemAssetsError {
    #[error(transparent)]
    Notify(#[from] notify::Error),
}

fn notify_watcher(
    base_dir: &std::path::Path,
    changed: Box<dyn Fn() -> () + Send + Sync + 'static>,
) -> notify::Result<Box<dyn Watcher>> {
    let mut watcher = notify::recommended_watcher(move |event_result| {
        let event: notify::Event = match event_result {
            Ok(event) => event,
            Err(error) => {
                log::error!("{error}");
                return;
            }
        };
    })?;

    notify::Watcher::watch(&mut watcher, base_dir, notify::RecursiveMode::Recursive)?;

    Ok(Box::new(watcher))
}

pub trait Watcher {}

impl Watcher for notify::RecommendedWatcher {}

#[cfg(feature = "filesystem-assets")]
impl I18nAssets for FileSystemAssets {
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>> {
        let full_path = self.base_dir.join(file_path);

        if !(full_path.is_file() && full_path.exists()) {
            return None;
        }

        match std::fs::read(full_path) {
            Ok(contents) => Some(Cow::from(contents)),
            Err(e) => {
                log::error!(
                    target: "i18n_embed::assets", 
                    "Unexpected error while reading localization asset file: {}", 
                    e);
                None
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

    // #[cfg(all(feature = "autoreload", feature = "filesystem-assets"))]
    fn subscribe_changed(
        &self,
        changed: Box<dyn Fn() -> () + Send + Sync + 'static>,
    ) -> Result<Box<dyn Watcher>, I18nEmbedError> {
        notify_watcher(&self.base_dir, changed)
            .map_err(Into::into)
    }
}

pub struct MultiAssets {
    assets: Vec<Box<dyn I18nAssets>>,
}

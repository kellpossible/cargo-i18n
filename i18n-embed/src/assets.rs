use std::{path::PathBuf, borrow::Cow};

/// A trait to handle the retrieval of localization assets.
pub trait I18nAssets {
    /// Get a localization asset (returns `None` if the asset does not exist).
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>>;
    /// Get an iterator over the filenames of the localization assets.
    fn filenames_iter(&self) ->  Box<dyn Iterator<Item = String>>;
}

#[cfg(feature = "rust-embed")]
impl<T> I18nAssets for T
where
    T: rust_embed::RustEmbed + 'static,
{
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>> {
        Self::get(file_path) 
    }

    fn filenames_iter(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(Self::iter().map(|filename| filename.to_string()))
    }
}

pub struct FileSystemAssets {
    base: PathBuf,
}

impl FileSystemAssets {
    /// Create a new `FileSystemAssets` instance.
    pub fn new<P: Into<PathBuf>>(base: P) -> Self {
        Self {
            base: base.into(),
        }
    }
}

impl I18nAssets for FileSystemAssets {
    fn get_file(&self, file_path: &str) -> Option<Cow<'_, [u8]>> {
        let full_path = self.base.join(file_path); 

        if !(full_path.is_file() && full_path.exists()) {
            return None;
        }

        match std::fs::read(full_path) {
            Ok(contents) => Some(Cow::from(contents)),
            Err(e) =>  {
                log::error!(
                    target: "i18n_embed::assets", 
                    "Unexpected error while reading localization asset file: {}", 
                    e);
                None
            }
        }
    }

    fn filenames_iter(&self) ->  Box<dyn Iterator<Item = String>> {
        Box::new(walkdir::WalkDir::new(&self.base).into_iter().filter_map(|f| {
            match f {
                Ok(f) => {
                    if f.file_type().is_file() {
                        match f.file_name().to_str() {
                            Some(filename) => {
                                Some(filename.to_string())
                            }
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
            }
        }))
    }
}

use std::fmt::Display;
use std::io;
use std::path::Path;
use thiserror::Error;
use tr::tr;

/// Type of path being represented in an error message.
#[derive(Debug)]
pub enum PathType {
    File,
    Directory,
    Symlink,
}

impl Display for PathType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathType::File => write!(f, "file"),
            PathType::Directory => write!(f, "directory"),
            PathType::Symlink => write!(f, "symbolic link"),
        }
    }
}

#[derive(Debug)]
pub enum PathErrorKind {
    NotValidUTF8 {
        for_item: String,
        path_type: PathType,
    },
    DoesNotExist,
    CannotCreateDirectory(io::Error),
    NotInsideDirectory(String, Box<Path>),
}

/// This error type collates all the various generic file/path related
/// errors in this application into one place, so that they can be
/// translated easily.
#[derive(Error, Debug)]
pub struct PathError {
    pub path: Box<Path>,
    pub kind: PathErrorKind,
}

impl PathError {
    /// An error for when a directory cannot be created.
    pub fn cannot_create_dir<P: Into<Box<Path>>>(path: P, source: io::Error) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::CannotCreateDirectory(source),
        }
    }
    
    /// An error for when the given path does not exist (when it was expected to exist).
    pub fn does_not_exist<P: Into<Box<Path>>>(path: P) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::DoesNotExist,
        }
    }

    /// An error for when the given path contains some characters
    /// which do not conform to the UTF-8 standard/encoding.
    pub fn not_valid_utf8<F: Into<String>, P: Into<Box<Path>>>(
        path: P,
        for_item: F,
        path_type: PathType,
    ) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::NotValidUTF8 {
                for_item: for_item.into(),
                path_type,
            },
        }
    }

    /// An error for when the given path is not inside another given
    /// path which is a directory.
    pub fn not_inside_dir<S: Into<String>, P: Into<Box<Path>>>(
        path: P,
        parent_name: S,
        parent_path: P,
    ) -> PathError {
        PathError {
            path: path.into(),
            kind: PathErrorKind::NotInsideDirectory(parent_name.into(), parent_path.into()),
        }
    }
}

impl Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self.kind {
            PathErrorKind::NotValidUTF8 {
                for_item,
                path_type,
            } => {
                
                tr!(
                    // {0} is the file path, {1} is the item which it is for, {2} is the type of item (file, directory, etc)
                    "The path (\"{0}\") for {1} {2} does not have valid a utf-8 encoding.",
                    self.path.to_string_lossy(),
                    for_item,
                    path_type
                )
            }
            PathErrorKind::DoesNotExist => tr!(
                "The path {0} does not exist on the filesystem.",
                self.path.to_string_lossy()
            ),
            PathErrorKind::CannotCreateDirectory(source) => tr!(
                "Cannot create the directory \"{0}\" because: \"{1}\".",
                self.path.to_string_lossy(),
                source
            ),
            PathErrorKind::NotInsideDirectory(parent_name, parent_dir) => tr!(
                "The path \"{0}\" is not inside the \"{1}\" directory {2}.",
                self.path.to_string_lossy(),
                parent_name,
                parent_dir.to_string_lossy(),
            ),
        };

        write!(f, "{}", message)
    }
}

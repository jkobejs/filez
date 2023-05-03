use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use async_trait::async_trait;
use tokio::{fs, fs::File, io::AsyncReadExt, io::AsyncWriteExt};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait Files: Send + Sync {
    /// Reads the content of a file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that represents the path to the file to be read.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use filez::{live, Files};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let files = live("path/to/root".to_string());
    ///     let content = files.read("path/to/file.txt").await.unwrap();
    ///     println!("{}", content);
    /// }
    /// ```
    async fn read(&self, path: &str) -> Result<String, ReadError>;
    /// Writes the specified content to a file at the specified path.
    /// If the directory does not exist, it will be created.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A string slice that represents the path to the file to be written.
    /// * `content` - A string slice that represents the content to be written to the file.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created or the file cannot be written to.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use filez::{live, Files};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let files = live("path/to/root".to_string());
    ///     files.write("path/to/file.txt", "Hello, world!").await.unwrap();
    /// }
    /// ```
    async fn write(&self, path: &str, content: &str) -> Result<(), WriteError>;
    /// Lists all files that match the specified glob expression.
    ///
    /// # Arguments
    ///
    /// * `expression` - A string slice that represents the glob expression to match files against.
    ///
    /// # Errors
    ///
    /// Returns an error if the glob expression cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use filez::{live, Files};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let files = live("path/to/root".to_string());
    ///     let files = files.list("path/to/*.txt").unwrap();
    ///     println!("{:?}", files);
    /// }
    ///
    fn list(&self, expresson: &str) -> Result<Vec<String>, ListError>;
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ReadError {
    pub path: String,
    pub source: std::io::Error,
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error reading  `{}`", self.path)
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct WriteError {
    pub path: String,
    pub source: std::io::Error,
}

impl Display for WriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error writing `{}`", self.path)
    }
}

impl Error for WriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ListError {
    pub expression: String,
    pub kind: ListErrorKind,
}

#[derive(Debug)]
pub enum ListErrorKind {
    ParseGlob(glob::PatternError),
    ReadPath(glob::GlobError),
}

impl Display for ListError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "error listing `{}`", self.expression)
    }
}

impl Error for ListError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            ListErrorKind::ParseGlob(err) => Some(err),
            ListErrorKind::ReadPath(err) => Some(err),
        }
    }
}

/// Creates a new instance of `ParentDirectory` that uses the specified parent directory.
pub fn live(parent: String) -> impl Files {
    ParentDirectory::new(parent)
}

#[derive(Clone)]
struct ParentDirectory {
    pub parent: String,
}

impl ParentDirectory {
    #[must_use]
    pub const fn new(root: String) -> Self {
        Self { parent: root }
    }

    fn with_parent(&self, path: &str) -> String {
        format!("{}/{}", self.parent, path)
    }
}

#[async_trait]
impl Files for ParentDirectory {
    async fn read(&self, path: &str) -> Result<String, ReadError> {
        let mut file = File::open(self.with_parent(path))
            .await
            .map_err(|source| ReadError {
                path: path.to_string(),
                source,
            })?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .await
            .map_err(|source| ReadError {
                path: path.to_string(),
                source,
            })?;
        Ok(content)
    }

    async fn write(&self, path: &str, content: &str) -> Result<(), WriteError> {
        let path_with_parent = self.with_parent(path);
        let dir_path = std::path::Path::new(&path_with_parent)
            .parent()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Could not get parent directory")
            })
            .map_err(|source| WriteError {
                path: path.to_string(),
                source,
            })?;
        fs::create_dir_all(dir_path)
            .await
            .map_err(|source| WriteError {
                path: path.to_string(),
                source,
            })?;
        let mut file = File::create(path_with_parent)
            .await
            .map_err(|source| WriteError {
                path: path.to_string(),
                source,
            })?;
        file.write_all(content.as_bytes())
            .await
            .map_err(|source| WriteError {
                path: path.to_string(),
                source,
            })?;
        Ok(())
    }

    fn list(&self, expression: &str) -> Result<Vec<String>, ListError> {
        let mut paths = vec![];

        for path_buf_result in
            glob::glob(&format!("{}/{expression}", self.parent)).map_err(|err| ListError {
                expression: expression.to_string(),
                kind: ListErrorKind::ParseGlob(err),
            })?
        {
            let path_buf = path_buf_result.map_err(|err| ListError {
                expression: expression.to_string(),
                kind: ListErrorKind::ReadPath(err),
            })?;
            if path_buf.is_file() {
                if let Some(path) = path_buf.as_path().to_str() {
                    paths.push(path.to_string());
                }
            }
        }
        Ok(paths)
    }
}

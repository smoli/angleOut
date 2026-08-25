//! The level asset: a `assets/levels/*.ron` file as the asset server hands it back.
//!
//! Going through the asset server rather than reading the files once at startup
//! is what buys hot reload - Bevy's file watcher notices a hand edit, re-runs
//! [`LevelAssetLoader`] and swaps the new value into `Assets<LevelAsset>` under
//! the same handle, so everything holding that handle sees the edit.

use std::error::Error;
use std::fmt;
use std::str::Utf8Error;

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::reflect::TypePath;

use crate::level::campaign::parse_level;
use crate::level::LevelDefinition;

/// One level file.
///
/// A newtype rather than an `impl Asset for LevelDefinition` so "the level on
/// disk" stays distinct from "the level a match or the editor works with" - the
/// editor edits and saves a [`LevelDefinition`], and only the asset server ever
/// makes a `LevelAsset`.
#[derive(Asset, TypePath, Debug, Clone, PartialEq)]
pub struct LevelAsset(pub LevelDefinition);

impl std::ops::Deref for LevelAsset {
    type Target = LevelDefinition;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Reads a [`LevelAsset`] out of the RON in a level file.
#[derive(Default, TypePath)]
pub struct LevelAssetLoader;

#[derive(Debug)]
pub enum LevelAssetError {
    Io(std::io::Error),
    Utf8(Utf8Error),
    Ron(ron::error::SpannedError),
}

impl fmt::Display for LevelAssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LevelAssetError::Io(e) => write!(f, "reading the level file: {e}"),
            LevelAssetError::Utf8(e) => write!(f, "a level file must be UTF-8: {e}"),
            LevelAssetError::Ron(e) => write!(f, "parsing the level: {e}"),
        }
    }
}

impl Error for LevelAssetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            LevelAssetError::Io(e) => Some(e),
            LevelAssetError::Utf8(e) => Some(e),
            LevelAssetError::Ron(e) => Some(e),
        }
    }
}

impl From<std::io::Error> for LevelAssetError {
    fn from(e: std::io::Error) -> Self {
        LevelAssetError::Io(e)
    }
}

impl From<Utf8Error> for LevelAssetError {
    fn from(e: Utf8Error) -> Self {
        LevelAssetError::Utf8(e)
    }
}

impl From<ron::error::SpannedError> for LevelAssetError {
    fn from(e: ron::error::SpannedError) -> Self {
        LevelAssetError::Ron(e)
    }
}

impl AssetLoader for LevelAssetLoader {
    type Asset = LevelAsset;
    type Settings = ();
    type Error = LevelAssetError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        Ok(LevelAsset(parse_level(std::str::from_utf8(&bytes)?)?))
    }

    /// Levels are always asked for as a `Handle<LevelAsset>`, and the asset
    /// server resolves a loader by asset type before it looks at an extension -
    /// so a second RON asset type added later still gets its own loader, and
    /// this list is not what routes a level here.
    ///
    /// It matters for one case: when a reload of an already-typed handle fails,
    /// the asset server retries the path untyped, and with nothing registered
    /// that retry logs `Could not find an asset loader matching` on top of the
    /// real parse error. Claiming `ron` keeps a typo in a level file reporting
    /// only what is actually wrong with it.
    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

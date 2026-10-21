//! @file theme.rs
//! @brief Theme packs: loads `themes/<name>/theme.ron` and tells the pieces which model to use.
//! @note The only place asset paths are built. Without a theme, pieces use placeholder shapes.

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadState};
use bevy::prelude::*;
use chess_core::{Color as Side, Piece, PieceKind};
use serde::Deserialize;

use crate::turn::GameSet;

/// @brief Environment variable that picks a theme, e.g. `CHESS_THEME=test` for the generated test theme.
const THEME_ENV: &str = "CHESS_THEME";

/// @brief The built-in theme: placeholder shapes, nothing to load.
/// @note Becomes "stone" once the stone pieces exist (beta art checkpoint).
const PLACEHOLDER_THEME: &str = "placeholder";

/// @brief Loads the chosen theme at startup and announces when its models can be used.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ThemeManifest>()
            .register_asset_loader(ThemeManifestLoader)
            .init_resource::<ThemeStatus>()
            .add_message::<ThemeReady>()
            .add_systems(Startup, load_theme)
            .add_systems(Update, finish_loading_theme.in_set(GameSet::Turn));
    }
}

/// @brief Contents of a `theme.ron`: the theme's name and a model file per piece and color.
/// @note Paths are relative to the theme folder. A missing entry falls back to the placeholder shape.
#[derive(Asset, TypePath, Deserialize, Debug, Clone, PartialEq)]
pub struct ThemeManifest {
    /// @brief Display name, e.g. "Stone".
    pub name: String,
    /// @brief White's models.
    pub white: PieceModels,
    /// @brief Black's models.
    pub black: PieceModels,
}

/// @brief One model file per piece kind; any of them may be left out.
#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default)]
pub struct PieceModels {
    /// @brief Pawn model, e.g. "white/pawn.glb".
    pub pawn: Option<String>,
    /// @brief Knight model.
    pub knight: Option<String>,
    /// @brief Bishop model.
    pub bishop: Option<String>,
    /// @brief Rook model.
    pub rook: Option<String>,
    /// @brief Queen model.
    pub queen: Option<String>,
    /// @brief King model.
    pub king: Option<String>,
}

impl PieceModels {
    /// @brief The model file for a piece kind, if the theme has one.
    fn get(&self, kind: PieceKind) -> Option<&str> {
        match kind {
            PieceKind::Pawn => self.pawn.as_deref(),
            PieceKind::Knight => self.knight.as_deref(),
            PieceKind::Bishop => self.bishop.as_deref(),
            PieceKind::Rook => self.rook.as_deref(),
            PieceKind::Queen => self.queen.as_deref(),
            PieceKind::King => self.king.as_deref(),
        }
    }
}

/// @brief Which theme is in use and whether it has finished loading.
#[derive(Resource, Debug, Default)]
pub enum ThemeStatus {
    /// @brief No theme: placeholder shapes.
    #[default]
    Placeholder,
    /// @brief The manifest is loading; pieces use placeholders until it's ready.
    Loading {
        /// @brief Theme folder inside `assets`, e.g. "themes/stone".
        folder: String,
        /// @brief The manifest being loaded.
        handle: Handle<ThemeManifest>,
    },
    /// @brief The manifest is loaded; pieces use its models.
    Ready {
        /// @brief Theme folder inside `assets`.
        folder: String,
        /// @brief The loaded manifest (boxed: it's much larger than the other variants).
        manifest: Box<ThemeManifest>,
    },
}

impl ThemeStatus {
    /// @brief Asset path of a piece's model, or None to use a placeholder.
    pub fn model_path(&self, piece: Piece) -> Option<String> {
        let ThemeStatus::Ready { folder, manifest } = self else {
            return None;
        };
        let models = match piece.color {
            Side::White => &manifest.white,
            Side::Black => &manifest.black,
        };
        models
            .get(piece.kind)
            .map(|file| format!("{folder}/{file}"))
    }
}

/// @brief The theme finished loading (or fell back to placeholders); pieces respawn with it.
#[derive(Message, Debug, Clone, Copy)]
pub struct ThemeReady;

/// @brief Reads `.ron` theme manifests through Bevy's asset system (works inside packaged builds too).
#[derive(Default, TypePath)]
struct ThemeManifestLoader;

/// @brief Why a manifest couldn't be read.
#[derive(Debug, thiserror::Error)]
enum ThemeLoadError {
    /// @brief The file couldn't be read.
    #[error("could not read theme manifest: {0}")]
    Io(#[from] std::io::Error),
    /// @brief The file isn't valid RON for a `ThemeManifest`.
    #[error("invalid theme manifest: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

impl AssetLoader for ThemeManifestLoader {
    type Asset = ThemeManifest;
    type Settings = ();
    type Error = ThemeLoadError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<ThemeManifest, ThemeLoadError> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(ron::de::from_bytes(&bytes)?)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// @brief Starts loading the theme named by `CHESS_THEME`; the placeholder theme needs no loading.
fn load_theme(asset_server: Res<AssetServer>, mut status: ResMut<ThemeStatus>) {
    let name = std::env::var(THEME_ENV).unwrap_or_else(|_| PLACEHOLDER_THEME.to_string());
    if name == PLACEHOLDER_THEME {
        info!("using placeholder pieces");
        return;
    }
    let folder = format!("themes/{name}");
    info!(theme = %name, "loading theme");
    *status = ThemeStatus::Loading {
        handle: asset_server.load(format!("{folder}/theme.ron")),
        folder,
    };
}

/// @brief Switches to the theme once its manifest has loaded, or back to placeholders if it failed.
fn finish_loading_theme(
    asset_server: Res<AssetServer>,
    manifests: Res<Assets<ThemeManifest>>,
    mut status: ResMut<ThemeStatus>,
    mut ready: MessageWriter<ThemeReady>,
) {
    let ThemeStatus::Loading { folder, handle } = &*status else {
        return;
    };
    let next = match asset_server.load_state(handle) {
        LoadState::Loaded => {
            let Some(manifest) = manifests.get(handle) else {
                return;
            };
            info!(theme = %manifest.name, "theme loaded");
            ThemeStatus::Ready {
                folder: folder.clone(),
                manifest: Box::new(manifest.clone()),
            }
        }
        LoadState::Failed(error) => {
            warn!(%error, "theme failed to load, using placeholder pieces");
            ThemeStatus::Placeholder
        }
        _ => return,
    };
    *status = next;
    ready.write(ThemeReady);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @brief A manifest parses, missing pieces stay empty, and paths include the theme folder.
    #[test]
    fn manifest_parses_and_builds_paths() {
        let manifest: ThemeManifest = ron::de::from_str(
            r#"(
                name: "Test",
                white: (pawn: Some("white/pawn.glb"), king: Some("white/king.glb")),
                black: (),
            )"#,
        )
        .expect("valid manifest");
        assert_eq!(manifest.white.get(PieceKind::Pawn), Some("white/pawn.glb"));
        assert_eq!(manifest.white.get(PieceKind::Queen), None);

        let status = ThemeStatus::Ready {
            folder: "themes/test".to_string(),
            manifest: Box::new(manifest),
        };
        let white_king = Piece {
            kind: PieceKind::King,
            color: Side::White,
        };
        let black_king = Piece {
            kind: PieceKind::King,
            color: Side::Black,
        };
        assert_eq!(
            status.model_path(white_king).as_deref(),
            Some("themes/test/white/king.glb")
        );
        assert_eq!(status.model_path(black_king), None);
        assert_eq!(ThemeStatus::Placeholder.model_path(white_king), None);
    }
}

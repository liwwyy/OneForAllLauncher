//! Local data and actions for the existing Wardrobe views.
use oneclient_auth::MinecraftAccount;
use oneclient_core::local_skin::{
    catalog,
    library::{self, Action, Kind, Snapshot},
};
use oneclient_mc::{MojangCape, MojangFullPlayerProfile, MojangSkin, SkinVariant};
use oneclient_polyplus::api::{OutfitInfo, OutfitList, SkinInfo, SkinList, SkinModel};
use uuid::Uuid;

use super::{CapeImportSource, WardrobeAction, mojang_read_message};
use crate::i18n::{t, tf};

pub fn look(account: Uuid, snapshot: &Snapshot) -> MojangFullPlayerProfile {
    let lib = &snapshot.library;
    let state = |id, selected| {
        if Some(id) == selected {
            "ACTIVE"
        } else {
            "INACTIVE"
        }
        .into()
    };
    let skins = lib
        .textures
        .iter()
        .filter(|skin| Some(skin.id) == lib.skin)
        .map(|skin| MojangSkin {
            id: skin.id.to_string(),
            url: skin.id.to_string(),
            state: state(skin.id, lib.skin),
            variant: if skin.slim {
                SkinVariant::Slim
            } else {
                SkinVariant::Classic
            },
        })
        .collect();
    let mut capes: Vec<_> = catalog::capes()
        .map(|cape| MojangCape {
            id: cape.id().to_string(),
            alias: cape.name.into(),
            url: cape.url(),
            state: state(cape.id(), lib.cape),
        })
        .collect();
    for cape in lib.textures.iter().filter(|item| item.kind == Kind::Cape) {
        let id = cape.id.to_string();
        if !capes.iter().any(|existing| existing.id == id) {
            capes.push(MojangCape {
                id,
                alias: cape.name.clone(),
                url: String::new(),
                state: state(cape.id, lib.cape),
            });
        }
    }
    MojangFullPlayerProfile {
        id: account.to_string(),
        username: String::new(),
        skins,
        capes,
    }
}

pub fn skins(snapshot: &Snapshot) -> SkinList {
    SkinList {
        limit: library::LIMIT,
        skins: snapshot
            .library
            .textures
            .iter()
            .filter(|item| item.kind == Kind::Skin)
            .map(|item| SkinInfo {
                id: item.number,
                name: item.name.clone(),
                url: item.id.to_string(),
                hash: item.id.to_string(),
                model: if item.slim {
                    SkinModel::Slim
                } else {
                    SkinModel::Classic
                },
                favorite: item.favorite,
                created_at: String::new(),
            })
            .collect(),
    }
}

pub fn outfits(snapshot: &Snapshot) -> OutfitList {
    let lib = &snapshot.library;
    OutfitList {
        limit: library::LIMIT,
        active_outfit_id: lib
            .outfits
            .iter()
            .find(|o| Some(o.id) == lib.outfit)
            .map(|o| o.number),
        outfits: lib
            .outfits
            .iter()
            .map(|o| OutfitInfo {
                id: o.number,
                name: o.name.clone(),
                favorite: o.favorite,
                skin_id: lib
                    .textures
                    .iter()
                    .find(|s| Some(s.id) == o.skin)
                    .map(|s| s.number),
                mojang_cape_id: o.cape.map(|id| id.to_string()),
                particle_color: None,
                blank: false,
                equipped: Default::default(),
                created_at: String::new(),
                updated_at: String::new(),
            })
            .collect(),
    }
}

async fn item_id(account: Uuid, number: i32, outfit: bool) -> Result<Uuid, String> {
    let lib = library::metadata(account)
        .await
        .map_err(|err| err.to_string())?;
    let id = if outfit {
        lib.outfits
            .iter()
            .find(|item| item.number == number)
            .map(|item| item.id)
    } else {
        lib.textures
            .iter()
            .find(|item| item.number == number && item.kind == Kind::Skin)
            .map(|item| item.id)
    };
    id.ok_or_else(|| t("fork.wardrobe.missing_item").into())
}

fn id(value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|_| t("fork.wardrobe.missing_item").into())
}

fn file_name(path: &std::path::Path) -> Option<String> {
    path.file_stem()
        .map(|stem| {
            stem.to_string_lossy()
                .chars()
                .filter(|c| !c.is_control())
                .take(64)
                .collect::<String>()
        })
        .filter(|name| !name.trim().is_empty())
}

pub(super) async fn run(
    account: MinecraftAccount,
    action: WardrobeAction,
) -> Result<String, String> {
    let state = crate::launcher::state().map_err(|err| err.to_string())?;
    let requester = &state.services.requester;
    let account_id = account.id;
    let outfit = matches!(
        &action,
        WardrobeAction::FavoriteOutfit { .. } | WardrobeAction::DeleteOutfit { .. }
    );
    let (change, status) = match action {
        WardrobeAction::MojangCape { cape, name, .. } => {
            let selected = cape.as_deref().map(id).transpose()?;
            if let Some(cape) = catalog::capes().find(|cape| Some(cape.id()) == selected) {
                library::select_catalog_cape(&account, cape, requester)
                    .await
                    .map_err(|err| err.to_string())?;
                return Ok(tf("oneclient.wardrobe.cape.equipped", &[&name]));
            }
            let message = if selected.is_some() {
                "oneclient.wardrobe.cape.equipped"
            } else {
                "oneclient.wardrobe.cape.removed"
            };
            (
                Action::Select {
                    kind: Kind::Cape,
                    id: selected,
                },
                tf(message, &[&name]),
            )
        }
        WardrobeAction::WearSkin { url, name, .. } => (
            Action::Select {
                kind: Kind::Skin,
                id: Some(id(&url)?),
            },
            tf("oneclient.wardrobe.wear.wearing_done", &[&name]),
        ),
        WardrobeAction::ResetLocalSkin { .. } => (
            Action::Select {
                kind: Kind::Skin,
                id: None,
            },
            t("fork.wardrobe.default_skin_active").into(),
        ),
        WardrobeAction::AddLocalCape { source, name, .. } => {
            let (bytes, name) = match source {
                CapeImportSource::File(path) => (
                    library::read_file(&path)
                        .await
                        .map_err(|err| tf("fork.wardrobe.cape_file_read_failed", &[&err]))?,
                    name.or_else(|| file_name(&path)),
                ),
                CapeImportSource::Url(url) => (
                    library::download(requester, &url)
                        .await
                        .map_err(|err| err.to_string())?,
                    name,
                ),
            };
            let name = name.unwrap_or_else(|| t("fork.wardrobe.imported_cape").into());
            let status = tf("oneclient.wardrobe.cape.equipped", &[&name]);
            (
                Action::Import {
                    kind: Kind::Cape,
                    name,
                    bytes,
                    slim: false,
                    wear: true,
                },
                status,
            )
        }
        WardrobeAction::DeleteLocalCape {
            id: cape_id, name, ..
        } => {
            let id = id(&cape_id)?;
            let lib = library::metadata(account_id)
                .await
                .map_err(|err| err.to_string())?;
            if catalog::capes().any(|cape| cape.id() == id)
                || !lib
                    .textures
                    .iter()
                    .any(|item| item.id == id && item.kind == Kind::Cape)
            {
                return Err(t("fork.wardrobe.missing_item").into());
            }
            (
                Action::Delete { id },
                tf("oneclient.wardrobe.deleted", &[&name]),
            )
        }
        WardrobeAction::AddSkinFile {
            path, slim, name, ..
        } => {
            let bytes = library::read_file(&path)
                .await
                .map_err(|err| tf("oneclient.wardrobe.skin.file_read_failed", &[&err]))?;
            let name = name
                .or_else(|| file_name(&path))
                .unwrap_or_else(|| t("fork.wardrobe.imported_skin").into());
            let status = tf("oneclient.wardrobe.wear.wearing_done", &[&name]);
            (
                Action::Import {
                    kind: Kind::Skin,
                    name,
                    bytes,
                    slim,
                    wear: true,
                },
                status,
            )
        }
        WardrobeAction::AddSkinUrl {
            url, slim, name, ..
        } => {
            let bytes = library::download(requester, &url)
                .await
                .map_err(|err| err.to_string())?;
            let name = name.unwrap_or_else(|| t("fork.wardrobe.imported_skin").into());
            let status = tf("oneclient.wardrobe.wear.wearing_done", &[&name]);
            (
                Action::Import {
                    kind: Kind::Skin,
                    name,
                    bytes,
                    slim,
                    wear: true,
                },
                status,
            )
        }
        WardrobeAction::AddSkinUsername { username, .. } => {
            let (uuid, name) = oneclient_mc::resolve_username(requester, &username)
                .await
                .map_err(|err| {
                    if matches!(err, oneclient_mc::McError::Minecraft(_))
                        || matches!(err.http_status(), Some(204 | 404))
                    {
                        tf("oneclient.wardrobe.skin.player_not_found", &[&username])
                    } else {
                        mojang_read_message(err)
                    }
                })?;
            let profile = oneclient_mc::fetch_player_profile(requester, &uuid)
                .await
                .map_err(mojang_read_message)?;
            let Some(url) = profile.skin_url.as_deref() else {
                return Err(tf("oneclient.wardrobe.skin.default_skin", &[&name]));
            };
            let bytes = library::download(requester, url)
                .await
                .map_err(|err| err.to_string())?;
            let status = tf("oneclient.wardrobe.skin.saved_player", &[&name]);
            (
                Action::Import {
                    kind: Kind::Skin,
                    name,
                    bytes,
                    slim: profile.is_slim,
                    wear: false,
                },
                status,
            )
        }
        WardrobeAction::RenameSkin { id, old, name, .. } => (
            Action::Rename {
                id: item_id(account_id, id, false).await?,
                name: name.clone(),
            },
            tf("oneclient.wardrobe.renamed", &[&old, &name.trim()]),
        ),
        WardrobeAction::RenameOutfit { id, old, name, .. } => (
            Action::Rename {
                id: item_id(account_id, id, true).await?,
                name: name.clone(),
            },
            tf("oneclient.wardrobe.renamed", &[&old, &name.trim()]),
        ),
        WardrobeAction::FavoriteSkin {
            id, favorite, name, ..
        }
        | WardrobeAction::FavoriteOutfit {
            id, favorite, name, ..
        } => {
            let message = if favorite {
                "oneclient.wardrobe.favorite.added"
            } else {
                "oneclient.wardrobe.favorite.removed"
            };
            (
                Action::Favorite {
                    id: item_id(account_id, id, outfit).await?,
                    value: favorite,
                },
                tf(message, &[&name]),
            )
        }
        WardrobeAction::SkinArms { id, slim, name, .. } => {
            let message = if slim {
                "oneclient.wardrobe.skin.arms.slim"
            } else {
                "oneclient.wardrobe.skin.arms.classic"
            };
            (
                Action::Model {
                    id: item_id(account_id, id, false).await?,
                    slim,
                },
                tf(message, &[&name]),
            )
        }
        WardrobeAction::DeleteSkin { id, name, .. }
        | WardrobeAction::DeleteOutfit { id, name, .. } => (
            Action::Delete {
                id: item_id(account_id, id, outfit).await?,
            },
            tf("oneclient.wardrobe.deleted", &[&name]),
        ),
        WardrobeAction::ApplyOutfit { id, name, .. } => (
            Action::ApplyOutfit {
                id: item_id(account_id, id, true).await?,
            },
            tf("oneclient.wardrobe.wear.wearing_done", &[&name]),
        ),
        WardrobeAction::CreateOutfit { .. } => {
            let lib = library::metadata(account_id)
                .await
                .map_err(|err| err.to_string())?;
            let name = (1..=library::LIMIT + 1)
                .map(|number| tf("fork.wardrobe.outfit_name", &[&number]))
                .find(|name| lib.outfits.iter().all(|item| item.name != *name))
                .unwrap_or_else(|| t("oneclient.wardrobe.preview.outfit").into());
            let status = tf("oneclient.wardrobe.outfit.saved", &[&name]);
            (Action::SaveOutfit { name }, status)
        }
        _ => return Err(t("fork.wardrobe.cosmetics_disabled").into()),
    };
    library::apply(&account, change)
        .await
        .map_err(|err| err.to_string())?;
    Ok(status)
}

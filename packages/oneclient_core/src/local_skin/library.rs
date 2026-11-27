//! Per-account local wardrobe. Paths are derived from UUIDs, never display names.
use super::{LocalSkin, catalog, directory};
use crate::{LauncherError, LauncherResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use uuid::Uuid;

pub const LIMIT: u64 = 256;

static WRITES: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Skin,
    Cape,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Texture {
    #[serde(default)]
    pub number: i32,
    pub id: Uuid,
    pub name: String,
    pub kind: Kind,
    pub slim: bool,
    pub favorite: bool,
    #[serde(default)]
    legacy: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outfit {
    #[serde(default)]
    pub number: i32,
    #[serde(default)]
    pub slim: bool,
    pub id: Uuid,
    pub name: String,
    pub skin: Option<Uuid>,
    pub cape: Option<Uuid>,
    pub favorite: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub outfit: Option<Uuid>,
    version: u32,
    #[serde(default)]
    last_number: i32,
    pub textures: Vec<Texture>,
    pub outfits: Vec<Outfit>,
    pub skin: Option<Uuid>,
    pub cape: Option<Uuid>,
}
impl Default for Library {
    fn default() -> Self {
        Self {
            version: 1,
            last_number: 0,
            outfit: None,
            textures: vec![],
            outfits: vec![],
            skin: None,
            cape: None,
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub library: Library,
    pub images: Vec<(Uuid, bytes::Bytes)>,
    pub unavailable: Vec<Uuid>,
}
impl Snapshot {
    pub fn image(&self, id: Option<Uuid>) -> Option<bytes::Bytes> {
        id.and_then(|id| {
            self.images
                .iter()
                .find(|(key, _)| *key == id)
                .map(|(_, b)| b.clone())
        })
    }
}
#[derive(Clone)]
pub enum Action {
    Import {
        name: String,
        kind: Kind,
        bytes: Vec<u8>,
        slim: bool,
        wear: bool,
    },
    SelectCatalogCape {
        cape: catalog::Cape,
        bytes: Vec<u8>,
    },
    Select {
        kind: Kind,
        id: Option<Uuid>,
    },
    Rename {
        id: Uuid,
        name: String,
    },
    Favorite {
        id: Uuid,
        value: bool,
    },
    Model {
        id: Uuid,
        slim: bool,
    },
    Delete {
        id: Uuid,
    },
    SaveOutfit {
        name: String,
    },
    ApplyOutfit {
        id: Uuid,
    },
}
fn invalid(message: impl Into<String>) -> LauncherError {
    LauncherError::Minecraft(message.into())
}
fn name(value: String) -> LauncherResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 64 || value.chars().any(char::is_control) {
        return Err(invalid(
            "Use a name between 1 and 64 characters, without control characters.",
        ));
    }
    Ok(value.into())
}
pub(super) fn selected_path(root: &Path, t: &Texture) -> PathBuf {
    root.join(if t.legacy {
        "skin.png".into()
    } else {
        format!("{}.png", t.id)
    })
}
fn validate_library(lib: &Library) -> LauncherResult<()> {
    if lib.version != 1 {
        return Err(invalid(
            "This wardrobe was saved by an unsupported launcher version.",
        ));
    }
    let mut ids = HashSet::new();
    let mut numbers = HashSet::new();
    for number in lib
        .textures
        .iter()
        .map(|t| t.number)
        .chain(lib.outfits.iter().map(|o| o.number))
    {
        if !(1..=lib.last_number).contains(&number) || !numbers.insert(number) {
            return Err(invalid("Invalid wardrobe item number."));
        }
    }

    for t in &lib.textures {
        if !ids.insert(t.id) || (t.legacy && t.kind != Kind::Skin) {
            return Err(invalid("Invalid wardrobe texture metadata."));
        }
    }
    for o in &lib.outfits {
        if !ids.insert(o.id) {
            return Err(invalid("Duplicate wardrobe identifier."));
        }
    }
    if lib
        .outfit
        .is_some_and(|id| !lib.outfits.iter().any(|o| o.id == id))
    {
        return Err(invalid("Unknown active outfit."));
    }
    for (skin, cape) in
        std::iter::once((lib.skin, lib.cape)).chain(lib.outfits.iter().map(|o| (o.skin, o.cape)))
    {
        for (id, kind) in [(skin, Kind::Skin), (cape, Kind::Cape)] {
            if let Some(id) = id
                && !lib.textures.iter().any(|t| t.id == id && t.kind == kind)
            {
                return Err(invalid("A wardrobe selection refers to a missing texture."));
            }
        }
    }
    Ok(())
}
async fn read(root: &Path, account: Uuid) -> LauncherResult<Library> {
    if root.join("wardrobe.json").is_file() {
        let mut lib: Library = polyio::read_json(root.join("wardrobe.json")).await?;
        for number in lib
            .textures
            .iter_mut()
            .map(|t| &mut t.number)
            .chain(lib.outfits.iter_mut().map(|o| &mut o.number))
        {
            if *number == 0 {
                *number = next_number(&mut lib.last_number)?;
            }
        }
        validate_library(&lib)?;
        return Ok(lib);
    }
    let mut lib = Library::default();
    if root.join("skin.json").is_file() {
        let skin: LocalSkin = polyio::read_json(root.join("skin.json")).await?;
        lib.textures.push(Texture {
            id: account,
            number: next_number(&mut lib.last_number)?,
            name: "Imported skin".into(),
            kind: Kind::Skin,
            slim: skin.slim,
            favorite: false,
            legacy: true,
        });
        lib.skin = Some(account);
    }
    Ok(lib)
}
fn next_number(last: &mut i32) -> LauncherResult<i32> {
    *last = last
        .checked_add(1)
        .ok_or_else(|| invalid("No wardrobe item numbers remain."))?;
    Ok(*last)
}

pub async fn metadata(account: Uuid) -> LauncherResult<Library> {
    let _lock = WRITES.lock().await;
    read(&directory(account)?, account).await
}

pub async fn select_catalog_cape(
    account: &oneclient_auth::MinecraftAccount,
    cape: catalog::Cape,
    requester: &oneclient_net::RequestClient,
) -> LauncherResult<()> {
    if !account.is_offline() {
        return Err(invalid(
            "The local wardrobe is only available for offline accounts.",
        ));
    }
    let root = directory(account.id)?;
    let id = cape.id();
    let path = root.join(format!("{id}.png"));
    let bytes = match read_file(&path).await {
        Ok(bytes) if cape.matches(&bytes) => bytes,
        _ => cape.download(requester).await?,
    };
    apply_at(&root, account.id, Action::SelectCatalogCape { cape, bytes }).await
}

pub async fn snapshot(account: Uuid) -> LauncherResult<Snapshot> {
    snapshot_at(&directory(account)?, account).await
}
async fn snapshot_at(root: &Path, account: Uuid) -> LauncherResult<Snapshot> {
    let _lock = WRITES.lock().await;
    let library = read(root, account).await?;
    let mut images = Vec::new();
    let mut unavailable = Vec::new();
    for t in &library.textures {
        match read_file(&selected_path(root, t)).await {
            Ok(bytes) if validate_texture(&bytes, t.kind).is_ok() => {
                images.push((t.id, bytes.into()))
            }
            _ => unavailable.push(t.id),
        }
    }
    Ok(Snapshot {
        library,
        images,
        unavailable,
    })
}

pub(super) async fn launch_selection(
    account: Uuid,
) -> LauncherResult<(Option<PathBuf>, Option<PathBuf>, bool)> {
    selection_at(&directory(account)?, account).await
}
async fn selection_at(
    root: &Path,
    account: Uuid,
) -> LauncherResult<(Option<PathBuf>, Option<PathBuf>, bool)> {
    let _lock = WRITES.lock().await;
    let lib = read(root, account).await?;
    let mut paths = [None, None];
    let mut slim = false;
    for (index, id) in [lib.skin, lib.cape].into_iter().enumerate() {
        if let Some(t) = lib.textures.iter().find(|t| Some(t.id) == id) {
            let path = selected_path(root, t);
            let bytes = read_file(&path).await?;
            validate_texture(&bytes, t.kind)?;
            paths[index] = Some(path);
            if index == 0 {
                slim = t.slim;
            }
        }
    }
    let [skin, cape] = paths;
    Ok((skin, cape, slim))
}

fn dimensions(bytes: &[u8]) -> LauncherResult<(u32, u32)> {
    if bytes.len() > 1024 * 1024 {
        return Err(invalid("The PNG must be smaller than 1 MB."));
    }
    image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png)
        .into_dimensions()
        .map_err(|_| invalid("Choose a valid PNG image."))
}

pub fn validate_texture(bytes: &[u8], kind: Kind) -> LauncherResult<()> {
    let expected = match kind {
        Kind::Skin => (64, 64),
        Kind::Cape => (64, 32),
    };
    if dimensions(bytes)? != expected {
        return Err(invalid(match kind {
            Kind::Skin => "Use a 64 × 64 PNG skin.",
            Kind::Cape => "Use a 64 × 32 PNG cape.",
        }));
    }
    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|_| invalid("Choose a valid PNG image."))?;
    Ok(())
}

fn prepare_import(bytes: Vec<u8>, kind: Kind, slim: bool) -> LauncherResult<(Vec<u8>, bool)> {
    if kind != Kind::Skin || dimensions(&bytes)? != (64, 32) {
        validate_texture(&bytes, kind)?;
        return Ok((bytes, kind == Kind::Skin && slim));
    }

    // Old skins only have right-side limbs. Mirror each UV face into the modern
    // left-side slots, leaving overlays transparent.
    let original = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .map_err(|_| invalid("Choose a valid PNG image."))?
        .into_rgba8();
    let mut converted = image::RgbaImage::new(64, 64);
    image::imageops::replace(&mut converted, &original, 0, 0);
    for (right, left) in [(0, 16), (40, 32)] {
        for (source, target, y, height) in [
            (4, 4, 16, 4),
            (8, 8, 16, 4),
            (0, 8, 20, 12),
            (4, 4, 20, 12),
            (8, 0, 20, 12),
            (12, 12, 20, 12),
        ] {
            let face = image::imageops::crop_imm(&original, right + source, y, 4, height);
            let mirrored = image::imageops::flip_horizontal(&*face);
            image::imageops::replace(
                &mut converted,
                &mirrored,
                i64::from(left + target),
                i64::from(y + 32),
            );
        }
    }
    let mut png = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(converted)
        .write_to(&mut png, image::ImageFormat::Png)
        .map_err(|_| invalid("Could not convert the legacy Minecraft skin."))?;
    Ok((png.into_inner(), false))
}
pub async fn apply(
    account: &oneclient_auth::MinecraftAccount,
    action: Action,
) -> LauncherResult<()> {
    if !account.is_offline() {
        return Err(invalid(
            "The local wardrobe is only available for offline accounts.",
        ));
    }
    apply_at(&directory(account.id)?, account.id, action).await
}
async fn apply_at(root: &Path, account: Uuid, action: Action) -> LauncherResult<()> {
    let _lock = WRITES.lock().await;
    let mut lib = read(root, account).await?;
    let mut discarded = None;
    match action {
        Action::Import {
            name: label,
            kind,
            bytes,
            slim,
            wear,
        } => {
            let label = name(label)?;
            if lib.textures.iter().filter(|t| t.kind == kind).count() as u64 >= LIMIT {
                return Err(invalid(
                    "This local wardrobe already contains 256 textures.",
                ));
            }
            let (bytes, slim) = prepare_import(bytes, kind, slim)?;
            let t = Texture {
                id: Uuid::new_v4(),
                number: next_number(&mut lib.last_number)?,
                name: label,
                kind,
                slim,
                favorite: false,
                legacy: false,
            };
            polyio::write_atomic(selected_path(root, &t), &bytes).await?;
            if wear {
                lib.outfit = None;
                match kind {
                    Kind::Skin => lib.skin = Some(t.id),
                    Kind::Cape => lib.cape = Some(t.id),
                }
            }
            lib.textures.push(t);
        }
        Action::SelectCatalogCape { cape, bytes } => {
            let id = cape.id();
            cape.validate(&bytes)?;
            if !lib.textures.iter().any(|t| t.id == id) {
                lib.textures.push(Texture {
                    id,
                    number: next_number(&mut lib.last_number)?,
                    name: cape.name.into(),
                    kind: Kind::Cape,
                    slim: false,
                    favorite: false,
                    legacy: false,
                });
            }
            polyio::write_atomic(root.join(format!("{id}.png")), &bytes).await?;
            lib.cape = Some(id);
            lib.outfit = None;
        }
        Action::Select { kind, id } => {
            lib.outfit = None;
            if id.is_some_and(|id| !lib.textures.iter().any(|t| t.id == id && t.kind == kind)) {
                return Err(invalid("Unknown wardrobe texture."));
            }
            if let Some(t) = lib.textures.iter().find(|t| Some(t.id) == id) {
                validate_texture(&read_file(&selected_path(root, t)).await?, kind)?;
            }
            match kind {
                Kind::Skin => lib.skin = id,
                Kind::Cape => lib.cape = id,
            }
        }
        Action::Rename { id, name: label } => {
            let label = name(label)?;
            if let Some(t) = lib.textures.iter_mut().find(|t| t.id == id) {
                t.name = label;
            } else if let Some(o) = lib.outfits.iter_mut().find(|o| o.id == id) {
                o.name = label;
            } else {
                return Err(invalid("Unknown wardrobe item."));
            }
        }
        Action::Favorite { id, value } => {
            if let Some(t) = lib.textures.iter_mut().find(|t| t.id == id) {
                t.favorite = value;
            } else if let Some(o) = lib.outfits.iter_mut().find(|o| o.id == id) {
                o.favorite = value;
            } else {
                return Err(invalid("Unknown wardrobe item."));
            }
        }
        Action::Model { id, slim } => {
            if lib.skin == Some(id) {
                lib.outfit = None;
            }
            let t = lib
                .textures
                .iter_mut()
                .find(|t| t.id == id && t.kind == Kind::Skin)
                .ok_or_else(|| invalid("Unknown skin."))?;
            t.slim = slim;
        }
        Action::Delete { id } => {
            if !lib.textures.iter().any(|t| t.id == id) && !lib.outfits.iter().any(|o| o.id == id) {
                return Err(invalid("Unknown wardrobe item."));
            }
            if lib.outfit == Some(id) {
                lib.outfit = None;
            }
            discarded = lib
                .textures
                .iter()
                .find(|t| t.id == id && !t.legacy)
                .map(|t| selected_path(root, t));
            if lib.skin == Some(id) {
                lib.skin = None;
            }
            if lib.cape == Some(id) {
                lib.cape = None;
            }
            for o in &mut lib.outfits {
                if o.skin == Some(id) {
                    o.skin = None;
                }
                if o.cape == Some(id) {
                    o.cape = None;
                }
            }
            lib.textures.retain(|t| t.id != id);
            lib.outfits.retain(|o| o.id != id);
            // Keep bytes until the manifest is safely persisted. UUID-only orphan
            // assets are harmless; never delete paths obtained from names/URLs.
        }
        Action::SaveOutfit { name: label } => {
            if lib.outfits.len() as u64 >= LIMIT {
                return Err(invalid("This local wardrobe already contains 256 outfits."));
            }
            let id = Uuid::new_v4();
            lib.outfits.push(Outfit {
                id,
                number: next_number(&mut lib.last_number)?,
                name: name(label)?,
                skin: lib.skin,
                cape: lib.cape,
                favorite: false,
                slim: lib
                    .textures
                    .iter()
                    .any(|t| Some(t.id) == lib.skin && t.slim),
            });
            lib.outfit = Some(id);
        }
        Action::ApplyOutfit { id } => {
            lib.outfit = Some(id);
            let o = lib
                .outfits
                .iter()
                .find(|o| o.id == id)
                .ok_or_else(|| invalid("Unknown outfit."))?;
            let (skin, cape, slim) = (o.skin, o.cape, o.slim);
            for id in [skin, cape].into_iter().flatten() {
                let t = lib
                    .textures
                    .iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| invalid("Unknown outfit texture."))?;
                validate_texture(&read_file(&selected_path(root, t)).await?, t.kind)?;
            }
            lib.skin = skin;
            lib.cape = cape;
            if let Some(t) = lib.textures.iter_mut().find(|t| Some(t.id) == skin) {
                t.slim = slim;
            }
        }
    }
    validate_library(&lib)?;
    polyio::write_json_atomic(root.join("wardrobe.json"), &lib).await?;
    if let Some(path) = discarded {
        if let Err(e) = tokio::fs::remove_file(path).await
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!("Could not remove an unused local wardrobe texture");
        }
    }
    Ok(())
}

/// Read only the selected skin for avatars and the game launcher, not the library.
pub(super) async fn active_skin(account: Uuid) -> LauncherResult<Option<(Vec<u8>, bool)>> {
    let _lock = WRITES.lock().await;
    let root = directory(account)?;
    let lib = read(&root, account).await?;
    if let Some(t) = lib.textures.iter().find(|t| Some(t.id) == lib.skin) {
        let bytes = read_file(&selected_path(&root, t)).await?;
        validate_texture(&bytes, Kind::Skin)?;
        return Ok(Some((bytes, t.slim)));
    }
    Ok(None)
}

pub async fn read_file(path: &Path) -> LauncherResult<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    let file = tokio::fs::File::open(path).await?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1).read_to_end(&mut bytes).await?;
    if bytes.len() > 1024 * 1024 {
        return Err(invalid("The PNG must be smaller than 1 MB."));
    }
    Ok(bytes)
}
/// Explicit user-requested import. No cloud wardrobe or authenticated writes.
pub async fn download(
    client: &oneclient_net::RequestClient,
    address: &str,
) -> LauncherResult<Vec<u8>> {
    let url =
        reqwest::Url::parse(address).map_err(|_| invalid("Enter a valid HTTP(S) image URL."))?;
    if !matches!(url.scheme(), "https" | "http")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(invalid(
            "Use an HTTP(S) image URL without embedded credentials.",
        ));
    }
    let request = client
        .http()
        .get(url)
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|_| invalid("Invalid image request."))?;
    let mut response = client
        .send(request)
        .await?
        .error_for_status()
        .map_err(|_| invalid("The image server rejected the download."))?;
    if response.content_length().is_some_and(|n| n > 1024 * 1024) {
        return Err(invalid("The PNG must be smaller than 1 MB."));
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| invalid("The image download was interrupted."))?
    {
        if bytes.len() + chunk.len() > 1024 * 1024 {
            return Err(invalid("The PNG must be smaller than 1 MB."));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png(kind: Kind) -> Vec<u8> {
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(64, if kind == Kind::Skin { 64 } else { 32 })
            .write_to(&mut out, image::ImageFormat::Png)
            .unwrap();
        out.into_inner()
    }
    fn root() -> polyio::testing::ScratchDir {
        polyio::testing::ScratchDir::new("offline_wardrobe")
    }
    async fn import(root: &Path, account: Uuid, kind: Kind) {
        apply_at(
            root,
            account,
            Action::Import {
                name: "Texture".into(),
                kind,
                bytes: png(kind),
                slim: kind == Kind::Skin,
                wear: true,
            },
        )
        .await
        .unwrap();
    }
    #[tokio::test]
    async fn legacy_upgrade_is_read_only_until_mutation_and_preserves_selection() {
        let r = root();
        let id = Uuid::new_v4();
        polyio::write_atomic(r.join("skin.png"), png(Kind::Skin))
            .await
            .unwrap();
        polyio::write_json_atomic(r.join("skin.json"), &LocalSkin { slim: true })
            .await
            .unwrap();
        let old = tokio::fs::read(r.join("skin.png")).await.unwrap();
        let s = snapshot_at(r.path(), id).await.unwrap();
        assert_eq!(s.library.skin, Some(id));
        assert!(s.library.textures[0].slim);
        assert!(!r.join("wardrobe.json").exists());
        import(r.path(), id, Kind::Cape).await;
        let s = snapshot_at(r.path(), id).await.unwrap();
        assert_eq!(s.library.skin, Some(id));
        assert!(s.library.cape.is_some());
        assert_eq!(tokio::fs::read(r.join("skin.png")).await.unwrap(), old);
        apply_at(
            r.path(),
            id,
            Action::Select {
                kind: Kind::Skin,
                id: None,
            },
        )
        .await
        .unwrap();
        let s = snapshot_at(r.path(), id).await.unwrap();
        assert!(s.library.skin.is_none());
        assert!(s.library.cape.is_some());
        assert!(r.join("skin.json").exists(), "keep the legacy files intact");
    }
    #[tokio::test]
    async fn outfits_restore_models_and_deleting_a_texture_clears_only_its_references() {
        let r = root();
        let id = Uuid::new_v4();
        import(r.path(), id, Kind::Skin).await;
        import(r.path(), id, Kind::Cape).await;
        apply_at(
            r.path(),
            id,
            Action::SaveOutfit {
                name: "Look".into(),
            },
        )
        .await
        .unwrap();
        let s = snapshot_at(r.path(), id).await.unwrap();
        let skin = s.library.skin.unwrap();
        let cape = s.library.cape.unwrap();
        let outfit = s.library.outfits[0].id;
        apply_at(
            r.path(),
            id,
            Action::Model {
                id: skin,
                slim: false,
            },
        )
        .await
        .unwrap();
        apply_at(r.path(), id, Action::ApplyOutfit { id: outfit })
            .await
            .unwrap();
        let (_, _, slim) = selection_at(r.path(), id).await.unwrap();
        assert!(slim);
        apply_at(
            r.path(),
            id,
            Action::Favorite {
                id: outfit,
                value: true,
            },
        )
        .await
        .unwrap();
        apply_at(
            r.path(),
            id,
            Action::Rename {
                id: outfit,
                name: "Renamed".into(),
            },
        )
        .await
        .unwrap();
        apply_at(r.path(), id, Action::Delete { id: skin })
            .await
            .unwrap();
        let s = snapshot_at(r.path(), id).await.unwrap();
        assert!(s.library.skin.is_none());
        assert_eq!(s.library.cape, Some(cape));
        assert_eq!(s.library.outfits[0].name, "Renamed");
        assert!(s.library.outfits[0].favorite);
        assert!(s.library.outfits[0].skin.is_none());
        assert_eq!(s.library.outfits[0].cape, Some(cape));
    }
    #[tokio::test]
    async fn imported_cape_can_be_equipped_with_a_skin_and_deleted_from_outfits() {
        let r = root();
        let account = Uuid::new_v4();
        import(r.path(), account, Kind::Skin).await;
        import(r.path(), account, Kind::Cape).await;
        let original = read(r.path(), account).await.unwrap();
        let cape = original.cape.unwrap();

        apply_at(
            r.path(),
            account,
            Action::SaveOutfit {
                name: "Saved look".into(),
            },
        )
        .await
        .unwrap();
        let (skin_path, cape_path, _) = selection_at(r.path(), account).await.unwrap();
        assert!(skin_path.is_some());
        let cape_path = cape_path.unwrap();
        assert_eq!(read_file(&cape_path).await.unwrap(), png(Kind::Cape));

        apply_at(r.path(), account, Action::Delete { id: cape })
            .await
            .unwrap();
        let library = read(r.path(), account).await.unwrap();
        assert_eq!(library.skin, original.skin);
        assert_eq!(library.outfits[0].skin, original.skin);
        assert!(library.cape.is_none());
        assert!(library.outfits[0].cape.is_none());
        assert!(!cape_path.exists());
    }

    #[tokio::test]
    async fn invalid_mutations_leave_the_manifest_unchanged_and_cannot_select_wrong_kinds() {
        let r = root();
        let id = Uuid::new_v4();
        import(r.path(), id, Kind::Cape).await;
        let s = snapshot_at(r.path(), id).await.unwrap();
        assert!(
            apply_at(
                r.path(),
                id,
                Action::Select {
                    kind: Kind::Skin,
                    id: s.library.cape
                }
            )
            .await
            .is_err()
        );
        assert!(
            apply_at(
                r.path(),
                id,
                Action::Rename {
                    id: s.library.cape.unwrap(),
                    name: "\n".into()
                }
            )
            .await
            .is_err()
        );
        assert!(
            apply_at(
                r.path(),
                id,
                Action::Import {
                    name: "Bad".into(),
                    kind: Kind::Skin,
                    bytes: b"not a PNG".to_vec(),
                    slim: false,
                    wear: true,
                }
            )
            .await
            .is_err()
        );
        assert_eq!(snapshot_at(r.path(), id).await.unwrap().library, s.library);
    }
    #[tokio::test]
    async fn item_numbers_are_stable_across_migration_and_deletion() {
        let r = root();
        let id = Uuid::new_v4();
        import(r.path(), id, Kind::Skin).await;
        import(r.path(), id, Kind::Skin).await;
        let path = r.path().join("wardrobe.json");
        let mut draft: serde_json::Value = polyio::read_json(&path).await.unwrap();
        draft.as_object_mut().unwrap().remove("last_number");
        for item in draft["textures"].as_array_mut().unwrap() {
            item.as_object_mut().unwrap().remove("number");
        }
        polyio::write_json_atomic(&path, &draft).await.unwrap();

        let lib = read(r.path(), id).await.unwrap();
        assert_eq!(lib.textures[0].number, 1);
        assert_eq!(lib.textures[1].number, 2);
        let untouched: serde_json::Value = polyio::read_json(&path).await.unwrap();
        assert_eq!(untouched, draft);
        apply_at(
            r.path(),
            id,
            Action::Delete {
                id: lib.textures[0].id,
            },
        )
        .await
        .unwrap();
        import(r.path(), id, Kind::Skin).await;
        let lib = read(r.path(), id).await.unwrap();
        assert_eq!(lib.textures[0].number, 2);
        assert_eq!(lib.textures[1].number, 3);
    }

    #[test]
    fn legacy_skin_import_mirrors_limbs_and_preserves_the_source_pixels() {
        let mut legacy = image::RgbaImage::new(64, 32);
        let red = image::Rgba([255, 0, 0, 255]);
        let blue = image::Rgba([0, 0, 255, 255]);
        legacy.put_pixel(4, 20, red);
        legacy.put_pixel(43, 20, blue);
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(legacy)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();

        let (png, slim) = prepare_import(bytes.into_inner(), Kind::Skin, true).unwrap();
        assert!(!slim);
        let skin = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
            .unwrap()
            .into_rgba8();
        assert_eq!(skin.dimensions(), (64, 64));
        assert_eq!(skin.get_pixel(4, 20), &red);
        assert_eq!(skin.get_pixel(23, 52), &red);
        assert_eq!(skin.get_pixel(40, 52), &blue);
        assert_eq!(skin.get_pixel(63, 63).0[3], 0);
    }

    #[tokio::test]
    async fn online_kinds_cannot_mutate_a_wardrobe() {
        for kind in [
            oneclient_auth::AccountKind::Microsoft,
            oneclient_auth::AccountKind::Elyby,
            oneclient_auth::AccountKind::Custom,
        ] {
            let mut account = oneclient_auth::offline_account("Player".into());
            account.kind = kind;
            assert!(
                apply(
                    &account,
                    Action::SaveOutfit {
                        name: "Never".into()
                    }
                )
                .await
                .is_err()
            );
        }
    }
    #[test]
    fn capes_and_skins_have_distinct_bounded_png_dimensions() {
        assert!(validate_texture(&png(Kind::Cape), Kind::Cape).is_ok());
        assert!(validate_texture(&png(Kind::Skin), Kind::Cape).is_err());
        assert!(validate_texture(&vec![0; 1024 * 1024 + 1], Kind::Cape).is_err());
        let mut bomb = png(Kind::Cape);
        bomb[16..20].copy_from_slice(&100000u32.to_be_bytes());
        assert!(validate_texture(&bomb, Kind::Cape).is_err());
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    fn cape() -> Vec<u8> {
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgba8(64, 32)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        png.into_inner()
    }
    #[tokio::test]
    async fn cape_only_launch_selection_needs_no_skin_and_unavailable_files_are_recoverable() {
        let root = polyio::testing::ScratchDir::new("cape_only");
        let id = Uuid::new_v4();
        apply_at(
            root.path(),
            id,
            Action::Import {
                name: "Cape".into(),
                kind: Kind::Cape,
                bytes: cape(),
                slim: false,
                wear: true,
            },
        )
        .await
        .unwrap();
        let (skin, path, slim) = selection_at(root.path(), id).await.unwrap();
        assert!(skin.is_none());
        assert!(!slim);
        assert!(path.is_some());
        let lib = read(root.path(), id).await.unwrap();
        let cape_id = lib.cape.unwrap();
        tokio::fs::remove_file(path.unwrap()).await.unwrap();
        let s = snapshot_at(root.path(), id).await.unwrap();
        assert_eq!(s.unavailable, vec![cape_id]);
        assert!(selection_at(root.path(), id).await.is_err());
        apply_at(
            root.path(),
            id,
            Action::Select {
                kind: Kind::Cape,
                id: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(
            selection_at(root.path(), id).await.unwrap(),
            (None, None, false)
        );
        assert!(
            apply_at(
                root.path(),
                id,
                Action::Select {
                    kind: Kind::Cape,
                    id: Some(cape_id)
                }
            )
            .await
            .is_err()
        );
        apply_at(root.path(), id, Action::Delete { id: cape_id })
            .await
            .unwrap();
        assert!(
            snapshot_at(root.path(), id)
                .await
                .unwrap()
                .unavailable
                .is_empty()
        );
    }
    #[tokio::test]
    async fn unknown_manifest_version_is_not_overwritten() {
        let root = polyio::testing::ScratchDir::new("future_wardrobe");
        let id = Uuid::new_v4();
        let mut lib = Library::default();
        lib.version = 99;
        polyio::write_json_atomic(root.join("wardrobe.json"), &lib)
            .await
            .unwrap();
        assert!(
            apply_at(root.path(), id, Action::SaveOutfit { name: "No".into() })
                .await
                .is_err()
        );
        let preserved: Library = polyio::read_json(root.join("wardrobe.json")).await.unwrap();
        assert_eq!(preserved.version, 99);
    }
    #[tokio::test]
    async fn image_download_limits_and_schemes_are_checked_before_import() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(std::time::Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 2048];
            stream.read(&mut request).unwrap();
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 2097152\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
        });
        let client =
            oneclient_net::RequestClient::new(oneclient_net::NetConfig::default()).unwrap();
        assert!(
            download(&client, &format!("http://{address}/image"))
                .await
                .is_err()
        );
        worker.join().unwrap();
        for value in [
            "file:///etc/passwd",
            "https://user:password@example.com/skin.png",
            "not a URL",
        ] {
            assert!(download(&client, value).await.is_err());
        }
    }
}

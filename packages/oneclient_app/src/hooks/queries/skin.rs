use bytes::Bytes;
use freya::prelude::*;

use crate::AppAssets;

pub fn use_player_skin(uuid: String) -> (Bytes, bool) {
    use_player_skin_when(uuid, true)
}

pub fn use_player_skin_when(uuid: String, enabled: bool) -> (Bytes, bool) {
    let local = uuid.strip_prefix("offline:").and_then(|id| uuid::Uuid::parse_str(id).ok());
    let local_skin = super::local_skin::use_local_skin(if enabled { local } else { None });
    let profile = super::use_player_profile_when(uuid.clone(), None::<String>, enabled && local.is_none());

    let (skin_url, is_slim) = if enabled && local.is_none() {
        crate::hooks::settled_or_loading(&profile)
            .map_or((None, false), |profile| (profile.skin_url, profile.is_slim))
    } else { (None, false) };

    let skin_query = super::use_cached_image(skin_url.clone(), 256);

    let steve = use_memo(|| AppAssets::get_bytes("steve.png").unwrap_or_default());
    let alex = use_memo(|| AppAssets::get_bytes("alex.png").unwrap_or_default());

    if enabled && local.is_some() && let Some(Some(skin)) = crate::hooks::settled_or_loading(&local_skin) { return skin; }

    let default_slim = (java_string_hash(&uuid) & 1) == 1;

    match crate::hooks::loaded_image(skin_url.as_deref(), &skin_query) {
        Some((_, bytes)) => (bytes, is_slim),
        None if default_slim => (alex.read().clone(), true),
        None => (steve.read().clone(), false),
    }
}

fn java_string_hash(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

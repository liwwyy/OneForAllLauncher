//! Public Mojang texture references for the offline cape picker.
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::library;
use crate::{LauncherError, LauncherResult};

#[derive(Clone, Copy)]
pub struct Cape {
    pub name: &'static str,
    texture: &'static str,
    sha256: &'static str,
}

impl Cape {
    pub fn id(self) -> Uuid {
        // Keep the selection stable across catalog ordering and display name changes.
        let mut bytes =
            u128::from_str_radix(&self.texture[..32], 16).expect("embedded Mojang texture key");
        // Use UUID version 8 for catalog IDs, separate from random uploads.
        bytes = (bytes & !(0xf << 76 | 0x3 << 62)) | (8 << 76) | (2 << 62);
        Uuid::from_u128(bytes)
    }

    pub fn url(self) -> String {
        format!("https://textures.minecraft.net/texture/{}", self.texture)
    }

    pub(super) fn matches(self, bytes: &[u8]) -> bool {
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
            == self.sha256
    }

    pub(super) async fn download(
        self,
        requester: &oneclient_net::RequestClient,
    ) -> LauncherResult<Vec<u8>> {
        library::download(requester, &self.url()).await
    }

    pub(super) fn validate(self, bytes: &[u8]) -> LauncherResult<()> {
        if !self.matches(bytes) {
            return Err(LauncherError::Minecraft(
                "The Minecraft cape did not match its expected checksum.".into(),
            ));
        }
        library::validate_texture(bytes, library::Kind::Cape)
    }
}

pub fn capes() -> impl Iterator<Item = Cape> {
    include_str!("capes.tsv").lines().map(|line| {
        let mut fields = line.split('\t');
        Cape {
            name: fields.next().expect("embedded cape name"),
            texture: fields.next().expect("embedded cape texture key"),
            sha256: fields.next().expect("embedded cape checksum"),
        }
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn catalog_contains_unique_mojang_texture_references() {
        let mut ids = HashSet::new();
        for cape in capes() {
            assert!(!cape.name.is_empty());
            assert!((60..=64).contains(&cape.texture.len()));
            assert_eq!(cape.sha256.len(), 64);
            assert!(
                cape.texture
                    .bytes()
                    .chain(cape.sha256.bytes())
                    .all(|b| b.is_ascii_hexdigit())
            );
            assert!(ids.insert(cape.id()), "{}", cape.name);
        }
        assert_eq!(ids.len(), 50);
    }
}

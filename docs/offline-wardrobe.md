# Offline wardrobe

Select a standalone offline account and open **Wardrobe**. Look for **Local**
beside the page title. The launcher uses the same cards, menus and 3D preview
as OneLauncher. The **Cosmetics** tab is disabled
for offline accounts. Local skins and capes are visible only in your game client,
and selections take effect the next time you launch Minecraft.

## Skins

Select **Add skin** and choose a PNG file, image URL or Minecraft username. A
file works without internet. URL and username imports need internet once. File
and URL imports save and equip the skin. Username imports save the skin and its
arm model to your library so you can preview it before equipping it.

Use a 64 × 64 PNG smaller than 1 MiB. The launcher also converts 64 × 32 legacy
skins to the modern layout with classic arms. Card menus let you rename,
favorite, delete and change the arm model. Select **Use default skin** to clear
the equipped skin and keep your cape. The library holds up to 256 skins per account.

## Capes

Open **Capes** and use **Add capes** beside **Custom capes** to equip a static
64 × 32 PNG from a file or URL. Choose a name if you want to override the file
name or default label. File imports work without internet. The card menu lets
you unequip or delete an imported cape. Deleting a cape removes its references
from saved outfits.

Select a **Minecraft capes** card to preview it. Equip a cape to download and
save its texture for this account. A saved cape can be equipped again without an
internet connection. You can wear a cape with the default skin. Use the equipped
cape's card to remove it and keep your skin.

The picker includes 50 cape textures verified on Mojang's texture CDN. It covers
the Java account capes tracked by the reference catalogs, including event,
staff, and one-off capes. The launcher stores only names and texture references
in its source. [MCProfiles cape catalog](https://mcprofiles.me/capes/official),
[Mojang's Test cape texture](https://textures.minecraft.net/texture/7a93b1867eb599f2b76e6e1c30a0ddb530e6f4c7bce6515d1ba72b206df30e39).

## Outfits

Use **+** above the preview to save the current look. An outfit remembers the
skin, arm model and cape. You can preview, equip, rename, favorite or delete it
from **Outfits**. Changing the equipped skin or cape leaves saved outfits
available through the outfit switcher. Deleting a skin also removes its
references from saved outfits.

## Storage and upgrades

The per-account library stays in `local-skins/<launcher-account-id>/` in the
fork's private configuration folder. Existing `skin.json` and `skin.png`
selections appear as an imported skin. The first saved change creates
`wardrobe.json` and leaves the legacy files intact. Missing PNGs stay listed so
you can delete the skin entry or equip the cape again to download a fresh copy.

The source-built appearance hook applies the selected textures only to the
offline player's own profile. Older Minecraft versions use classic arms.
In-game cape visibility and mods that replace the appearance renderer may affect
what you see. See [validation](validation.md) for the loader and game checks.

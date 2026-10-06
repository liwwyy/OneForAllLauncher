package org.polyfrost.vanillahud.util;

import com.mojang.authlib.GameProfile;
import java.util.UUID;

/** Headless fixture with the same preview method signature as VanillaHUD. */
public final class TabListManager {
    private GameProfile getProfile(UUID id) { return new GameProfile(id, null); }
    public GameProfile preview(UUID id) { return getProfile(id); }
}

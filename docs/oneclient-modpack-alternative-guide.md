# Community setup: OneClient with an offline account

The community route recommended by this fork's maintainer is to use a standalone
launcher that supports both **offline accounts without a Microsoft sign-in** and
**Modrinth modpacks**, then install the official OneClient modpack through it.
This guide uses [Freesm Launcher](https://github.com/FreesmTeam/FreesmLauncher)
as the example; another launcher with both capabilities can follow the same route.

An offline account is a local player name. It does not authenticate you with
Microsoft, Mojang or Polyfrost. You need internet access for the initial game,
Java, loader and modpack downloads; an offline account does not supply missing
installation files.

## 1. Install Freesm

Download the current build for your operating system and processor from
[Freesm's releases](https://github.com/FreesmTeam/FreesmLauncher/releases/latest).
Open it and complete its first-run setup. Allow its automatic Java management,
or select a Java version that matches the Minecraft version required by the pack.

## 2. Add an offline account

1. Open **Settings → Accounts** (or the account menu's account-management screen).
2. Choose **Add Offline**, enter your player name and confirm.
3. Select the new account and use **Set Default** so the instance launches with it.

Freesm supports adding this account without first adding a Microsoft account.
Keep the same player name when returning to an existing offline world/server;
changing names can change the offline player identity used for inventory and
permissions. Ely.by accounts are also supported!

## 3. Install the official OneClient modpack

1. In Freesm, choose **Add Instance → Modrinth**.
2. Search for **OneClient** and select the pack from Polyfrost. Check it against
   the [official OneClient Modrinth page](https://modrinth.com/modpack/oneclient-modpack).
3. Select an available pack version and install it. Keep the Minecraft and loader
   versions supplied by that pack.
4. Let the launcher finish downloading the game, Java and all modpack files.
   Complete any download prompts it presents before launching.

Alternatively, download the chosen version's `.mrpack` from that same Modrinth
page, then use **Add Instance → Import** in Freesm to select the downloaded file.
Importing the pack still downloads its dependencies; the `.mrpack` alone is not
a complete offline installation.

## 4. Launch and use the installed pack

Select the installed OneClient instance, check that your offline account is
selected and press **Launch**. Finish one launch while connected so installation
errors can be resolved before disconnecting. After the required files are
installed, use your launcher's offline-launch option when playing without internet.

Single-player and servers that accept offline identities can work with an
offline account. Realms, servers requiring Microsoft/Mojang authentication and
authenticated PolyPlus features such as cosmetics or P2P hosting still require
the corresponding valid account/session. Authentication errors for those services
do not mean that the local modpack installation failed.

[Return to the README](../README.md#installation).

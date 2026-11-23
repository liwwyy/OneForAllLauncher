{ lib
, appimageTools
, fetchurl
, makeDesktopItem
, makeWrapper
}:

let
  pname = "oneforall-launcher";
  version = "2.11.0";

  src = fetchurl {
    url = "https://github.com/liwwyy/OneForAllLauncher/releases/download/oneforall-${version}/OneForAllLauncher_${version}_linux_x86_64.AppImage";
    hash = "sha256-+GWImKY4T7eJGrci85HVm0gGMjoMkE+1ojVzh16okh4=";
  };

  extracted = appimageTools.extract { inherit pname version src; };

  desktopItem = makeDesktopItem {
    name = pname;
    desktopName = "OneForAllLauncher";
    comment = "Minecraft launcher by Polyfrost";
    exec = "${pname} %U";
    icon = pname;
    terminal = false;
    categories = [ "Game" ];
    mimeTypes = [ "x-scheme-handler/oneforall" ];
  };
in
appimageTools.wrapType2 {
  inherit pname version src;

  nativeBuildInputs = [ makeWrapper ];

  extraPkgs = pkgs: with pkgs; [
    libgpg-error
    libgcrypt
    vulkan-loader
    libxkbcommon
    wayland
    jdk8
    jdk17
    jdk21
    libGL
    openal
    libpulseaudio
    libxcursor
    libxrandr
    libxxf86vm
  ];

  extraInstallCommands = ''
    wrapProgram $out/bin/${pname} --unset GIO_EXTRA_MODULES

    install -Dm444 ${desktopItem}/share/applications/*.desktop \
      -t $out/share/applications

    find ${extracted} -maxdepth 2 -name '*.png' | head -n1 | while read -r icon; do
      install -Dm444 "$icon" $out/share/pixmaps/${pname}.png
    done
  '';

  meta = {
    description = "OneForAllLauncher, a fork of Polyfrost OneClient";
    homepage = "https://github.com/liwwyy/OneForAllLauncher";
    sourceProvenance = [ lib.sourceTypes.binaryNativeCode ];
    platforms = [ "x86_64-linux" ];
    mainProgram = pname;
  };
}

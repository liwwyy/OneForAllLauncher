{
  description = "OneForAllLauncher (Polyfrost OneClient fork) for NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
      };

      oneforall-launcher = pkgs.callPackage ./nix/package.nix { };
    in
    {
      packages.${system} = {
        inherit oneforall-launcher;
        default = oneforall-launcher;
      };

      apps.${system}.default = {
        type = "app";
        program = "${oneforall-launcher}/bin/oneforall-launcher";
      };

      overlays.default = final: prev: {
        oneforall-launcher = final.callPackage ./nix/package.nix { };
      };
    };
}

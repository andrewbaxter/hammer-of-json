{ pkgs, lib }:
let
  naersk = pkgs.callPackage (fetchTarball "https://github.com/nix-community/naersk/archive/378614f37a6bee5a3f2ef4f825a73d948d3ae921.zip") { };
in
naersk.buildPackage {
  root = ./.;
}

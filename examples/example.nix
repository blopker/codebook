# Commment with a typo
{
  description = "A useful dev shell for the packge";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flakeUtils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flakeUtils }:
    flakeUtils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        verion = "0.1.0";
      in
      {
        devShells.defualt = pkgs.mkShell {
          buildInputs = [ pkgs.cargo pkgs.rustc ];
          shellHook = ''
            echo "Entring dev shell version ${verion}"
          '';
        };
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "my-projet";
          version = verion;
          src = ./.;
        };
      });
}

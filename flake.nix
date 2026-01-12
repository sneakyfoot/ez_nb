{
  description = "Simple markdown notebook tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:

    let
      lib = nixpkgs.lib;

      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      forAllSystems = f: lib.genAttrs systems (system: f system);

      pythonSpec = "3.14.2";

      appName = "notebook";
      entrypoint = "notebook";

    in
    {

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          toolchain = [
            pkgs.uv
            pkgs.ruff
            pkgs.cacert
            pkgs.makeWrapper
            pkgs.ty
            pkgs.zlib
            pkgs.openssl
            pkgs.stdenv.cc
          ];

        in
        {
          default = pkgs.mkShell {
            packages = toolchain;

            env = {
              UV_MANAGED_PYTHON = "1";
              UV_PROJECT_ENVIRONMENT = ".venv";
            };

            shellHook = ''
              set -euo pipefail
            '';
          };
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          toolchain = [
            pkgs.uv
            pkgs.ruff
            pkgs.cacert
            pkgs.makeWrapper
            pkgs.ty
            pkgs.zlib
            pkgs.openssl
            pkgs.stdenv.cc
          ];

          uvBundle = pkgs.stdenvNoCC.mkDerivation {
            pname = "${appName}-uv-bundle";
            version = "0.1.1";
            src = ./.;

            # Requires relaxed sandbox / network
            __noChroot = true;
            allowSubstitutes = true;
            dontFixup = true;

            nativeBuildInputs = toolchain;

            installPhase = ''
              set -euo pipefail


              export HOME="$TMPDIR/home"
              mkdir -p "$HOME"

              export UV_CACHE_DIR="$TMPDIR/uv-cache"
              export UV_MANAGED_PYTHON=1

              export UV_PYTHON_INSTALL_DIR="$out/python"
              export UV_PROJECT_ENVIRONMENT="$out/venv"

              uv python install ${pythonSpec}
              uv venv --python ${pythonSpec}
              uv sync --frozen --no-dev --no-editable
            '';
          };

          cli = pkgs.stdenvNoCC.mkDerivation {
            pname = appName;
            version = "0.1.1";

            dontUnpack = true;
            nativeBuildInputs = [ pkgs.makeWrapper ];

            installPhase = ''
              set -euo pipefail
              mkdir -p "$out/bin"


              makeWrapper "${uvBundle}/venv/bin/${entrypoint}" "$out/bin/${entrypoint}"
            '';
          };

        in
        {
          uv-bundle = uvBundle;
          ${appName} = cli;

          default = cli;
        }
      );

      apps = forAllSystems (
        system:
        let
          cli = self.packages.${system}.${appName};
        in
        {
          ${appName} = {
            type = "app";
            program = "${cli}/bin/${entrypoint}";
          };

          default = {
            type = "app";
            program = "${cli}/bin/${entrypoint}";
          };
        }
      );

    };
}

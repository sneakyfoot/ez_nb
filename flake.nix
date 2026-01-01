{
  description = "Simple markdown notebook tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:

    let
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs systems;

      pythonSpec = "3.14";

      appName = "notebook";
      entrypoint = "notebook";

      mkForSystem =
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

          # Heavy runtime bundle (python + venv). Do NOT install this into your profile.
          uvBundle = pkgs.stdenvNoCC.mkDerivation {
            pname = "${appName}-uv-bundle";
            version = "0.1.2";
            src = self;

            # Build with: --option sandbox relaxed
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

              # Optional: keep a runnable wrapper inside the bundle for testing
              mkdir -p "$out/bin"
              if [ -x "$out/venv/bin/${entrypoint}" ]; then
                makeWrapper "$out/venv/bin/${entrypoint}" "$out/bin/${entrypoint}" \
                  --set PYTHONNOUSERSITE 1
              else
                echo "ERROR: expected entrypoint missing: $out/venv/bin/${entrypoint}" >&2
                echo "Hint: set entrypoint=... to match your [project.scripts] name." >&2
                exit 1
              fi
            '';
          };

          # Thin CLI package: installs only bin/notebook and points at uvBundle's venv.
          cli = pkgs.stdenvNoCC.mkDerivation {
            pname = appName;
            version = "0.1.2";

            dontUnpack = true;
            nativeBuildInputs = [ pkgs.makeWrapper ];

            installPhase = ''
              set -euo pipefail
              mkdir -p "$out/bin"

              if [ -x "${uvBundle}/venv/bin/${entrypoint}" ]; then
                makeWrapper "${uvBundle}/venv/bin/${entrypoint}" "$out/bin/${entrypoint}" \
                  --set PYTHONNOUSERSITE 1
              else
                echo "ERROR: expected entrypoint missing: ${uvBundle}/venv/bin/${entrypoint}" >&2
                echo "Hint: set entrypoint=... to match your [project.scripts] name." >&2
                exit 1
              fi
            '';
          };
        in
        {
          devShells.default = pkgs.mkShell {
            packages = toolchain;
            env = {
              UV_MANAGED_PYTHON = "1";
              UV_PROJECT_ENVIRONMENT = ".venv";
            };
            shellHook = ''
              source $UV_PROJECT_ENVIRONMENT/bin/activate
            '';
          };

          packages = {
            uv-bundle = uvBundle;
            ${appName} = cli;
            default = cli;
          };

          apps = {
            ${appName} = {
              type = "app";
              program = "${cli}/bin/${entrypoint}";
            };
            default = {
              type = "app";
              program = "${cli}/bin/${entrypoint}";
            };
          };
        };
    in
    {
      devShells = forAllSystems (system: (mkForSystem system).devShells);
      packages = forAllSystems (system: (mkForSystem system).packages);
      apps = forAllSystems (system: (mkForSystem system).apps);
    };
}

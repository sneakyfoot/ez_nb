{
  description = "Simple markdown notebook tool";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:

  let

    systems = [ "x86_64-linux" "aarch64-darwin" ];
    forAllSystems = nixpkgs.lib.genAttrs systems;

    pythonSpec = "3.14";

    appName = "notebook";
    entrypoint = "notebook";

    mkForSystem = system:
      let
        pkgs = import nixpkgs { inherit system; };
        toolchain =
          [ pkgs.uv pkgs.ruff pkgs.cacert pkgs.makeWrapper pkgs.ty pkgs.zlib pkgs.openssl pkgs.stdenv.cc ];
        uvBundle = pkgs.stdenvNoCC.mkDerivation {
          pname = "${appName}-uv-bundle";
          version = "0.1.1";
          src = self;

          # Build with: --option sandbox relaxed
          __noChroot = true;
          preferLocalBuild = true;
          allowSubstitutes = false;
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

            mkdir -p "$out/bin"

            # Main wrapper.
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
      in
      {
        devShells.default = pkgs.mkShell {
          packages = toolchain;
          env = {
            UV_MANAGED_PYTHON = "1";
            # UV_PYTHON_INSTALL_DIR = ".uv-python";
            UV_PROJECT_ENVIRONMENT = ".venv";
          };
          shellHook = ''
            source $UV_PROJECT_ENVIRONMENT/bin/activate
          '';
        };

        packages = {
          ${appName} = uvBundle;
          default = uvBundle;
        };

        apps.default = {
          type = "app";
          program = "${uvBundle}/bin/${entrypoint}";
        };
      };

  in {

    devShells = forAllSystems (system: (mkForSystem system).devShells);
    packages = forAllSystems (system: (mkForSystem system).packages);
    apps = forAllSystems (system: (mkForSystem system).apps);

  };
}

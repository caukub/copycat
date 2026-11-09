{
  description = "copycat";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f:
        nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs:
        let
          unwrapped = pkgs.rustPlatform.buildRustPackage {
            pname = "copycat";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./askama.toml
                ./src
                ./frontend
              ];
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };

            nativeBuildInputs = [ pkgs.pkg-config pkgs.tailwindcss_4 ];
            
            buildInputs = [ pkgs.openssl pkgs.onnxruntime ];

            NIX_LDFLAGS = "-L${pkgs.onnxruntime}/lib";

            ORT_LIB_LOCATION = "${pkgs.onnxruntime}/lib";
            ORT_PREFER_DYNAMIC_LINK = "1";

            preBuild = ''
              tailwindcss -m -i frontend/assets/input.css -o frontend/assets/styles.css
            '';

            postInstall = ''
              mkdir -p $out/share/copycat
              cp -r frontend $out/share/copycat/frontend
            '';
          };

          copycat = pkgs.writeShellApplication {
            name = "copycat";
            text = ''
              workdir="''${XDG_CACHE_HOME:-$HOME/.cache}/copycat"
              mkdir -p "$workdir"
              mkdir -p "$PWD/data"
              ln -sfn ${unwrapped}/share/copycat/frontend "$workdir/frontend"
              ln -sfn "$PWD/configuration" "$workdir/configuration"
              ln -sfn "$PWD/data" "$workdir/data"
              cd "$workdir"
              exec ${unwrapped}/bin/copycat "$@"
            '';
          };
        in
        {
          default = copycat;
          inherit copycat unwrapped;
        });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/copycat";
        };
      });
    };
}
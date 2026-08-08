{
  description = "A Nix(OS) Flake for todo-tree!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = {
    self,
    naersk,
    nixpkgs,
    rust-overlay,
    ...
  }: let
    overlays = [
      rust-overlay.overlays.default
      (final: prev: {
        rustToolchain =
          if builtins.pathExists ./rust-toolchain.toml
          then prev.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml
          else if builtins.pathExists ./rust-toolchain
          then prev.rust-bin.fromRustupToolchainFile ./rust-toolchain
          else prev.rust-bin.stable.latest.default;
      })
    ];

    supportedSystems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

    forEachSupportedSystem = f:
      nixpkgs.lib.genAttrs supportedSystems (
        system:
          f {pkgs = import nixpkgs {inherit overlays system;};}
      );

    todoTreePackages = forEachSupportedSystem ({pkgs}: let
      naersklib = pkgs.callPackage naersk {};
      package = naersklib.buildPackage {
        name = "todo-tree";
        pname = "todo-tree";
        version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
        src = self;
        cargoBuildOptions = opts: opts ++ ["--package" "todo-tree"];
        nativeBuildInputs = with pkgs; [pkg-config];
        meta = {mainProgram = "todo-tree";};
      };
    in {
      todo-tree = package;
      default = package;
    });
  in {
    devShells = forEachSupportedSystem ({pkgs}: {
      default = pkgs.mkShell {
        env.RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        packages = with pkgs; [
          rustToolchain
          pkg-config
          cargo-deny
          cargo-edit
          cargo-watch
          cargo-flamegraph
          rust-analyzer
          just
          bacon
        ];
      };
    });

    packages = todoTreePackages;
  };
}

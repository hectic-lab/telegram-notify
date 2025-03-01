{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    util = {
      url = "github:hectic-lab/util.nix";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = {self, nixpkgs, rust-overlay, util}: 
  let
    overlays = [ (import rust-overlay) util.overlays.default ];
  in
  util.lib.forAllSystemsWithPkgs overlays ({ system, pkgs }:
  let
    rustToolchain = pkgs.pkgsBuildHost.rust-bin.stable."1.81.0".default;
    nativeBuildInputs = [ rustToolchain pkgs.pkg-config ];
  in
  {
    devShells.${system}.default =
      pkgs.mkShell {
        inherit nativeBuildInputs;
        buildInputs = [pkgs.openssl];
      };
    packages.${system}.defalut =
      let
        src = ./.;
        cargo = util.lib.cargoToml src;
      in
      pkgs.rustPlatform.buildRustPackage {
        pname = cargo.package.name;
        version = cargo.package.version;

        inherit nativeBuildInputs src;

        cargoLock.lockFile = ./Cargo.lock;

        doCheck = true;
      };
    }) // {
      nixosModules = {
        vproxy-rest = { config, system, pkgs, ... }:
	let
	  lib = pkgs.lib;
	  cfg = config.hectic.telegram-notify;
        in
	{
	  options = {
            hectic.telegram-notify.enable = lib.mkOption {
              type = lib.types.bool;
              default = false;
            };
	  };
	  config = lib.mkIf cfg.enable {
	  };
        };
      };
    };

}

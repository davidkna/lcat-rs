{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "https://flakehub.com/f/ipetkov/crane/0.20.*.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    {
      self,
      crane,
      flake-utils,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        commonArgs = craneLib.crateNameFromCargoToml { cargoToml = ./lcat/Cargo.toml; } // {
          inherit src;
          strictDeps = true;

          buildInputs = lib.optionals pkgs.stdenv.isDarwin (
            with pkgs;
            [
              iconv
              darwin.apple_sdk.frameworks.Security
            ]
          );
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        lcat = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
      in
      {
        formatter = nixpkgs.legacyPackages.${system}.nixfmt-rfc-style;

        checks = {
          inherit lcat;
          lcat-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          lcat-fmt = craneLib.cargoFmt commonArgs;
          lcat-deny = craneLib.cargoDeny commonArgs;
        };

        packages.default = lcat;

        apps.default = flake-utils.lib.mkApp { drv = lcat; };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};
        };
      }
    );
}

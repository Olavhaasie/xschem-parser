{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    git-hooks.url = "github:cachix/git-hooks.nix";
    git-hooks.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      advisory-db,
      git-hooks,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        inherit (nixpkgs) lib;
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = (crane.mkLib pkgs).overrideScope (
          _final: _prev: {
            rustfmt = pkgs.rustfmt.override { asNightly = true; };
          }
        );

        root = ./.;
        fileset = craneLib.fileset.commonCargoSources root;
        testFileset = lib.fileset.unions [
          fileset
          ./assets
        ];
        src = lib.fileset.toSource { inherit root fileset; };
        testSrc = lib.fileset.toSource {
          inherit root;
          fileset = testFileset;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        fileSetForCrate =
          crates:
          lib.fileset.toSource {
            inherit root;
            fileset = lib.fileset.unions (
              [
                ./Cargo.toml
                ./Cargo.lock
              ]
              ++ (map craneLib.fileset.commonCargoSources crates)
            );
          };
        buildCrate =
          name: deps:
          craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              pname = name;
              cargoExtraArgs = "-p ${name}";
              src = fileSetForCrate ([ ./crates/${name} ] ++ (map (c: ./crates/${c}) deps));
              # NB: we disable tests since we'll run them all via cargo-nextest
              doCheck = false;
            }
          );
        xschem-parser = buildCrate "xschem-parser" [ ];
        xschem-parser-cli = buildCrate "xschem-parser-cli" [ "xschem-parser" ];
      in
      {
        packages = {
          default = xschem-parser-cli;
          inherit xschem-parser-cli;
        };

        checks = {
          inherit xschem-parser xschem-parser-cli;

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              src = testSrc;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          fmt = craneLib.cargoFmt {
            inherit src;
          };

          toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
          };

          audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          deny = craneLib.cargoDeny {
            inherit src;
          };

          doctest = craneLib.cargoDocTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              src = testSrc;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );

          pre-commit-check = git-hooks.lib.${system}.run {
            src = ./.;
            settings.rust.check.cargoDeps = pkgs.rustPlatform.importCargoLock {
              lockFile = ./Cargo.lock;
            };
            hooks = {
              # General
              end-of-file-fixer.enable = true;
              trim-trailing-whitespace.enable = true;
              # Nix
              deadnix.enable = true;
              nixfmt-rfc-style.enable = true;
              statix.enable = true;
              # Rust
              clippy.enable = true;
              clippy.settings.allFeatures = true;
              rustfmt.enable = true;
              # File formats
              taplo.enable = true;
              markdownlint.enable = true;
            };
          };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            cargo-msrv
            rust-analyzer
          ];

          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}

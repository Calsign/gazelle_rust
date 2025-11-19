{
  description = "Bazel environment for making gazelle_rust work";
  inputs = {
    nixpkgs = {
      type = "github";
      owner = "NixOS";
      repo = "nixpkgs";
      ref = "nixpkgs-unstable";
    };
    flake-utils = {
      type = "github";
      owner = "numtide";
      repo = "flake-utils";
    };
  };
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        bazel-env = pkgs.buildFHSEnv {
          name = "bazel-env";
          targetPkgs = _pkgs:
            [
              pkgs.bazelisk
              pkgs.bazel-buildtools
              pkgs.zlib
            ];
          extraBuildCommands = ''
            ln -s /usr/bin/bazelisk $out/usr/bin/bazel
          '';
        };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [ bazel-env ];
          shellHook = ''
            bazel-env
          '';
        };
      });
}

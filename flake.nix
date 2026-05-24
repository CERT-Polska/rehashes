{
  description = "Python flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
    ...
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux" = {
      default = pkgs.mkShell {
        packages = [
          pkgs.python311
          pkgs.ssdeep
          # pkgs.python311Packages.pytest
        ];
        shellHook = ''
          export VENV=.venv
          if [ ! -d "$VENV" ]; then
            python -m venv $VENV
            source $VENV/bin/activate
            pip install --upgrade pip
            pip install pytest
          else
            source $VENV/bin/activate
          fi
        '';
      };
    };
  };
}

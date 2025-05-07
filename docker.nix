{ pkgs ? import <nixpkgs> {} }:

pkgs.dockerTools.buildImage {
  name = "myproject-container";
  tag = "latest";

  # NEW: Use copyToRoot instead of contents
  copyToRoot = pkgs.buildEnv {
    name = "myproject-env";
    paths = [
      pkgs.bash
      pkgs.cargo
      pkgs.sqlite
      pkgs.gcc
    ];
  };

  config = {
    Cmd = [ "/bin/bash" ];
    WorkingDir = "/root";
  };
}

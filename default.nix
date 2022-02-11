{ pkgs ? import <nixpkgs> {} }:
let
  sources = import ./nix/sources.nix;
  gis = import sources.gis {};
in pkgs.callPackage ./nix/package.nix { gis = gis; }

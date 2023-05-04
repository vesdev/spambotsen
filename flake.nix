{
  description = "spambotsen dev env";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-stable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  
  outputs = with pkgs ; {
    buildInputs = [
      openssl
      cargo-watch
      rust-bin.stable.latest.default
    ]
  }

}
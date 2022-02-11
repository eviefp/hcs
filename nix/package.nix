{ lib
, rustPlatform
, pkg-config
, openssl
, gis
}:

rustPlatform.buildRustPackage rec {
  pname = "hcs";
  version = "0.1.0";
  src = (gis.gitignoreSource ../.);
  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];
  cargoLock = {
    lockFile = ../Cargo.lock;
  };
}

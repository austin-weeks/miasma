{ rustPlatform
, lib
, pkg-config
, sqlite
}:
let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage rec {
  pname = cargoToml.package.name;
  version = cargoToml.workspace.package.version;

  src = builtins.path {
    path = ../.;
  };

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    sqlite
  ];

  # tests require internet connection, if I'm not mistaken
  doCheck = false;

  LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${lib.makeLibraryPath buildInputs}";

  cargoLock.lockFile = ../Cargo.lock;

  meta = {
    description = cargoToml.package.description;
    license = lib.licenses.gpl3;
    mainProgram = pname;
  };
}

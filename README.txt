Najciekawszym plikiem w tym projekcie jest ./src/Shaders/metaball_geometry.glsl - tutaj znajduje się implementacja marching cubes na gpu.

Cały kod oprócz shaderów jest napisany w języku Rust - https://www.rust-lang.org/
main.rs jest napisany raczej brzydko i wymaga refactoringu... Kiedy i jeśli to zrobię to prawdopodobnie opublikuję kod na GitHubie

Instrukcje budowania:

1) zainstalować Rusta (polecam multirust (Unix), lub multirust-rs (Windows) po czym w konsoli `multirust update nightly; multirust default nightly`)
2) komenda `cargo run` wywołana w głównym folderze projektu (folder z Cargo.toml) powinna ściągnąć wszystkie zależności, zbudować, i uruchomić program
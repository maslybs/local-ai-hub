fn main() {
  // Ensure `cargo` rebuilds when app icons change, otherwise `tauri dev` can keep showing
  // the old/default icon due to build script caching.
  println!("cargo:rerun-if-changed=tauri.conf.json");
  println!("cargo:rerun-if-changed=icons/icon.icns");
  println!("cargo:rerun-if-changed=icons/icon.ico");
  println!("cargo:rerun-if-changed=icons/32x32.png");
  println!("cargo:rerun-if-changed=icons/128x128.png");
  println!("cargo:rerun-if-changed=icons/128x128@2x.png");

  tauri_build::build()
}

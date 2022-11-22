pub fn get_skins_path() -> String {
    return dotenvy::var("SKINS_PATH").unwrap();
}

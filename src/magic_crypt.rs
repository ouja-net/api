use magic_crypt::{new_magic_crypt, MagicCryptTrait};

pub fn encrypt(string: &str) -> String {
    let mc = new_magic_crypt!(dotenvy::var("KEY").unwrap(), 256);
    return mc.encrypt_str_to_base64(string);
}

pub fn decrypt(string: &str) -> String {
    let mc = new_magic_crypt!(dotenvy::var("KEY").unwrap(), 256);
    return mc.decrypt_base64_to_string(string).unwrap();
}

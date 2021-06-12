use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};

use rand::Rng;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}



#[derive(Serialize, Deserialize)]
pub struct Key {
    tag: String,
    key: String
}

impl Key {
    pub fn get_key(&self) -> String {
        self.key.clone()
    }
    pub fn get_tag(&self) -> String {
        self.tag.clone()
    }
}

fn expand_key(seed : [u8;3]) -> Vec<u8> {
    let mut expanded = seed.to_vec();
    for _ in 0..6 {
        expanded.append(&mut seed.to_vec());
    }
    expanded.resize(16,0);
    return expanded;
}

pub fn generate_key() -> (Key, String) {
    let tag = rand::thread_rng().gen::<[u8; 12]>();
    let key = rand::thread_rng().gen::<[u8; 16]>();
    let nonce = rand::thread_rng().gen::<[u8; 12]>();
    let mut puzzle = rand::thread_rng().gen::<[u8; 3]>();
    let lastpuzzle = puzzle.get_mut(2).unwrap();
    *lastpuzzle = *lastpuzzle & 0x3;

    let puzzlekey = expand_key(puzzle);

    let mut ptxt = tag.to_vec();
    ptxt.append(&mut key.to_vec());

    let cipher = Aes128Gcm::new(GenericArray::from_slice(&puzzlekey));
    let mut ctxt = nonce.to_vec();
    ctxt.append(&mut cipher.encrypt(GenericArray::from_slice(&nonce), ptxt.as_ref()).unwrap());

    return (Key {tag: base64::encode(tag), key: base64::encode(key) }, base64::encode(ctxt));
}


pub fn recover_key(puzzle: String) -> Key {
    let bytes = base64::decode(puzzle).unwrap();
    let nonce = &bytes[..12];
    let ctxt = &bytes[12..];

    for b1 in 0..=255 {
        for b2 in 0..=255 {
            for b3 in 0..=3 {
                let candidate = expand_key([b1,b2,b3]);
                let cipher = Aes128Gcm::new(GenericArray::from_slice(&candidate));
                let ptxt = cipher.decrypt(GenericArray::from_slice(&nonce), ctxt);
                if ptxt.is_ok() {
                    let data = ptxt.unwrap();
                    let tag = &data[..12];
                    let key = &data[12..];
                    return Key {tag: base64::encode(tag), key: base64::encode(key)}
                }
            }
        }
    }
    return Key {tag: "".to_string(), key: "".to_string() }
}


#[derive(Serialize, Deserialize)]
pub struct Keybase {
    keys: HashMap<String,String>
}


impl Keybase {
    pub fn add_key(&mut self, k: Key) -> () {
        self.keys.insert(k.get_tag(), k.get_key());
    }

    pub fn get_key(&self, tag:&String) -> Option<Key> {
        let key = self.keys.get(tag);
        match key {
            None => None,
            Some(k) => Some( Key {key: k.clone(), tag: tag.clone()} )
        }
    }
}

pub fn get_keybase() -> Keybase {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    let json = storage.get("secretkeys").unwrap();
    if json.is_none() {
        let mut kbase =  Keybase { keys : HashMap::new() };
        kbase.add_key(Key {key : "AAAAAAAAAAAAAAAAAAAAAA==".to_string(),
                           tag: "AAAAAAAAAAAAAAAA".to_string()});
        return kbase
    } else {
        let kbase : Keybase = serde_json::from_str(&json.unwrap()).expect("Failed to parse Keybase Format");
        return kbase;
    }
}


pub fn set_keybase(kbase : Keybase) -> () {
    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    storage.set("secretkeys", &serde_json::to_string(&kbase).unwrap()).unwrap();
}

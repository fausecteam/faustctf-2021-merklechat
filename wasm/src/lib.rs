mod utils;
mod log;

use rand::Rng;

use aes_gcm::Aes128Gcm;
use aes_gcm::aead::{Aead, NewAead, generic_array::GenericArray};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use wasm_bindgen::prelude::*;

use std::convert::From;


// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}


#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct Ciphertext {
    tag: String,
    base64: String
}

impl From<Ciphertext> for js_sys::Map {
    fn from(ctxt:Ciphertext) -> Self {
        let map = js_sys::Map::new();
        map.set(&js_sys::JsString::from("tag"),
                &js_sys::JsString::from(ctxt.tag));
        map.set(&js_sys::JsString::from("base64"),
                &js_sys::JsString::from(ctxt.base64));
        map
    }
}

impl From<js_sys::Map> for Ciphertext {
    fn from(map:js_sys::Map) -> Self {
        Ciphertext{ tag:    map.get(&js_sys::JsString::from("tag")).as_string().unwrap(),
                    base64: map.get(&js_sys::JsString::from("base64")).as_string().unwrap() }
    }
}

#[wasm_bindgen(start)]
pub fn setup() {
    utils::set_panic_hook();
    console!("Enabled Panic Hook");
}


#[wasm_bindgen]
pub fn generate_uuid() -> String {
    let uuid = Uuid::new_v4();
    return format!("{}", uuid);
}



#[wasm_bindgen]
pub fn generate_keys() -> js_sys::Array {
    let pubkeys = js_sys::Array::new();
    let mut keybase = utils::get_keybase();
    for _ in 0..1091 {
        let (privkey, pubkey) = utils::generate_key();
        keybase.add_key(privkey);
        pubkeys.push(&js_sys::JsString::from(pubkey));
    }
    utils::set_keybase(keybase);
    return pubkeys;
}


#[wasm_bindgen]
pub fn encrypt_message(msg: String, keys: js_sys::Array) -> JsValue {
    let elt:u32 =
        if msg.len() > 12 {
            msg[..12].as_bytes().iter().fold(0, |acc, x| (acc + u32::from(*x)) % 1091)
        } else {
            msg.as_bytes().iter().fold(0, |acc, x| (acc + u32::from(*x)) % 1091)
        };
    let key = utils::recover_key(keys.get(elt).as_string().unwrap());
    let nonce = rand::thread_rng().gen::<[u8; 12]>();

    let cipher = Aes128Gcm::new(GenericArray::from_slice(&base64::decode(key.get_key()).unwrap()));
    let mut ctxt = nonce.to_vec();
    ctxt.append(&mut cipher.encrypt(GenericArray::from_slice(&nonce), msg.as_ref()).expect("Encryption Failure") );

    JsValue::from_serde( &Ciphertext { tag: key.get_tag(), base64: base64::encode(ctxt) } ).unwrap()
}


#[wasm_bindgen]
pub fn fake_encrypt_message(msg: String) -> JsValue {
    let key = [0;16];
    let nonce = rand::thread_rng().gen::<[u8; 12]>();

    let cipher = Aes128Gcm::new(GenericArray::from_slice(&key));
    let mut ctxt = nonce.to_vec();
    ctxt.append(&mut cipher.encrypt(GenericArray::from_slice(&nonce), msg.as_ref()).expect("Encryption Failure") );

    JsValue::from_serde( &Ciphertext { tag: base64::encode(&[0;12]), base64: base64::encode(ctxt) } ).unwrap()
}


#[wasm_bindgen]
pub fn decrypt_message(raw_ctxt: JsValue) -> String {
    let keybase = utils::get_keybase();
    let ctxt : Ciphertext = raw_ctxt.into_serde().unwrap();
    let key = keybase.get_key(&ctxt.tag).unwrap();

    let decoded = base64::decode(ctxt.base64).expect("Not valid Base64");
    let nonce = GenericArray::from_slice(&decoded[..12]);
    let ctxtbytes = &decoded[12..];

    let cipher = Aes128Gcm::new(GenericArray::from_slice(&base64::decode(key.get_key()).unwrap()));
    let plaintext = cipher.decrypt(nonce, ctxtbytes.as_ref()).expect("Decryption Failure");

    return String::from_utf8(plaintext).unwrap();
}

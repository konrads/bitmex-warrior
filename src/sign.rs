use chrono::{Duration, Utc};
use hex::encode as hexify;
use ring::hmac;


pub fn sign(s: &str, api_secret: &str) -> String {
    let signed_key = hmac::Key::new(hmac::HMAC_SHA256, api_secret.as_bytes());
    let signature = hexify(hmac::sign(&signed_key, s.as_bytes()));
    signature.to_uppercase()
}
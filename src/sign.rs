use hex::encode as hexify;
use ring::hmac;


/// Sign with bitmex payload with api_secret:
/// ```
/// # use bitmex_warrior::sign::sign;
///
/// let signed = sign("xxx", "yyy");
/// assert_eq!(signed, "0AF3DD476BCCBCB337D9396AFA6463486023C4511216AB1A358D7FB11330AF1E");
/// ```
pub fn sign(s: &str, api_secret: &str) -> String {
    let signed_key = hmac::Key::new(hmac::HMAC_SHA256, api_secret.as_bytes());
    let signature = hexify(hmac::sign(&signed_key, s.as_bytes()));
    signature.to_uppercase()
}
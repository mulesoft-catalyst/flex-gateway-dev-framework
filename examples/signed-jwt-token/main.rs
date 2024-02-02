use log::*;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Serialize, Deserialize};
use rsa::{RsaPrivateKey,pkcs1::DecodeRsaPrivateKey,sha2::Sha256};
use rsa::pkcs1v15::{SigningKey, VerifyingKey};
use rsa::signature::{Keypair, RandomizedSigner, SignatureEncoding, Verifier};
use serde_json::json;
use base64::{encode_config, URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use jwt_compact::{prelude::*};

fn main() {
    let private_key_pem = Some("-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEAySajPaaXBchE7ty3FDedfBXCRiXrb1OBPl6SzBrRHfpaftLZ\nZ60b8p1Y57GBv9fyT2IEzq17N7nobiDJxG+/xWyZYZDwhCdK8qLZC9JgazKhWBUI\nj6arjK+n89f7toBhsZjceVsXEOUweDCuoQRwGlw2bY+TNe5VZFUIuXI5b4eDI8Ze\nc+qzEpArj8zoMLpuP8wTVUR04GGe2HngdNGFhHuayHrKaNvKSwO6Bzp8AvOau1sR\nQvDRneu6abfAJx1koSeNvK/JBvEn2MGgAJd1SL5CYl7o/Uxk4CEyMGFRiqejpYk2\nllEnudI0i856MSiwaM2ehU7Sw9XCC/Lglb9Y4wIDAQABAoIBAGWMAOr1t9YudUZU\n3IPzU6i531rEd+e6s0uGOPubKijFI3xU+3YQeURw1NoazZLI9MXIiP7Bq6vFSaaX\nHOTzOU/0dDZCEnnU0ExPk90Y9p4HcFZkP+8tR/t9Df/W8HcAttEOh3coWiuoWGDE\nytP0xpc4KC4FRl76k9dT6lScaox3ap8tpNQTctGx9FWjbf95s4CWpR1i2/HS0t3H\nVMXuo1hVSqONG5SnGOw2ZI+jAkIVk539VXFNQg/Wbyw+N0j8/IHK5h47KVLMJBk1\nAIv3n76w7q5qd0cbBY9DGIpIQHxP3N4Rmbr6N0GJ0+sgtzCVPF4mwFq97dJ04eHn\nyUb6kLECgYEA667dVhXBADjrijhGeVqbWo1J9wlhrZmSZpil+iPb3LaeCN6hLMS5\nucIWrG9iZvggmLrIMZDbeQYccnF+PCK1+aUM49YVDuzKDgWbebkmnV84Xe3La+ge\nOVPYxknN8ib2SGj2FiE/TyKf3bImb+2UCkYxAdb3AH85qhbCT+CvN1sCgYEA2n20\nt+n45eriYlha2YAmlyJqqvKKGjoDX7TnjofW7F6X3pZHGD75PH9t8Nj5pCQ+YqwI\nV+kXtgJEo06qqMEDxMgof9CSusyCdca6tNxjnztEGKiYXZ320U03TLMXSyS0prfH\nEjcvN2LrowkRWs+BRDxKxlgsqApqzVpgoU5soxkCgYEAl43P2M6OWHVByZUchGbm\nZZlbidbnj/mkMgNRIAQ1Gr336941qJMsly4e5G2PyoUeePgLSgxLhYyPeGxQ75ym\nhk1Dd7MvwXQ1DkU+0Exmb2ZO2d/TIqWV4elwCTpmi6PQVYnhTSO+EUZDpQ0tOGZ+\n/whZ9UPle9VUIHMYKsNiCOcCgYEAjiAtWWFaZf0nq/97gXZKJA/gQTkqOqs4HJ6p\n7d5RIUrbEZDByHTWXdMqqfwKUAq/hiIP/1eB9AR/r5dxjfqYP/5P0ezUNazoxEI0\nm4etShwcTMoG6wbhUf8flMX+xgOne7eaSDAbh7TSQSiZ4G+Qz9lEbsdWrSyxkslS\niDtikakCgYBCzLBzXghBJ0eSgcLZNv4tPZj87DJsT7TyxajJjiYjMCD/Zfui9Khy\nRz6qMGxD6k7H4taYQkIIxDFXEFXFezwgEDbTFy8wImKrqHiNFXuMIb8pYUxKK6+i\npPUXk2V2+WuOxKVZ7LhIIn4nUgEY/E8BGAPsIUZiXTxA3cNFgfC2aA==\n-----END RSA PRIVATE KEY-----\n".to_string());

    let private_key = RsaPrivateKey::from_pkcs1_pem(&private_key_pem.as_ref().expect("Private Key has not been configured")).expect("failed to generate a key");
    let signing_key = Some(SigningKey::<Sha256>::new(private_key));
    let verifying_key = Some(signing_key.as_ref().expect("Signing Key has not been created").verifying_key());
}
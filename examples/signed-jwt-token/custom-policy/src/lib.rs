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

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(CustomRootContext {
            config: CustomConfig::default(),
            keys: CustomKeys::default()
        })
    });
}}

// Policy configuration
#[derive(Default, Clone, Deserialize)]
struct CustomConfig {

    #[serde(alias = "header")]
    header: Option<String>,

    #[serde(alias = "privateKeyPem")]
    private_key_pem: Option<String>,
}

// Keys
#[derive(Default, Clone)]
struct CustomKeys {
    signing_key: Option<SigningKey<Sha256>>,
    verifying_key: Option<VerifyingKey<Sha256>>
}


// ROOT CONTEXT
// The struct will implement the trait RootContext and contain the Policy configuration
struct CustomRootContext {
    config: CustomConfig,
    keys: CustomKeys
}

impl Context for CustomRootContext {}

// The trait RootContext is required by Proxy WASM
impl RootContext for CustomRootContext {

    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.config = serde_json::from_slice(config_bytes.as_slice()).unwrap();

            // UNCOMMENT THIS IF THE PRIVATE KEY IS NOT SET IN THE POLICY CONFIGURATION
//          self.config.private_key_pem = Some("-----BEGIN RSA PRIVATE KEY-----
// MIIEpAIBAAKCAQEAySajPaaXBchE7ty3FDedfBXCRiXrb1OBPl6SzBrRHfpaftLZ
// Z60b8p1Y57GBv9fyT2IEzq17N7nobiDJxG+/xWyZYZDwhCdK8qLZC9JgazKhWBUI
// j6arjK+n89f7toBhsZjceVsXEOUweDCuoQRwGlw2bY+TNe5VZFUIuXI5b4eDI8Ze
// c+qzEpArj8zoMLpuP8wTVUR04GGe2HngdNGFhHuayHrKaNvKSwO6Bzp8AvOau1sR
// QvDRneu6abfAJx1koSeNvK/JBvEn2MGgAJd1SL5CYl7o/Uxk4CEyMGFRiqejpYk2
// llEnudI0i856MSiwaM2ehU7Sw9XCC/Lglb9Y4wIDAQABAoIBAGWMAOr1t9YudUZU
// 3IPzU6i531rEd+e6s0uGOPubKijFI3xU+3YQeURw1NoazZLI9MXIiP7Bq6vFSaaX
// HOTzOU/0dDZCEnnU0ExPk90Y9p4HcFZkP+8tR/t9Df/W8HcAttEOh3coWiuoWGDE
// ytP0xpc4KC4FRl76k9dT6lScaox3ap8tpNQTctGx9FWjbf95s4CWpR1i2/HS0t3H
// VMXuo1hVSqONG5SnGOw2ZI+jAkIVk539VXFNQg/Wbyw+N0j8/IHK5h47KVLMJBk1
// AIv3n76w7q5qd0cbBY9DGIpIQHxP3N4Rmbr6N0GJ0+sgtzCVPF4mwFq97dJ04eHn
// yUb6kLECgYEA667dVhXBADjrijhGeVqbWo1J9wlhrZmSZpil+iPb3LaeCN6hLMS5
// ucIWrG9iZvggmLrIMZDbeQYccnF+PCK1+aUM49YVDuzKDgWbebkmnV84Xe3La+ge
// OVPYxknN8ib2SGj2FiE/TyKf3bImb+2UCkYxAdb3AH85qhbCT+CvN1sCgYEA2n20
// t+n45eriYlha2YAmlyJqqvKKGjoDX7TnjofW7F6X3pZHGD75PH9t8Nj5pCQ+YqwI
// V+kXtgJEo06qqMEDxMgof9CSusyCdca6tNxjnztEGKiYXZ320U03TLMXSyS0prfH
// EjcvN2LrowkRWs+BRDxKxlgsqApqzVpgoU5soxkCgYEAl43P2M6OWHVByZUchGbm
// ZZlbidbnj/mkMgNRIAQ1Gr336941qJMsly4e5G2PyoUeePgLSgxLhYyPeGxQ75ym
// hk1Dd7MvwXQ1DkU+0Exmb2ZO2d/TIqWV4elwCTpmi6PQVYnhTSO+EUZDpQ0tOGZ+
// /whZ9UPle9VUIHMYKsNiCOcCgYEAjiAtWWFaZf0nq/97gXZKJA/gQTkqOqs4HJ6p
// 7d5RIUrbEZDByHTWXdMqqfwKUAq/hiIP/1eB9AR/r5dxjfqYP/5P0ezUNazoxEI0
// m4etShwcTMoG6wbhUf8flMX+xgOne7eaSDAbh7TSQSiZ4G+Qz9lEbsdWrSyxkslS
// iDtikakCgYBCzLBzXghBJ0eSgcLZNv4tPZj87DJsT7TyxajJjiYjMCD/Zfui9Khy
// Rz6qMGxD6k7H4taYQkIIxDFXEFXFezwgEDbTFy8wImKrqHiNFXuMIb8pYUxKK6+i
// pPUXk2V2+WuOxKVZ7LhIIn4nUgEY/E8BGAPsIUZiXTxA3cNFgfC2aA==
// -----END RSA PRIVATE KEY-----".to_string());

            let private_key = RsaPrivateKey::from_pkcs1_pem(&self.config.private_key_pem.as_ref().expect("Private Key has not been configured")).expect("failed to generate a key");
            self.keys.signing_key = Some(SigningKey::<Sha256>::new(private_key));
            self.keys.verifying_key = Some(self.keys.signing_key.as_ref().expect("Signing Key has not been created").verifying_key())
        }

        true
    }

    // Other implemented methods
    // ...

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CustomHttpContext {
            config: self.config.clone(),
            keys: self.keys.clone(),
            context_id: context_id
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

// HTTP CONTEXT
// The struct will implement the trait Http Context to support the HTTP headers and body operations

struct CustomHttpContext {
    pub config: CustomConfig,
    pub keys: CustomKeys,
    context_id: u32,
}

impl Context for CustomHttpContext {}

fn base64url_encode(input: &[u8]) -> String {
    encode_config(input, URL_SAFE_NO_PAD)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct CustomClaims {
    /// `sub` is a standard claim which denotes claim subject:
    /// https://tools.ietf.org/html/rfc7519#section-4.1.2
    #[serde(rename = "sub")]
    subject: String,
}

impl HttpContext for CustomHttpContext {

    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!("on_http_request_headers");
        debug!("Config header: {}", self.config.header.clone().unwrap_or("No string here".to_string()));
        
        // OPTIONALLY USE JWT_COMPACT TO GENERATE THE CLAIM
        let time_options = TimeOptions::default();
        let jwt_payload = Claims::new(CustomClaims { subject: "alice".to_owned() })
            .set_duration_and_issuance(&time_options, Duration::days(7))
            .set_not_before(Utc::now() - Duration::hours(1));
        let payload = serde_json::to_string(&jwt_payload).unwrap();
        // OR GENERATE IT MANUALLY
        // let payload = json!({"sub": "alice"}).to_string();

        let header = json!({"alg": "RS256","typ": "JWT"}).to_string();
        let encoded_header = base64url_encode(header.as_bytes());
        let encoded_payload = base64url_encode(payload.as_bytes());
        let header_payload = format!("{}.{}", encoded_header, encoded_payload);
        
        debug!("Header and Payload: {:#?}", header_payload);

        // Sign
        let mut rng = rand::thread_rng();
        let signature = self.keys.signing_key.as_ref().expect("Signing Key has not been created").sign_with_rng(&mut rng, header_payload.as_bytes());
        assert_ne!(signature.to_bytes().as_ref(), header_payload.as_bytes());

        // Verify (THIS IS OPTIONAL - NOT RECOMMENDED IN PRODUCTION)
        self.keys.verifying_key.as_ref().expect("Verifying Key has not been created").verify(header_payload.as_bytes(), &signature).expect("failed to verify");

        let encoded_signature = base64url_encode(&signature.to_bytes().as_ref());
        debug!("Signature: {:#?}", encoded_signature);

        let jwt = format!("{}.{}", header_payload, encoded_signature);
        debug!("JWT Token: {:#?}", jwt);

        Action::Continue
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        debug!("on_http_request_body");
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!("on_http_response_headers");
        for (name, value) in &self.get_http_response_headers() {
            debug!("#{} <- {}: {}", self.context_id, name, value);
        }
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        debug!("on_http_response_body");
        Action::Continue
    }

}
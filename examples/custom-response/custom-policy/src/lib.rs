use log::*;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(CustomRootContext {
            config: CustomConfig::default(),
        })
    });
}}

// Policy configuration
#[derive(Default, Clone, Deserialize)]
struct CustomConfig {

    #[serde(alias = "header")]
    header: Option<String>,
}


// ROOT CONTEXT
// The struct will implement the trait RootContext and contain the Policy configuration
struct CustomRootContext {
    config: CustomConfig,
}

impl Context for CustomRootContext {}

// The trait RootContext is required by Proxy WASM
impl RootContext for CustomRootContext {

    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            self.config = serde_json::from_slice(config_bytes.as_slice()).unwrap();
        }

        true
    }

    // Other implemented methods
    // ...

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CustomHttpContext {
            config: self.config.clone(),
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
    #[allow(dead_code)]
    context_id: u32,
}

impl Context for CustomHttpContext {}

impl CustomHttpContext {
    fn log_attribute(&mut self, context: &str, attribute: &str) {
        let property_path = vec![context, attribute];
        match self.get_property(property_path) {
            None => debug!("No {} {}", context, attribute),
            Some(property) => {
                let property_string = String::from_utf8(property);
                debug!("{}.{} {:?}", context, attribute, property_string.unwrap());
            },
        }
    }
}

impl HttpContext for CustomHttpContext {

    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        debug!("on_http_request_headers");
        debug!("Config header: {}", self.config.header.clone().unwrap_or("No string here".to_string()));
        self.set_http_request_header("aHeader", Some("aValue"));

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

        self.log_attribute("response", "code_details");
        self.log_attribute("response", "flags");
        self.log_attribute("response", "grpc_status");
        self.log_attribute("response", "size");
        self.log_attribute("upstream", "transport_failure_reason");

        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        debug!("on_http_response_body");
        Action::Continue
    }

}
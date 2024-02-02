use jsonpath_lib::select;
use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use url::Url;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(ServiceCalloutPolicyRoot {
            config: ServiceCalloutPolicyConfig::default(),
        })
    });
}}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Header {
    header_name: String,
    header_value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Parameter {
    parameter_name: String,
    response_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct ServiceCalloutPolicyConfig {
    url: String,
    service_name: String,
    headers: Option<Vec<Header>>,
    request_type: String,
    request_content_type: Option<String>,
    request_body: Option<String>,
    time_out: u64,
    parameters: Option<Vec<Parameter>>,
}

struct ServiceCalloutPolicyRoot {
    config: ServiceCalloutPolicyConfig,
}

impl Context for ServiceCalloutPolicyRoot {}

impl RootContext for ServiceCalloutPolicyRoot {
    fn on_configure(&mut self, _: usize) -> bool {
        if let Some(config_bytes) = self.get_plugin_configuration() {
            match serde_json::from_slice::<ServiceCalloutPolicyConfig>(config_bytes.as_slice()) {
                Ok(config) => {
                    // info!("Service Callout Policy configuration: {:#?}", config);
                    self.config = config;
                }
                Err(e) => {
                    info!(
                        "Failed to parse Service Callout Policy configuration: {:?}",
                        e
                    );
                }
            }
        } else {
            info!("Failed to get configuration.");
        }
        true
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(ServiceCalloutPolicyContext {
            config: self.config.clone(),
        }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

// HTTP context
struct ServiceCalloutPolicyContext {
    config: ServiceCalloutPolicyConfig,
}

impl HttpContext for ServiceCalloutPolicyContext {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        // info!("ServiceCalloutPolicyConfig: {:?}", self.config.clone());
        // info!("url: {:?}", &self.config.url);
        // info!("service name: {:?}", &self.config.service_name);
        // info!("request type: {:?}", &self.config.request_type);
        // info!("headers: {:?}", &self.config.headers);
        // info!(
        //     "request contesnt type: {:?}",
        //     &self.config.request_content_type
        // );
        // info!("request_body: {:?}", &self.config.request_body);
        // info!("timeout: {:?}", &self.config.time_out);
        // info!("parameter: {:?}", &self.config.parameters);
        if self.check_cache_hit() {
            // If it's a cache hit, continue the request.
            return Action::Continue;
        }

        match self.prepare_http_call_data() {
            Some((service_name, request_headers, request_body, time_out)) => {
                self.make_http_call(&service_name, request_headers, request_body, time_out);
                Action::Pause
            }
            None => {
                // info!("Failed to parse request URL. with error: {:?}", e);
                // return None;
                info!("Failed to parse configuration.");
                let error_response = self.generate_error_response_json(400);
                self.send_http_response(
                    400,
                    vec![("Content-Type", "application/json")],
                    Some(error_response.as_bytes()),
                );
                Action::Continue
            }
        }
    }
}

impl Context for ServiceCalloutPolicyContext {
    fn on_http_call_response(&mut self, _: u32, _: usize, body_size: usize, _: usize) {
        // info!("on call response");

        // let http_response_headers = self.get_http_call_response_headers();
        // info!(
        //     "get_http_call_response_headers(): {:?}",
        //     http_response_headers
        // );

        // let http_response_trailers = self.get_http_call_response_trailers();
        // info!(
        //     "get_http_call_response_trailers(): {:?}",
        //     http_response_trailers
        // );

        // Check if the HTTP response status code is successful (e.g., 200 OK).
        let status_code = self
            .get_http_call_response_header(":status")
            .unwrap_or_default();
        info!("get_http_call_response_header(:status): {}", &status_code);
        if !status_code.starts_with("20") {
            // Return a 503 error.

            let error_response = self.generate_error_response_json(503);
            self.send_http_response(
                503,
                vec![("Content-Type", "application/json")],
                Some(error_response.as_bytes()),
            );
            return;
        }
        if let Some(body) = self.get_http_call_response_body(0, body_size) {
            // info!("formatted body: {:?}", std::str::from_utf8(&body));
            self.handle_http_response(&body);
        }

        self.resume_http_request();
        return;
    }
}

impl ServiceCalloutPolicyContext {
    fn check_cache_hit(&self) -> bool {
        if let Some(value) = self.get_http_request_header("X-Cache-Status") {
            if value == "hit" {
                return true;
            }
        }
        info!("cache miss");
        false
    }

    fn check_composit_auth(&self) -> Option<(String, String)> {
        match (
            self.get_http_request_header("X-Composit-auth-URL"),
            self.get_http_request_header("X-Composit-auth-Service-name"),
        ) {
            (Some(str_url), Some(service_name)) => Some((str_url, service_name)),
            _ => None,
        }
    }

    fn prepare_http_call_data(
        &self,
    ) -> Option<(String, Vec<(String, String)>, Option<&[u8]>, u64)> {
        // Take values form either headers for composit auth or config

        let (str_url, mut service_name) = match self.check_composit_auth() {
            Some((composit_url, composit_service_name)) => (composit_url, composit_service_name),
            None => (self.config.url.clone(), self.config.service_name.clone()),
        };

        service_name = format!("{}{}", &service_name, ".default.svc");

        let url = match Url::parse(&str_url) {
            Ok(url) => url,
            Err(e) => {
                info!("Failed to parse request URL. with error: {:?}", e);
                // return None;
                let error_response = self.generate_error_response_json(400);
                self.send_http_response(
                    400,
                    vec![("Content-Type", "application/json")],
                    Some(error_response.as_bytes()),
                );
                return None;
            }
        };
        let mut request_headers: Vec<(String, String)> = Vec::new();
        // info!("external call body {:?}", &self.config.request_body);
        let request_type = self.config.request_type.clone();
        // let request_body = self.config.request_body;
        let request_body = if request_type == "POST" {
            self.config.request_body.as_ref().map(|s| s.as_bytes())
        } else {
            None
        };
        if let Some(headers) = &self.config.headers {
            for header in headers {
                request_headers.push((header.header_name.clone(), header.header_value.clone()));
            }
        }
        let mut path = url.path().to_owned();
        if let Some(query) = url.query() {
            path = format!("{}?{}", path, query);
            // path = string_path.as_str();
        }

        let mut default_headers: Vec<(String, String)> = vec![
            (":method".to_string(), request_type.clone()),
            (":path".to_string(), path.clone()),
            (
                ":authority".to_string(),
                url.host_str().unwrap_or("").to_string(),
            ),
            (":scheme".to_string(), url.scheme().to_string()),
            // ("content-type", &request_content_type),
        ];
        if let Some(content_type) = &self.config.request_content_type {
            if content_type.len() > 0 && request_type == "POST" {
                default_headers.push(("content-type".to_string(), content_type.to_owned()));
            }
        }
        request_headers.append(&mut default_headers);

        info!("external call headers: {:?} ", request_headers);

        let time_out = self.config.time_out;
        return Some((
            service_name,
            request_headers.clone(),
            request_body,
            time_out,
        ));
    }

    fn make_http_call(
        &self,
        service_name: &str,
        request_headers: Vec<(String, String)>,
        request_body: Option<&[u8]>,
        time_out: u64,
    ) {
        let converted_vec: Vec<(&str, &str)> = request_headers
            .iter()
            .map(|(first, second)| (first.as_str(), second.as_str()))
            .collect();
        // Dispatch HTTP call
        match self.dispatch_http_call(
            service_name, //&new_upstream, //
            converted_vec,
            request_body,
            vec![],
            Duration::from_secs(time_out),
        ) {
            Ok(resp) => {
                info!("Service Invocation response: {:?}", resp)
            }
            Err(err) => {
                info!("request >>> Error: {:?}", err);
                let error_response = self.generate_error_response_json(503);
                self.send_http_response(
                    503,
                    vec![("Content-Type", "application/json")],
                    Some(error_response.as_bytes()),
                );
            }
        }
    }

    fn handle_http_response(&self, body: &[u8]) {
        // Parse the JSON response body into a structured data format.
        let response_data: Result<Value, _> = serde_json::from_slice(&body);
        // let mut append_vec: Vec<String> = vec![];
        match response_data {
            Ok(json_data) => {
                // info!("Parsed response data: {:?}", json_data);
                // Extract values using JSONPath based on the Parameter.response_path.
                if let Some(parameters) = &self.config.parameters {
                    for param in parameters {
                        match select(&json_data, &param.response_path) {
                            Ok(selected_values) => {
                                if selected_values.len() == 1 {
                                    // Extract the single value from selected_values.
                                    let extracted_value = &selected_values[0];

                                    // Remove the double quotes from the value.
                                    let value_without_quotes = &extracted_value.to_string()
                                        [1..extracted_value.to_string().len() - 1];
                                    // Set the HTTP request header using the Parameter.parameter_name and extracted_value.
                                    self.set_http_request_header(
                                        &param.parameter_name,
                                        Some(value_without_quotes),
                                    );

                                    info!(
                                        "Set header: {} = {}",
                                        &param.parameter_name,
                                        &extracted_value.to_string()
                                    );
                                } else {
                                    info!(
                                        "Expected a single value for JSONPath: {}, but got multiple values",
                                        &param.response_path
                                        // Note: Should Return 400
                                    );

                                    let error_response = self.generate_error_response_json(400);
                                    self.send_http_response(
                                        400,
                                        vec![("Content-Type", "application/json")],
                                        Some(error_response.as_bytes()),
                                    );
                                }
                            }
                            Err(e) => {
                                info!(
                                    "Failed to find values for JSONPath: {}, with error {:?}",
                                    &param.response_path,
                                    e // Note: Should Return 400
                                );

                                let error_response = self.generate_error_response_json(400);
                                self.send_http_response(
                                    400,
                                    vec![("Content-Type", "application/json")],
                                    Some(error_response.as_bytes()),
                                );
                            }
                        }
                    }
                }
                // info!("headers for chache policy {:?}", append_vec);
            }
            Err(e) => {
                info!("Failed to parse response data as JSON: {:?}", e);
                // Return a 400 Bad Request response.
                let error_response = self.generate_error_response_json(400);
                self.send_http_response(
                    400,
                    vec![("Content-Type", "application/json")],
                    Some(error_response.as_bytes()),
                );
            }
        }
    }

    fn generate_error_response_json(&self, error_code: u32) -> String {
        #[derive(Serialize, Deserialize)]
        struct ErrorResponse {
            status: u32,
            message: String,
        }

        let (status_text, default_message) = match error_code {
            400 => (
                "Bad Request",
                "The request is invalid or missing some required parameters.",
            ),
            403 => ("Forbidden", "Access to this resource is forbidden."),
            404 => ("Not Found", "The requested resource could not be found."),
            429 => (
                "Too Many Requests",
                "You have exceeded the rate limit for making requests.",
            ),
            500 => (
                "Internal Server Error",
                "An internal server error occurred.",
            ),
            503 => (
                "Service Unavailable",
                "The service is currently unavailable.",
            ),
            _ => ("Unknown Error", "An unknown error occurred."),
        };

        let error_response = ErrorResponse {
            status: error_code,
            message: format!(
                "Error {} - {} ({})",
                error_code, status_text, default_message
            ),
        };

        serde_json::to_string(&error_response).unwrap()
    }
}

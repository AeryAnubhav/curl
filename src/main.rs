// Assignmnet 3
// Student Name: Anubhav Aery
// Student Number: 1005839513
use reqwest::blocking::Client;
use serde_json::Value;
use std::env;
use url::{ParseError, Url};

fn sort_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted_map = serde_json::Map::new();
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            for key in keys {
                sorted_map.insert(key.clone(), sort_json(&map[key]));
            }
            Value::Object(sorted_map)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(sort_json).collect()),
        _ => value.clone(),
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    // Variables to store command line options
    let mut url_str = String::new();
    let mut method = "GET".to_string();
    let mut data = String::new();
    let mut is_json = false;

    // Iterate over the arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-X" => {
                i += 1;
                if i < args.len() {
                    method = args[i].to_uppercase();
                } else {
                    eprintln!("Error: No method specified after -X.");
                    return;
                }
            }
            "-d" => {
                i += 1;
                if i < args.len() {
                    data = args[i].clone();
                    method = "POST".to_string(); // If data is provided, default to POST
                } else {
                    eprintln!("Error: No data specified after -d.");
                    return;
                }
            }
            "--json" => {
                i += 1;
                if i < args.len() {
                    data = args[i].clone();
                    method = "POST".to_string(); // JSON implies POST
                    is_json = true;
                } else {
                    eprintln!("Error: No JSON data specified after --json.");
                    return;
                }
            }
            _ => {
                // Assume this is the URL
                if url_str.is_empty() {
                    url_str = args[i].clone();
                } else {
                    eprintln!("Error: Unknown argument '{}'.", args[i]);
                    return;
                }
            }
        }
        i += 1;
    }

    if url_str.is_empty() {
        eprintln!("Usage: curl <URL> [-d <data>] [--json <JSON data>] [-X <method>]");
        return;
    }

    // Validate the HTTP method
    if method != "GET" && method != "POST" {
        eprintln!(
            "Error: Unsupported HTTP method '{}'. Only GET and POST are supported.",
            method
        );
        return;
    }

    // Display the URL
    println!("Requesting URL: {}", url_str);
    println!("Method: {}", method);

    // Display data if provided
    if !data.is_empty() {
        if is_json {
            println!("JSON: {}", data);
        } else {
            println!("Data: {}", data);
        }
    }

    /*
    Validate URLs and handle specific errors, 3 cases:
        1) Missing or invalid protocols (e.g., data://example.com, http//example.com)
        2) Invalid IP addresses (e.g., https://255.255.255.256)
        3) Invalid port numbers (e.g., http://127.0.0.1:65536)
    */

    // Parse and validate the URL
    let parsed_url = match Url::parse(&url_str) {
        Ok(url) => {
            // Ensure protocol is HTTP or HTTPS
            if url.scheme() != "http" && url.scheme() != "https" {
                eprintln!("Error: The URL does not have a valid base protocol.");
                return;
            }
            url
        }
        Err(e) => {
            // Handle various URL parsing errors
            match e {
                ParseError::InvalidIpv4Address => {
                    eprintln!("Error: The URL contains an invalid IPv4 address.");
                }
                ParseError::InvalidIpv6Address => {
                    eprintln!("Error: The URL contains an invalid IPv6 address.");
                }
                ParseError::InvalidPort => {
                    eprintln!("Error: The URL contains an invalid port number.");
                }
                ParseError::RelativeUrlWithoutBase => {
                    eprintln!("Error: The URL does not have a valid base protocol.");
                }
                _ => {
                    eprintln!("Error: The URL does not contain a valid host.");
                }
            }
            return;
        }
    };

    // Validate host
    if parsed_url.host().is_none() {
        eprintln!("Error: The URL does not contain a valid host.");
        return;
    }

    // Validate port number
    if let Some(port) = parsed_url.port() {
        if port == 0 {
            eprintln!("Error: The URL contains an invalid port number.");
            return;
        }
    }

    /*
        Process Send Request
        1) Make the HTTP Request
        2) Prepare the request
        3) Send the HTTP Request

    */

    //Make the HTTP Request
    let client = Client::new();

    //Prepare the request
    let request = if method == "POST" {
        if is_json {
            // Validate JSON data and include error message from the parser
            let _json_value: Value =
                serde_json::from_str(&data).unwrap_or_else(|e| panic!("Invalid JSON: {:?}", e));
            client
                .post(parsed_url.as_str())
                .header("Content-Type", "application/json")
                .body(data.clone())
        } else {
            // Parse into key-value pairs
            let params: Vec<(&str, &str)> = data
                .split('&')
                .filter_map(|pair| {
                    let mut split = pair.splitn(2, '=');
                    match (split.next(), split.next()) {
                        (Some(key), Some(value)) => Some((key, value)),
                        (Some(key), None) => Some((key, "")), // Handle case where value is missing
                        _ => None,
                    }
                })
                .collect();
            client.post(parsed_url.as_str()).form(&params) // Prepare POST request with form data
        }
    } else {
        client.get(parsed_url.as_str()) //Prepare GET request
    };

    //Send the HTTP request
    match request.send() {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                // Successful response
                match response.text() {
                    Ok(body) => {
                        // Check if response body is JSON
                        if let Ok(_json_value) = serde_json::from_str::<Value>(&body) {
                            let sorted_json = sort_json(&_json_value); // Sort JSON keys
                            println!("Response body (JSON with sorted keys):");
                            println!("{}", serde_json::to_string_pretty(&sorted_json).unwrap())
                        } else {
                            // Print response body directly
                            println!("Response body:");
                            println!("{}", body);
                        }
                    }
                    Err(e) => eprintln!("Error reading response body: {}", e),
                }
            } else {
                // Handle non-successful status codes
                eprintln!(
                    "Error: Request failed with status code: {}.",
                    status.as_u16()
                );
            }
        }
        Err(e) => {
            // Handle connection errors
            if e.is_connect() {
                eprintln!("Error: Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.");
            } else {
                eprintln!("Error making {} request: {}.", method, e);
            }
        }
    }
}

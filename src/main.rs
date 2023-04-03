use std::io;

mod open_ai {
    use std::{io, io::Read};
    use serde::Deserialize;
    use serde_json::{json, Value};
    use reqwest::{header};
    use toml;

    const OPEN_AI_URL: &str = "https://api.openai.com/v1/chat/completions";

    #[derive(Deserialize)]
    struct Configuration {
	api_key: String,
	model: String
    }

    #[derive(Deserialize)]
    struct OpenaiResponse {
	id: String,
	object: String,
	created: i64,
	model: String,
	usage: Value,
	choices: Vec<OpenaiChoice>
    }

    #[derive(Deserialize)]
    struct OpenaiChoice {
	message: OpenaiMessage
    }

    #[derive(Deserialize)]
    struct OpenaiMessage {
	role: String,
	content: String,
//	finish_reason: String,
//	index: i32
    }

    /// Load configurations for the trade application
    fn load_config(filename: &str) -> io::Result<Configuration> {
	let mut file = std::fs::File::open(filename)?;
	let mut contents = String::new();
	file.read_to_string(&mut contents)?;
	let config = toml::from_str(&contents);
	if let Err(c) = config {
	    panic!("Couldn't read config file: {:?}", c);
	}
	Ok(config.unwrap())
    }

    fn get_headers(api_key: &str) -> header::HeaderMap {
	let hv = header::HeaderValue::from_str(&format!("Bearer {}",
							api_key))
	    .expect("Invalid key format");
	let mut headers = header::HeaderMap::new();
	headers.insert(header::AUTHORIZATION, hv);
	headers.insert(header::CONTENT_TYPE,
		       header::HeaderValue::from_static("application/json"));

	return headers;
    }

    fn call_service(prompt: &str) -> Result<String, ()> {
	let config = load_config("./.openAi.yml")
	    .unwrap_or_else(|e| panic!("Couldn't load config file: {}", e));
	let headers = get_headers(&config.api_key);
	let url = OPEN_AI_URL;
	let body = json!({
	    "model": config.model,
	    "messages": [{"role": "user", "content": prompt}],
	    "temperature": 0.7
	});
	let client = reqwest::blocking::Client::builder()
	    .default_headers(headers)
	    .build();

	let resp = client
	    .expect("OpenAI client failed to initialize")
	    .post(url)
	    .json(&body)
	    .send();

	if resp.is_err() {
	    panic!("Internal error getting response from OpenAI: {:?}", resp);
	}
	let resp = resp.expect("Response failed");
	if resp.status() != 200 {
	    panic!("[Invalid Status: {}]\nResponse from OpenAi: {:?}",
		   resp.status(), resp);
	}
	let r = resp.text()
	    .expect("Response body from OpenAI is invalid");
	let outcome: OpenaiResponse = serde_json::from_str(&r)
	    .expect("Couldn't parse response to JSON");

	let answer = &outcome.choices
	    .get(0)
	    .expect("No choices found")
	    .message
	    .content;
	return Ok(answer.to_string());
    }

    pub fn get_answer(question: &str) -> String {

	let r = call_service(question);
	if let Ok(answer) = r {
	    return answer;
	}
	return String::from(""); // TODO Return response
    }
}


fn main() -> Result<(), io::Error> {
    loop {
	let mut question = String::new();
	println!("Type your question: ");
	io::stdin()
	    .read_line(&mut question)
	    .expect("Cannot read user input");
	let question = question.trim();

	if question == "q" {
	    break;
	}
	let answer = open_ai::get_answer(&question);
	println!("{}", answer);
    } 

    println!("bye");
    Ok(())
}

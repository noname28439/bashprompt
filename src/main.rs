use std::{env, io::{stdin, stdout, Write}, process::{exit, Command}};
use reqwest::Client;
use serde_json::Value;

fn run_shell_command(full_command:String){
    let mut child = Command::new("bash")
    .arg("-c")
    .arg(full_command)
    .spawn().expect("Lauch failed...");
    let _result = child.wait().expect("Failed to wait for execution");

}

fn build_prompt_request(prompt:String) -> String{
    format!(
        r#"{{
            "model": "llama3-8b-8192",
            "messages": [
                {{
                    "role": "system",
                    "content": "You are a useful digital assistant that creates linux bash commands. Output only the raw command and but no explainations or any other text or extra characters. Never output anything else that the command that the user can run to accomlish the prompt. Never wrap the commands in quotes!!!"
                }},
                {{
                    "role": "user",
                    "content": "{}"
                }}
            ],
            "temperature": 0,
            "max_tokens": 1024,
            "top_p": 1,
            "stop": null,
            "stream": false
        }}"#,
        prompt
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key:String;
    //run_shell_command("iwconfig | grep \"A\"".to_string());

    match env::var("GROQ_API_KEY"){
        Ok(v) => {api_key = v},
        Err(_) => {
            println!("GROQ_API_KEY not set");
            exit(-1);
        }
    }
    let prompt = env::args().skip(1).collect::<Vec<String>>().join(" ");
    if prompt == "" {
        println!("No prompt supplied");
        exit(-1);
    }


    let client = Client::new();
    let response = client.post("https://api.groq.com/openai/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(build_prompt_request(prompt))
        .send()
        .await?;


    let body = response.text().await?;
    let json: Value = serde_json::from_str(&body)?;
    let command = json["choices"][0]["message"]["content"].as_str().expect(&format!("Invalid response ({})", json));

    print!("Run '{}'?", command);
    stdout().flush().unwrap();

    let mut buffer:String = String::new();
    stdin().read_line(&mut buffer).expect("Could not read terminal input");

    if buffer.trim_end() == ""{
        println!("Executing... \n------------");
        run_shell_command(command.to_string());
    }else{
        println!("Cancelled!");
    }

    Ok(())
}
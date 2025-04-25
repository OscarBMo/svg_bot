// This is a simple svg logo generation agent. It gathers information from the terminal
// and generates an SVG file based on the input.
use rig::{
    agent::Agent,
    completion::{CompletionModel, Prompt},
    providers::openai,
};
use std::io::Write;

// Data model for the input data
#[derive(serde::Deserialize)]
struct InputItem {
    desired_output_filename: String,
    desired_svg_description: String,
    title: String,
}

// Function to process one logo request
async fn generate_svg_logo<M: CompletionModel>(llm_agent: Agent<M>, item: &InputItem) -> String {
    let svg_generation_prompt = include_str!("../prompts/svg_generation_prompt.md")
        .replace("{logo_title}", &item.title)
        .replace("{logo_description}", &item.desired_svg_description);

    // Generate SVG code
    llm_agent
        .prompt(svg_generation_prompt)
        .await
        .expect("Failed to generate SVG code")
}

fn save_svg_to_file(svg_code: String, filename: &str) {
    let svg_file_path = format!("./output/{}.svg", filename);
    let svg_file_path = std::path::Path::new(&svg_file_path);
    println!("Output SVG file path: {}", svg_file_path.display());

    println!("Saving SVG file...");
    if svg_file_path.exists() {
        println!("File already exists. Overwriting...");

        std::fs::write(svg_file_path, svg_code).expect("Unable to write file");
        println!("SVG file updated successfully.");
    } else {
        println!("Creating new file...");
        // Create the file using File::create
        let file = std::fs::File::create(svg_file_path).expect("Unable to create file");
        // Write the SVG content to the file
        let mut writer = std::io::BufWriter::new(file);
        writer
            .write_all(svg_code.as_bytes())
            .expect("Unable to write data to file");
        println!("SVG file created successfully.");
    }
}

#[tokio::main]
async fn main() {
    // Read the user input from ./input.json
    let input_file_path = "./input.json";
    let input_file_path = std::path::Path::new(input_file_path);
    let input_file = std::fs::File::open(input_file_path).expect("Unable to open file");
    let reader = std::io::BufReader::new(input_file);
    let input_data: Vec<InputItem> = serde_json::from_reader(reader).expect("Unable to parse JSON");

    // Initialize OpenAI client
    dotenvy::dotenv().ok();
    let client = openai::Client::from_env();

    // Generate SVG code
    for item in &input_data {
        let agent = client.agent(openai::GPT_4).build();
        let generated_svg = generate_svg_logo(agent, item).await;

        // Save the SVG code to a file
        save_svg_to_file(generated_svg, &item.desired_output_filename);
    }
}

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
async fn generate_logo_design<M: CompletionModel>(llm_agent: Agent<M>, item: &InputItem) -> String {
    let logo_design_prompt = include_str!("../prompts/logo_design_prompt.md")
        .replace("{logo_title}", &item.title)
        .replace("{logo_description}", &item.desired_svg_description);

    // Generate logo design
    llm_agent
        .prompt(logo_design_prompt)
        .await
        .expect("Failed to generate logo design")
}

async fn generate_svg_logo_from_design<M: CompletionModel>(
    llm_agent: Agent<M>,
    logo_design_instructions: &str,
) -> String {
    let svg_generation_prompt = include_str!("../prompts/svg_generation_from_design.md")
        .replace("{design_instructions}", logo_design_instructions);

    // Generate SVG code
    llm_agent
        .prompt(svg_generation_prompt)
        .await
        .expect("Failure on SVG code generation step")
}

async fn logo_review_and_correction_step<M: CompletionModel>(
    llm_agent: Agent<M>,
    logo_generated_code: &str,
) -> String {
    let svg_correction_prompt = include_str!("../prompts/logo_review_and_correction.md")
        .replace("{logo_generated_code}", logo_generated_code);

    // Generate SVG code
    llm_agent
        .prompt(svg_correction_prompt)
        .await
        .expect("Failure on SVG code review and correction step")
}

fn save_output_to_text_file(
    file_content: &str,
    stage_name: &str,
    extension: &str,
    desired_logo_filename: &str,
) {
    let output_folder = format!("./output/{}/", desired_logo_filename);
    // Create the output folder if it doesn't exist
    std::fs::create_dir_all(&output_folder).expect("Unable to create output folder");

    let file_path = format!("{}{}.{}", output_folder, stage_name, extension);
    let output_file_path = std::path::Path::new(&file_path);
    println!("Output file path: {}", output_file_path.display());

    println!("Saving text file...");
    if output_file_path.exists() {
        println!("File already exists. Overwriting...");

        std::fs::write(output_file_path, file_content).expect("Unable to write file");
        println!("Text file updated successfully.");
    } else {
        println!("Creating new file...");
        // Create the file using File::create
        let file = std::fs::File::create(output_file_path).expect("Unable to create file");
        // Write the SVG content to the file
        let mut writer = std::io::BufWriter::new(file);
        writer
            .write_all(file_content.as_bytes())
            .expect("Unable to write data to file");
        println!("Text file created successfully.");
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
        let design_agent = client.agent(openai::GPT_4).build();
        let logo_design = generate_logo_design(design_agent, item).await;
        save_output_to_text_file(
            &logo_design,
            "logo_design",
            "txt",
            &item.desired_output_filename,
        );

        let svg_generation_agent = client.agent(openai::GPT_4).build();
        let generated_svg = generate_svg_logo_from_design(svg_generation_agent, &logo_design).await;
        save_output_to_text_file(
            &generated_svg,
            "generated_svg",
            "svg",
            &item.desired_output_filename,
        );

        let svg_review_agent = client.agent(openai::GPT_4).build();
        let reviewed_svg = logo_review_and_correction_step(svg_review_agent, &generated_svg).await;
        save_output_to_text_file(
            &reviewed_svg,
            "reviewed_svg",
            "svg",
            &item.desired_output_filename,
        );
    }
}

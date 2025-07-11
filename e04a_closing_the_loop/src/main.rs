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

struct SvgLintResult {
    valid: bool,
    error_message: Option<String>,
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

struct PrevCodeWithFeedback {
    previous_svg_code: String,
    review_feedback: String,
}
async fn generate_svg_logo_from_design<M: CompletionModel>(
    llm_agent: Agent<M>,
    logo_design_instructions: &str,
    prev_feedback_with_code: Option<PrevCodeWithFeedback>,
) -> String {
    // Make sure that previous svg code is present if review feedback is provided
    // If this is not the case, we will panic
    let svg_generation_prompt = match prev_feedback_with_code {
        Some(prev_code_w_feedbck) => {
            // If review feedback is provided, use it to generate the SVG code with the corresponding
            // prompt
            include_str!("../prompts/svg_generation_from_design_including_feedback.md")
                .replace(
                    "{previous_svg_code}",
                    &prev_code_w_feedbck.previous_svg_code,
                )
                .replace("{design_instructions}", logo_design_instructions)
                .replace("{review_feedback}", &prev_code_w_feedbck.review_feedback)
        }
        None => {
            // if no review feedback is provided, that means it is the first iteration
            include_str!("../prompts/svg_generation_from_design.md")
                .replace("{design_instructions}", logo_design_instructions)
        }
    };

    // Generate SVG code
    llm_agent
        .prompt(svg_generation_prompt)
        .await
        .expect("Failure on SVG code generation step")
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
        // Make sure that the directory exists
        if let Some(parent) = output_file_path.parent() {
            // Only create the parent directory if it doesn't exist
            if !parent.exists() {
                println!("Parent directory does not exist. Creating...");
                std::fs::create_dir_all(parent).expect("Unable to create parent directory");
            }
        }
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

fn check_svg_file(file_path: &str) -> SvgLintResult {
    // Check if the file is correct by using `svglint` on the command line
    let output = std::process::Command::new("svglint")
        .arg(file_path)
        .output()
        .expect("Failed to execute command");

    // Print the output
    if output.status.success() {
        SvgLintResult {
            valid: true,
            error_message: None,
        }
    } else {
        let error_msg_stderr = String::from_utf8_lossy(&output.stderr);
        let error_msg_stout = String::from_utf8_lossy(&output.stdout);

        let error_message = format!(
            "Error from stdout: {}\nError from stderr: {}",
            error_msg_stout, error_msg_stderr
        );

        SvgLintResult {
            valid: false,
            error_message: Some(error_message),
        }
    }
}

async fn generation_loop(
    logo_design: &str,
    max_iter: usize,
    model_for_svg_generation: &str,
    client: &openai::Client,
    item: &InputItem,
) -> Option<String> {
    let mut prev_feedback_with_code = None;

    for i in 0..max_iter {
        // Generate SVG code
        let svg_generation_agent = client.agent(model_for_svg_generation).build();
        let generated_svg = generate_svg_logo_from_design(
            svg_generation_agent,
            logo_design,
            prev_feedback_with_code,
        )
        .await;
        let stage_name = format!("iter_{}/generated_svg", i);
        save_output_to_text_file(
            &generated_svg,
            &stage_name,
            "svg",
            &item.desired_output_filename,
        );

        // Check the generated SVG file
        let svg_lint_result = check_svg_file(&generated_svg);
        if svg_lint_result.valid {
            println!("Iteration {}: SVG file is valid.", i);
            return Some(generated_svg);
        } else {
            println!("Iteration {}: SVG file is invalid.", i);
            prev_feedback_with_code = Some(PrevCodeWithFeedback {
                previous_svg_code: generated_svg.clone(),
                review_feedback: svg_lint_result
                    .error_message
                    .unwrap_or_else(|| "No error message provided".to_string()),
            });
        }
    }

    println!("Maximum iterations reached. SVG generation failed.");
    None
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

        let model_for_svg_generation = openai::GPT_4;
        let max_iter = 5;
        let generated_svg = generation_loop(
            &logo_design,
            max_iter,
            model_for_svg_generation,
            &client,
            item,
        )
        .await;

        if let Some(svg_code) = generated_svg {
            // Save the final SVG code
            save_output_to_text_file(&svg_code, "final_svg", "svg", &item.desired_output_filename);
        } else {
            println!(
                "Failed to generate a valid SVG after {} iterations.",
                max_iter
            );
        }
    }
}

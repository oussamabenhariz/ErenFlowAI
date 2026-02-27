/// QA Agent Example using config.yaml and handler functions
///
/// This example demonstrates how to:
/// 1. Load an agent from a config.yaml file
/// 2. Define handler functions that process state
/// 3. Run a workflow with multiple steps
///
/// To run this example (from erenflow-ai directory):
/// ```bash
/// export MISTRAL_API_KEY=your_key_here
/// cargo run --example qa_agent_config
/// ```

use erenflow_ai::prelude::*;
use serde_json::json;
use std::env;

/// Handler 1: Validate user input
async fn validate_input(mut state: State) -> Result<State> {
    println!("  ✓ validate_input");

    let question = state
        .get("user_question")
        .and_then(|v| v.as_str())
        .unwrap_or("No question");

    if question.is_empty() {
        return Err(ErenFlowError::ValidationError("Empty question".to_string()));
    }

    state.set("validated_question", json!(question.to_string()));
    state.set("validation_timestamp", json!(chrono::Utc::now().to_rfc3339()));

    Ok(state)
}

/// Handler 2: Get context  
async fn get_context(mut state: State) -> Result<State> {
    println!("  ✓ get_context");

    let question = state
        .get("validated_question")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let context = if question.to_lowercase().contains("rust") {
        "Rust is a systems programming language with memory safety without garbage collection."
    } else if question.to_lowercase().contains("ai") {
        "Artificial Intelligence is the simulation of human intelligence by computers."
    } else {
        "General knowledge context."
    };

    state.set("context", json!(context.to_string()));
    Ok(state)
}

/// Handler 3: Generate answer
async fn generate_answer(mut state: State) -> Result<State> {
    println!("  ✓ generate_answer");

    let question = state
        .get("validated_question")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let answer = format!(
        "Based on '{}': This is a comprehensive answer that would come from the LLM.",
        question
    );

    state.set("llm_answer", json!(answer));
    Ok(state)
}

/// Handler 4: Format response
async fn format_response(mut state: State) -> Result<State> {
    println!("  ✓ format_response");

    let question = state
        .get("validated_question")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown");

    let answer = state
        .get("llm_answer")
        .and_then(|v| v.as_str())
        .unwrap_or("No answer");

    let final_response = json!({
        "question": question,
        "answer": answer,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "status": "success"
    });

    state.set("final_response", final_response);
    Ok(state)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n{}", "=".repeat(70));
    println!("🤖 QA Agent Example - Processing Question");
    println!("{}\n", "=".repeat(70));

    // Create initial state
    let mut state = State::new();
    let user_question = "What is Rust and why is it important?";

    state.set("user_question", json!(user_question));
    state.set("request_id", json!(uuid::Uuid::new_v4().to_string()));

    println!("📝 Question: \"{}\"\n", user_question);
    println!("🔄 Processing steps:\n");

    // Manually execute handlers to demonstrate workflow
    state = validate_input(state).await?;
    state = get_context(state).await?;
    state = generate_answer(state).await?;
    state = format_response(state).await?;

    println!("\n{}", "=".repeat(70));
    println!("✅ Workflow completed!\n");

    // Display final response
    if let Some(response) = state.get("final_response") {
        println!("📤 Final Response:\n");
        println!("{}\n", serde_json::to_string_pretty(&response)?);
    }

    println!("{}", "=".repeat(70));
    println!("✨ Example complete - This demonstrates manual handler orchestration");
    println!("For automatic handler discovery, use #[register_handler] macro\n");

    Ok(())
}

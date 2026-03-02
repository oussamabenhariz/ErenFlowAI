# Handler Development Guide

Write the custom functions that power your agent's workflow.

## Must Know

Handlers are async functions that take state in, do something, and return updated state.

```rust
pub async fn my_handler(mut state: State) -> Result<State> {
    // Read from state
    let input = state.get("input")?;
    
    // Do something
    let result = process(input);
    
    // Update state
    state.set("output", json!(result));
    
    Ok(state)
}
```

That's it - every handler follows this pattern.

## Simple Handler

```rust
pub async fn validate_input(mut state: State) -> Result<State> {
    let input = state.get_str("user_input")?;
    
    if input.is_empty() {
        state.set("valid", json!(false));
        state.set("error", json!("Input cannot be empty"));
    } else {
        state.set("valid", json!(true));
        state.set("cleaned_input", json!(input.trim()));
    }
    
    Ok(state)
}
```

## Using LLM in Handlers

```rust
pub async fn generate_response(mut state: State) -> Result<State> {
    let llm = state.get_llm_client()?;
    let input = state.get_str("input")?;
    
    // Call LLM
    let messages = vec![Message::new("user", input)];
    let response = llm.chat(messages).await?;
    
    state.set("response", json!(response));
    Ok(state)
}
```

## Using RAG in Handlers

```rust
pub async fn retrieve_context(mut state: State) -> Result<State> {
    let rag = state.get_rag_client()?;
    let query = state.get_str("user_query")?;
    
    // Retrieve documents
    let docs = rag.retrieve(query, 5).await?;
    
    // Format as context
    let context = docs.iter()
        .map(|d| d.content.clone())
        .collect::<Vec<_>>()
        .join("\n---\n");
    
    state.set("context", json!(context));
    state.set("docs", json!(docs));
    
    Ok(state)
}
```

## Using MCP Tools in Handlers

```rust
pub async fn search_web(mut state: State) -> Result<State> {
    let mcp = state.get_mcp_client()?;
    let query = state.get_str("query")?;
    
    // Execute tool
    let results = mcp.execute_tool(
        "web_search",
        json!({"query": query, "max_results": 5})
    ).await?;
    
    state.set("search_results", results);
    Ok(state)
}
```

## Error Handling

```rust
pub async fn safe_handler(mut state: State) -> Result<State> {
    // Option 1: Handle and continue
    let value = state.get("field").unwrap_or(json!(null));
    
    // Option 2: Propagate error
    let required = state.get_str("required_field")?;
    
    // Option 3: Recover with default
    let count = state.get_i64("count")
        .map_err(|_| anyhow!("Invalid count"))?;
    
    Ok(state)
}
```

## Complex Logic

```rust
pub async fn multi_step_handler(mut state: State) -> Result<State> {
    // Step 1: Validate
    let input = state.get_str("input")?;
    if input.is_empty() {
        return Err(anyhow!("Empty input"));
    }
    
    // Step 2: Process
    let processed = input.to_uppercase();
    state.set("step1_result", json!(processed));
    
    // Step 3: Enrich
    let llm = state.get_llm_client()?;
    let analysis = llm.chat(vec![
        Message::new("user", format!("Analyze: {}", processed))
    ]).await?;
    
    state.set("step2_result", json!(analysis));
    
    Ok(state)
}
```

## Async Operations

```rust
pub async fn concurrent_handler(mut state: State) -> Result<State> {
    let mcp = state.get_mcp_client()?;
    
    // Run multiple tools concurrently
    let (search_result, calc_result) = tokio::join!(
        mcp.execute_tool("web_search", json!({"query": "..."})),
        mcp.execute_tool("calculator", json!({"expr": "..."}))
    );
    
    state.set("search", search_result?);
    state.set("calc", calc_result?);
    
    Ok(state)
}
```

## Testing Handlers

```rust
#[tokio::test]
async fn test_my_handler() {
    let mut state = State::new();
    state.set("input", json!("test input"));
    
    let result = my_handler(state).await;
    
    assert!(result.is_ok());
    let final_state = result.unwrap();
    assert_eq!(
        final_state.get_str("output").unwrap(),
        "expected output"
    );
}
```

## Common Patterns

### Passthrough
```rust
pub async fn logging_handler(mut state: State) -> Result<State> {
    println!("Current state: {:#?}", state);
    Ok(state)  // No changes
}
```

### Transformation
```rust
pub async fn transform_handler(mut state: State) -> Result<State> {
    let input = state.get("data")?;
    let transformed = transform(input);
    state.set("data", transformed);
    Ok(state)
}
```

### Branching
```rust
pub async fn branch_handler(mut state: State) -> Result<State> {
    let value = state.get_i64("score")?;
    
    if value > 70 {
        state.set("path", json!("complex"));
    } else {
        state.set("path", json!("simple"));
    }
    
    Ok(state)
}
```

## Best Practices

1. One handler, one job
2. Use clear names (not cryptic abbreviations)
3. Document what state it reads and writes
4. Return errors, don't panic
5. Keep handlers lightweight and fast
6. Write unit tests

---

See [state/README.md](../state/README.md) for state management details.

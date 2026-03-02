# LLM Configuration Guide

Set up any language model provider you prefer - OpenAI, Anthropic, Mistral, Groq, Azure, or even run Ollama locally.

## Supported Providers

- OpenAI (GPT-4, GPT-3.5-turbo)
- Anthropic (Claude)
- Mistral
- Groq
- Azure OpenAI
- Ollama (local models)

## Quick Setup

### OpenAI

```yaml
llm:
  provider: openai
  model: gpt-4
  api_key: ${OPENAI_API_KEY}
  temperature: 0.7
  max_tokens: 2000
```

Set environment variable:
```bash
export OPENAI_API_KEY="sk-..."
```

### Anthropic (Claude)

```yaml
llm:
  provider: anthropic
  model: claude-3-opus-20240229
  api_key: ${ANTHROPIC_API_KEY}
  temperature: 0.7
```

### Mistral

```yaml
llm:
  provider: mistral
  model: mistral-large
  api_key: ${MISTRAL_API_KEY}
  temperature: 0.7
```

### Groq (Fast inference)

```yaml
llm:
  provider: groq
  model: mixtral-8x7b-32768
  api_key: ${GROQ_API_KEY}
```

### Azure OpenAI

```yaml
llm:
  provider: azure
  model: gpt-4
  api_key: ${AZURE_API_KEY}
  endpoint: ${AZURE_ENDPOINT}
  deployment: my-deployment
```

### Ollama (Local)

```yaml
llm:
  provider: ollama
  model: mistral
  base_url: http://localhost:11434
```

## Fallback Providers

Automatically try alternative providers if the main one fails:

```yaml
llm:
  provider: openai
  model: gpt-4
  api_key: ${OPENAI_API_KEY}
  fallbacks:
    - provider: anthropic
      model: claude-3-opus-20240229
      api_key: ${ANTHROPIC_API_KEY}
    - provider: mistral
      model: mistral-large
      api_key: ${MISTRAL_API_KEY}
```

Flow:
1. Try OpenAI GPT-4
2. If fails → try Claude
3. If fails → try Mistral
4. If all fail → error

## Parameters Explained

**temperature** - Controls randomness in responses
- 0.0: Deterministic (always the same answer)
- 0.7: Balanced (recommended)
- 1.0: Creative (lots of variation)

**max_tokens** - Maximum response length (default: 2000)

**timeout** - Request timeout in seconds (default: 30)

**top_p** - Nucleus sampling (0.0 - 1.0)

## Usage in Code

```rust
let llm = state.get_llm_client()?;
let response = llm.chat(messages).await?;
```

## Best Practices

1. Always use environment variables for API keys
2. Pick the right temperature for your task
3. Add fallback providers for reliability
4. Monitor token usage and costs
5. Test with Ollama locally before going live

---

See [configuration/CONFIG_GUIDE.md](../configuration/CONFIG_GUIDE.md) for complete reference.

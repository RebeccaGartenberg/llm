# llm
Online LLM

##### Run API:
cargo run

##### Examples of using the /chat/completions endpoint:

curl -X POST http://localhost:8000/chat/completions \
-d '{"role": "system", "content": "you are a helpful assistant"}'

curl -X POST http://localhost:8000/chat/completions \
-d '{"role": "user", "content": "why is the sky blue? limit to 10 words."}'

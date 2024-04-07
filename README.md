# llm
Online LLM

##### Run API:
cargo run

##### Example of using the /chat/completions endpoint:

curl http://localhost:8000/chat/completions -X POST -d '{"input": "what color is the sky? Limit to 10 words.}'

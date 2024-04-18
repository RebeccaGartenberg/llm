# llm
Online LLM

#### Set up Environment

In the project directory assign the following environment variables:

export OPENAI_API_BASE=https://api.openai.com/v1

export OPENAI_API_KEY=sk-xxxxxxx (your OpenAI API key)

(Make sure they are available in both the root directory to be accessed by test.py and in rust_rest_api directory to be accessed by the API)

##### Run API (using binary executable):

./target/release/rust_rest_api

(You may need to give permissions on your local machine to run this file)

#### Run API (using cargo)

With cargo package manager set up the binary executable file can be updated using:

cargo build --release

The API can be also be run using:

cargo run

##### Test the /chat/completions endpoint:

Install the requirements in python_requirements.txt

Run: python3 test.py

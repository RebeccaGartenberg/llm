from openai import OpenAI
import os

client = OpenAI(base_url='http://localhost:8000/', api_key=os.environ['OPENAI_API_KEY'])

completion = client.chat.completions.create(
    model="gpt-4-turbo-preview",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "What is the top news about Gen AI today? "},
    ],
)

# print(completion.choices[0].message)

from flask import Flask, request, jsonify
import whisper
from openai import OpenAI
import os

app = Flask(__name__)
model = whisper.load_model("base")
openai = OpenAI(api_key=os.environ.get("OPENAI_API_KEY"))

# noinspection PyArgumentList
@app.route("/transcribe", methods=["POST"])
def transcribe_audio():
    audio_file = request.files["file"]
    path = f"/tmp/{audio_file.filename}"
    audio_file.save(path)
    result = model.transcribe(audio=path, fp16=False)
    return jsonify(result)

@app.route("/prompt", methods=["POST"])
def prompt_ai():
    system_prompt = request.json["system_prompt"]
    user_prompt = request.json["user_prompt"]
    response = openai.chat.completions.create(
        model="gpt-4o-mini",
        messages=[
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ]
    )

    return jsonify({
        "content": response.choices[0].message.content
    })


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5010)
from flask import Flask, request, jsonify
import whisper

app = Flask(__name__)
model = whisper.load_model("base")


# noinspection PyArgumentList
@app.route("/transcribe", methods=["POST"])
def transcribe_audio():
    audio_file = request.files["file"]
    path = f"/tmp/{audio_file.filename}"
    audio_file.save(path)
    result = model.transcribe(audio=path, fp16=False)
    return jsonify(result)




if __name__ == "__main__":
    app.run(host="0.0.0.0", port=5010)


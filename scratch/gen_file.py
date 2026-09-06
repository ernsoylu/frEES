#!/usr/bin/env python3
import sys
import os
import re
import json
import urllib.request

def detect_lang(prompt_text, target_file):
    m = re.search(r'```([a-zA-Z0-9_\-\+]+)', prompt_text)
    if m:
        return m.group(1)
    ext = os.path.splitext(target_file)[1].lower()
    mapping = {
        '.rs': 'rust',
        '.ts': 'typescript',
        '.tsx': 'tsx',
        '.js': 'javascript',
        '.jsx': 'jsx',
        '.py': 'python',
        '.json': 'json',
        '.toml': 'toml',
        '.md': 'markdown',
        '.sh': 'bash',
        '.yml': 'yaml',
        '.yaml': 'yaml',
    }
    return mapping.get(ext, 'text')

def main():
    if len(sys.argv) < 3:
        print("Usage: python3 gen_file.py <prompt_file> <target_file>")
        sys.exit(1)

    prompt_file = sys.argv[1]
    target_file = sys.argv[2]

    with open(prompt_file, 'r', encoding='utf-8') as f:
        prompt_content = f.read().strip()

    lang = detect_lang(prompt_content, target_file)

    # Construct raw ChatML prompt pre-closing thinking tags and opening the language code fence
    # so generation starts immediately on line 1
    raw_prompt = (
        f"<|im_start|>system\nYou are an expert {lang} engineer.<|im_end|>\n"
        f"<|im_start|>user\n{prompt_content}<|im_end|>\n"
        f"<|im_start|>assistant\n<think>\n</think>\n```{lang}\n"
    )

    payload = {
        "model": "qwopus3.5-coder:4b",
        "prompt": raw_prompt,
        "raw": True,
        "stream": False,
        "options": {
            "temperature": 0.1,
            "repeat_penalty": 1.15,
            "num_predict": 4096,
            "num_ctx": 16384
        }
    }

    req = urllib.request.Request(
        "http://192.168.1.104:11434/api/generate",
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"}
    )

    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except Exception as e:
        print(f"Error calling Ollama API: {e}", file=sys.stderr)
        sys.exit(1)

    response_text = data.get("response", "")

    # Clean up response: strip trailing markdown fence or tags
    end_idx = response_text.find("```")
    if end_idx != -1:
        clean_code = response_text[:end_idx]
    else:
        end_tag = response_text.find("<|im_end|>")
        if end_tag != -1:
            clean_code = response_text[:end_tag]
        else:
            clean_code = response_text

    parent_dir = os.path.dirname(os.path.abspath(target_file))
    if parent_dir:
        os.makedirs(parent_dir, exist_ok=True)
    with open(target_file, 'w', encoding='utf-8') as f:
        f.write(clean_code.rstrip() + "\n")

    eval_count = data.get("eval_count", 0)
    eval_duration_ns = data.get("eval_duration", 0)
    total_duration_ns = data.get("total_duration", 0)
    prompt_eval_count = data.get("prompt_eval_count", 0)

    speed = (eval_count / (eval_duration_ns / 1e9)) if eval_duration_ns > 0 else 0.0
    resp_time = total_duration_ns / 1e9

    telemetry = f"[Ollama Stats] Tokens: {eval_count} | Speed: {speed:.1f} tok/s | Response Time: {resp_time:.1f}s | Prompt: {prompt_eval_count} tokens"
    print(telemetry)

if __name__ == "__main__":
    main()

"""
木桃 Mutao - Python AI 手术刀

职责：
  1. 接收用户上传的图片 + 描述文本
  2. 调用 LLM API 提取标签 (tags) 和价值梯度 (value_tier)
  3. 返回结构化 JSON 供 Rust 引擎消费

用法：
  python scalpel.py --image path/to/img.jpg --description "用了两年的机械键盘"
"""

import argparse
import json
import os
import sys

LLM_API_KEY = os.environ.get("MUTAO_LLM_KEY", "")
LLM_ENDPOINT = os.environ.get(
    "MUTAO_LLM_ENDPOINT", "https://api.openai.com/v1/chat/completions"
)
LLM_MODEL = os.environ.get("MUTAO_LLM_MODEL", "gpt-4o-mini")


def extract_tags_local(description: str) -> dict:
    """离线规则引擎：不用 LLM 时的降级方案"""
    desc = description.lower()
    tag_map = {
        "键盘": "机械键盘",
        "书": "书籍",
        "小说": "小说",
        "耳机": "耳机",
        "音箱": "音箱",
        "杯子": "杯子",
        "相机": "相机",
        "背包": "背包",
        "衣服": "衣服",
        "桌子": "家具",
    }
    found_tags = [tag for kw, tag in tag_map.items() if kw in desc]
    if not found_tags:
        found_tags = ["其他"]

    high = ["相机", "镜头", "电脑", "ipad", "switch", "游戏机"]
    mid = ["键盘", "耳机", "音箱", "背包", "kindle"]
    value_tier = 1
    if any(kw in desc for kw in high):
        value_tier = 4
    elif any(kw in desc for kw in mid):
        value_tier = 3

    return {"tags": found_tags, "value_tier": value_tier, "method": "local_rule"}


def extract_tags_llm(description: str, image_path: str = "") -> dict:
    """调用 LLM 提取结构化标签"""
    import urllib.request

    messages = [
        {
            "role": "system",
            "content": (
                "你是一个闲置物品分析助手。"
                "分析用户的物品描述，返回 JSON 格式："
                '{"tags": ["标签1", "标签2"], "value_tier": 1-5}。'
                "value_tier: 1=低值日用, 2=普通, 3=中等, 4=高值, 5=贵重。"
                "tags 使用中文，每个标签2-4个字。"
            ),
        },
        {"role": "user", "content": f"物品描述：{description}"},
    ]

    body = json.dumps(
        {
            "model": LLM_MODEL,
            "messages": messages,
            "temperature": 0.1,
            "max_tokens": 256,
        }
    ).encode()

    req = urllib.request.Request(
        LLM_ENDPOINT,
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {LLM_API_KEY}",
        },
        method="POST",
    )

    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            result = json.loads(resp.read())
            content = result["choices"][0]["message"]["content"]
            parsed = json.loads(content)
            return {
                "tags": parsed.get("tags", ["其他"]),
                "value_tier": parsed.get("value_tier", 1),
                "method": "llm",
            }
    except Exception as e:
        print(f"LLM 调用失败，降级到规则引擎: {e}", file=sys.stderr)
        return extract_tags_local(description)


def main():
    parser = argparse.ArgumentParser(description="木桃 AI 手术刀")
    parser.add_argument("--image", default="", help="物品图片路径")
    parser.add_argument("--description", default="", help="物品描述文本")
    parser.add_argument("--llm", action="store_true", help="使用 LLM 提取")
    args = parser.parse_args()

    if not args.description:
        print(json.dumps({"error": "需要 --description 参数"}, ensure_ascii=False))
        sys.exit(1)

    if args.llm and LLM_API_KEY:
        result = extract_tags_llm(args.description, args.image)
    else:
        result = extract_tags_local(args.description)

    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()

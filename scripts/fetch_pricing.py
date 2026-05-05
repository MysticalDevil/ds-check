#!/usr/bin/env python3
"""Fetch DeepSeek pricing from official docs and cache it locally.

Usage:
    python3 scripts/fetch_pricing.py          # Write to $XDG_CACHE_HOME/ds-check/pricing.json
    python3 scripts/fetch_pricing.py -o path  # Write to specific path
    python3 scripts/fetch_pricing.py -c       # Check if cached data is up-to-date
"""

import argparse
import json
import os
import re
import sys
import urllib.request
from html.parser import HTMLParser

URL = "https://api-docs.deepseek.com/zh-cn/quick_start/pricing"


def default_cache_path() -> str:
    """Return the default pricing.json path under XDG_CACHE_HOME."""
    cache_home = os.environ.get("XDG_CACHE_HOME")
    if not cache_home:
        cache_home = os.path.expanduser("~/.cache")
    return os.path.join(cache_home, "ds-check", "pricing.json")


class PricingTableParser(HTMLParser):
    """Extract text cells from the pricing <table>, ignoring <del> strikethrough."""

    def __init__(self):
        super().__init__()
        self.in_table = False
        self.in_td = False
        self.in_del = False
        self.current_row = []
        self.current_cell = ""
        self.table_data = []

    def handle_starttag(self, tag, _attrs):
        if tag == "table":
            self.in_table = True
        elif tag == "tr" and self.in_table:
            self.current_row = []
        elif tag == "td" and self.in_table:
            self.in_td = True
            self.current_cell = ""
        elif tag == "del":
            self.in_del = True

    def handle_endtag(self, tag):
        if tag == "table":
            self.in_table = False
        elif tag == "tr" and self.in_table:
            if self.current_row:
                self.table_data.append(self.current_row)
        elif tag == "td" and self.in_table:
            self.in_td = False
            self.current_row.append(self.current_cell.strip())
        elif tag == "del":
            self.in_del = False

    def handle_data(self, data):
        if self.in_td and self.in_table and not self.in_del:
            self.current_cell += data


def extract_models(table_data):
    """Parse table rows into model pricing objects."""
    models = []
    model_names = []

    for row in table_data:
        if not row:
            continue

        # Model name row: first cell is "模型"
        if row[0] == "模型" and len(row) >= 3:
            for cell in row[1:]:
                name = re.sub(r"\(\d+\)", "", cell).strip()
                if name and "deepseek-" in name:
                    model_names.append(name)
            continue

        # Price rows
        if len(row) >= 4 and row[0] == "价格":
            label = row[1]
            prices = row[2:]
        elif len(row) >= 3 and "百万tokens" in row[0]:
            label = row[0]
            prices = row[1:]
        else:
            continue

        for i, price_str in enumerate(prices):
            if i >= len(models):
                models.append({})

            # Remove Chinese parentheses and footnote markers
            cleaned = re.sub(r"（[^）]*）", "", price_str)
            cleaned = re.sub(r"\(\d+\)", "", cleaned)

            match = re.search(r"(\d+\.?\d*)\s*元", cleaned)
            if match:
                price = match.group(1)
                if "缓存命中" in label:
                    models[i]["input_cache_hit"] = price
                elif "缓存未命中" in label:
                    models[i]["input_cache_miss"] = price
                elif "输出" in label:
                    models[i]["output"] = price

    result = []
    for i, name in enumerate(model_names):
        if i < len(models) and models[i]:
            result.append({
                "model": name,
                "input_cache_hit": models[i].get("input_cache_hit", ""),
                "input_cache_miss": models[i].get("input_cache_miss", ""),
                "output": models[i].get("output", ""),
            })

    return result


def clean_text(text):
    """Remove null bytes and other control characters from text."""
    return text.replace("\x00", "").replace("\x01", "").replace("\x02", "").strip()


def extract_note(html_text):
    """Extract discount footnote (3) text."""
    match = re.search(r"\(3\)\s*<strong>(.*?)</strong>", html_text, re.DOTALL)
    if match:
        note = match.group(1)
        note = re.sub(r"<[^>]+>", "", note)
        note = note.replace("当前 ", "")
        return clean_text(note)
    return ""


def fetch_pricing():
    """Download the pricing page and parse it."""
    req = urllib.request.Request(
        URL,
        headers={
            "User-Agent": "Mozilla/5.0 (compatible; ds-check pricing fetcher)",
        },
    )

    with urllib.request.urlopen(req, timeout=30) as response:
        html = response.read().decode("utf-8")

    parser = PricingTableParser()
    parser.feed(html)

    models = extract_models(parser.table_data)
    note = extract_note(html)

    if not models:
        print("ERROR: Could not extract pricing data from page", file=sys.stderr)
        sys.exit(1)

    return {
        "currency": "CNY",
        "unit": "per 1M tokens",
        "note": note,
        "models": models,
    }


def main():
    parser = argparse.ArgumentParser(description="Fetch DeepSeek pricing from official docs")
    parser.add_argument(
        "-o", "--output",
        help="Output file path (default: $XDG_CACHE_HOME/ds-check/pricing.json)",
    )
    parser.add_argument(
        "-c", "--check",
        action="store_true",
        help="Check if cached data differs from the live page",
    )
    args = parser.parse_args()

    data = fetch_pricing()
    output = json.dumps(data, indent=2, ensure_ascii=False) + "\n"

    out_path = args.output or default_cache_path()

    if args.check:
        try:
            with open(out_path, "r", encoding="utf-8") as f:
                existing = f.read()
            if existing == output:
                print(f"Pricing data is up-to-date ({out_path}).", file=sys.stderr)
                sys.exit(0)
            else:
                print(f"Pricing data has changed! ({out_path})", file=sys.stderr)
                sys.exit(1)
        except FileNotFoundError:
            print(f"No cached pricing found at {out_path}", file=sys.stderr)
            sys.exit(1)

    # Ensure parent directory exists
    parent = os.path.dirname(out_path)
    if parent:
        os.makedirs(parent, exist_ok=True)

    with open(out_path, "w", encoding="utf-8") as f:
        f.write(output)
    print(f"Pricing written to {out_path}", file=sys.stderr)


if __name__ == "__main__":
    main()

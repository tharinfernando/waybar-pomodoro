#!/usr/bin/env python3
"""Patches or unpatches the Waybar config and style files."""

import os
import re
import sys

ACTION = sys.argv[1] if len(sys.argv) > 1 else "install"

HOME = os.path.expanduser("~")
CONFIG = os.path.join(HOME, ".config/waybar/config.jsonc")
STYLE  = os.path.join(HOME, ".config/waybar/style.css")

SCRIPT_DIR  = os.path.dirname(os.path.abspath(__file__))
PROJECT_DIR = os.path.dirname(SCRIPT_DIR)

BEGIN_JSONC = "// waybar-pomodoro-begin"
END_JSONC   = "// waybar-pomodoro-end"
BEGIN_CSS   = "/* waybar-pomodoro-begin */"
END_CSS     = "/* waybar-pomodoro-end */"


def read(path):
    with open(path) as f:
        return f.read()


def write(path, content):
    with open(path, "w") as f:
        f.write(content)


def remove_marked_section(text, begin, end):
    pattern = re.compile(
        r"\n?" + r"[ \t]*" + re.escape(begin) + r".*?" + re.escape(end) + r"[ \t]*\n?",
        re.DOTALL,
    )
    return pattern.sub("", text)


def install():
    # ── config.jsonc ──────────────────────────────────────────────────────────
    config = read(CONFIG)
    if BEGIN_JSONC in config:
        print("  config: already patched")
    else:
        module = read(os.path.join(PROJECT_DIR, "config/waybar-module.jsonc"))
        # Add "custom/pomodoro" to modules-center
        config = re.sub(
            r'("modules-center"\s*:\s*\[[^\]]*)\]',
            lambda m: m.group(0)[:-1] + ', "custom/pomodoro"]',
            config,
        )
        # Insert module block before "custom/voxtype"
        config = config.replace(
            '  "custom/voxtype":',
            module.rstrip("\n") + '\n  "custom/voxtype":',
            1,
        )
        write(CONFIG, config)
        print("  config: patched")

    # ── style.css ─────────────────────────────────────────────────────────────
    style = read(STYLE)
    if BEGIN_CSS in style:
        print("  style:  already patched")
    else:
        snippet = read(os.path.join(PROJECT_DIR, "config/waybar-style.css"))
        write(STYLE, style.rstrip("\n") + "\n\n" + snippet)
        print("  style:  patched")


def uninstall():
    # ── config.jsonc ──────────────────────────────────────────────────────────
    config = read(CONFIG)
    config = remove_marked_section(config, BEGIN_JSONC, END_JSONC)
    config = re.sub(r',\s*"custom/pomodoro"', "", config)
    write(CONFIG, config)
    print("  config: unpatched")

    # ── style.css ─────────────────────────────────────────────────────────────
    style = read(STYLE)
    style = remove_marked_section(style, BEGIN_CSS, END_CSS)
    write(STYLE, style.rstrip("\n") + "\n")
    print("  style:  unpatched")


if ACTION == "install":
    install()
elif ACTION == "uninstall":
    uninstall()
else:
    print(f"usage: {sys.argv[0]} [install|uninstall]", file=sys.stderr)
    sys.exit(1)

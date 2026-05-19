from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    if new in text:
        return
    if old not in text:
        raise RuntimeError(f"Expected text not found in {path}: {old}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="")


def main() -> None:
    replace_once(
        ROOT / "src" / "config.cpp",
        "body->haptics_gain < 1.0f",
        "body->haptics_gain < 0.1f",
    )
    replace_once(
        ROOT / "src" / "config.h",
        "float haptics_gain; // [1.0,2.0]",
        "float haptics_gain; // [0.1,2.0]",
    )


if __name__ == "__main__":
    main()

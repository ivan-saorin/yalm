#!/usr/bin/env python3
"""
Closure checker for YALM dictionaries.

Verifies that every word in every definition and example is itself defined
in the dictionary (or is a recognized inflection/exception).

Usage:
    python closure_check.py dictionaries/dict18.md
    python closure_check.py dictionaries/grammar18.md --against dictionaries/dict18.md
"""

import re
import sys
import argparse
from collections import defaultdict


# --- Exception lists ---

PRONOUNS = {
    "i", "me", "myself", "she", "he", "her", "his", "him",
    "we", "they", "them", "their", "yourself", "your",
    "ourselves", "themselves", "its", "those", "these",
    "one's",
}

NUMBER_WORDS = {
    "zero", "one", "two", "three", "four", "five", "six", "seven",
    "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen",
    "fifteen", "sixteen", "seventeen", "eighteen", "nineteen", "twenty",
    "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    "hundred", "thousand", "million", "billion",
}

COMPOUND_WORDS = {
    "everyone", "everything", "everywhere", "anywhere", "someone",
    "something", "somehow", "sometime", "sometimes", "somewhere",
    "nobody", "nothing", "nowhere", "whenever", "whatever",
    "whoever", "either", "neither", "anyone", "everyone's",
    "onto", "overall", "underground", "waterproof", "inborn",
}

OTHER_EXCEPTIONS = {
    "my", "fully", "forth", "ever", "slightly", "b", "x",
    "person's", "bird's", "herself", "himself",
    "million", "millions", "billion", "billions",
    "then", "never", "than",
}

EXCEPTIONS = PRONOUNS | NUMBER_WORDS | COMPOUND_WORDS | OTHER_EXCEPTIONS


# --- Tokenizer (mirrors Rust tokenize function) ---

SPLIT_CHARS = set(' \t\n\r.,?!";:()' + '\u201c\u201d')

def tokenize(text):
    """Lowercase, split on whitespace/punctuation, strip non-alphanumeric."""
    text = text.lower()
    # Split on whitespace and punctuation (including smart quotes)
    tokens = []
    current = []
    for ch in text:
        if ch in SPLIT_CHARS:
            if current:
                tokens.append(''.join(current))
                current = []
        else:
            current.append(ch)
    if current:
        tokens.append(''.join(current))
    # Strip non-alphanumeric from edges
    result = []
    for t in tokens:
        t = re.sub(r'^[^a-z0-9]+', '', t)
        t = re.sub(r'[^a-z0-9]+$', '', t)
        if t:
            result.append(t)
    return result


# --- Stemmer (mirrors Rust stem_to_entry function) ---

SUFFIXES = ["iest", "ier", "ing", "est", "er", "ly", "es", "ed", "s"]

# Common irregular forms that the suffix stripper can't handle
IRREGULAR_FORMS = {
    "wolves": "wolf", "knives": "knife", "lives": "life", "wives": "wife",
    "halves": "half", "selves": "self", "leaves": "leaf", "thieves": "thief",
    "children": "child", "men": "man", "women": "woman", "people": "person",
    "feet": "foot", "teeth": "tooth", "mice": "mouse", "geese": "goose",
    "oxen": "ox", "lice": "louse",
    "lying": "lie", "dying": "die", "tying": "tie",
    "was": "be", "were": "be", "been": "be", "am": "be", "are": "be",
    "had": "have", "has": "have",
    "did": "do", "does": "do", "done": "do",
    "went": "go", "gone": "go", "goes": "go",
    "said": "say", "says": "say",
    "made": "make", "makes": "make",
    "took": "take", "taken": "take",
    "gave": "give", "given": "give",
    "came": "come", "comes": "come",
    "knew": "know", "known": "know",
    "thought": "think", "thinks": "think",
    "felt": "feel", "feels": "feel",
    "found": "find", "finds": "find",
    "told": "tell", "tells": "tell",
    "got": "get", "gets": "get", "gotten": "get",
    "became": "become", "becomes": "become",
    "began": "begin", "begun": "begin",
    "ran": "run", "runs": "run",
    "wrote": "write", "written": "write",
    "spoke": "speak", "spoken": "speak",
    "broke": "break", "broken": "break",
    "chose": "choose", "chosen": "choose",
    "drove": "drive", "driven": "drive",
    "ate": "eat", "eaten": "eat",
    "fell": "fall", "fallen": "fall",
    "grew": "grow", "grown": "grow",
    "held": "hold", "holds": "hold",
    "kept": "keep", "keeps": "keep",
    "led": "lead", "leads": "lead",
    "left": "leave", "leaves": "leave",
    "lost": "lose", "loses": "lose",
    "meant": "mean", "means": "mean",
    "met": "meet", "meets": "meet",
    "paid": "pay", "pays": "pay",
    "put": "put", "puts": "put",
    "read": "read", "reads": "read",
    "rose": "rise", "risen": "rise",
    "sat": "sit", "sits": "sit",
    "sent": "send", "sends": "send",
    "set": "set", "sets": "set",
    "shook": "shake", "shaken": "shake",
    "shot": "shoot", "shoots": "shoot",
    "showed": "show", "shown": "show",
    "shut": "shut", "shuts": "shut",
    "sang": "sing", "sung": "sing",
    "slept": "sleep", "sleeps": "sleep",
    "stood": "stand", "stands": "stand",
    "stole": "steal", "stolen": "steal",
    "struck": "strike", "strikes": "strike",
    "swam": "swim", "swum": "swim",
    "taught": "teach", "teaches": "teach",
    "threw": "throw", "thrown": "throw",
    "understood": "understand",
    "woke": "wake", "woken": "wake",
    "won": "win", "wins": "win",
    "wore": "wear", "worn": "wear",
    "built": "build", "builds": "build",
    "bought": "buy", "buys": "buy",
    "caught": "catch", "catches": "catch",
    "cut": "cut", "cuts": "cut",
    "dealt": "deal", "deals": "deal",
    "drew": "draw", "drawn": "draw",
    "drank": "drink", "drunk": "drink",
    "fought": "fight", "fights": "fight",
    "flew": "fly", "flown": "fly",
    "forgave": "forgive", "forgiven": "forgive",
    "froze": "freeze", "frozen": "freeze",
    "hung": "hang", "hangs": "hang",
    "hid": "hide", "hidden": "hide",
    "hit": "hit", "hits": "hit",
    "hurt": "hurt", "hurts": "hurt",
    "lay": "lie", "lain": "lie",
    "laid": "lay",
    "lit": "light", "lights": "light",
    "rid": "rid", "rids": "rid",
    "rode": "ride", "ridden": "ride",
    "rang": "ring", "rung": "ring",
    "saw": "see", "seen": "see",
    "sought": "seek", "seeks": "seek",
    "sold": "sell", "sells": "sell",
    "sank": "sink", "sunk": "sink",
    "spent": "spend", "spends": "spend",
    "split": "split", "splits": "split",
    "spread": "spread", "spreads": "spread",
    "stuck": "stick", "sticks": "stick",
    "swept": "sweep", "sweeps": "sweep",
    "swore": "swear", "sworn": "swear",
    "tore": "tear", "torn": "tear",
    "wove": "weave", "woven": "weave",
    "wound": "wind", "winds": "wind",
    "brought": "bring", "brings": "bring",
    "sped": "speed", "speeds": "speed",
    "died": "die", "dies": "die", "dying": "die",
    "killed": "kill", "kills": "kill",
    "adjusted": "adjust", "adjusts": "adjust",
    "imposed": "impose", "imposes": "impose",
    "inspired": "inspire", "inspires": "inspire",
    "convinced": "convince", "convinces": "convince",
    "defended": "defend", "defends": "defend",
    "destroyed": "destroy", "destroys": "destroy",
    "elected": "elect", "elects": "elect",
    "examined": "examine", "examines": "examine",
    "expanded": "expand", "expands": "expand",
    "welcomed": "welcome", "welcomes": "welcome",
    "donated": "donate", "donates": "donate",
    "released": "release", "releases": "release",
    "recorded": "record", "records": "record",
    "appeared": "appear", "appears": "appear",
    "argued": "argue", "argues": "argue",
    "assembled": "assemble", "assembles": "assemble",
    "attended": "attend", "attends": "attend",
    "carved": "carve", "carves": "carve",
    "saved": "save", "saves": "save",
    "scored": "score", "scores": "score",
    "varied": "vary", "varies": "vary",
    "suited": "suit", "suits": "suit",
    "tended": "tend", "tends": "tend",
    "waited": "wait", "waits": "wait",
    "prepared": "prepare", "prepares": "prepare",
    "pleased": "please", "pleases": "please",
    "pressed": "press", "presses": "press",
    "updated": "update", "updates": "update",
    "approved": "approve", "approves": "approve",
    "defined": "define", "defines": "define",
    "injured": "injure", "injures": "injure",
    "blew": "blow", "blown": "blow", "blows": "blow",
    "dug": "dig", "digs": "dig",
    "beats": "beat", "beat": "beat", "beaten": "beat",
    "shone": "shine", "shines": "shine",
    "woke": "wake", "woken": "wake",
    "fed": "feed", "feeds": "feed",
    "laid": "lay", "lays": "lay",
    "cried": "cry", "cries": "cry",
    "forbade": "forbid", "forbidden": "forbid",
    "sailed": "sail", "sails": "sail",
    "marched": "march", "marches": "march",
    "served": "serve", "serves": "serve",
    "deserved": "deserve", "deserves": "deserve",
    "escaped": "escape", "escapes": "escape",
    "painted": "paint", "paintings": "paint",
    "washed": "wash", "washes": "wash",
    "shot": "shoot", "shots": "shoot",
}

def stem_to_entry(token, entry_set):
    """Try to reduce an inflected token to its base entry word."""
    lower = token.lower()

    # Direct match
    if lower in entry_set:
        return lower

    # Special case: "an" -> "a"
    if lower == "an":
        return "a"

    # Check irregular forms
    if lower in IRREGULAR_FORMS:
        base = IRREGULAR_FORMS[lower]
        if base in entry_set:
            return base

    # Hyphenated words: check if all parts are defined
    if '-' in lower:
        parts = lower.split('-')
        if all(stem_to_entry(p, entry_set) is not None for p in parts if p):
            return lower  # All parts resolve

    # Try removing common suffixes
    for suffix in SUFFIXES:
        if lower.endswith(suffix) and len(lower) > len(suffix):
            stem = lower[:-len(suffix)]
            if stem in entry_set:
                return stem
            # e-restoration: "liv" -> "live"
            with_e = stem + "e"
            if with_e in entry_set:
                return with_e
            # Consonant doubling: "bigg" -> "big"
            if len(stem) >= 2 and stem[-1] == stem[-2]:
                undoubled = stem[:-1]
                if undoubled and undoubled in entry_set:
                    return undoubled
            # y->i transformation: "happi" -> "happy"
            if stem.endswith("i"):
                with_y = stem[:-1] + "y"
                if with_y in entry_set:
                    return with_y

    # Try "un-" prefix: "unfair" -> "fair", "uncommon" -> "common"
    if lower.startswith("un") and len(lower) > 3:
        without_un = lower[2:]
        if without_un in entry_set:
            return without_un

    # Try derivational suffixes (only check if base is in entry_set)
    DERIVATIONAL = [
        ("tion", "te"),    # production -> produce (produc + te)
        ("tion", "t"),     # destruction -> destruct -> destroy? No, too complex
        ("ation", "e"),    # observation -> observe (observ + e)
        ("ation", ""),     # information -> inform (inform + "")
        ("ment", ""),      # judgment -> judge? No. agreement -> agree
        ("ness", ""),      # fairness -> fair, awareness -> aware
        ("ness", "e"),     # closeness -> close
        ("ful", ""),       # painful -> pain, plentiful -> plenti? No
        ("less", ""),      # waterproof handled by hyphen. homeless -> home
        ("able", ""),      # acceptable -> accept
        ("able", "e"),     # valuable -> value (valu + e)
        ("ible", ""),      # sensible -> sense? No
        ("ous", ""),       # violent -> violence? No. dangerous -> danger
        ("ive", ""),       # effective -> effect
        ("ive", "e"),      # productive -> produce (productiv + e? No)
        ("al", ""),        # traditional -> tradition, emotional -> emotion
        ("al", "e"),       # central -> centre? Not quite
        ("ity", ""),       # electricity -> electric? No suffix match
        ("ence", ""),      # violence -> violent? No. excellence -> excell?
        ("ance", ""),      # performance -> perform? (perform + ance)
        ("ion", ""),       # discussion -> discuss (discuss + ion)
        ("ion", "e"),      # destruction -> destructe? No
    ]
    for suffix, restore in DERIVATIONAL:
        if lower.endswith(suffix) and len(lower) > len(suffix) + 1:
            stem = lower[:-len(suffix)] + restore
            if stem in entry_set:
                return stem
            # Also try without the restore
            if restore and lower[:-len(suffix)] in entry_set:
                return lower[:-len(suffix)]

    return None


# --- Dictionary parser ---

ENTRY_RE = re.compile(r'^\*\*([a-z][a-z0-9 \-]*?)\*\*\s*[\u2014]|^\*\*([a-z][a-z0-9 \-]*?)\*\*\s*---')
EXAMPLE_RE = re.compile(r'^-\s*["\u201c](.+?)["\u201d]\s*$')
SECTION_RE = re.compile(r'^##\s+')

def parse_dictionary(content):
    """Parse a dictionary markdown file. Returns (entries, entry_set).
    entries: list of (word, definition, examples, line_number)
    entry_set: set of all entry words
    """
    entries = []
    entry_set = set()
    current_word = None
    current_def = None
    current_examples = []
    current_line = 0

    for i, line in enumerate(content.split('\n'), 1):
        stripped = line.strip()

        # Try to match entry line
        # Handle both em-dash and triple-dash
        entry_match = None
        if stripped.startswith('**'):
            # Find the closing **
            close = stripped.find('**', 2)
            if close > 2:
                word = stripped[2:close].strip().lower()
                rest = stripped[close+2:].strip()
                # Check for em-dash or triple-dash separator
                if rest.startswith('\u2014') or rest.startswith('---') or rest.startswith('—'):
                    # Extract definition (after separator)
                    if rest.startswith('---'):
                        definition = rest[3:].strip()
                    else:
                        definition = rest[1:].strip()
                    entry_match = (word, definition)

        if entry_match:
            # Save previous entry
            if current_word is not None:
                entries.append((current_word, current_def, current_examples, current_line))
            current_word = entry_match[0]
            current_def = entry_match[1]
            current_examples = []
            current_line = i
            entry_set.add(current_word)
            continue

        # Try to match example line
        ex_match = EXAMPLE_RE.match(stripped)
        if ex_match and current_word:
            current_examples.append((ex_match.group(1), i))
            continue

    # Save last entry
    if current_word is not None:
        entries.append((current_word, current_def, current_examples, current_line))

    return entries, entry_set


def parse_grammar(content, entry_set):
    """Parse grammar file and check all words against provided entry_set.
    Returns list of violations.
    """
    violations = []
    for i, line in enumerate(content.split('\n'), 1):
        stripped = line.strip()
        # Skip headers, separators, blockquotes, empty lines
        if not stripped or stripped.startswith('#') or stripped.startswith('---') or stripped.startswith('>'):
            continue
        tokens = tokenize(stripped)
        for token in tokens:
            # Skip pure numbers
            if token.isdigit():
                continue
            resolved = stem_to_entry(token, entry_set)
            if resolved is None and token not in EXCEPTIONS:
                violations.append(("_grammar_", "text", token, i))
    return violations


# --- Main checker ---

def check_closure(entries, entry_set):
    """Check that all words in definitions and examples are in entry_set.
    Returns list of (entry_word, location, undefined_token, line_number).
    """
    violations = []

    for word, definition, examples, line_num in entries:
        # Check definition
        if definition:
            tokens = tokenize(definition)
            for token in tokens:
                if token.isdigit():
                    continue
                resolved = stem_to_entry(token, entry_set)
                if resolved is None and token not in EXCEPTIONS:
                    violations.append((word, "def", token, line_num))

        # Check examples
        for ex_text, ex_line in examples:
            tokens = tokenize(ex_text)
            for token in tokens:
                if token.isdigit():
                    continue
                resolved = stem_to_entry(token, entry_set)
                if resolved is None and token not in EXCEPTIONS:
                    violations.append((word, "example", token, ex_line))

    return violations


def print_report(violations, total_entries, filename):
    """Print a closure report."""
    print(f"\nCLOSURE CHECK: {filename}")
    print(f"Entries: {total_entries}")

    if not violations:
        print("Status: CLOSED (zero violations)")
        return

    # Count by location type
    def_violations = [v for v in violations if v[1] == "def"]
    ex_violations = [v for v in violations if v[1] == "example"]
    grammar_violations = [v for v in violations if v[1] == "text"]

    if def_violations:
        print(f"Definition violations: {len(def_violations)}")
    if ex_violations:
        print(f"Example violations: {len(ex_violations)}")
    if grammar_violations:
        print(f"Grammar text violations: {len(grammar_violations)}")
    print(f"Total violations: {len(violations)}")

    # Count unique undefined words
    undefined_counts = defaultdict(int)
    for _, _, token, _ in violations:
        undefined_counts[token] += 1

    print(f"Unique undefined words: {len(undefined_counts)}")

    # Print violations
    print("\nVIOLATIONS:")
    for entry_word, location, token, line_num in violations:
        loc_label = f"[{location.upper():7s}]"
        print(f"  {loc_label} entry \"{entry_word}\" uses undefined word \"{token}\" (line {line_num})")

    # Print summary sorted by frequency
    print(f"\nSUGGESTED ADDITIONS ({len(undefined_counts)} unique words, sorted by frequency):")
    for token, count in sorted(undefined_counts.items(), key=lambda x: (-x[1], x[0])):
        print(f"  {token} (used {count}x)")


def main():
    parser = argparse.ArgumentParser(description="YALM dictionary closure checker")
    parser.add_argument("dict_file", help="Path to the dictionary .md file to check")
    parser.add_argument("--against", help="Check grammar file against this dictionary's entries (for grammar files)")
    args = parser.parse_args()

    with open(args.dict_file, 'r', encoding='utf-8') as f:
        content = f.read()

    if args.against:
        # Grammar mode: check dict_file (grammar) against --against (dictionary)
        with open(args.against, 'r', encoding='utf-8') as f:
            dict_content = f.read()
        _, entry_set = parse_dictionary(dict_content)
        violations = parse_grammar(content, entry_set)
        print_report(violations, len(entry_set), args.dict_file)
    else:
        # Dictionary mode: self-closure check
        entries, entry_set = parse_dictionary(content)
        violations = check_closure(entries, entry_set)
        print_report(violations, len(entries), args.dict_file)

    # Exit code: 0 if closed, 1 if violations
    sys.exit(0 if not violations else 1)


if __name__ == "__main__":
    main()

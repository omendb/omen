#!/usr/bin/env python3
"""Clean up native.mojo by removing batch functions temporarily."""

import re

with open("omendb/native.mojo", "r") as f:
    lines = f.readlines()

# Find function boundaries
in_batch_function = False
batch_functions = [
    "add_vector_batch",
    "search_vectors_concurrent",
    "search_batch_concurrent",
]
current_function = None
cleaned_lines = []
skip_until_next_fn = False

i = 0
while i < len(lines):
    line = lines[i]

    # Check if we're at a function definition
    if line.strip().startswith("fn "):
        skip_until_next_fn = False
        fn_name = line.strip().split("(")[0].replace("fn ", "")
        if any(bf in fn_name for bf in batch_functions):
            skip_until_next_fn = True
            cleaned_lines.append(
                f"# TODO: Re-enable {fn_name} after fixing handle management\n"
            )

    if not skip_until_next_fn:
        cleaned_lines.append(line)

    i += 1

# Write back
with open("omendb/native_clean.mojo", "w") as f:
    f.writelines(cleaned_lines)

print("Created cleaned native module")

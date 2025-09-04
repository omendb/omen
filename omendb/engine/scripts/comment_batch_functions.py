#!/usr/bin/env python3
"""Comment out batch functions in native.mojo temporarily."""

import re

with open("omendb/native.mojo", "r") as f:
    content = f.read()

# Find and comment out the three batch functions
patterns = [
    (
        r"(# TODO: Re-enable after fixing handle management\n# fn add_vector_batch\([\s\S]*?)(?=# End of commented add_vector_batch)",
        lambda m: "# " + "\n# ".join(m.group(1).split("\n")),
    ),
    (
        r"(# TODO: Re-enable after fixing handle management\n# fn search_vectors_concurrent\([\s\S]*?)(?=# End of commented search_vectors_concurrent)",
        lambda m: "# " + "\n# ".join(m.group(1).split("\n")),
    ),
    (
        r"(# TODO: Re-enable after fixing handle management\n# fn search_batch_concurrent\([\s\S]*?)(?=# End of commented search_batch_concurrent)",
        lambda m: "# " + "\n# ".join(m.group(1).split("\n")),
    ),
]

# Apply the patterns
for pattern, replacement in patterns:
    content = re.sub(pattern, replacement, content, flags=re.MULTILINE)

# Write back
with open("omendb/native.mojo", "w") as f:
    f.write(content)

print("Commented out batch functions")

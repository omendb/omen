#!/usr/bin/env python3
"""Unified test runner for OmenDB."""

import sys
import subprocess
from pathlib import Path


def run_tests(category=None):
    """Run tests by category."""

    categories = {
        "unit": "test/unit/",
        "integration": "test/integration/",
        "benchmarks": "test/benchmarks/",
        "regression": "test/regression/",
        "all": "test/",
    }

    if category and category in categories:
        path = categories[category]
        print(f"🧪 Running {category} tests...")
        cmd = f"pytest {path} -v"
    else:
        print("🧪 Running all tests...")
        cmd = "pytest test/ -v"

    subprocess.run(cmd, shell=True)


if __name__ == "__main__":
    category = sys.argv[1] if len(sys.argv) > 1 else "all"
    run_tests(category)

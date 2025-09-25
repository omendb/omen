#!/usr/bin/env python3
"""
Comprehensive project audit for OmenDB.
Identifies issues with organization, documentation, and test coverage.
"""

import os
import glob
import json
from pathlib import Path
from typing import Dict, List, Set


class ProjectAuditor:
    def __init__(self, root_path: str = "."):
        self.root = Path(root_path)
        self.issues = []
        self.stats = {}

    def audit_root_files(self) -> List[str]:
        """Identify files in root that should be moved."""
        root_files_to_move = []

        # Files that should be in specific directories
        patterns = {
            "test_*.py": "test/debug/",
            "debug_*.py": "test/debug/",
            "benchmark_*.py": "benchmarks/",
            "cleanup_*.py": "scripts/cleanup/",
            "reorganize_*.py": "scripts/cleanup/",
            "*_results_*.json": "results/",
            "profile_results_*.json": "results/profiling/",
        }

        for pattern, target_dir in patterns.items():
            files = glob.glob(pattern)
            for file in files:
                if os.path.isfile(file):
                    root_files_to_move.append((file, target_dir))

        return root_files_to_move

    def audit_directory_structure(self) -> Dict:
        """Check if directory structure follows best practices."""
        expected_structure = {
            "omendb/": "Core Mojo/Python package",
            "python/": "Python bindings",
            "test/": "All tests",
            "benchmarks/": "Performance benchmarks",
            "docs/": "Documentation",
            "examples/": "Usage examples",
            "scripts/": "Utility scripts",
            "ci/": "CI/CD configuration",
            ".github/": "GitHub specific files",
        }

        issues = []
        for dir_name, purpose in expected_structure.items():
            if not os.path.exists(dir_name.rstrip("/")):
                issues.append(f"Missing {dir_name}: {purpose}")

        # Check for unexpected directories in root
        all_dirs = [
            d for d in os.listdir(".") if os.path.isdir(d) and not d.startswith(".")
        ]
        expected = {d.rstrip("/") for d in expected_structure.keys()}
        unexpected = (
            set(all_dirs) - expected - {".git", ".pixi", "build", "dist", "__pycache__"}
        )

        if "results" in unexpected:
            unexpected.remove("results")  # Results dir is ok
        if "profiling" in unexpected:
            unexpected.remove("profiling")  # Profiling dir is ok
        if "tools" in unexpected:
            unexpected.remove("tools")  # Tools dir is ok
        if "embeddings_cache" in unexpected:
            issues.append("embeddings_cache/ should be in .gitignore")

        return {"issues": issues, "unexpected_dirs": list(unexpected)}

    def audit_documentation(self) -> Dict:
        """Check documentation accuracy and placement."""
        doc_issues = []

        # Check for private docs in public repo
        public_docs = glob.glob("docs/**/*.md", recursive=True)
        private_keywords = [
            "investor",
            "business",
            "revenue",
            "pricing",
            "monetization",
            "competitive",
            "strategy",
            "roadmap",
            "internal",
        ]

        for doc in public_docs:
            with open(doc, "r") as f:
                content = f.read().lower()
                for keyword in private_keywords:
                    if keyword in content:
                        doc_issues.append(
                            f"{doc} may contain private info (keyword: {keyword})"
                        )
                        break

        # Check if README is up to date
        if os.path.exists("README.md"):
            with open("README.md", "r") as f:
                readme = f.read()
                # Check for outdated performance claims
                if "156,937" in readme or "99,261" in readme:
                    doc_issues.append("README.md has outdated performance numbers")
                if "roargraph" in readme.lower() or "hnsw" in readme.lower():
                    doc_issues.append("README.md references removed algorithms")

        # Check CHANGELOG
        if os.path.exists("CHANGELOG.md"):
            with open("CHANGELOG.md", "r") as f:
                changelog = f.read()
                if "2025-08-11" not in changelog:
                    doc_issues.append("CHANGELOG.md not updated with latest changes")

        return {"issues": doc_issues}

    def audit_test_coverage(self) -> Dict:
        """Assess test coverage completeness."""
        test_gaps = []

        # Check for specific test types
        test_categories = {
            "edge_cases": ["empty", "single", "duplicate", "null", "overflow"],
            "error_handling": ["invalid", "exception", "error", "fail"],
            "concurrency": ["thread", "concurrent", "parallel", "race"],
            "memory": ["leak", "memory", "allocation"],
            "persistence": ["save", "load", "checkpoint", "recover"],
            "performance": ["benchmark", "performance", "speed", "latency"],
        }

        all_tests = glob.glob("test/**/*.py", recursive=True)
        test_content = ""
        for test_file in all_tests:
            with open(test_file, "r") as f:
                test_content += f.read().lower()

        for category, keywords in test_categories.items():
            found = False
            for keyword in keywords:
                if keyword in test_content:
                    found = True
                    break
            if not found:
                test_gaps.append(f"Missing tests for: {category}")

        # Check for specific edge cases
        edge_cases_needed = [
            "Zero-length vectors",
            "Vectors with NaN/Inf values",
            "Very high dimensional vectors (>1000D)",
            "Database with 1M+ vectors",
            "Concurrent reads while writing",
            "Recovery after crash",
            "Out of memory handling",
        ]

        for case in edge_cases_needed:
            if case.lower() not in test_content:
                test_gaps.append(f"Missing edge case test: {case}")

        return {"gaps": test_gaps, "total_tests": len(all_tests)}

    def audit_github_actions(self) -> Dict:
        """Check if GitHub Actions are properly configured."""
        issues = []

        workflow_file = ".github/workflows/performance.yml"
        if os.path.exists(workflow_file):
            with open(workflow_file, "r") as f:
                workflow = f.read()

            # Check for issues
            if "pixi.toml" in workflow and not os.path.exists("pixi.toml"):
                issues.append("Workflow references pixi.toml but file doesn't exist")
            if "native.mojo" in workflow:
                # Check if path is correct
                if not os.path.exists("omendb/native.mojo"):
                    issues.append("Workflow has incorrect path to native.mojo")
            if "benchmark_suite.py" in workflow:
                if not os.path.exists("benchmarks/benchmark_suite.py"):
                    issues.append("Workflow has incorrect path to benchmark_suite.py")
        else:
            issues.append("Performance workflow not found")

        return {"issues": issues}

    def audit_api_consistency(self) -> Dict:
        """Review API for consistency and best practices."""
        api_issues = []

        # Check Python API
        api_file = "python/omendb/api.py"
        if os.path.exists(api_file):
            with open(api_file, "r") as f:
                api_content = f.read()

            # Check for consistency issues
            if "add(" in api_content and "add_batch(" in api_content:
                if "add_many(" not in api_content:
                    api_issues.append(
                        "Inconsistent naming: add_batch should be add_many"
                    )

            if "search(" in api_content:
                if "top_k" in api_content:
                    api_issues.append("Inconsistent parameter: top_k should be limit")

            if "delete(" not in api_content:
                api_issues.append("Missing delete() method")

            if "update(" not in api_content:
                api_issues.append("Missing update() method")

            # Check for proper error handling
            if "raise" not in api_content:
                api_issues.append("No error handling in API")

        return {"issues": api_issues}

    def run_full_audit(self) -> Dict:
        """Run complete project audit."""
        print("🔍 Running OmenDB Project Audit...")
        print("=" * 60)

        results = {
            "root_files": self.audit_root_files(),
            "directory_structure": self.audit_directory_structure(),
            "documentation": self.audit_documentation(),
            "test_coverage": self.audit_test_coverage(),
            "github_actions": self.audit_github_actions(),
            "api_consistency": self.audit_api_consistency(),
        }

        return results

    def print_audit_report(self, results: Dict):
        """Print formatted audit report."""
        print("\n📊 AUDIT REPORT")
        print("=" * 60)

        # Root files to move
        if results["root_files"]:
            print("\n🗂️  Files to move from root:")
            for file, target in results["root_files"]:
                print(f"  {file} → {target}")

        # Directory structure issues
        if results["directory_structure"]["issues"]:
            print("\n📁 Directory structure issues:")
            for issue in results["directory_structure"]["issues"]:
                print(f"  ❌ {issue}")

        # Documentation issues
        if results["documentation"]["issues"]:
            print("\n📝 Documentation issues:")
            for issue in results["documentation"]["issues"]:
                print(f"  ⚠️  {issue}")

        # Test coverage gaps
        if results["test_coverage"]["gaps"]:
            print("\n🧪 Test coverage gaps:")
            for gap in results["test_coverage"]["gaps"][:10]:  # Show first 10
                print(f"  ❌ {gap}")
            if len(results["test_coverage"]["gaps"]) > 10:
                print(f"  ... and {len(results['test_coverage']['gaps']) - 10} more")

        # GitHub Actions issues
        if results["github_actions"]["issues"]:
            print("\n🔧 GitHub Actions issues:")
            for issue in results["github_actions"]["issues"]:
                print(f"  ❌ {issue}")

        # API consistency issues
        if results["api_consistency"]["issues"]:
            print("\n🔌 API consistency issues:")
            for issue in results["api_consistency"]["issues"]:
                print(f"  ⚠️  {issue}")

        # Summary
        total_issues = (
            len(results["root_files"])
            + len(results["directory_structure"]["issues"])
            + len(results["documentation"]["issues"])
            + len(results["test_coverage"]["gaps"])
            + len(results["github_actions"]["issues"])
            + len(results["api_consistency"]["issues"])
        )

        print("\n" + "=" * 60)
        print(f"📊 Total issues found: {total_issues}")

        # Priority actions
        print("\n🎯 Priority Actions:")
        print("  1. Move root-level test/debug files to proper directories")
        print("  2. Update README with current performance numbers")
        print("  3. Add missing test coverage for edge cases")
        print("  4. Fix GitHub Actions workflow paths")
        print("  5. Review and improve API consistency")


def main():
    auditor = ProjectAuditor()
    results = auditor.run_full_audit()
    auditor.print_audit_report(results)

    # Save results
    with open("audit_results.json", "w") as f:
        # Convert tuples to lists for JSON serialization
        results_json = results.copy()
        results_json["root_files"] = [[f, t] for f, t in results["root_files"]]
        json.dump(results_json, f, indent=2)
    print("\n💾 Audit results saved to audit_results.json")


if __name__ == "__main__":
    main()

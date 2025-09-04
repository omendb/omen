"""
Setup script for OmenDB Python bindings.
"""

import os
import platform
from pathlib import Path
from setuptools import setup, find_packages

# Read version from package
version_file = Path(__file__).parent / "omendb" / "__init__.py"
version = None
with open(version_file) as f:
    for line in f:
        if line.startswith("__version__"):
            version = line.split('"')[1]
            break

if not version:
    raise RuntimeError("Could not determine version")

# Read long description from README
readme_file = Path(__file__).parent.parent / "README.md"
long_description = ""
if readme_file.exists():
    with open(readme_file, encoding="utf-8") as f:
        long_description = f.read()


# Platform-specific shared library
def get_library_name():
    """Get platform-specific library name."""
    if platform.system() == "Windows":
        return "omendb.dll"
    elif platform.system() == "Darwin":
        return "libomendb.dylib"
    else:
        return "libomendb.so"


# Package data - include the shared library when built
package_data = {"omendb": [get_library_name()]}

setup(
    name="omendb",
    version=version,
    description="High-performance embedded vector database with dual-mode deployment",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="OmenDB Team",
    author_email="support@omendb.io",
    url="https://github.com/omendb/omendb",
    license="Apache-2.0",
    packages=find_packages(),
    package_data=package_data,
    include_package_data=True,
    python_requires=">=3.8",
    install_requires=[
        # No external dependencies - only uses standard library
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-cov>=2.0",
            "black>=21.0",
            "isort>=5.0",
            "mypy>=0.900",
        ],
        "docs": [
            "sphinx>=4.0",
            "sphinx-rtd-theme>=1.0",
        ],
    },
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Intended Audience :: Science/Research",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Topic :: Database",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    keywords="vector database, machine learning, embeddings, similarity search, AI",
    project_urls={
        "Documentation": "https://omendb.io/docs",
        "Website": "https://omendb.io",
        "Source": "https://github.com/omendb/omendb",
        "Tracker": "https://github.com/omendb/omendb/issues",
    },
    zip_safe=False,  # Contains native library
)

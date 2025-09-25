"""
MLOps Module for OmenDB
=======================

This module provides MLOps vector versioning and model lifecycle management
capabilities for complete ML workflow integration.
"""

from .vector_versioning import (
    MLOpsVectorVersioning,
    ModelVersionStore,
    VectorABTesting,
    DistributionDriftDetector,
    ModelLifecycleManager,
    ModelVersion,
    DriftAlert,
    VersionComparison
)
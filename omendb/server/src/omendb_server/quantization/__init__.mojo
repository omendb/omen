"""
Quantization Module for OmenDB
==============================

This module provides advanced quantization techniques including Matryoshka
representations for adaptive precision and cost optimization.
"""

from .matryoshka import (
    MatryoshkaVector,
    AdaptivePrecision, 
    MatryoshkaSearchEngine,
    QueryType,
    CostSavings,
    PerformanceEntry
)
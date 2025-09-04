"""
Query processing pipeline stages for OmenDB.

This module defines the trait-based pipeline architecture for query processing.
"""

from collections import List, Optional, Dict
from core.vector import Vector
from core.record import VectorRecord
from storage.vector_store import QueryFilter
from util.logging import Logger, LogLevel

struct QueryResult(Copyable, Movable):
    """
    Result of a query operation containing vector records and their scores.
    
    Attributes:
        records: List of vector records matching the query
        scores: Dictionary mapping record IDs to similarity scores
    """
    var records: List[VectorRecord[DType.float32]]
    var scores: Dict[String, Float64]
    
    fn __init__(out self):
        self.records = List[VectorRecord[DType.float32]]()
        self.scores = Dict[String, Float64]()
    
    fn __copyinit__(out self, other: Self):
        self.records = other.records
        self.scores = other.scores
        
    fn add_record(mut self, record: VectorRecord[DType.float32], score: Float64):
        """Add a record with its score to the result set."""
        self.records.append(record)
        self.scores[record.id] = score
        
    fn size(self) -> Int:
        """Get the number of records in the result set."""
        return len(self.records)
        
    fn sort_by_score(mut self) raises:
        """Sort the records by score in descending order (highest score first)."""
        # Create a list of (index, score) pairs
        var idx_score_pairs = List[(Int, Float64)]()
        
        for i in range(len(self.records)):
            var id = self.records[i].id
            var score = self.scores[id]
            idx_score_pairs.append((i, score))
        
        # Sort the pairs by score (descending) using bubble sort
        for i in range(len(idx_score_pairs)):
            for j in range(len(idx_score_pairs) - i - 1):
                var current_score = idx_score_pairs[j][1]
                var next_score = idx_score_pairs[j + 1][1]
                if current_score < next_score:
                    var temp = idx_score_pairs[j]
                    idx_score_pairs[j] = idx_score_pairs[j + 1]
                    idx_score_pairs[j + 1] = temp
        
        # Create a new sorted list of records
        var sorted_records = List[VectorRecord[DType.float32]]()
        
        for pair in idx_score_pairs:
            var idx = pair[0]
            sorted_records.append(self.records[idx])
        
        # Update the records list
        self.records = sorted_records
        
    fn limit(mut self, limit: Int):
        """Limit the number of records in the result set."""
        if limit < len(self.records):
            var limited_records = List[VectorRecord[DType.float32]]()
            for i in range(limit):
                limited_records.append(self.records[i])
            self.records = limited_records

trait QueryStage(Copyable, Movable):
    """
    Trait defining the interface for query pipeline stages.
    
    Query stages process input data and pass results to the next stage
    in the pipeline, allowing for flexible query execution.
    """
    
    fn execute(
        self, 
        input_result: QueryResult,
        query_vector: Optional[Vector[DType.float32]] = None,
        filters: Optional[List[QueryFilter]] = None
    ) raises -> QueryResult:
        """
        Execute this query stage.
        
        Args:
            input_result: Input from the previous stage (or empty for first stage)
            query_vector: Optional vector to find similar vectors to
            filters: Optional list of metadata filters to apply
            
        Returns:
            Modified query result to pass to the next stage
            
        Raises:
            Error: If there was an error during stage execution
        """
        ...
    
    fn name(self) -> String:
        """
        Get the name of this query stage.
        
        Returns:
            String identifier for this stage
        """
        ...
    
    fn configure(mut self, options: Dict[String, String]) raises:
        """
        Configure this query stage with options.
        
        Args:
            options: Dictionary of configuration options
            
        Raises:
            Error: If there was an error during configuration
        """
        ...

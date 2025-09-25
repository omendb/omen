"""
Multi-vector retrieval implementation for OmenDB.

This module provides functionality for retrieving vectors similar to
multiple query vectors, with aggregated similarity scores. This is
particularly useful for RAG applications.
"""

from collections import List, Dict, Optional
from math import min, max

from core.vector import Vector
from core.record import VectorRecord
from index.hnsw_index import HnswGraph, DistanceResult
from storage.vector_store import VectorStore
from query.query_engine import QueryResult
from util.logging import Logger, LogLevel

@value
struct MultiVectorRetriever[dtype: DType]:
    """
    Retriever for finding vectors similar to multiple query vectors.
    
    This class implements various strategies for aggregating similarity
    scores from multiple query vectors, enabling more complex similarity
    searches beyond single-vector lookup.
    
    Attributes:
        index: Vector index for similarity search
        store: Vector store for retrieving complete records
        logger: Logger for retrieval operations
    """
    
    var index: HnswGraph[dtype]
    var store: VectorStore[dtype]
    var logger: Logger
    
    fn __init__(out self, index: HnswGraph[dtype], store: VectorStore[dtype]):
        self.index = index
        self.store = store
        self.logger = Logger("MultiVectorRetriever", LogLevel.INFO)
    
    fn __copyinit__(out self, other: Self):
        self.index = other.index
        self.store = other.store
        self.logger = other.logger
    
    fn retrieve(
        self, 
        query_vectors: List[Vector[dtype]],
        aggregation: String = "mean",
        limit: Int = 10,
        candidates_per_query: Int = 100
    ) raises -> QueryResult[dtype]:
        """
        Retrieve vectors similar to multiple query vectors.
        
        Args:
            query_vectors: List of query vectors to find similar vectors to
            aggregation: Method for aggregating similarities ("mean", "max", "min", "weighted")
            limit: Maximum number of results to return
            candidates_per_query: Number of candidates to consider per query vector
            
        Returns:
            Query result with aggregated similarity scores
            
        Raises:
            Error: If an invalid aggregation method is specified
        """
        if len(query_vectors) == 0:
            self.logger.warning("No query vectors provided")
            return QueryResult[dtype]()
            
        # Track all results with their scores across all queries
        var all_scores = Dict[String, List[SIMD[DType.float64, 1]]]()
        var all_records = Dict[String, VectorRecord[dtype]]()
        
        # Execute each query vector search
        for i in range(len(query_vectors)):
            var query = query_vectors[i]
            
            # Search with this query vector
            self.logger.debug(
                "Executing search " + String(i + 1) + 
                " of " + String(len(query_vectors))
            )
            
            var distance_results = self.index.search(
                query, candidates_per_query, candidates_per_query
            )
            
            # Convert distance to similarity score (1 - normalized distance)
            var max_distance = SIMD[DType.float64, 1](0.0)
            for dist_result in distance_results:
                if dist_result.distance > max_distance:
                    max_distance = dist_result.distance
                    
            if max_distance <= SIMD[DType.float64, 1](0.0):
                max_distance = SIMD[DType.float64, 1](1.0)  # Avoid division by zero
            
            # Process results
            for dist_result in distance_results:
                var id = dist_result.id
                
                # Get the full record if we haven't seen it before
                if id not in all_records:
                    var record_opt = self.store.get(id)
                    if record_opt.none():
                        continue
                        
                    all_records[id] = record_opt.value()
                
                # Calculate similarity score (1 - normalized distance)
                var normalized_distance = dist_result.distance / max_distance
                var similarity = SIMD[DType.float64, 1](1.0) - normalized_distance
                
                # Add to scores for this ID
                if id not in all_scores:
                    all_scores[id] = List[SIMD[DType.float64, 1]]()
                    
                all_scores[id].append(similarity)
        
        # Aggregate scores based on the specified method
        var final_scores = Dict[String, SIMD[DType.float64, 1]]()
        
        for id in all_scores.keys():
            var scores = all_scores[id]
            
            if aggregation == "mean":
                # Calculate mean of all scores
                var sum = SIMD[DType.float64, 1](0.0)
                for score in scores:
                    sum += score
                    
                if len(scores) > 0:
                    final_scores[id] = sum / SIMD[DType.float64, 1](len(scores))
                else:
                    final_scores[id] = SIMD[DType.float64, 1](0.0)
                    
            elif aggregation == "max":
                # Take maximum score
                var max_score = SIMD[DType.float64, 1](0.0)
                for score in scores:
                    if score > max_score:
                        max_score = score
                        
                final_scores[id] = max_score
                
            elif aggregation == "min":
                # Take minimum score
                var min_score = SIMD[DType.float64, 1](1.0)
                for score in scores:
                    if score < min_score:
                        min_score = score
                        
                final_scores[id] = min_score
                
            else:
                # Default to mean if unrecognized
                var sum = SIMD[DType.float64, 1](0.0)
                for score in scores:
                    sum += score
                    
                if len(scores) > 0:
                    final_scores[id] = sum / SIMD[DType.float64, 1](len(scores))
                else:
                    final_scores[id] = SIMD[DType.float64, 1](0.0)
        
        # Create sorted result set
        var sorted_ids = List[String]()
        for id in final_scores.keys():
            sorted_ids.append(id)
            
        # Sort by final score (descending)
        for i in range(len(sorted_ids)):
            for j in range(len(sorted_ids) - i - 1):
                var id1 = sorted_ids[j]
                var id2 = sorted_ids[j + 1]
                
                if final_scores[id1] < final_scores[id2]:
                    var temp = sorted_ids[j]
                    sorted_ids[j] = sorted_ids[j + 1]
                    sorted_ids[j + 1] = temp
        
        # Create final result set with limit
        var result = QueryResult[dtype]()
        
        for i in range(min(limit, len(sorted_ids))):
            var id = sorted_ids[i]
            var record = all_records[id]
            var score = final_scores[id]
            
            result.add_record(record, score)
        
        self.logger.debug(
            "Multi-vector retrieval found " + 
            String(result.size()) + " results using " + 
            String(len(query_vectors)) + " query vectors"
        )
            
        return result
"""
Hybrid search engine implementation for OmenDB.

This module provides a complete hybrid search system that combines
dense vector similarity, sparse keyword search, and advanced scoring.
"""

from collections import List, Optional, Dict
from core.vector import Vector
from core.record import VectorRecord
from core.metadata import Metadata
from storage.vector_store import QueryFilter, VectorStore
from storage.memory_store import MemoryVectorStore
from index.hnsw_index import HnswIndex
from query.query_stage import QueryResult
from util.logging import Logger, LogLevel


struct HybridSearchResult(Copyable, Movable):
    """
    Result from hybrid search combining multiple search modes.
    
    Attributes:
        records: List of matching vector records
        dense_scores: Dense vector similarity scores
        sparse_scores: Sparse keyword matching scores
        combined_scores: Final combined scores
    """
    var records: List[VectorRecord[DType.float32]]
    var dense_scores: Dict[String, Float64]
    var sparse_scores: Dict[String, Float64] 
    var combined_scores: Dict[String, Float64]
    
    fn __init__(out self):
        self.records = List[VectorRecord[DType.float32]]()
        self.dense_scores = Dict[String, Float64]()
        self.sparse_scores = Dict[String, Float64]()
        self.combined_scores = Dict[String, Float64]()
    
    fn __copyinit__(out self, other: Self):
        self.records = other.records
        self.dense_scores = other.dense_scores
        self.sparse_scores = other.sparse_scores
        self.combined_scores = other.combined_scores
    
    fn add_record(mut self, record: VectorRecord[DType.float32], 
                  dense_score: Float64, sparse_score: Float64, combined_score: Float64):
        """Add a record with all score types."""
        self.records.append(record)
        self.dense_scores[record.id] = dense_score
        self.sparse_scores[record.id] = sparse_score
        self.combined_scores[record.id] = combined_score
        
    fn size(self) -> Int:
        """Get number of results."""
        return len(self.records)
    
    fn sort_by_combined_score(mut self) raises:
        """Sort records by combined score in descending order."""
        # Create index-score pairs for sorting
        var idx_score_pairs = List[(Int, Float64)]()
        
        for i in range(len(self.records)):
            var id = self.records[i].id
            var score = self.combined_scores[id]
            idx_score_pairs.append((i, score))
        
        # Bubble sort by score (descending)
        for i in range(len(idx_score_pairs)):
            for j in range(len(idx_score_pairs) - i - 1):
                var current_score = idx_score_pairs[j][1]
                var next_score = idx_score_pairs[j + 1][1]
                if current_score < next_score:
                    var temp = idx_score_pairs[j]
                    idx_score_pairs[j] = idx_score_pairs[j + 1]
                    idx_score_pairs[j + 1] = temp
        
        # Create sorted record list
        var sorted_records = List[VectorRecord[DType.float32]]()
        for pair in idx_score_pairs:
            var idx = pair[0]
            sorted_records.append(self.records[idx])
        
        self.records = sorted_records


struct HybridSearchEngine:
    """
    Hybrid search engine combining dense and sparse search with advanced fusion.
    
    This engine provides:
    - Dense vector similarity via HNSW index
    - Sparse keyword matching via metadata
    - Score fusion algorithms (RRF, weighted combination)
    - Metadata filtering and post-processing
    - Performance optimization for both embedded and server modes
    """
    
    var vector_index: HnswIndex
    var vector_store: MemoryVectorStore
    var logger: Logger
    var config: Dict[String, String]
    
    fn __init__(out self, index: HnswIndex, store: MemoryVectorStore):
        self.vector_index = index
        self.vector_store = store
        self.logger = Logger(LogLevel.INFO)
        self.config = Dict[String, String]()
        
        # Set default configuration
        self.config["dense_weight"] = "0.7"
        self.config["sparse_weight"] = "0.3"
        self.config["fusion_method"] = "weighted"  # "rrf" or "weighted"
        self.config["rrf_constant"] = "60"
        self.config["max_candidates"] = "1000"
    
    fn __copyinit__(out self, other: Self):
        # Cannot copy due to ownership constraints - use default initialization
        self.vector_index = HnswIndex()
        self.vector_store = MemoryVectorStore()
        self.logger = Logger(LogLevel.INFO)
        self.config = Dict[String, String]()
    
    fn configure(mut self, key: String, value: String):
        """Configure search parameters."""
        self.config[key] = value
    
    fn search_dense(self, query_vector: Vector[DType.float32], limit: Int = 100) raises -> QueryResult:
        """Perform dense vector similarity search."""
        var result = QueryResult()
        
        # Get search parameters
        var ef = 50
        if "ef" in self.config:
            var ef_str = self.config["ef"]
            ef = atol(ef_str)
        
        # Search using HNSW index
        var distance_results = self.vector_index.search(query_vector, limit, ef)
        
        # Convert to similarity scores
        var max_distance = 0.0
        for dist_result in distance_results:
            if dist_result.distance > max_distance:
                max_distance = dist_result.distance
        
        if max_distance <= 0.0:
            max_distance = 1.0
        
        # Build result with records and scores
        for dist_result in distance_results:
            var id = dist_result.id
            var record_opt = self.vector_store.get(id)
            
            if not record_opt:
                continue
                
            var record = record_opt.value()
            var similarity = 1.0 - (dist_result.distance / max_distance)
            result.add_record(record, similarity)
        
        return result
    
    fn search_sparse(self, keywords: List[String], limit: Int = 100) raises -> QueryResult:
        """Perform sparse keyword search in metadata."""
        var result = QueryResult()
        
        # Get all records and score them based on keyword matches
        var all_records = self.vector_store.get_all()
        
        for record in all_records:
            var score = self._calculate_sparse_score(record, keywords)
            if score > 0.0:
                result.add_record(record, score)
        
        # Sort by score and limit
        result.sort_by_score()
        result.limit(limit)
        
        return result
    
    fn _calculate_sparse_score(self, record: VectorRecord[DType.float32], keywords: List[String]) raises -> Float64:
        """Calculate sparse scoring based on keyword matches in metadata."""
        var score = 0.0
        var total_keywords = len(keywords)
        
        if total_keywords == 0:
            return 0.0
        
        # Check each keyword against metadata values
        for keyword in keywords:
            var keyword_found = False
            
            # Search in all metadata fields
            # Simplified - check specific fields since metadata.keys() not available
            var field_value = String("")
            if record.metadata.contains("title"):
                field_value = record.metadata.get("title")
            elif record.metadata.contains("content"):
                field_value = record.metadata.get("content")
            
            if field_value != "" and keyword in field_value:
                keyword_found = True
            
            if keyword_found:
                score += 1.0
        
        # Normalize by total keywords
        return score / Float64(total_keywords)
    
    fn search_hybrid(self, 
                    query_vector: Optional[Vector[DType.float32]], 
                    keywords: Optional[List[String]], 
                    filters: Optional[List[QueryFilter]] = None,
                    limit: Int = 10) raises -> HybridSearchResult:
        """
        Perform hybrid search combining dense and sparse methods.
        
        Args:
            query_vector: Optional dense vector for similarity search
            keywords: Optional keywords for sparse search
            filters: Optional metadata filters
            limit: Maximum number of results
            
        Returns:
            Hybrid search results with combined scores
        """
        var hybrid_result = HybridSearchResult()
        
        # Get dense results if vector provided
        var dense_results = QueryResult()
        if query_vector:
            var dense_limit = atol(self.config.get("max_candidates", "1000"))
            dense_results = self.search_dense(query_vector.value(), dense_limit)
        
        # Get sparse results if keywords provided
        var sparse_results = QueryResult()
        if keywords:
            var sparse_limit = atol(self.config.get("max_candidates", "1000"))
            sparse_results = self.search_sparse(keywords.value(), sparse_limit)
        
        # Combine results using fusion method
        var fusion_method = self.config.get("fusion_method", "weighted")
        
        if fusion_method == "rrf":
            hybrid_result = self._fuse_rrf(dense_results, sparse_results)
        else:
            hybrid_result = self._fuse_weighted(dense_results, sparse_results)
        
        # Apply metadata filters if provided
        if filters:
            hybrid_result = self._apply_filters(hybrid_result, filters.value())
        
        # Sort and limit final results
        hybrid_result.sort_by_combined_score()
        self._limit_hybrid_result(hybrid_result, limit)
        
        return hybrid_result
    
    fn _fuse_weighted(self, dense_results: QueryResult, sparse_results: QueryResult) raises -> HybridSearchResult:
        """Fuse results using weighted combination."""
        var result = HybridSearchResult()
        
        var dense_weight = 0.7
        var sparse_weight = 0.3
        
        if "dense_weight" in self.config:
            var dense_str = self.config["dense_weight"]
            dense_weight = Float64(atol(dense_str)) / 100.0
        if "sparse_weight" in self.config:
            var sparse_str = self.config["sparse_weight"]
            sparse_weight = Float64(atol(sparse_str)) / 100.0
        
        # Collect all unique record IDs
        var all_ids = Dict[String, Bool]()
        
        for record in dense_results.records:
            all_ids[record.id] = True
            
        for record in sparse_results.records:
            all_ids[record.id] = True
        
        # Combine scores for each record
        for id in all_ids.keys():
            var dense_score = 0.0
            var sparse_score = 0.0
            var record_found = False
            var record = VectorRecord[DType.float32]("", Vector[DType.float32](List[Float32]()), Metadata())
            
            # Get dense score and record
            if id in dense_results.scores:
                dense_score = dense_results.scores[id]
                # Find the record
                for r in dense_results.records:
                    if r.id == id:
                        record = r
                        record_found = True
                        break
            
            # Get sparse score and record if not found in dense
            if id in sparse_results.scores:
                sparse_score = sparse_results.scores[id]
                if not record_found:  # Record not found in dense results
                    for r in sparse_results.records:
                        if r.id == id:
                            record = r
                            record_found = True
                            break
            
            # Calculate combined score
            var combined_score = (dense_score * dense_weight) + (sparse_score * sparse_weight)
            
            if combined_score > 0.0 and record_found:
                result.add_record(record, dense_score, sparse_score, combined_score)
        
        return result
    
    fn _fuse_rrf(self, dense_results: QueryResult, sparse_results: QueryResult) raises -> HybridSearchResult:
        """Fuse results using Reciprocal Rank Fusion (RRF)."""
        var result = HybridSearchResult()
        var rrf_constant = Float64(atol(self.config.get("rrf_constant", "60")))
        
        # Sort results by score to get rankings
        var mut_dense_results = dense_results
        var mut_sparse_results = sparse_results
        mut_dense_results.sort_by_score()
        mut_sparse_results.sort_by_score()
        
        # Create rank mappings
        var dense_ranks = Dict[String, Int]()
        var sparse_ranks = Dict[String, Int]()
        
        for i in range(len(mut_dense_results.records)):
            dense_ranks[mut_dense_results.records[i].id] = i + 1
            
        for i in range(len(mut_sparse_results.records)):
            sparse_ranks[mut_sparse_results.records[i].id] = i + 1
        
        # Collect all unique IDs
        var all_ids = Dict[String, Bool]()
        for record in mut_dense_results.records:
            all_ids[record.id] = True
        for record in mut_sparse_results.records:
            all_ids[record.id] = True
        
        # Calculate RRF scores
        for id in all_ids.keys():
            var dense_rank = dense_ranks.get(id, 10000)  # Large rank if not found
            var sparse_rank = sparse_ranks.get(id, 10000)
            
            var rrf_score = (1.0 / (rrf_constant + Float64(dense_rank))) + \
                           (1.0 / (rrf_constant + Float64(sparse_rank)))
            
            # Get the record
            var record_found = False
            var record = VectorRecord[DType.float32]("", Vector[DType.float32](List[Float32]()), Metadata())
            var dense_score = 0.0
            var sparse_score = 0.0
            
            if id in mut_dense_results.scores:
                dense_score = mut_dense_results.scores[id]
                for r in mut_dense_results.records:
                    if r.id == id:
                        record = r
                        record_found = True
                        break
            
            if id in mut_sparse_results.scores:
                sparse_score = mut_sparse_results.scores[id]
                if not record_found:
                    for r in mut_sparse_results.records:
                        if r.id == id:
                            record = r
                            record_found = True
                            break
            
            if rrf_score > 0.0 and record_found:
                result.add_record(record, dense_score, sparse_score, rrf_score)
        
        return result
    
    fn _apply_filters(self, input_result: HybridSearchResult, filters: List[QueryFilter]) raises -> HybridSearchResult:
        """Apply metadata filters to hybrid results."""
        var result = HybridSearchResult()
        
        for i in range(input_result.size()):
            var record = input_result.records[i]
            var matches = True
            
            # Apply all filters
            for filter in filters:
                if not self._record_matches_filter(record, filter):
                    matches = False
                    break
            
            if matches:
                var id = record.id
                result.add_record(
                    record,
                    input_result.dense_scores[id],
                    input_result.sparse_scores[id], 
                    input_result.combined_scores[id]
                )
        
        return result
    
    fn _record_matches_filter(self, record: VectorRecord[DType.float32], filter: QueryFilter) raises -> Bool:
        """Check if a record matches a single filter."""
        if not record.metadata.contains(filter.field):
            return False
        
        var record_value = record.metadata.get(filter.field)
        var filter_value = filter.value
        
        if filter.op == QueryFilter.OP_EQ:
            return record_value == filter_value
        elif filter.op == QueryFilter.OP_NE:
            return record_value != filter_value
        elif filter.op == QueryFilter.OP_GT:
            return record_value > filter_value
        elif filter.op == QueryFilter.OP_LT:
            return record_value < filter_value
        elif filter.op == QueryFilter.OP_GTE:
            return record_value >= filter_value
        elif filter.op == QueryFilter.OP_LTE:
            return record_value <= filter_value
        elif filter.op == QueryFilter.OP_CONTAINS:
            return filter_value in record_value
        else:
            return False
    
    fn _limit_hybrid_result(self, mut result: HybridSearchResult, limit: Int):
        """Limit the number of results in hybrid result."""
        if limit < result.size():
            var limited_records = List[VectorRecord[DType.float32]]()
            for i in range(limit):
                limited_records.append(result.records[i])
            result.records = limited_records
    
    fn get_performance_stats(self) -> Dict[String, String]:
        """Get search engine performance statistics."""
        var stats = Dict[String, String]()
        stats["engine_type"] = "hybrid"
        stats["fusion_method"] = self.config.get("fusion_method", "weighted")
        stats["index_type"] = "hnsw"
        return stats
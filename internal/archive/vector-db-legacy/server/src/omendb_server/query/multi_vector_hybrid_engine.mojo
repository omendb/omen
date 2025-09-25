"""
Multi-vector hybrid search engine for OmenDB.

This module integrates multi-vector document support with the hybrid search pipeline,
enabling cross-modal search with dense vector similarity, sparse keyword matching,
and unified ranking across different vector types.

Dual-mode compatible: Optimized for both embedded and server deployments.
"""

from collections import List, Dict, Optional
from algorithm import min, max

from core.multi_vector_doc import MultiVectorDocument, VectorType
from core.vector import Vector
from core.metadata import Metadata
from storage.multi_vector_store import MultiVectorStore, CrossModalQuery, MultiVectorSearchResult
from storage.vector_store import QueryFilter
from query.multi_vector_ranking import MultiVectorRanker, RankingStrategy, BoostingRule
from query.hybrid_engine import HybridSearchEngine, HybridSearchResult
from util.logging import Logger, LogLevel


@value
struct MultiVectorHybridQuery[dtype: DType = DType.float32](Copyable):
    """
    Comprehensive query specification for multi-vector hybrid search.
    
    Combines dense vector search, sparse keyword search, and cross-modal
    capabilities in a single query interface.
    """
    
    var dense_vectors: Dict[String, Vector[dtype]]  # Vector type -> query vector
    var sparse_keywords: List[String]              # Keywords for sparse search
    var metadata_filters: List[QueryFilter]        # Metadata filtering
    var vector_type_weights: Dict[String, Float64] # Importance weights per type
    var ranking_strategy: String                   # Fusion strategy
    var similarity_threshold: Float64              # Minimum similarity
    var cross_modal_enabled: Bool                  # Enable cross-modal search
    
    fn __init__(
        out self,
        dense_vectors: Optional[Dict[String, Vector[dtype]]] = None,
        sparse_keywords: Optional[List[String]] = None,
        metadata_filters: Optional[List[QueryFilter]] = None,
        vector_type_weights: Optional[Dict[String, Float64]] = None,
        ranking_strategy: String = RankingStrategy.ADAPTIVE,
        similarity_threshold: Float64 = 0.0,
        cross_modal_enabled: Bool = True
    ):
        # Initialize dense vectors
        if dense_vectors:
            self.dense_vectors = dense_vectors.value()
        else:
            self.dense_vectors = Dict[String, Vector[dtype]]()
        
        # Initialize sparse keywords
        if sparse_keywords:
            self.sparse_keywords = sparse_keywords.value()
        else:
            self.sparse_keywords = List[String]()
        
        # Initialize metadata filters
        if metadata_filters:
            self.metadata_filters = metadata_filters.value()
        else:
            self.metadata_filters = List[QueryFilter]()
        
        # Initialize weights
        if vector_type_weights:
            self.vector_type_weights = vector_type_weights.value()
        else:
            self.vector_type_weights = Dict[String, Float64]()
        
        self.ranking_strategy = ranking_strategy
        self.similarity_threshold = similarity_threshold
        self.cross_modal_enabled = cross_modal_enabled


@value
struct MultiVectorHybridResult[dtype: DType = DType.float32](Copyable):
    """
    Comprehensive result from multi-vector hybrid search.
    
    Contains detailed scoring information across all search modalities
    and vector types for transparency and debugging.
    """
    
    var documents: List[MultiVectorDocument[dtype]]    # Retrieved documents
    var document_scores: Dict[String, Float64]         # Final document scores
    var dense_scores: Dict[String, Dict[String, Float64]]  # doc_id -> {type: score}
    var sparse_scores: Dict[String, Float64]           # Document sparse scores
    var cross_modal_scores: Dict[String, Float64]     # Cross-modal fusion scores
    var ranking_metrics: Dict[String, Float64]        # Performance metrics
    
    fn __init__(out self):
        self.documents = List[MultiVectorDocument[dtype]]()
        self.document_scores = Dict[String, Float64]()
        self.dense_scores = Dict[String, Dict[String, Float64]]()
        self.sparse_scores = Dict[String, Float64]()
        self.cross_modal_scores = Dict[String, Float64]()
        self.ranking_metrics = Dict[String, Float64]()
    
    fn add_document(
        mut self,
        doc: MultiVectorDocument[dtype],
        final_score: Float64,
        type_scores: Dict[String, Float64],
        sparse_score: Float64 = 0.0,
        cross_modal_score: Float64 = 0.0
    ):
        """Add document with comprehensive scoring information."""
        self.documents.append(doc)
        self.document_scores[doc.id] = final_score
        self.dense_scores[doc.id] = type_scores
        self.sparse_scores[doc.id] = sparse_score
        self.cross_modal_scores[doc.id] = cross_modal_score
    
    fn size(self) -> Int:
        """Get number of documents in result."""
        return len(self.documents)
    
    fn get_top_documents(self, limit: Int) -> List[MultiVectorDocument[dtype]]:
        """Get top documents up to limit."""
        var result = List[MultiVectorDocument[dtype]]()
        for i in range(min(limit, len(self.documents))):
            result.append(self.documents[i])
        return result


struct MultiVectorHybridEngine[dtype: DType = DType.float32](Copyable, Movable):
    """
    Advanced hybrid search engine for multi-vector documents.
    
    This engine provides comprehensive search capabilities:
    - Multi-modal dense vector search across different vector types
    - Sparse keyword search in document metadata
    - Cross-modal similarity fusion
    - Advanced ranking with multiple fusion strategies
    - Performance optimization for <5ms query latency
    - Dual-mode compatibility (embedded/server)
    
    Architecture:
    - Multi-vector store for document storage and basic search
    - Multi-vector ranker for advanced fusion and ranking
    - Hybrid search engine for sparse keyword integration
    - Performance monitoring and optimization
    """
    
    var multi_vector_store: MultiVectorStore[dtype]
    var multi_vector_ranker: MultiVectorRanker[dtype]
    var logger: Logger
    var config: Dict[String, String]
    
    # Performance tracking
    var query_count: Int
    var total_query_time: Float64
    var cache_hit_rate: Float64
    
    # ========================================
    # Constructors and Configuration
    # ========================================
    
    fn __init__(
        out self,
        store: MultiVectorStore[dtype],
        ranker: Optional[MultiVectorRanker[dtype]] = None
    ):
        """Initialize multi-vector hybrid search engine."""
        self.multi_vector_store = store
        
        if ranker:
            self.multi_vector_ranker = ranker.value()
        else:
            self.multi_vector_ranker = MultiVectorRanker[dtype](RankingStrategy.ADAPTIVE)
        
        self.logger = Logger(LogLevel.INFO)
        self.config = Dict[String, String]()
        
        # Set default configuration
        self.config["dense_weight"] = "0.6"
        self.config["sparse_weight"] = "0.2" 
        self.config["cross_modal_weight"] = "0.2"
        self.config["max_candidates"] = "500"
        self.config["enable_caching"] = "true"
        self.config["parallel_search"] = "false"  # For server mode
        
        # Performance tracking
        self.query_count = 0
        self.total_query_time = 0.0
        self.cache_hit_rate = 0.0
    
    fn __copyinit__(out self, existing: Self):
        """Create copy of hybrid engine."""
        self.multi_vector_store = existing.multi_vector_store
        self.multi_vector_ranker = existing.multi_vector_ranker
        self.logger = existing.logger
        self.config = existing.config
        self.query_count = existing.query_count
        self.total_query_time = existing.total_query_time
        self.cache_hit_rate = existing.cache_hit_rate
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor for efficient transfers."""
        self.multi_vector_store = existing.multi_vector_store^
        self.multi_vector_ranker = existing.multi_vector_ranker^
        self.logger = existing.logger^
        self.config = existing.config^
        self.query_count = existing.query_count
        self.total_query_time = existing.total_query_time
        self.cache_hit_rate = existing.cache_hit_rate
    
    # ========================================
    # Configuration Methods
    # ========================================
    
    fn configure(mut self, key: String, value: String):
        """Configure engine parameters."""
        self.config[key] = value
        
        # Update ranker configuration if relevant
        if key == "ranking_strategy":
            self.multi_vector_ranker.default_strategy = value
        elif key == "similarity_threshold":
            try:
                var threshold = Float64(atol(value)) / 100.0
                self.multi_vector_ranker.set_similarity_threshold(threshold)
            except:
                pass
    
    fn add_vector_type_weight(mut self, vector_type: String, weight: Float64):
        """Set importance weight for specific vector type."""
        self.multi_vector_ranker.set_vector_type_weight(vector_type, weight)
    
    fn add_boosting_rule(mut self, rule: BoostingRule):
        """Add metadata-based boosting rule."""
        self.multi_vector_ranker.add_boosting_rule(rule)
    
    # ========================================
    # Core Search Methods
    # ========================================
    
    fn search(
        mut self,
        query: MultiVectorHybridQuery[dtype],
        limit: Int = 10
    ) raises -> MultiVectorHybridResult[dtype]:
        """
        Execute comprehensive multi-vector hybrid search.
        
        Args:
            query: Multi-vector hybrid query specification
            limit: Maximum number of results to return
            
        Returns:
            Comprehensive search results with detailed scoring
        """
        var start_time = 0  # TODO: Add timing when available
        
        # Phase 1: Dense vector search across modalities
        var dense_results = self._execute_dense_search(query, limit)
        
        # Phase 2: Sparse keyword search if keywords provided
        var sparse_results = self._execute_sparse_search(query, limit)
        
        # Phase 3: Cross-modal search if enabled
        var cross_modal_results = List[MultiVectorSearchResult[dtype]]()
        if query.cross_modal_enabled and len(query.dense_vectors) > 1:
            cross_modal_results = self._execute_cross_modal_search(query, limit)
        
        # Phase 4: Advanced fusion and ranking
        var final_result = self._fuse_and_rank_results(
            dense_results, sparse_results, cross_modal_results, query, limit
        )
        
        # Update performance metrics
        self.query_count += 1
        
        self.logger.debug(
            "Multi-vector hybrid search completed: " + 
            String(final_result.size()) + " results in query #" + 
            String(self.query_count)
        )
        
        return final_result
    
    fn search_by_document(
        mut self,
        reference_doc_id: String,
        vector_types: Optional[List[String]] = None,
        include_sparse: Bool = True,
        limit: Int = 10
    ) raises -> MultiVectorHybridResult[dtype]:
        """
        Search for documents similar to a reference document.
        
        Args:
            reference_doc_id: ID of reference document
            vector_types: Optional list of vector types to use
            include_sparse: Include sparse keyword matching
            limit: Maximum number of results
            
        Returns:
            Similar documents with comprehensive scoring
        """
        # Get reference document
        var doc_opt = self.multi_vector_store.get_document(reference_doc_id)
        if not doc_opt:
            raise Error("Reference document not found: " + reference_doc_id)
        
        var ref_doc = doc_opt.value()
        
        # Build query from reference document
        var query_vectors = Dict[String, Vector[dtype]]()
        var weights = Dict[String, Float64]()
        var keywords = List[String]()
        
        # Extract vectors from reference document
        var types_to_use = List[String]()
        if vector_types:
            types_to_use = vector_types.value()
        else:
            types_to_use = ref_doc.get_vector_types()
        
        for vector_type in types_to_use:
            if ref_doc.has_vector(vector_type):
                var entry = ref_doc.get_vector(vector_type)
                query_vectors[vector_type] = entry.vector
                weights[vector_type] = entry.weight
        
        # Extract keywords from metadata if sparse search enabled
        if include_sparse:
            try:
                var title = ref_doc.get_global_metadata("title")
                keywords.append(title)
            except:
                pass
            
            try:
                var content = ref_doc.get_global_metadata("content")
                keywords.append(content)
            except:
                pass
        
        # Create hybrid query
        var hybrid_query = MultiVectorHybridQuery[dtype](
            query_vectors, keywords, List[QueryFilter](), weights,
            RankingStrategy.ADAPTIVE, 0.0, True
        )
        
        # Execute search and filter out reference document
        var results = self.search(hybrid_query, limit + 1)
        return self._filter_reference_document(results, reference_doc_id, limit)
    
    # ========================================
    # Search Phase Implementations
    # ========================================
    
    fn _execute_dense_search(
        self,
        query: MultiVectorHybridQuery[dtype],
        limit: Int
    ) raises -> Dict[String, List[MultiVectorSearchResult[dtype]]]:
        """Execute dense vector search for each vector type."""
        var results_by_type = Dict[String, List[MultiVectorSearchResult[dtype]]]()
        var max_candidates = atol(self.config.get("max_candidates", "500"))
        
        for vector_type in query.dense_vectors.keys():
            var query_vector = query.dense_vectors[vector_type]
            
            # Create single-type cross-modal query
            var single_type_vectors = Dict[String, Vector[dtype]]()
            single_type_vectors[vector_type] = query_vector
            
            var single_type_weights = Dict[String, Float64]()
            if vector_type in query.vector_type_weights:
                single_type_weights[vector_type] = query.vector_type_weights[vector_type]
            else:
                single_type_weights[vector_type] = 1.0
            
            var cross_modal_query = CrossModalQuery[dtype](
                single_type_vectors, single_type_weights, "weighted_mean", 0.0
            )
            
            # Execute search for this vector type
            var type_results = self.multi_vector_store.cross_modal_search(
                cross_modal_query, max_candidates
            )
            
            results_by_type[vector_type] = type_results
        
        return results_by_type
    
    fn _execute_sparse_search(
        self,
        query: MultiVectorHybridQuery[dtype],
        limit: Int
    ) raises -> List[String]:
        """Execute sparse keyword search and return matching document IDs."""
        var matching_doc_ids = List[String]()
        
        if len(query.sparse_keywords) == 0:
            return matching_doc_ids
        
        # Simple keyword matching implementation
        # TODO: Implement proper sparse search with TF-IDF or BM25
        var all_doc_ids = self.multi_vector_store.list_documents(1000, 0)
        
        for doc_id in all_doc_ids:
            var doc_opt = self.multi_vector_store.get_document(doc_id)
            if not doc_opt:
                continue
            
            var doc = doc_opt.value()
            var keyword_matches = 0
            
            # Check keywords against metadata
            for keyword in query.sparse_keywords:
                try:
                    var title = doc.get_global_metadata("title")
                    if keyword in title:
                        keyword_matches += 1
                        continue
                except:
                    pass
                
                try:
                    var content = doc.get_global_metadata("content")
                    if keyword in content:
                        keyword_matches += 1
                except:
                    pass
            
            # If at least one keyword matches, include document
            if keyword_matches > 0:
                matching_doc_ids.append(doc_id)
        
        return matching_doc_ids
    
    fn _execute_cross_modal_search(
        self,
        query: MultiVectorHybridQuery[dtype],
        limit: Int
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """Execute cross-modal search across all vector types."""
        var cross_modal_query = CrossModalQuery[dtype](
            query.dense_vectors,
            query.vector_type_weights,
            query.ranking_strategy,
            query.similarity_threshold
        )
        
        var max_candidates = atol(self.config.get("max_candidates", "500"))
        return self.multi_vector_store.cross_modal_search(cross_modal_query, max_candidates)
    
    # ========================================
    # Fusion and Ranking
    # ========================================
    
    fn _fuse_and_rank_results(
        mut self,
        dense_results: Dict[String, List[MultiVectorSearchResult[dtype]]],
        sparse_doc_ids: List[String],
        cross_modal_results: List[MultiVectorSearchResult[dtype]],
        query: MultiVectorHybridQuery[dtype],
        limit: Int
    ) raises -> MultiVectorHybridResult[dtype]:
        """Fuse and rank all search results using advanced algorithms."""
        var final_result = MultiVectorHybridResult[dtype]()
        
        # Collect all unique document IDs
        var all_doc_ids = Dict[String, Bool]()
        
        # From dense results
        for vector_type in dense_results.keys():
            var type_results = dense_results[vector_type]
            for result in type_results:
                all_doc_ids[result.document_id] = True
        
        # From sparse results
        for doc_id in sparse_doc_ids:
            all_doc_ids[doc_id] = True
        
        # From cross-modal results
        for result in cross_modal_results:
            all_doc_ids[result.document_id] = True
        
        # Get configuration weights
        var dense_weight = Float64(atol(self.config.get("dense_weight", "60"))) / 100.0
        var sparse_weight = Float64(atol(self.config.get("sparse_weight", "20"))) / 100.0
        var cross_modal_weight = Float64(atol(self.config.get("cross_modal_weight", "20"))) / 100.0
        
        # Calculate final scores for each document
        var doc_final_scores = List[(String, Float64)]()
        
        for doc_id in all_doc_ids.keys():
            # Get document
            var doc_opt = self.multi_vector_store.get_document(doc_id)
            if not doc_opt:
                continue
            var doc = doc_opt.value()
            
            # Calculate dense score (weighted average across vector types)
            var dense_score = self._calculate_dense_score(doc_id, dense_results, query)
            
            # Calculate sparse score
            var sparse_score = 1.0 if doc_id in sparse_doc_ids else 0.0
            
            # Calculate cross-modal score
            var cross_modal_score = self._find_cross_modal_score(doc_id, cross_modal_results)
            
            # Combine scores
            var final_score = (dense_score * dense_weight) + 
                            (sparse_score * sparse_weight) + 
                            (cross_modal_score * cross_modal_weight)
            
            if final_score >= query.similarity_threshold:
                doc_final_scores.append((doc_id, final_score))
        
        # Sort by final score
        self._sort_doc_scores(doc_final_scores)
        
        # Build final result with top documents
        var result_limit = limit if limit < len(doc_final_scores) else len(doc_final_scores)
        for i in range(result_limit):
            var doc_id = doc_final_scores[i][0]
            var final_score = doc_final_scores[i][1]
            
            var doc_opt = self.multi_vector_store.get_document(doc_id)
            if doc_opt:
                var doc = doc_opt.value()
                var type_scores = self._get_type_scores(doc_id, dense_results)
                var sparse_score = 1.0 if doc_id in sparse_doc_ids else 0.0
                var cross_modal_score = self._find_cross_modal_score(doc_id, cross_modal_results)
                
                final_result.add_document(doc, final_score, type_scores, sparse_score, cross_modal_score)
        
        return final_result
    
    # ========================================
    # Utility Methods
    # ========================================
    
    fn _calculate_dense_score(
        self,
        doc_id: String,
        dense_results: Dict[String, List[MultiVectorSearchResult[dtype]]],
        query: MultiVectorHybridQuery[dtype]
    ) raises -> Float64:
        """Calculate weighted dense score across vector types."""
        var total_score = 0.0
        var total_weight = 0.0
        
        for vector_type in dense_results.keys():
            var type_results = dense_results[vector_type]
            var weight = 1.0
            if vector_type in query.vector_type_weights:
                weight = query.vector_type_weights[vector_type]
            
            # Find score for this document in this vector type
            for result in type_results:
                if result.document_id == doc_id:
                    total_score += result.similarity_score * weight
                    total_weight += weight
                    break
        
        return total_score / total_weight if total_weight > 0 else 0.0
    
    fn _find_cross_modal_score(
        self,
        doc_id: String,
        cross_modal_results: List[MultiVectorSearchResult[dtype]]
    ) -> Float64:
        """Find cross-modal score for document."""
        for result in cross_modal_results:
            if result.document_id == doc_id:
                return result.similarity_score
        return 0.0
    
    fn _get_type_scores(
        self,
        doc_id: String,
        dense_results: Dict[String, List[MultiVectorSearchResult[dtype]]]
    ) raises -> Dict[String, Float64]:
        """Get scores by vector type for document."""
        var type_scores = Dict[String, Float64]()
        
        for vector_type in dense_results.keys():
            var type_results = dense_results[vector_type]
            for result in type_results:
                if result.document_id == doc_id:
                    type_scores[vector_type] = result.similarity_score
                    break
        
        return type_scores
    
    fn _sort_doc_scores(self, mut doc_scores: List[(String, Float64)]):
        """Sort document scores in descending order."""
        # Simple bubble sort
        for i in range(len(doc_scores)):
            for j in range(len(doc_scores) - i - 1):
                if doc_scores[j][1] < doc_scores[j + 1][1]:
                    var temp = doc_scores[j]
                    doc_scores[j] = doc_scores[j + 1]
                    doc_scores[j + 1] = temp
    
    fn _filter_reference_document(
        self,
        results: MultiVectorHybridResult[dtype],
        reference_doc_id: String,
        limit: Int
    ) -> MultiVectorHybridResult[dtype]:
        """Filter out reference document from results."""
        var filtered_result = MultiVectorHybridResult[dtype]()
        var count = 0
        
        for doc in results.documents:
            if doc.id != reference_doc_id and count < limit:
                var type_scores = results.dense_scores[doc.id]
                var sparse_score = results.sparse_scores[doc.id]
                var cross_modal_score = results.cross_modal_scores[doc.id]
                var final_score = results.document_scores[doc.id]
                
                filtered_result.add_document(doc, final_score, type_scores, sparse_score, cross_modal_score)
                count += 1
        
        return filtered_result
    
    # ========================================
    # Performance Monitoring
    # ========================================
    
    fn get_performance_metrics(self) -> Dict[String, Float64]:
        """Get comprehensive performance metrics."""
        var metrics = Dict[String, Float64]()
        
        metrics["query_count"] = Float64(self.query_count)
        var divisor = 1 if self.query_count == 0 else self.query_count
        metrics["average_query_time"] = self.total_query_time / Float64(divisor)
        metrics["cache_hit_rate"] = self.cache_hit_rate
        
        # Get store statistics
        var store_stats = self.multi_vector_store.get_statistics()
        metrics["total_documents"] = Float64(store_stats.total_documents)
        metrics["total_vectors"] = Float64(store_stats.total_vectors)
        metrics["memory_usage_mb"] = Float64(store_stats.memory_usage) / (1024.0 * 1024.0)
        
        return metrics


# ========================================
# Factory Functions
# ========================================

fn create_text_image_hybrid_engine[dtype: DType = DType.float32](
    store: MultiVectorStore[dtype]
) -> MultiVectorHybridEngine[dtype]:
    """Create hybrid engine optimized for text-image search."""
    var ranker = MultiVectorRanker[dtype](RankingStrategy.WEIGHTED_MEAN)
    ranker.set_vector_type_weight(VectorType.TEXT, 0.6)
    ranker.set_vector_type_weight(VectorType.IMAGE, 0.4)
    
    var engine = MultiVectorHybridEngine[dtype](store, ranker)
    engine.configure("dense_weight", "70")
    engine.configure("sparse_weight", "20")
    engine.configure("cross_modal_weight", "10")
    
    return engine

fn create_multimodal_hybrid_engine[dtype: DType = DType.float32](
    store: MultiVectorStore[dtype]
) -> MultiVectorHybridEngine[dtype]:
    """Create hybrid engine for comprehensive multimodal search."""
    var ranker = MultiVectorRanker[dtype](RankingStrategy.ADAPTIVE)
    ranker.set_vector_type_weight(VectorType.TEXT, 0.25)
    ranker.set_vector_type_weight(VectorType.IMAGE, 0.25)
    ranker.set_vector_type_weight(VectorType.AUDIO, 0.25)
    ranker.set_vector_type_weight(VectorType.VIDEO, 0.25)
    
    var engine = MultiVectorHybridEngine[dtype](store, ranker)
    engine.configure("ranking_strategy", RankingStrategy.ADAPTIVE)
    engine.configure("dense_weight", "60")
    engine.configure("sparse_weight", "20")
    engine.configure("cross_modal_weight", "20")
    
    return engine


# ========================================
# Type Aliases
# ========================================

alias Float32MultiVectorHybridEngine = MultiVectorHybridEngine[DType.float32]
alias Float64MultiVectorHybridEngine = MultiVectorHybridEngine[DType.float64]
alias DefaultMultiVectorHybridEngine = Float32MultiVectorHybridEngine
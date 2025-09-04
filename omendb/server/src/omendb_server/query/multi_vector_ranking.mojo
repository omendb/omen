"""
Unified ranking system for multi-vector documents in OmenDB.

This module provides sophisticated ranking algorithms for cross-modal search,
combining similarities from different vector types with configurable fusion
strategies, boosting factors, and performance optimizations.

Dual-mode compatible: Supports both embedded and server deployments.
"""

from collections import List, Dict, Optional
from math import exp, log, sqrt
from algorithm import min, max
from algorithm import sort

from core.multi_vector_doc import MultiVectorDocument, MultiVectorEntry
from core.vector import Vector
from storage.multi_vector_store import MultiVectorSearchResult, CrossModalQuery


@value
struct RankingStrategy:
    """Enumeration of ranking strategies for multi-vector fusion."""
    
    alias WEIGHTED_MEAN = "weighted_mean"
    alias RRF = "reciprocal_rank_fusion"  # Reciprocal Rank Fusion
    alias CombSUM = "comb_sum"            # Combination by sum
    alias CombMNZ = "comb_mnz"            # Combination with match count
    alias LEARNED_FUSION = "learned_fusion"  # ML-based fusion
    alias ADAPTIVE = "adaptive"           # Context-aware fusion


@value
struct BoostingRule(Copyable):
    """
    Rule for boosting scores based on metadata or vector type.
    
    Attributes:
        condition_field: Metadata field to check
        condition_value: Value to match
        boost_factor: Multiplier for score (1.0 = no change)
        vector_type: Optional specific vector type to boost
    """
    
    var condition_field: String
    var condition_value: String
    var boost_factor: Float64
    var vector_type: Optional[String]
    
    fn __init__(
        out self,
        condition_field: String,
        condition_value: String,
        boost_factor: Float64,
        vector_type: Optional[String] = None
    ):
        self.condition_field = condition_field
        self.condition_value = condition_value
        self.boost_factor = max(0.0, boost_factor)  # Ensure non-negative
        self.vector_type = vector_type


@value 
struct RankingMetrics(Copyable):
    """Performance metrics for ranking operations."""
    
    var total_documents: Int
    var cross_modal_matches: Int
    var fusion_time_ms: Float64
    var average_confidence: Float64
    var type_coverage: Dict[String, Int]
    
    fn __init__(out self):
        self.total_documents = 0
        self.cross_modal_matches = 0
        self.fusion_time_ms = 0.0
        self.average_confidence = 0.0
        self.type_coverage = Dict[String, Int]()


struct MultiVectorRanker[dtype: DType = DType.float32](Copyable, Movable):
    """
    Unified ranking system for multi-vector documents.
    
    This ranker implements multiple fusion strategies for combining similarities
    from different vector types, with support for:
    - Cross-modal similarity fusion
    - Metadata-based boosting
    - Performance-optimized ranking algorithms
    - Adaptive strategies based on query context
    - Real-time ranking for <5ms latency targets
    
    Dual-mode design:
    - Embedded: Optimized for single-threaded, low-memory ranking
    - Server: Supports parallel ranking and caching for scale
    """
    
    var default_strategy: String
    var vector_type_weights: Dict[String, Float64]
    var boosting_rules: List[BoostingRule]  
    var similarity_threshold: Float64
    var max_results: Int
    var enable_normalization: Bool
    
    # Performance tracking
    var ranking_metrics: RankingMetrics
    
    # ========================================
    # Constructors and Configuration
    # ========================================
    
    fn __init__(
        out self,
        default_strategy: String = RankingStrategy.WEIGHTED_MEAN,
        similarity_threshold: Float64 = 0.0,
        max_results: Int = 100
    ):
        """Initialize multi-vector ranker with default configuration."""
        self.default_strategy = default_strategy
        self.vector_type_weights = Dict[String, Float64]()
        self.boosting_rules = List[BoostingRule]()
        self.similarity_threshold = similarity_threshold
        self.max_results = max_results
        self.enable_normalization = True
        self.ranking_metrics = RankingMetrics()
    
    fn __copyinit__(out self, existing: Self):
        """Create copy of ranker configuration."""
        self.default_strategy = existing.default_strategy
        self.vector_type_weights = existing.vector_type_weights
        self.boosting_rules = existing.boosting_rules
        self.similarity_threshold = existing.similarity_threshold
        self.max_results = existing.max_results
        self.enable_normalization = existing.enable_normalization
        self.ranking_metrics = existing.ranking_metrics
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor for efficient transfers."""
        self.default_strategy = existing.default_strategy
        self.vector_type_weights = existing.vector_type_weights^
        self.boosting_rules = existing.boosting_rules^
        self.similarity_threshold = existing.similarity_threshold
        self.max_results = existing.max_results
        self.enable_normalization = existing.enable_normalization
        self.ranking_metrics = existing.ranking_metrics^
    
    # ========================================
    # Configuration Methods
    # ========================================
    
    fn set_vector_type_weight(mut self, vector_type: String, weight: Float64):
        """Set importance weight for specific vector type."""
        self.vector_type_weights[vector_type] = max(0.0, weight)
    
    fn add_boosting_rule(mut self, rule: BoostingRule):
        """Add metadata-based boosting rule."""
        self.boosting_rules.append(rule)
    
    fn set_similarity_threshold(mut self, threshold: Float64):
        """Set minimum similarity threshold for results."""
        self.similarity_threshold = max(0.0, min(1.0, threshold))
    
    fn enable_score_normalization(mut self, enabled: Bool):
        """Enable/disable score normalization across vector types."""
        self.enable_normalization = enabled
    
    # ========================================
    # Core Ranking Methods
    # ========================================
    
    fn rank_cross_modal_results(
        mut self,
        results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]],
        query: CrossModalQuery[dtype],
        strategy: Optional[String] = None
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """
        Rank cross-modal search results using specified fusion strategy.
        
        Args:
            results: Initial search results from multi-vector store
            documents: Full document data for advanced ranking
            query: Original cross-modal query
            strategy: Optional ranking strategy override
            
        Returns:
            Re-ranked and filtered results
        """
        var start_time = 0  # TODO: Add timing when available
        
        var fusion_strategy = self.default_strategy
        if strategy:
            fusion_strategy = strategy.value()
        
        # Apply fusion strategy
        var ranked_results = List[MultiVectorSearchResult[dtype]]()
        
        if fusion_strategy == RankingStrategy.WEIGHTED_MEAN:
            ranked_results = self._rank_weighted_mean(results, documents, query)
        elif fusion_strategy == RankingStrategy.RRF:
            ranked_results = self._rank_reciprocal_rank_fusion(results, documents, query)
        elif fusion_strategy == RankingStrategy.CombSUM:
            ranked_results = self._rank_comb_sum(results, documents, query)
        elif fusion_strategy == RankingStrategy.CombMNZ:
            ranked_results = self._rank_comb_mnz(results, documents, query)
        elif fusion_strategy == RankingStrategy.ADAPTIVE:
            ranked_results = self._rank_adaptive(results, documents, query)
        else:
            # Default to weighted mean
            ranked_results = self._rank_weighted_mean(results, documents, query)
        
        # Apply boosting rules
        self._apply_boosting_rules(ranked_results, documents)
        
        # Filter by threshold and limit
        var final_results = self._filter_and_limit_results(ranked_results)
        
        # Update metrics
        self.ranking_metrics.total_documents = len(results)
        self.ranking_metrics.cross_modal_matches = len(final_results)
        
        return final_results
    
    fn compute_unified_similarity(
        self,
        doc: MultiVectorDocument[dtype],
        query_vectors: Dict[String, Vector[dtype]],
        strategy: String = RankingStrategy.WEIGHTED_MEAN
    ) raises -> Float64:
        """
        Compute unified similarity score between document and query vectors.
        
        Args:
            doc: Multi-vector document
            query_vectors: Query vectors by type
            strategy: Fusion strategy to use
            
        Returns:
            Unified similarity score
        """
        var type_similarities = Dict[String, Float64]()
        var type_weights = Dict[String, Float64]()
        
        # Calculate similarities for each matching vector type
        for query_type in query_vectors.keys():
            if doc.has_vector(query_type):
                var doc_entry = doc.get_vector(query_type)
                var query_vector = query_vectors[query_type]
                
                var similarity = doc_entry.vector.cosine_similarity(query_vector)
                type_similarities[query_type] = similarity
                
                # Get weight (document weight * global type weight)
                var doc_weight = doc_entry.weight
                var global_weight = 1.0
                if query_type in self.vector_type_weights:
                    global_weight = self.vector_type_weights[query_type]
                
                type_weights[query_type] = doc_weight * global_weight
        
        if len(type_similarities) == 0:
            return 0.0
        
        # Apply fusion strategy
        return self._fuse_similarities(type_similarities, type_weights, strategy)
    
    # ========================================
    # Fusion Strategy Implementations
    # ========================================
    
    fn _rank_weighted_mean(
        self,
        results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]],
        query: CrossModalQuery[dtype]
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """Weighted mean fusion of vector type similarities."""
        var ranked_results = List[MultiVectorSearchResult[dtype]]()
        
        for result in results:
            if result.document_id in documents:
                var doc = documents[result.document_id]
                var unified_score = self.compute_unified_similarity(
                    doc, query.query_vectors, RankingStrategy.WEIGHTED_MEAN
                )
                
                var new_result = result
                new_result.similarity_score = unified_score
                ranked_results.append(new_result)
        
        # Sort by score
        self._sort_results_by_score(ranked_results)
        return ranked_results
    
    fn _rank_reciprocal_rank_fusion(
        self,
        results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]],
        query: CrossModalQuery[dtype]
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """
        Reciprocal Rank Fusion (RRF) for combining rankings from different vector types.
        
        RRF Score = Σ(1 / (k + rank_i)) where k is a constant (usually 60)
        """
        var k = 60.0  # RRF constant
        var type_rankings = Dict[String, List[String]]()  # type -> [doc_id by rank]
        
        # Create rankings for each vector type
        for vector_type in query.query_vectors.keys():
            var type_results = List[MultiVectorSearchResult[dtype]]()
            
            # Filter results that have this vector type
            for result in results:
                if vector_type in result.type_scores:
                    var type_result = result
                    type_result.similarity_score = result.type_scores[vector_type]
                    type_results.append(type_result)
            
            # Sort by type-specific score
            self._sort_results_by_score(type_results)
            
            # Store ranking
            var ranking = List[String]()
            for type_result in type_results:
                ranking.append(type_result.document_id)
            type_rankings[vector_type] = ranking
        
        # Calculate RRF scores
        var rrf_scores = Dict[String, Float64]()
        for doc_id in [result.document_id for result in results]:
            var rrf_score = 0.0
            
            for vector_type in type_rankings.keys():
                var ranking = type_rankings[vector_type]
                var rank = self._find_rank(doc_id, ranking)
                if rank >= 0:
                    rrf_score += 1.0 / (k + rank + 1)  # +1 for 1-based ranking
            
            rrf_scores[doc_id] = rrf_score
        
        # Create final ranked results
        var ranked_results = List[MultiVectorSearchResult[dtype]]()
        for result in results:
            var new_result = result
            new_result.similarity_score = rrf_scores[result.document_id]
            ranked_results.append(new_result)
        
        self._sort_results_by_score(ranked_results)
        return ranked_results
    
    fn _rank_comb_sum(
        self,
        results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]],
        query: CrossModalQuery[dtype]
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """CombSUM: Simple sum of normalized scores across vector types."""
        var ranked_results = List[MultiVectorSearchResult[dtype]]()
        
        # Normalize scores by vector type first
        var type_max_scores = Dict[String, Float64]()
        for result in results:
            for vector_type in result.type_scores.keys():
                var score = result.type_scores[vector_type]
                if vector_type not in type_max_scores or score > type_max_scores[vector_type]:
                    type_max_scores[vector_type] = score
        
        # Calculate CombSUM scores
        for result in results:
            var comb_sum = 0.0
            var num_types = 0
            
            for vector_type in result.type_scores.keys():
                var raw_score = result.type_scores[vector_type]
                var max_score = type_max_scores[vector_type]
                
                # Normalize and weight
                var normalized_score = raw_score / max_score if max_score > 0 else 0.0
                var weight = 1.0
                if vector_type in self.vector_type_weights:
                    weight = self.vector_type_weights[vector_type]
                
                comb_sum += normalized_score * weight
                num_types += 1
            
            var new_result = result
            new_result.similarity_score = comb_sum
            ranked_results.append(new_result)
        
        self._sort_results_by_score(ranked_results)
        return ranked_results
    
    fn _rank_comb_mnz(
        self,
        results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]],
        query: CrossModalQuery[dtype]
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """CombMNZ: CombSUM multiplied by number of matching types."""
        var comb_sum_results = self._rank_comb_sum(results, documents, query)
        
        # Apply match count multiplier
        for i in range(len(comb_sum_results)):
            var result = comb_sum_results[i]
            var match_count = len(result.matching_vector_types)
            result.similarity_score *= match_count
            comb_sum_results[i] = result
        
        self._sort_results_by_score(comb_sum_results)
        return comb_sum_results
    
    fn _rank_adaptive(
        self,
        results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]],
        query: CrossModalQuery[dtype]
    ) raises -> List[MultiVectorSearchResult[dtype]]:
        """
        Adaptive ranking that chooses strategy based on query characteristics.
        
        Strategy selection based on:
        - Number of query vector types
        - Score distribution
        - Document coverage across types
        """
        var num_query_types = len(query.query_vectors)
        var avg_type_coverage = 0.0
        
        # Calculate average type coverage
        var total_coverage = 0
        for result in results:
            total_coverage += len(result.matching_vector_types)
        if len(results) > 0:
            avg_type_coverage = Float64(total_coverage) / Float64(len(results))
        
        # Choose strategy based on characteristics
        var chosen_strategy = RankingStrategy.WEIGHTED_MEAN
        
        if num_query_types > 3 and avg_type_coverage > 2.0:
            # High dimensionality - use RRF
            chosen_strategy = RankingStrategy.RRF
        elif avg_type_coverage < 1.5:
            # Low coverage - use CombMNZ to boost multi-type matches
            chosen_strategy = RankingStrategy.CombMNZ
        else:
            # Default to weighted mean
            chosen_strategy = RankingStrategy.WEIGHTED_MEAN
        
        # Apply chosen strategy
        if chosen_strategy == RankingStrategy.RRF:
            return self._rank_reciprocal_rank_fusion(results, documents, query)
        elif chosen_strategy == RankingStrategy.CombMNZ:
            return self._rank_comb_mnz(results, documents, query)
        else:
            return self._rank_weighted_mean(results, documents, query)
    
    # ========================================
    # Boosting and Post-processing
    # ========================================
    
    fn _apply_boosting_rules(
        self,
        mut results: List[MultiVectorSearchResult[dtype]],
        documents: Dict[String, MultiVectorDocument[dtype]]
    ):
        """Apply metadata-based boosting rules to results."""
        for i in range(len(results)):
            var result = results[i]
            
            if result.document_id in documents:
                var doc = documents[result.document_id]
                var boost_factor = 1.0
                
                # Apply each boosting rule
                for rule in self.boosting_rules:
                    try:
                        var metadata_value = doc.get_global_metadata(rule.condition_field)
                        if metadata_value == rule.condition_value:
                            # Check if rule applies to specific vector type
                            if rule.vector_type:
                                var target_type = rule.vector_type.value()
                                if target_type in result.matching_vector_types:
                                    boost_factor *= rule.boost_factor
                            else:
                                # Apply to overall score
                                boost_factor *= rule.boost_factor
                    except:
                        # Field doesn't exist, skip rule
                        pass
                
                # Apply boost
                result.similarity_score *= boost_factor
                results[i] = result
    
    fn _filter_and_limit_results(
        self,
        results: List[MultiVectorSearchResult[dtype]]
    ) -> List[MultiVectorSearchResult[dtype]]:
        """Filter results by threshold and apply limit."""
        var filtered_results = List[MultiVectorSearchResult[dtype]]()
        
        for result in results:
            if result.similarity_score >= self.similarity_threshold:
                filtered_results.append(result)
                
                if len(filtered_results) >= self.max_results:
                    break
        
        return filtered_results
    
    # ========================================
    # Utility Methods
    # ========================================
    
    fn _fuse_similarities(
        self,
        similarities: Dict[String, Float64],
        weights: Dict[String, Float64],
        strategy: String
    ) -> Float64:
        """Fuse individual similarities into unified score."""
        if len(similarities) == 0:
            return 0.0
        
        if strategy == RankingStrategy.WEIGHTED_MEAN:
            var weighted_sum = 0.0
            var weight_sum = 0.0
            
            for vector_type in similarities.keys():
                var sim = similarities[vector_type]
                var weight = weights[vector_type] if vector_type in weights else 1.0
                
                weighted_sum += sim * weight
                weight_sum += weight
            
            return weighted_sum / weight_sum if weight_sum > 0 else 0.0
        
        elif strategy == RankingStrategy.CombSUM:
            var sum = 0.0
            for sim in similarities.values():
                sum += sim
            return sum
        
        else:
            # Default weighted mean
            var weighted_sum = 0.0
            var weight_sum = 0.0
            
            for vector_type in similarities.keys():
                var sim = similarities[vector_type]
                var weight = weights[vector_type] if vector_type in weights else 1.0
                
                weighted_sum += sim * weight
                weight_sum += weight
            
            return weighted_sum / weight_sum if weight_sum > 0 else 0.0
    
    fn _sort_results_by_score(self, mut results: List[MultiVectorSearchResult[dtype]]):
        """Sort results by similarity score in descending order."""
        # Simple bubble sort for now
        for i in range(len(results)):
            for j in range(len(results) - i - 1):
                if results[j].similarity_score < results[j + 1].similarity_score:
                    var temp = results[j]
                    results[j] = results[j + 1]
                    results[j + 1] = temp
    
    fn _find_rank(self, doc_id: String, ranking: List[String]) -> Int:
        """Find document rank in a ranking list (-1 if not found)."""
        for i in range(len(ranking)):
            if ranking[i] == doc_id:
                return i
        return -1
    
    # ========================================
    # Performance Monitoring
    # ========================================
    
    fn get_ranking_metrics(self) -> RankingMetrics:
        """Get current ranking performance metrics."""
        return self.ranking_metrics
    
    fn reset_metrics(mut self):
        """Reset performance metrics."""
        self.ranking_metrics = RankingMetrics()


# ========================================
# Factory Functions and Utilities
# ========================================

fn create_text_image_ranker[dtype: DType = DType.float32]() -> MultiVectorRanker[dtype]:
    """Create ranker optimized for text-image cross-modal search."""
    var ranker = MultiVectorRanker[dtype](RankingStrategy.WEIGHTED_MEAN)
    
    # Set balanced weights for text and image
    ranker.set_vector_type_weight("text", 0.6)
    ranker.set_vector_type_weight("image", 0.4)
    
    return ranker

fn create_multimodal_ranker[dtype: DType = DType.float32]() -> MultiVectorRanker[dtype]:
    """Create ranker for comprehensive multimodal search."""
    var ranker = MultiVectorRanker[dtype](RankingStrategy.ADAPTIVE)
    
    # Set weights for different modalities
    ranker.set_vector_type_weight("text", 0.3)
    ranker.set_vector_type_weight("image", 0.3)
    ranker.set_vector_type_weight("audio", 0.2)
    ranker.set_vector_type_weight("video", 0.2)
    
    return ranker


# ========================================
# Type Aliases
# ========================================

alias Float32MultiVectorRanker = MultiVectorRanker[DType.float32]
alias Float64MultiVectorRanker = MultiVectorRanker[DType.float64]
alias DefaultMultiVectorRanker = Float32MultiVectorRanker
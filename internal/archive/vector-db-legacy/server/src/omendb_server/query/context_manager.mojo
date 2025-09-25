"""
Context Manager Module for OmenDB.

This module provides the context management functionality for OmenDB,
enabling efficient retrieval and selection of context snippets for
retrieval-augmented generation (RAG) applications.
"""

from collections import List, Dict, Optional
from core.vector import Vector
from core.context import ContextSnippet, VectorContext, ScoredSnippet, ContextResult, BudgetParams
from storage.context_store import ContextStore
from storage.vector_store import VectorStore, QueryFilter
from index.hnsw_index import HnswIndex
from util.logging import Logger, LogLevel

@value
struct ContextManager[dtype: DType]:
    """
    Manager for retrieving and selecting contextual information.
    
    This struct provides methods for retrieving context snippets associated
    with vectors and selecting the most relevant snippets within token budget
    constraints.
    
    Attributes:
        store: The vector store
        context_store: The context storage implementation
        index: The vector index
        logger: Logger for context manager operations
    """
    
    var store: VectorStore[dtype]
    var context_store: ContextStore[dtype] 
    var index: HnswIndex[dtype]
    var logger: Logger
    
    fn __init__(inout self, store: VectorStore[dtype], context_store: ContextStore[dtype], index: HnswIndex[dtype]):
        """
        Initialize the context manager.
        
        Args:
            store: The vector store
            context_store: The context storage implementation
            index: The vector index
        """
        self.store = store
        self.context_store = context_store
        self.index = index
        self.logger = Logger("ContextManager", LogLevel.INFO)
    
    fn get_context_for_id(self, vector_id: String) raises -> Optional[VectorContext[dtype]]:
        """
        Get context for a specific vector ID.
        
        Args:
            vector_id: The vector ID
            
        Returns:
            The vector context if found, None otherwise
        """
        self.logger.debug("Getting context for vector " + vector_id)
        return self.context_store.get_context(vector_id)
    
    fn get_nearest_context(self, query_vector: Vector[dtype], k: Int = 5, filters: Optional[List[QueryFilter]] = None) raises -> List[VectorContext[dtype]]:
        """
        Get context for the k nearest vectors to the query vector.
        
        Args:
            query_vector: The query vector
            k: Number of nearest neighbors to consider
            filters: Optional metadata filters
            
        Returns:
            List of vector contexts for the nearest vectors
        """
        self.logger.debug("Getting context for " + String(k) + " nearest vectors")
        
        var contexts = List[VectorContext[dtype]]()
        var results = self.index.search(query_vector, k)
        
        for result in results:
            var vector_id = result.id
            
            # Apply filters if provided
            if not filters.none() and filters.value().size() > 0:
                var record = self.store.get(vector_id)
                var passes_filters = True
                
                for filter in filters.value():
                    if not self._matches_filter(record.metadata, filter):
                        passes_filters = False
                        break
                
                if not passes_filters:
                    continue
            
            var context_opt = self.context_store.get_context(vector_id)
            if not context_opt.none():
                contexts.append(context_opt.value())
        
        return contexts
    
    fn search_context(self, query: String, filters: Optional[List[QueryFilter]] = None) raises -> List[ScoredSnippet]:
        """
        Search for context snippets matching a text query.
        
        Args:
            query: The text query
            filters: Optional metadata filters
            
        Returns:
            List of scored snippets matching the query
        """
        self.logger.debug("Searching context for query: " + query)
        return self.context_store.search_snippets(query, filters)
    
    fn select_context_for_vector(
        self, 
        vector_id: String, 
        budget: BudgetParams = BudgetParams()
    ) raises -> ContextResult:
        """
        Select context snippets for a specific vector within budget constraints.
        
        Args:
            vector_id: The vector ID
            budget: Budget parameters for token and snippet limits
            
        Returns:
            Context result with selected snippets
        """
        self.logger.debug("Selecting context for vector " + vector_id + " with token budget " + String(budget.max_tokens))
        
        var context_opt = self.context_store.get_context(vector_id)
        if context_opt.none():
            return ContextResult()
        
        var context = context_opt.value()
        var result = ContextResult()
        
        # Score snippets (in this simple implementation, all have the same score)
        var scored_snippets = List[ScoredSnippet]()
        for snippet in context.snippets:
            scored_snippets.append(ScoredSnippet(snippet, 1.0))
        
        # Sort by score
        for i in range(1, scored_snippets.size()):
            var j = i
            var temp = scored_snippets[i]
            
            while j > 0 and scored_snippets[j-1].score < temp.score:
                scored_snippets[j] = scored_snippets[j-1]
                j -= 1
            
            scored_snippets[j] = temp
        
        # Select within budget
        var tokens_used = 0
        var snippets_used = 0
        
        for scored_snippet in scored_snippets:
            if scored_snippet.score < budget.min_relevance:
                continue
            
            if snippets_used >= budget.max_snippets:
                break
            
            if tokens_used + scored_snippet.snippet.token_count > budget.max_tokens:
                continue
            
            result.add_snippet(scored_snippet.snippet, scored_snippet.score)
            tokens_used += scored_snippet.snippet.token_count
            snippets_used += 1
        
        return result
    
    fn select_context_for_query(
        self, 
        query_vector: Vector[dtype], 
        k: Int = 5, 
        budget: BudgetParams = BudgetParams(),
        filters: Optional[List[QueryFilter]] = None
    ) raises -> ContextResult:
        """
        Select context snippets for a query vector within budget constraints.
        
        Args:
            query_vector: The query vector
            k: Number of nearest neighbors to consider
            budget: Budget parameters for token and snippet limits
            filters: Optional metadata filters
            
        Returns:
            Context result with selected snippets
        """
        self.logger.debug("Selecting context for query vector with token budget " + String(budget.max_tokens))
        
        var contexts = self.get_nearest_context(query_vector, k, filters)
        var all_snippets = List[ScoredSnippet]()
        
        # Collect all snippets with vector similarity scores
        var results = self.index.search(query_vector, k)
        var scores_by_id = Dict[String, Float32]()
        
        for result in results:
            scores_by_id[result.id] = result.score
        
        for context in contexts:
            var vector_score = scores_by_id[context.vector_id] if context.vector_id in scores_by_id else 0.5
            
            for snippet in context.snippets:
                # Score is based on vector similarity
                all_snippets.append(ScoredSnippet(snippet, vector_score))
        
        # Sort by score
        for i in range(1, all_snippets.size()):
            var j = i
            var temp = all_snippets[i]
            
            while j > 0 and all_snippets[j-1].score < temp.score:
                all_snippets[j] = all_snippets[j-1]
                j -= 1
            
            all_snippets[j] = temp
        
        # Select within budget
        var result = ContextResult()
        var tokens_used = 0
        var snippets_used = 0
        
        for scored_snippet in all_snippets:
            if scored_snippet.score < budget.min_relevance:
                continue
            
            if snippets_used >= budget.max_snippets:
                break
            
            if tokens_used + scored_snippet.snippet.token_count > budget.max_tokens:
                continue
            
            result.add_snippet(scored_snippet.snippet, scored_snippet.score)
            tokens_used += scored_snippet.snippet.token_count
            snippets_used += 1
        
        return result
    
    fn select_context_hybrid(
        self, 
        query_vector: Vector[dtype], 
        query_text: String, 
        k: Int = 5, 
        budget: BudgetParams = BudgetParams(),
        filters: Optional[List[QueryFilter]] = None,
        vector_weight: Float32 = 0.7,
        text_weight: Float32 = 0.3
    ) raises -> ContextResult:
        """
        Select context using both vector similarity and text search.
        
        Args:
            query_vector: The query vector
            query_text: The text query
            k: Number of nearest neighbors to consider
            budget: Budget parameters for token and snippet limits
            filters: Optional metadata filters
            vector_weight: Weight for vector similarity (0.0-1.0)
            text_weight: Weight for text matching (0.0-1.0)
            
        Returns:
            Context result with selected snippets
        """
        self.logger.debug("Selecting context using hybrid method with token budget " + String(budget.max_tokens))
        
        # Get context from vector similarity
        var vector_contexts = self.get_nearest_context(query_vector, k, filters)
        
        # Get context from text search
        var text_snippets = self.search_context(query_text, filters)
        
        # Build a map of snippet IDs to hybrid scores
        var hybrid_scores = Dict[String, Float32]()
        var all_snippets = Dict[String, ContextSnippet]()
        
        # Process vector results
        for context in vector_contexts:
            var vector_id = context.vector_id
            var results = self.index.search(query_vector, k)
            var vector_score = 0.0
            
            for result in results:
                if result.id == vector_id:
                    vector_score = result.score
                    break
            
            for snippet in context.snippets:
                var snippet_key = vector_id + "_" + snippet.id
                all_snippets[snippet_key] = snippet
                hybrid_scores[snippet_key] = vector_score * vector_weight
            }
        
        # Process text results
        for scored_snippet in text_snippets:
            var snippet = scored_snippet.snippet
            var vector_context_opt = self.context_store.get_context_for_snippet(snippet.id)
            
            if vector_context_opt.none():
                continue
            
            var vector_id = vector_context_opt.value().vector_id
            var snippet_key = vector_id + "_" + snippet.id
            
            all_snippets[snippet_key] = snippet
            
            if snippet_key in hybrid_scores:
                hybrid_scores[snippet_key] += scored_snippet.score * text_weight
            else:
                hybrid_scores[snippet_key] = scored_snippet.score * text_weight
        
        # Create a list of scored snippets with hybrid scores
        var scored_snippets = List[ScoredSnippet]()
        
        for key in all_snippets.keys():
            var snippet = all_snippets[key]
            var score = hybrid_scores[key]
            scored_snippets.append(ScoredSnippet(snippet, score))
        
        # Sort by score
        for i in range(1, scored_snippets.size()):
            var j = i
            var temp = scored_snippets[i]
            
            while j > 0 and scored_snippets[j-1].score < temp.score:
                scored_snippets[j] = scored_snippets[j-1]
                j -= 1
            
            scored_snippets[j] = temp
        
        # Select within budget
        var result = ContextResult()
        var tokens_used = 0
        var snippets_used = 0
        
        for scored_snippet in scored_snippets:
            if scored_snippet.score < budget.min_relevance:
                continue
            
            if snippets_used >= budget.max_snippets:
                break
            
            if tokens_used + scored_snippet.snippet.token_count > budget.max_tokens:
                continue
            
            result.add_snippet(scored_snippet.snippet, scored_snippet.score)
            tokens_used += scored_snippet.snippet.token_count
            snippets_used += 1
        
        return result
    
    fn _matches_filter(self, metadata: Dict[String, String], filter: QueryFilter) -> Bool:
        """Check if metadata matches a filter."""
        if filter.field not in metadata:
            return False
        
        var value = metadata[filter.field]
        
        if filter.op == QueryFilter.OP_EQ:
            return value == filter.value
        elif filter.op == QueryFilter.OP_NEQ:
            return value != filter.value
        elif filter.op == QueryFilter.OP_GT:
            return value > filter.value
        elif filter.op == QueryFilter.OP_GTE:
            return value >= filter.value
        elif filter.op == QueryFilter.OP_LT:
            return value < filter.value
        elif filter.op == QueryFilter.OP_LTE:
            return value <= filter.value
        elif filter.op == QueryFilter.OP_CONTAINS:
            return value.contains(filter.value)
        
        return False
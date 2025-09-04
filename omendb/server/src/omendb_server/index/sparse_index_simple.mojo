"""
Simplified Sparse Vector Index implementation for OmenDB.

This module provides a working sparse vector index with basic functionality
while avoiding complex copying and move semantics issues.
"""

from collections import Dict, List
from math import sqrt, log
from core.sparse_vector import SparseVector
from core.distance import BM25Plus, TfIdfScorer
from core.metadata import Metadata
from util.logging import Logger, LogLevel


struct SimpleSparseSearchResult(Copyable, Movable):
    """Result from sparse vector search with scoring information."""
    
    var document_id: String
    var score: Float64
    var matched_terms: Int
    
    fn __init__(out self, document_id: String, score: Float64, matched_terms: Int = 0):
        self.document_id = document_id
        self.score = score
        self.matched_terms = matched_terms
    
    fn __copyinit__(out self, existing: Self):
        self.document_id = existing.document_id
        self.score = existing.score
        self.matched_terms = existing.matched_terms
    
    fn __moveinit__(out self, owned existing: Self):
        self.document_id = existing.document_id^
        self.score = existing.score
        self.matched_terms = existing.matched_terms


struct SimplePostingList(Copyable, Movable):
    """Simple posting list for term indexing."""
    
    var document_ids: List[String]
    var term_frequencies: List[Float32]
    
    fn __init__(out self):
        self.document_ids = List[String]()
        self.term_frequencies = List[Float32]()
    
    fn __copyinit__(out self, existing: Self):
        self.document_ids = existing.document_ids
        self.term_frequencies = existing.term_frequencies
    
    fn __moveinit__(out self, owned existing: Self):
        self.document_ids = existing.document_ids^
        self.term_frequencies = existing.term_frequencies^
    
    fn add_document(mut self, document_id: String, term_frequency: Float32):
        """Add document to posting list."""
        # Check if document already exists
        for i in range(len(self.document_ids)):
            if self.document_ids[i] == document_id:
                # Update existing frequency
                self.term_frequencies[i] = term_frequency
                return
        
        # Add new document
        self.document_ids.append(document_id)
        self.term_frequencies.append(term_frequency)
    
    fn remove_document(mut self, document_id: String) -> Bool:
        """Remove document from posting list."""
        for i in range(len(self.document_ids)):
            if self.document_ids[i] == document_id:
                # Remove at position i by creating new lists
                var new_ids = List[String]()
                var new_freqs = List[Float32]()
                
                for j in range(len(self.document_ids)):
                    if j != i:
                        new_ids.append(self.document_ids[j])
                        new_freqs.append(self.term_frequencies[j])
                
                self.document_ids = new_ids
                self.term_frequencies = new_freqs
                return True
        return False
    
    fn get_term_frequency(self, document_id: String) -> Float32:
        """Get term frequency for document."""
        for i in range(len(self.document_ids)):
            if self.document_ids[i] == document_id:
                return self.term_frequencies[i]
        return 0.0
    
    fn size(self) -> Int:
        """Get number of documents in posting list."""
        return len(self.document_ids)


struct SimpleSparseIndex(Copyable, Movable):
    """
    Simplified sparse vector index for keyword and text search.
    
    This implementation provides core sparse indexing functionality with:
    - Basic inverted index for term lookup
    - BM25+ scoring for text retrieval
    - Document frequency tracking
    - Simple search operations
    """
    
    var posting_lists: Dict[Int, SimplePostingList]
    var document_vectors: Dict[String, SparseVector]
    var document_lengths: Dict[String, Float64]
    var document_metadata: Dict[String, Metadata]
    var document_frequencies: Dict[Int, Int]
    var total_documents: Int
    var average_document_length: Float64
    var dimension: Int
    
    # BM25+ parameters - stored as values to avoid copy issues
    var bm25_k1: Float32
    var bm25_b: Float32
    var bm25_delta: Float32
    
    fn __init__(
        out self,
        dimension: Int,
        bm25_k1: Float32 = 1.2,
        bm25_b: Float32 = 0.75,
        bm25_delta: Float32 = 1.0
    ):
        """Initialize simplified sparse index."""
        self.dimension = max(dimension, 1)
        self.posting_lists = Dict[Int, SimplePostingList]()
        self.document_vectors = Dict[String, SparseVector]()
        self.document_lengths = Dict[String, Float64]()
        self.document_metadata = Dict[String, Metadata]()
        self.document_frequencies = Dict[Int, Int]()
        self.total_documents = 0
        self.average_document_length = 0.0
        
        self.bm25_k1 = bm25_k1
        self.bm25_b = bm25_b
        self.bm25_delta = bm25_delta
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.posting_lists = existing.posting_lists
        self.document_vectors = existing.document_vectors
        self.document_lengths = existing.document_lengths
        self.document_metadata = existing.document_metadata
        self.document_frequencies = existing.document_frequencies
        self.total_documents = existing.total_documents
        self.average_document_length = existing.average_document_length
        self.dimension = existing.dimension
        self.bm25_k1 = existing.bm25_k1
        self.bm25_b = existing.bm25_b
        self.bm25_delta = existing.bm25_delta
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.posting_lists = existing.posting_lists^
        self.document_vectors = existing.document_vectors^
        self.document_lengths = existing.document_lengths^
        self.document_metadata = existing.document_metadata^
        self.document_frequencies = existing.document_frequencies^
        self.total_documents = existing.total_documents
        self.average_document_length = existing.average_document_length
        self.dimension = existing.dimension
        self.bm25_k1 = existing.bm25_k1
        self.bm25_b = existing.bm25_b
        self.bm25_delta = existing.bm25_delta
    
    fn add_document(mut self, document_id: String, sparse_vector: SparseVector, metadata: Metadata = Metadata()) raises:
        """Add document to the index."""
        if document_id in self.document_vectors:
            raise Error("Document ID already exists: " + document_id)
        
        if sparse_vector.dimension_size() != self.dimension:
            raise Error("Vector dimension mismatch")
        
        # Calculate document length
        var doc_length = Float64(0.0)
        for i in range(sparse_vector.nnz()):
            doc_length += Float64(sparse_vector.values[i])
        
        # Add to posting lists and track document frequencies
        for i in range(sparse_vector.nnz()):
            var term_idx = sparse_vector.indices[i]
            var term_freq = sparse_vector.values[i]
            
            # Create posting list if it doesn't exist
            if term_idx not in self.posting_lists:
                self.posting_lists[term_idx] = SimplePostingList()
                self.document_frequencies[term_idx] = 0
            
            # Add document to posting list
            var was_new_doc = not self.posting_lists[term_idx].get_term_frequency(document_id) > 0.0
            self.posting_lists[term_idx].add_document(document_id, term_freq)
            
            # Update document frequency if this is a new document for this term
            if was_new_doc:
                self.document_frequencies[term_idx] += 1
        
        # Store document data
        self.document_vectors[document_id] = sparse_vector
        self.document_lengths[document_id] = doc_length
        self.document_metadata[document_id] = metadata
        
        # Update statistics
        self.total_documents += 1
        self._update_average_document_length()
    
    fn remove_document(mut self, document_id: String) raises:
        """Remove document from the index."""
        if document_id not in self.document_vectors:
            raise Error("Document ID not found: " + document_id)
        
        var vector = self.document_vectors[document_id]
        
        # Remove from posting lists and update document frequencies
        for i in range(vector.nnz()):
            var term_idx = vector.indices[i]
            
            if term_idx in self.posting_lists:
                var removed = self.posting_lists[term_idx].remove_document(document_id)
                if removed:
                    self.document_frequencies[term_idx] -= 1
                    
                    # Remove empty posting list
                    if self.posting_lists[term_idx].size() == 0:
                        _ = self.posting_lists.pop(term_idx)
                        _ = self.document_frequencies.pop(term_idx)
        
        # Remove document data
        _ = self.document_vectors.pop(document_id)
        _ = self.document_lengths.pop(document_id)
        if document_id in self.document_metadata:
            _ = self.document_metadata.pop(document_id)
        
        # Update statistics
        self.total_documents -= 1
        self._update_average_document_length()
    
    fn contains_document(self, document_id: String) -> Bool:
        """Check if document exists in the index."""
        return document_id in self.document_vectors
    
    fn search_bm25(self, query: SparseVector, k: Int = 10) raises -> List[SimpleSparseSearchResult]:
        """Search using BM25+ scoring."""
        var results = List[SimpleSparseSearchResult]()
        
        # Get candidate documents
        var candidates = Dict[String, Bool]()
        for i in range(query.nnz()):
            var term_idx = query.indices[i]
            if term_idx in self.posting_lists:
                var posting_list = self.posting_lists[term_idx]
                for doc_id in posting_list.document_ids:
                    candidates[doc_id] = True
        
        # Score each candidate
        for item in candidates.items():
            var doc_id = item.key
            var doc_vector = self.document_vectors[doc_id]
            var doc_length = Float32(self.document_lengths[doc_id])
            var score = Float64(0.0)
            var matched_terms = 0
            
            # Calculate BM25+ score for each query term
            for i in range(query.nnz()):
                var term_idx = query.indices[i]
                var query_tf = query.values[i]
                var doc_tf = doc_vector.get_value(term_idx)
                
                if doc_tf > 0.0:
                    matched_terms += 1
                    var df = Float32(self._get_document_frequency(term_idx))
                    var total_docs = Float32(self.total_documents)
                    
                    # BM25+ calculation
                    var idf = log((total_docs + 1.0) / (df + 1.0))
                    var length_norm = 1.0 - self.bm25_b + self.bm25_b * (doc_length / Float32(self.average_document_length))
                    var tf_component = (doc_tf * (self.bm25_k1 + 1.0)) / (doc_tf + self.bm25_k1 * length_norm)
                    var term_score = (tf_component + self.bm25_delta) * idf
                    
                    score += Float64(Float32(query_tf) * term_score)
            
            if score > 0.0:
                results.append(SimpleSparseSearchResult(doc_id, score, matched_terms))
        
        # Sort results by score (simple bubble sort)
        self._sort_results_by_score(results)
        
        # Return top k results
        var top_results = List[SimpleSparseSearchResult]()
        var limit = min(k, len(results))
        for i in range(limit):
            top_results.append(results[i])
        
        return top_results
    
    fn search_cosine(self, query: SparseVector, k: Int = 10) raises -> List[SimpleSparseSearchResult]:
        """Search using cosine similarity."""
        var results = List[SimpleSparseSearchResult]()
        
        # Get candidate documents
        var candidates = Dict[String, Bool]()
        for i in range(query.nnz()):
            var term_idx = query.indices[i]
            if term_idx in self.posting_lists:
                var posting_list = self.posting_lists[term_idx]
                for doc_id in posting_list.document_ids:
                    candidates[doc_id] = True
        
        # Score each candidate
        for item in candidates.items():
            var doc_id = item.key
            var doc_vector = self.document_vectors[doc_id]
            var score = query.cosine_similarity(doc_vector)
            var matched_terms = self._count_matching_terms(query, doc_vector)
            
            if score > 0.0:
                results.append(SimpleSparseSearchResult(doc_id, score, matched_terms))
        
        # Sort results by score
        self._sort_results_by_score(results)
        
        # Return top k results
        var top_results = List[SimpleSparseSearchResult]()
        var limit = min(k, len(results))
        for i in range(limit):
            top_results.append(results[i])
        
        return top_results
    
    fn get_document_count(self) -> Int:
        """Get total number of documents."""
        return self.total_documents
    
    fn get_vocabulary_size(self) -> Int:
        """Get vocabulary size."""
        return len(self.posting_lists)
    
    fn get_average_document_length(self) -> Float64:
        """Get average document length."""
        return self.average_document_length
    
    # Private helper methods
    
    fn _update_average_document_length(mut self):
        """Update average document length statistic."""
        if self.total_documents == 0:
            self.average_document_length = 0.0
            return
        
        var total_length = Float64(0.0)
        for item in self.document_lengths.items():
            total_length += item.value
        
        self.average_document_length = total_length / Float64(self.total_documents)
    
    fn _get_document_frequency(self, term_idx: Int) raises -> Int:
        """Get document frequency for a term."""
        if term_idx in self.document_frequencies:
            return self.document_frequencies[term_idx]
        return 0
    
    fn _sort_results_by_score(self, mut results: List[SimpleSparseSearchResult]):
        """Sort search results by score in descending order."""
        var n = len(results)
        for i in range(n):
            for j in range(0, n - i - 1):
                if results[j].score < results[j + 1].score:
                    # Swap results
                    var temp = results[j]
                    results[j] = results[j + 1]
                    results[j + 1] = temp
    
    fn _count_matching_terms(self, query: SparseVector, document: SparseVector) -> Int:
        """Count number of matching terms between query and document."""
        var matches = 0
        var i = 0
        var j = 0
        
        # Two-pointer technique for sorted sparse vectors
        while i < query.nnz() and j < document.nnz():
            if query.indices[i] == document.indices[j]:
                matches += 1
                i += 1
                j += 1
            elif query.indices[i] < document.indices[j]:
                i += 1
            else:
                j += 1
        
        return matches
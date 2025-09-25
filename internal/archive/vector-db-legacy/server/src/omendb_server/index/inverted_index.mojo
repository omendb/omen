"""
Inverted Index implementation for OmenDB sparse vector indexing.

This module provides the core inverted index data structure used by the sparse
index for efficient term-to-document mapping and fast candidate retrieval.
"""

from collections import Dict, List
from memory import UnsafePointer
from algorithm import vectorize
from math import sqrt, log
from core.sparse_vector import SparseVector
from util.logging import Logger, LogLevel


struct PostingList(Copyable, Movable):
    """Posting list for a specific term containing document IDs and frequencies."""
    
    var document_ids: List[String]
    var term_frequencies: List[Float32]
    var document_count: Int
    
    fn __init__(out self):
        """Initialize empty posting list."""
        self.document_ids = List[String]()
        self.term_frequencies = List[Float32]()
        self.document_count = 0
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.document_ids = existing.document_ids
        self.term_frequencies = existing.term_frequencies
        self.document_count = existing.document_count
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.document_ids = existing.document_ids^
        self.term_frequencies = existing.term_frequencies^
        self.document_count = existing.document_count
    
    fn add_document(mut self, document_id: String, term_frequency: Float32):
        """Add document to posting list.
        
        Args:
            document_id: Document identifier
            term_frequency: Frequency of term in document
        """
        # Check if document already exists
        for i in range(len(self.document_ids)):
            if self.document_ids[i] == document_id:
                # Update existing frequency
                self.term_frequencies[i] = term_frequency
                return
        
        # Add new document
        self.document_ids.append(document_id)
        self.term_frequencies.append(term_frequency)
        self.document_count += 1
    
    fn remove_document(mut self, document_id: String) -> Bool:
        """Remove document from posting list.
        
        Args:
            document_id: Document identifier to remove
            
        Returns:
            True if document was found and removed
        """
        for i in range(len(self.document_ids)):
            if self.document_ids[i] == document_id:
                # Remove document at position i
                var new_ids = List[String]()
                var new_freqs = List[Float32]()
                
                for j in range(len(self.document_ids)):
                    if j != i:
                        new_ids.append(self.document_ids[j])
                        new_freqs.append(self.term_frequencies[j])
                
                self.document_ids = new_ids
                self.term_frequencies = new_freqs
                self.document_count -= 1
                return True
        
        return False
    
    fn contains_document(self, document_id: String) -> Bool:
        """Check if document is in posting list.
        
        Args:
            document_id: Document identifier to check
            
        Returns:
            True if document is in posting list
        """
        for doc_id in self.document_ids:
            if doc_id == document_id:
                return True
        return False
    
    fn get_term_frequency(self, document_id: String) -> Float32:
        """Get term frequency for specific document.
        
        Args:
            document_id: Document identifier
            
        Returns:
            Term frequency, or 0.0 if document not found
        """
        for i in range(len(self.document_ids)):
            if self.document_ids[i] == document_id:
                return self.term_frequencies[i]
        return 0.0
    
    fn is_empty(self) -> Bool:
        """Check if posting list is empty.
        
        Returns:
            True if no documents in posting list
        """
        return self.document_count == 0
    
    fn size(self) -> Int:
        """Get number of documents in posting list.
        
        Returns:
            Number of documents
        """
        return self.document_count
    
    fn memory_footprint(self) -> Int:
        """Estimate memory footprint in bytes.
        
        Returns:
            Approximate memory usage
        """
        var base_size = 16  # Struct overhead
        var ids_size = 0
        for doc_id in self.document_ids:
            ids_size += len(doc_id) + 8  # String overhead
        var freqs_size = len(self.term_frequencies) * 4  # Float32 is 4 bytes
        return base_size + ids_size + freqs_size


struct DocumentFrequency(Copyable, Movable):
    """Document frequency tracking for terms."""
    
    var term_counts: Dict[Int, Int]
    var total_documents: Int
    
    fn __init__(out self):
        """Initialize empty document frequency tracker."""
        self.term_counts = Dict[Int, Int]()
        self.total_documents = 0
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.term_counts = existing.term_counts
        self.total_documents = existing.total_documents
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.term_counts = existing.term_counts^
        self.total_documents = existing.total_documents
    
    fn add_document_terms(mut self, terms: List[Int]):
        """Add terms from a new document.
        
        Args:
            terms: List of term indices in the document
        """
        var unique_terms = Dict[Int, Bool]()
        
        # Get unique terms in document
        for term in terms:
            unique_terms[term] = True
        
        # Increment document frequency for each unique term
        for item in unique_terms.items():
            var term = item.key
            if term in self.term_counts:
                self.term_counts[term] += 1
            else:
                self.term_counts[term] = 1
        
        self.total_documents += 1
    
    fn remove_document_terms(mut self, terms: List[Int]):
        """Remove terms from a deleted document.
        
        Args:
            terms: List of term indices in the document
        """
        var unique_terms = Dict[Int, Bool]()
        
        # Get unique terms in document
        for term in terms:
            unique_terms[term] = True
        
        # Decrement document frequency for each unique term
        for item in unique_terms.items():
            var term = item.key
            if term in self.term_counts:
                self.term_counts[term] -= 1
                if self.term_counts[term] <= 0:
                    self.term_counts.pop(term)
        
        self.total_documents -= 1
    
    fn get_document_frequency(self, term: Int) -> Int:
        """Get document frequency for a term.
        
        Args:
            term: Term index
            
        Returns:
            Number of documents containing the term
        """
        if term in self.term_counts:
            return self.term_counts[term]
        return 0
    
    fn get_total_documents(self) -> Int:
        """Get total number of documents.
        
        Returns:
            Total document count
        """
        return self.total_documents
    
    fn get_vocabulary_size(self) -> Int:
        """Get vocabulary size (number of unique terms).
        
        Returns:
            Number of unique terms
        """
        return len(self.term_counts)


struct InvertedIndex(Copyable, Movable):
    """
    High-performance inverted index for sparse vector indexing.
    
    This implementation provides:
    - Efficient term-to-document mapping
    - Fast candidate document retrieval
    - Document frequency tracking for IDF calculations
    - Memory-efficient storage for large vocabularies
    - Real-time index updates
    
    Architecture:
    - Dictionary mapping term indices to posting lists
    - Document frequency tracking for scoring
    - Optimized for sparse vector operations
    """
    
    var posting_lists: Dict[Int, PostingList]
    var document_frequency: DocumentFrequency
    var dimension: Int
    var logger: Logger
    
    fn __init__(out self, dimension: Int, log_level: Int = LogLevel.INFO):
        """Initialize inverted index.
        
        Args:
            dimension: Maximum vocabulary size
            log_level: Logging level for index operations
        """
        self.dimension = max(dimension, 1)
        self.posting_lists = Dict[Int, PostingList]()
        self.document_frequency = DocumentFrequency()
        self.logger = Logger(log_level)
        
        self.logger.debug("Initialized InvertedIndex with dimension: " + String(dimension))
    
    fn __copyinit__(out self, existing: Self):
        """Copy constructor."""
        self.posting_lists = existing.posting_lists
        self.document_frequency = existing.document_frequency
        self.dimension = existing.dimension
        self.logger = existing.logger
    
    fn __moveinit__(out self, owned existing: Self):
        """Move constructor."""
        self.posting_lists = existing.posting_lists^
        self.document_frequency = existing.document_frequency^
        self.dimension = existing.dimension
        self.logger = existing.logger^
    
    # ========================================
    # Document Management
    # ========================================
    
    fn add_document(mut self, document_id: String, sparse_vector: SparseVector):
        """Add document to inverted index.
        
        Args:
            document_id: Unique document identifier
            sparse_vector: Sparse vector representation of document
        """
        var terms = List[Int]()
        
        # Add document to posting lists for each term
        for i in range(sparse_vector.nnz()):
            var term_idx = sparse_vector.indices[i]
            var term_freq = sparse_vector.values[i]
            
            if term_idx >= 0 and term_idx < self.dimension:
                # Create posting list if it doesn't exist
                if term_idx not in self.posting_lists:
                    self.posting_lists[term_idx] = PostingList()
                
                # Add document to posting list
                self.posting_lists[term_idx].add_document(document_id, term_freq)
                terms.append(term_idx)
        
        # Update document frequency tracking
        self.document_frequency.add_document_terms(terms)
        
        self.logger.debug("Added document " + document_id + " with " + String(sparse_vector.nnz()) + " terms to inverted index")
    
    fn remove_document(mut self, document_id: String, sparse_vector: SparseVector):
        """Remove document from inverted index.
        
        Args:
            document_id: Document identifier to remove
            sparse_vector: Sparse vector representation of document
        """
        var terms = List[Int]()
        
        # Remove document from posting lists
        for i in range(sparse_vector.nnz()):
            var term_idx = sparse_vector.indices[i]
            
            if term_idx >= 0 and term_idx < self.dimension and term_idx in self.posting_lists:
                var removed = self.posting_lists[term_idx].remove_document(document_id)
                if removed:
                    terms.append(term_idx)
                
                # Remove empty posting list
                if self.posting_lists[term_idx].is_empty():
                    self.posting_lists.pop(term_idx)
        
        # Update document frequency tracking
        self.document_frequency.remove_document_terms(terms)
        
        self.logger.debug("Removed document " + document_id + " from inverted index")
    
    # ========================================
    # Search Operations
    # ========================================
    
    fn get_candidate_documents(self, query: SparseVector) -> List[String]:
        """Get candidate documents that contain any query terms.
        
        Args:
            query: Query sparse vector
            
        Returns:
            List of document IDs that contain at least one query term
        """
        var candidates = Dict[String, Bool]()
        
        # Collect candidates from posting lists of query terms
        for i in range(query.nnz()):
            var term_idx = query.indices[i]
            
            if term_idx >= 0 and term_idx < self.dimension and term_idx in self.posting_lists:
                var posting_list = self.posting_lists[term_idx]
                for doc_id in posting_list.document_ids:
                    candidates[doc_id] = True
        
        # Convert dict to list
        var candidate_list = List[String]()
        for item in candidates.items():
            candidate_list.append(item.key)
        
        return candidate_list
    
    fn get_documents_with_term(self, term_idx: Int) -> List[String]:
        """Get all documents containing a specific term.
        
        Args:
            term_idx: Term index to search for
            
        Returns:
            List of document IDs containing the term
        """
        if term_idx >= 0 and term_idx < self.dimension and term_idx in self.posting_lists:
            return self.posting_lists[term_idx].document_ids
        return List[String]()
    
    fn get_term_frequency(self, term_idx: Int, document_id: String) -> Float32:
        """Get term frequency for specific term and document.
        
        Args:
            term_idx: Term index
            document_id: Document identifier
            
        Returns:
            Term frequency, or 0.0 if not found
        """
        if term_idx >= 0 and term_idx < self.dimension and term_idx in self.posting_lists:
            return self.posting_lists[term_idx].get_term_frequency(document_id)
        return 0.0
    
    # ========================================
    # Statistics and Information
    # ========================================
    
    fn get_document_frequency(self, term_idx: Int) -> Int:
        """Get document frequency for a term.
        
        Args:
            term_idx: Term index
            
        Returns:
            Number of documents containing the term
        """
        return self.document_frequency.get_document_frequency(term_idx)
    
    fn get_vocabulary_size(self) -> Int:
        """Get vocabulary size (number of unique terms).
        
        Returns:
            Number of unique terms in index
        """
        return self.document_frequency.get_vocabulary_size()
    
    fn get_total_documents(self) -> Int:
        """Get total number of documents in index.
        
        Returns:
            Total document count
        """
        return self.document_frequency.get_total_documents()
    
    fn contains_term(self, term_idx: Int) -> Bool:
        """Check if term exists in index.
        
        Args:
            term_idx: Term index to check
            
        Returns:
            True if term exists in index
        """
        return term_idx in self.posting_lists
    
    fn get_posting_list_size(self, term_idx: Int) -> Int:
        """Get size of posting list for a term.
        
        Args:
            term_idx: Term index
            
        Returns:
            Number of documents containing the term
        """
        if term_idx in self.posting_lists:
            return self.posting_lists[term_idx].size()
        return 0
    
    # ========================================
    # Index Optimization
    # ========================================
    
    fn optimize(mut self):
        """Optimize index for better performance.
        
        This method can be called periodically to:
        - Compact posting lists
        - Optimize memory layout
        - Clean up empty entries
        """
        self.logger.debug("Starting inverted index optimization...")
        
        # Remove empty posting lists
        var terms_to_remove = List[Int]()
        for item in self.posting_lists.items():
            if item.value.is_empty():
                terms_to_remove.append(item.key)
        
        for term in terms_to_remove:
            self.posting_lists.pop(term)
        
        self.logger.debug("Inverted index optimization completed. Removed " + String(len(terms_to_remove)) + " empty posting lists")
    
    fn estimate_memory_usage(self) -> Int:
        """Estimate total memory usage of inverted index.
        
        Returns:
            Approximate memory usage in bytes
        """
        var total_memory = 64  # Base struct overhead
        
        # Posting lists memory
        for item in self.posting_lists.items():
            total_memory += 8  # Dict entry overhead
            total_memory += item.value.memory_footprint()
        
        # Document frequency tracking
        total_memory += len(self.document_frequency.term_counts) * 16  # Int key + Int value
        
        return total_memory
    
    fn get_index_statistics(self) -> String:
        """Get comprehensive index statistics.
        
        Returns:
            String containing index statistics
        """
        var stats = String("InvertedIndex Statistics:")
        stats += "\n  Dimension: " + String(self.dimension)
        stats += "\n  Total Documents: " + String(self.get_total_documents())
        stats += "\n  Vocabulary Size: " + String(self.get_vocabulary_size())
        stats += "\n  Posting Lists: " + String(len(self.posting_lists))
        stats += "\n  Memory Usage: " + String(self.estimate_memory_usage()) + " bytes"
        
        # Average posting list size
        if len(self.posting_lists) > 0:
            var total_postings = 0
            for item in self.posting_lists.items():
                total_postings += item.value.size()
            var avg_posting_size = Float64(total_postings) / Float64(len(self.posting_lists))
            stats += "\n  Average Posting List Size: " + String(avg_posting_size)
        
        return stats
    
    # ========================================
    # Advanced Operations
    # ========================================
    
    fn get_term_statistics(self, term_idx: Int) -> String:
        """Get statistics for a specific term.
        
        Args:
            term_idx: Term index
            
        Returns:
            String containing term statistics
        """
        if term_idx not in self.posting_lists:
            return "Term " + String(term_idx) + " not found in index"
        
        var posting_list = self.posting_lists[term_idx]
        var doc_freq = self.get_document_frequency(term_idx)
        
        var stats = String("Term " + String(term_idx) + " Statistics:")
        stats += "\n  Document Frequency: " + String(doc_freq)
        stats += "\n  Posting List Size: " + String(posting_list.size())
        stats += "\n  Memory Usage: " + String(posting_list.memory_footprint()) + " bytes"
        
        # Calculate average term frequency
        if posting_list.size() > 0:
            var total_tf = Float32(0.0)
            for tf in posting_list.term_frequencies:
                total_tf += tf
            var avg_tf = total_tf / Float32(posting_list.size())
            stats += "\n  Average Term Frequency: " + String(avg_tf)
        
        return stats
    
    fn get_most_frequent_terms(self, k: Int = 10) -> List[Int]:
        """Get the k most frequent terms (by document frequency).
        
        Args:
            k: Number of terms to return
            
        Returns:
            List of term indices sorted by document frequency (descending)
        """
        var term_freqs = List[List[Int]]()  # [[term_idx, doc_freq]]
        
        # Collect all terms with their document frequencies
        for item in self.posting_lists.items():
            var term_idx = item.key
            var doc_freq = self.get_document_frequency(term_idx)
            var term_freq_pair = List[Int]()
            term_freq_pair.append(term_idx)
            term_freq_pair.append(doc_freq)
            term_freqs.append(term_freq_pair)
        
        # Simple selection sort for top k
        var result = List[Int]()
        var limit = min(k, len(term_freqs))
        
        for _ in range(limit):
            var max_freq = -1
            var max_idx = -1
            var max_pos = -1
            
            # Find maximum
            for i in range(len(term_freqs)):
                if term_freqs[i][1] > max_freq:
                    max_freq = term_freqs[i][1]
                    max_idx = term_freqs[i][0]
                    max_pos = i
            
            if max_pos >= 0:
                result.append(max_idx)
                # Remove from consideration
                var new_term_freqs = List[List[Int]]()
                for i in range(len(term_freqs)):
                    if i != max_pos:
                        new_term_freqs.append(term_freqs[i])
                term_freqs = new_term_freqs
        
        return result
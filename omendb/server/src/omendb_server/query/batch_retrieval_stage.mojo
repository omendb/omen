"""
Batch retrieval query stage for OmenDB.

This module implements a query stage for retrieving vectors in batches,
taking advantage of the enhanced storage abstraction's batch operations
for improved performance.
"""

from collections import List, Dict, Optional
from core.vector import Vector
from core.record import VectorRecord
from storage.vector_store import VectorStore, QueryFilter, TransactionContext
from query.query_stage import QueryStage, QueryResult
from util.logging import Logger, LogLevel

@value
struct BatchRetrievalStage[dtype: DType](QueryStage[dtype]):
    """
    Query stage for retrieving vectors in batches from the store.
    
    This stage leverages the batch operations in the vector store
    for improved performance when fetching multiple vectors.
    
    Attributes:
        store: Vector store for retrieving records
        logger: Logger for batch operations
        options: Configuration options for the stage
    """
    
    var store: VectorStore[dtype]
    var logger: Logger
    var options: Dict[String, String]
    var transaction_context: Optional[TransactionContext]
    
    fn __init__(out self, store: VectorStore[dtype]):
        self.store = store
        self.logger = Logger("BatchRetrievalStage", LogLevel.INFO)
        self.options = Dict[String, String]()
        self.transaction_context = None
    
    fn __copyinit__(out self, other: Self):
        self.store = other.store
        self.logger = other.logger
        self.options = other.options
        self.transaction_context = other.transaction_context
    
    fn execute(
        self, 
        input_result: QueryResult[dtype],
        query_vector: Optional[Vector[dtype]] = None,
        filters: Optional[List[QueryFilter]] = None
    ) raises -> QueryResult[dtype]:
        """
        Retrieve vector records in batch from the store.
        
        Args:
            input_result: Input from the previous stage (usually contains IDs to fetch)
            query_vector: Not used in this stage
            filters: Not used in this stage (filtering happens in FilterStage)
            
        Returns:
            Query result with complete vector records
        """
        if input_result.size() == 0:
            # No input results to process
            return input_result
            
        # Extract IDs from the input
        var ids = List[String]()
        for record in input_result.records:
            ids.append(record.id)
            
        # Determine batch size from options
        var batch_size = 50  # Default
        if "batch_size" in self.options:
            batch_size = atol(self.options["batch_size"])
        
        # Process in batches for improved performance
        var result = QueryResult[dtype]()
        
        # Preserve scores from input_result
        var scores_map = input_result.scores
        
        # Start transaction if needed
        if "use_transaction" in self.options and self.options["use_transaction"] == "true":
            if self.transaction_context.none():
                self.transaction_context = Optional[TransactionContext](self.store.begin_transaction())
                self.logger.debug("Started transaction for batch retrieval")
        
        # Process in batches
        for i in range(0, len(ids), batch_size):
            var end = min(i + batch_size, len(ids))
            var batch_ids = List[String]()
            
            for j in range(i, end):
                batch_ids.append(ids[j])
            
            self.logger.debug(
                "Processing batch " + String(i / batch_size + 1) + 
                " of " + String((len(ids) + batch_size - 1) / batch_size) + 
                " (" + String(len(batch_ids)) + " records)"
            )
            
            # Retrieve records in batch
            var records = self.store.get_batch(batch_ids)
            
            # Add to result, maintaining original scores
            for record in records:
                if record.id in scores_map:
                    result.add_record(record, scores_map[record.id])
                else:
                    # If no score available, use default score (1.0)
                    result.add_record(record, SIMD[DType.float64, 1](1.0))
        
        # Commit transaction if started
        if "use_transaction" in self.options and self.options["use_transaction"] == "true":
            if not self.transaction_context.none():
                self.store.commit_transaction(self.transaction_context.value())
                self.transaction_context = None
                self.logger.debug("Committed transaction for batch retrieval")
        
        self.logger.debug(
            "Batch retrieval complete, retrieved " + 
            String(result.size()) + " of " + 
            String(len(ids)) + " requested records"
        )
            
        return result
    
    fn name(self) -> String:
        return "BatchRetrievalStage"
    
    fn configure(mut self, options: Dict[String, String]) raises:
        self.options = options

@value
struct BatchMetadataFilterStage[dtype: DType](QueryStage[dtype]):
    """
    Query stage for filtering vectors based on metadata criteria using batch operations.
    
    This stage leverages batch operations for improved filtering performance,
    especially when working with large datasets.
    """
    
    var store: VectorStore[dtype]
    var logger: Logger
    var options: Dict[String, String]
    var transaction_context: Optional[TransactionContext]
    
    fn __init__(out self, store: VectorStore[dtype]):
        self.store = store
        self.logger = Logger("BatchMetadataFilterStage", LogLevel.INFO)
        self.options = Dict[String, String]()
        self.transaction_context = None
    
    fn __copyinit__(out self, other: Self):
        self.store = other.store
        self.logger = other.logger
        self.options = other.options
        self.transaction_context = other.transaction_context
    
    fn execute(
        self, 
        input_result: QueryResult[dtype],
        query_vector: Optional[Vector[dtype]] = None,
        filters: Optional[List[QueryFilter]] = None
    ) raises -> QueryResult[dtype]:
        """
        Apply metadata filters to the vector records using batch operations.
        
        Args:
            input_result: Input from the previous stage
            query_vector: Not used in this stage
            filters: Metadata filters to apply
            
        Returns:
            Filtered query result
        """
        if filters.none() or len(filters.value()) == 0:
            # No filters to apply, pass through
            return input_result
        
        # Determine if we should process everything from storage
        var process_all = False
        if "process_all" in self.options and self.options["process_all"] == "true":
            process_all = True
        
        var result = QueryResult[dtype]()
        
        # Get all IDs if processing everything, otherwise use input
        var all_ids = List[String]()
        
        if process_all:
            # Start transaction if needed
            if "use_transaction" in self.options and self.options["use_transaction"] == "true":
                if self.transaction_context.none():
                    self.transaction_context = Optional[TransactionContext](self.store.begin_transaction())
            
            # Get all IDs from storage
            all_ids = self.store.get_all_ids()
            
            self.logger.debug(
                "Processing all " + String(len(all_ids)) + 
                " records from storage for filtering"
            )
        else:
            # Use IDs from input result
            for record in input_result.records:
                all_ids.append(record.id)
                
            self.logger.debug(
                "Processing " + String(len(all_ids)) + 
                " records from input for filtering"
            )
        
        # Determine batch size from options
        var batch_size = 50  # Default
        if "batch_size" in self.options:
            batch_size = atol(self.options["batch_size"])
        
        # Process in batches
        for i in range(0, len(all_ids), batch_size):
            var end = min(i + batch_size, len(all_ids))
            var batch_ids = List[String]()
            
            for j in range(i, end):
                batch_ids.append(all_ids[j])
            
            self.logger.debug(
                "Processing filter batch " + String(i / batch_size + 1) + 
                " of " + String((len(all_ids) + batch_size - 1) / batch_size)
            )
            
            # Get records for this batch
            var records = self.store.get_batch(batch_ids)
            
            # Apply filters to each record
            for record in records:
                var matches = True
                
                # Apply all filters
                for filter in filters.value():
                    var field = filter.field
                    var op = filter.op
                    var value = filter.value
                    
                    # Skip if the field doesn't exist in the record's metadata
                    if field not in record.metadata:
                        matches = False
                        break
                    
                    var record_value = record.metadata[field]
                    
                    # Apply the filter operation
                    if op == QueryFilter.OP_EQ:  # Equal
                        if record_value != value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_NE:  # Not equal
                        if record_value == value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_GT:  # Greater than
                        # Simple string comparison
                        if record_value <= value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_LT:  # Less than
                        if record_value >= value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_GTE:  # Greater than or equal
                        if record_value < value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_LTE:  # Less than or equal
                        if record_value > value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_CONTAINS:  # Contains
                        if value not in record_value:
                            matches = False
                            break
                    elif op == QueryFilter.OP_IN:  # In list (simplified implementation)
                        # Assume value is comma-separated list
                        var values = value.split(",")
                        var found = False
                        for val in values:
                            if record_value == val:
                                found = True
                                break
                        if not found:
                            matches = False
                            break
                
                # If all filters match, add to result
                if matches:
                    # Get score from input_result if available
                    var score = SIMD[DType.float64, 1](1.0)  # Default score
                    
                    if not process_all and record.id in input_result.scores:
                        score = input_result.scores[record.id]
                    
                    result.add_record(record, score)
        
        # Commit transaction if started
        if "use_transaction" in self.options and self.options["use_transaction"] == "true":
            if not self.transaction_context.none():
                self.store.commit_transaction(self.transaction_context.value())
                self.transaction_context = None
        
        self.logger.debug(
            "Batch filtering complete, " + String(result.size()) + 
            " records matched the filters"
        )
            
        return result
    
    fn name(self) -> String:
        return "BatchMetadataFilterStage"
    
    fn configure(mut self, options: Dict[String, String]) raises:
        self.options = options
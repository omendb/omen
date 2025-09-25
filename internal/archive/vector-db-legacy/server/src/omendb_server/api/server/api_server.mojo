"""
OmenDB API Server implementation.

This module provides the core server implementation that handles
vector database operations like insert, get, update, delete, and search.
It acts as a bridge between the API interfaces (REST) and the
underlying vector store and index components.
"""

from collections import List, Dict, Optional
from core.vector import Vector
from core.record import VectorRecord
from storage.vector_store import VectorStore, QueryFilter
from index.hnsw_index import HnswIndex
from query.pipeline_builder import QueryPipelineBuilder
from query.query_engine import QueryEngine
from util.logging import Logger, LogLevel

@value
struct OmenDBServer[dtype: DType]:
    """
    Core server implementation for OmenDB.
    
    This struct handles the actual database operations, serving as the implementation
    behind the REST API interface.
    
    Attributes:
        store: The vector store implementation
        index: The vector index implementation
        logger: Logger for server operations
    """
    
    var store: VectorStore[dtype]
    var index: HnswIndex[dtype]
    var logger: Logger
    
    fn __init__(inout self, store: VectorStore[dtype], index: HnswIndex[dtype]):
        """
        Initialize the server with the provided store and index.
        
        Args:
            store: The vector store implementation
            index: The vector index implementation
        """
        self.store = store
        self.index = index
        self.logger = Logger("OmenDBServer", LogLevel.INFO)
    
    fn insert_vector(mut self, id: String, vector: Vector[dtype], metadata: Dict[String, String]) raises -> (Bool, String):
        """
        Insert a vector into the database.
        
        Args:
            id: Unique identifier for the vector
            vector: The vector to insert
            metadata: Metadata associated with the vector
            
        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug("Inserting vector with ID: " + id)
        
        var record = VectorRecord[dtype](id, vector)
        for key in metadata.keys():
            record.metadata[key] = metadata[key]
        
        if self.store.contains(id):
            return (False, "Vector with ID already exists. Use update instead.")
        
        var tx_context = self.store.begin_transaction()
        var success = self.store.insert(record)
        
        if success:
            self.index.insert(record)
            self.store.commit_transaction(tx_context)
            return (True, "Vector inserted successfully")
        else:
            self.store.rollback_transaction(tx_context)
            return (False, "Failed to insert vector")
    
    fn get_vector(self, id: String, include_vector: Bool, include_metadata: Bool) raises -> (Bool, String, String, Optional[Vector[dtype]], Optional[Dict[String, String]]):
        """
        Retrieve a vector from the database.
        
        Args:
            id: The ID of the vector to retrieve
            include_vector: Whether to include the vector data in the response
            include_metadata: Whether to include metadata in the response
            
        Returns:
            A tuple containing (success, message, id, vector, metadata)
        """
        self.logger.debug("Getting vector with ID: " + id)
        
        if not self.store.contains(id):
            return (False, "Vector not found", "", None, None)
        
        var record = self.store.get(id)
        var vector_opt = None
        var metadata_opt = None
        
        if include_vector:
            vector_opt = Optional[Vector[dtype]](record.vector)
        
        if include_metadata:
            metadata_opt = Optional[Dict[String, String]](record.metadata)
        
        return (True, "Vector retrieved successfully", record.id, vector_opt, metadata_opt)
    
    fn update_vector(mut self, id: String, vector: Optional[Vector[dtype]], metadata: Optional[Dict[String, String]]) raises -> (Bool, String):
        """
        Update a vector in the database.
        
        Args:
            id: The ID of the vector to update
            vector: Optional new vector data (if None, the existing vector is preserved)
            metadata: Optional new metadata (if None, the existing metadata is preserved)
            
        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug("Updating vector with ID: " + id)
        
        if not self.store.contains(id):
            return (False, "Vector not found")
        
        var tx_context = self.store.begin_transaction()
        var current_record = self.store.get(id)
        var new_record = current_record
        
        if not vector.none():
            new_record.vector = vector.value()
        
        if not metadata.none():
            for key in metadata.value().keys():
                new_record.metadata[key] = metadata.value()[key]
        
        var success = self.store.update(new_record)
        
        if success:
            if not vector.none():
                self.index.update(new_record)
            
            self.store.commit_transaction(tx_context)
            return (True, "Vector updated successfully")
        else:
            self.store.rollback_transaction(tx_context)
            return (False, "Failed to update vector")
    
    fn delete_vector(mut self, id: String) raises -> (Bool, String):
        """
        Delete a vector from the database.
        
        Args:
            id: The ID of the vector to delete
            
        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug("Deleting vector with ID: " + id)
        
        if not self.store.contains(id):
            return (False, "Vector not found")
        
        var tx_context = self.store.begin_transaction()
        var success = self.store.delete(id)
        
        if success:
            self.index.delete(id)
            self.store.commit_transaction(tx_context)
            return (True, "Vector deleted successfully")
        else:
            self.store.rollback_transaction(tx_context)
            return (False, "Failed to delete vector")
    
    fn search_vectors(
        self, 
        query_vector: Vector[dtype], 
        k: Int, 
        include_vectors: Bool, 
        include_metadata: Bool, 
        include_distances: Bool,
        filters: Optional[List[QueryFilter]] = None
    ) raises -> (Bool, String, List[VectorRecord[dtype]], Dict[String, Float32]):
        """
        Search for similar vectors.
        
        Args:
            query_vector: The query vector
            k: Number of results to return
            include_vectors: Whether to include vector data in the results
            include_metadata: Whether to include metadata in the results
            include_distances: Whether to include distances in the results
            filters: Optional list of metadata filters
            
        Returns:
            A tuple containing (success, message, records, scores)
        """
        self.logger.debug("Searching for vectors similar to query (k=" + String(k) + ")")
        
        var builder = QueryPipelineBuilder[dtype]()
            .with_store(self.store)
            .with_index(self.index)
            .add_vector_search(Dict[String, String]{"k": String(k * 2), "ef": String(k * 10)})
        
        if not filters.none() and filters.value().size() > 0:
            builder = builder.add_filter()
        
        builder = builder.add_reranking()
        
        var pipeline = builder.build()
        var results = pipeline.execute(
            query_vector=query_vector,
            filters=filters.value() if not filters.none() else List[QueryFilter](),
            limit=k
        )
        
        return (True, "Search completed successfully", results.records, results.scores)
    
    fn count_vectors(self, filters: Optional[List[QueryFilter]] = None) raises -> Int64:
        """
        Count vectors in the database.
        
        Args:
            filters: Optional list of metadata filters
            
        Returns:
            The number of vectors matching the filters
        """
        self.logger.debug("Counting vectors" + (" with filters" if not filters.none() and filters.value().size() > 0 else ""))
        
        if filters.none() or filters.value().size() == 0:
            return Int64(self.store.count())
        
        var count: Int64 = 0
        var ids = self.store.get_all_ids()
        
        for id in ids:
            var record = self.store.get(id)
            var match = True
            
            for filter in filters.value():
                if not self._matches_filter(record, filter):
                    match = False
                    break
            
            if match:
                count += 1
        
        return count
    
    fn batch_insert_vectors(mut self, records: List[VectorRecord[dtype]]) raises -> (Int, Int, String):
        """
        Insert multiple vectors in a batch.
        
        Args:
            records: List of vector records to insert
            
        Returns:
            A tuple containing (successful_count, failed_count, message)
        """
        self.logger.debug("Batch inserting " + String(records.size()) + " vectors")
        
        var successful_count = 0
        var failed_count = 0
        var tx_context = self.store.begin_transaction()
        
        try:
            var inserted = self.store.insert_batch(records)
            successful_count = inserted
            failed_count = records.size() - inserted
            
            if successful_count > 0:
                for record in records:
                    try:
                        self.index.insert(record)
                    except:
                        self.logger.warning("Failed to index vector " + record.id)
            
            self.store.commit_transaction(tx_context)
            
            if failed_count == 0:
                return (successful_count, failed_count, "All vectors inserted successfully")
            else:
                return (successful_count, failed_count, String(successful_count) + " vectors inserted, " + String(failed_count) + " failed")
        except:
            self.store.rollback_transaction(tx_context)
            return (0, records.size(), "Batch insertion failed")
    
    fn list_vector_ids(self) raises -> List[String]:
        """
        List all vector IDs in the database.
        
        Returns:
            List of vector IDs
        """
        self.logger.debug("Listing all vector IDs")
        return self.store.get_all_ids()
    
    fn _matches_filter(self, record: VectorRecord[dtype], filter: QueryFilter) -> Bool:
        """
        Check if a record matches a filter condition.
        
        Args:
            record: The vector record to check
            filter: The filter condition
            
        Returns:
            True if the record matches the filter, False otherwise
        """
        if filter.field not in record.metadata:
            return False
        
        var value = record.metadata[filter.field]
        
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
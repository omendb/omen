"""
OmenDB gRPC Service implementation.

This module provides the gRPC service implementation for OmenDB,
handling the transformation between gRPC requests/responses and
the core API server operations.
"""

from collections import List, Dict, Optional
from core.vector import Vector
from core.record import VectorRecord
from storage.vector_store import QueryFilter
from api.server.api_server import OmenDBServer
from util.logging import Logger, LogLevel

@value
struct OmenDBGrpcService[dtype: DType]:
    """
    gRPC service implementation for OmenDB.
    
    This struct handles the gRPC interface, transforming between Protocol Buffer
    messages and the internal data structures used by the API server.
    
    Attributes:
        server: The underlying API server
        logger: Logger for gRPC service operations
    """
    
    var server: OmenDBServer[dtype]
    var logger: Logger
    
    fn __init__(inout self, server: OmenDBServer[dtype]):
        """
        Initialize the gRPC service with the API server.
        
        Args:
            server: The OmenDB API server
        """
        self.server = server
        self.logger = Logger("OmenDBGrpcService", LogLevel.INFO)
    
    # Note: The following methods would typically interact directly with Protocol Buffer messages,
    # but Mojo doesn't have built-in gRPC support yet. These methods serve as a blueprint for
    # how the gRPC service would be implemented once proper gRPC bindings are available.
    
    fn handle_insert(mut self, id: String, values: List[Float32], metadata: Dict[String, String]) raises -> (Bool, String):
        """
        Handle an insert request from gRPC.
        
        Args:
            id: Vector ID
            values: Vector values
            metadata: Metadata key-value pairs
            
        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug("Handling gRPC insert request for vector " + id)
        
        var vector = Vector[dtype](len(values))
        for i in range(len(values)):
            vector.data[i] = values[i]
        
        return self.server.insert_vector(id, vector, metadata)
    
    fn handle_get(self, id: String, include_vector: Bool, include_metadata: Bool) raises -> (Bool, String, String, Optional[List[Float32]], Optional[Dict[String, String]]):
        """
        Handle a get request from gRPC.
        
        Args:
            id: Vector ID
            include_vector: Whether to include vector values
            include_metadata: Whether to include metadata
            
        Returns:
            A tuple containing (success, message, id, vector values, metadata)
        """
        self.logger.debug("Handling gRPC get request for vector " + id)
        
        var result = self.server.get_vector(id, include_vector, include_metadata)
        var success = result[0]
        var message = result[1]
        var vector_id = result[2]
        var vector_opt = result[3]
        var metadata_opt = result[4]
        
        var values_opt = None
        
        if success and include_vector and not vector_opt.none():
            var values = List[Float32]()
            var vector = vector_opt.value()
            for i in range(vector.dim):
                values.append(vector.data[i])
            values_opt = Optional[List[Float32]](values)
        
        return (success, message, vector_id, values_opt, metadata_opt)
    
    fn handle_update(mut self, id: String, values: Optional[List[Float32]], metadata: Optional[Dict[String, String]]) raises -> (Bool, String):
        """
        Handle an update request from gRPC.
        
        Args:
            id: Vector ID
            values: Optional new vector values
            metadata: Optional new metadata
            
        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug("Handling gRPC update request for vector " + id)
        
        var vector_opt = None
        
        if not values.none():
            var vector = Vector[dtype](len(values.value()))
            for i in range(len(values.value())):
                vector.data[i] = values.value()[i]
            vector_opt = Optional[Vector[dtype]](vector)
        
        return self.server.update_vector(id, vector_opt, metadata)
    
    fn handle_delete(mut self, id: String) raises -> (Bool, String):
        """
        Handle a delete request from gRPC.
        
        Args:
            id: Vector ID
            
        Returns:
            A tuple containing (success, message)
        """
        self.logger.debug("Handling gRPC delete request for vector " + id)
        
        return self.server.delete_vector(id)
    
    fn handle_search(
        self, 
        values: List[Float32], 
        k: Int, 
        include_vectors: Bool, 
        include_metadata: Bool, 
        include_distances: Bool,
        filter_operations: Optional[List[Tuple[String, String, String]]] = None
    ) raises -> (Bool, String, List[Tuple[String, Optional[List[Float32]], Optional[Dict[String, String]], Float32]]):
        """
        Handle a search request from gRPC.
        
        Args:
            values: Query vector values
            k: Number of results to return
            include_vectors: Whether to include vector values
            include_metadata: Whether to include metadata
            include_distances: Whether to include distances
            filter_operations: Optional filter operations (field, operator, value)
            
        Returns:
            A tuple containing (success, message, results)
            where results is a list of (id, vector values, metadata, distance)
        """
        self.logger.debug("Handling gRPC search request with k=" + String(k))
        
        var query_vector = Vector[dtype](len(values))
        for i in range(len(values)):
            query_vector.data[i] = values[i]
        
        var filters_opt = None
        if not filter_operations.none() and filter_operations.value().size() > 0:
            var filters = List[QueryFilter]()
            for op in filter_operations.value():
                var field = op[0]
                var operator = op[1]
                var value = op[2]
                filters.append(QueryFilter(field, operator, value))
            filters_opt = Optional[List[QueryFilter]](filters)
        
        var result = self.server.search_vectors(
            query_vector, 
            k, 
            include_vectors, 
            include_metadata, 
            include_distances,
            filters_opt
        )
        
        var success = result[0]
        var message = result[1]
        var records = result[2]
        var scores = result[3]
        
        var search_results = List[Tuple[String, Optional[List[Float32]], Optional[Dict[String, String]], Float32]]()
        
        for i in range(records.size()):
            var record = records[i]
            var id = record.id
            var score = scores[id]
            
            var vector_values_opt = None
            if include_vectors:
                var vector_values = List[Float32]()
                for j in range(record.vector.dim):
                    vector_values.append(record.vector.data[j])
                vector_values_opt = Optional[List[Float32]](vector_values)
            
            var metadata_opt = None
            if include_metadata:
                metadata_opt = Optional[Dict[String, String]](record.metadata)
            
            search_results.append((id, vector_values_opt, metadata_opt, score if include_distances else 0.0))
        
        return (success, message, search_results)
    
    fn handle_count(self, filter_operations: Optional[List[Tuple[String, String, String]]] = None) raises -> Int64:
        """
        Handle a count request from gRPC.
        
        Args:
            filter_operations: Optional filter operations (field, operator, value)
            
        Returns:
            The count of vectors matching the filters
        """
        self.logger.debug("Handling gRPC count request")
        
        var filters_opt = None
        if not filter_operations.none() and filter_operations.value().size() > 0:
            var filters = List[QueryFilter]()
            for op in filter_operations.value():
                var field = op[0]
                var operator = op[1]
                var value = op[2]
                filters.append(QueryFilter(field, operator, value))
            filters_opt = Optional[List[QueryFilter]](filters)
        
        return self.server.count_vectors(filters_opt)
    
    fn handle_batch_insert(mut self, batch_items: List[Tuple[String, List[Float32], Dict[String, String]]]) raises -> (Int, Int, String):
        """
        Handle a batch insert request from gRPC.
        
        Args:
            batch_items: List of (id, values, metadata) tuples
            
        Returns:
            A tuple containing (successful_count, failed_count, message)
        """
        self.logger.debug("Handling gRPC batch insert request with " + String(batch_items.size()) + " items")
        
        var records = List[VectorRecord[dtype]]()
        
        for item in batch_items:
            var id = item[0]
            var values = item[1]
            var metadata = item[2]
            
            var vector = Vector[dtype](len(values))
            for i in range(len(values)):
                vector.data[i] = values[i]
            
            var record = VectorRecord[dtype](id, vector)
            for key in metadata.keys():
                record.metadata[key] = metadata[key]
            
            records.append(record)
        
        return self.server.batch_insert_vectors(records)
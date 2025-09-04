"""
OmenDB REST Service implementation.

This module provides the REST service implementation for OmenDB,
handling HTTP requests/responses using the Lightbug API framework
and EmberJson for JSON serialization/deserialization.
"""

from collections import List, Dict, Optional
from lightbug_api import App
from lightbug_http import HTTPRequest, HTTPResponse, OK, NotFound, BadRequest
from emberjson import parse as parse_json, to_string as json_to_string, JSON, Value, Object, Array

from core.vector import Vector
from core.record import VectorRecord
from storage.vector_store import QueryFilter
from api.server.api_server import OmenDBServer
from api.context_endpoints import ContextEndpoints
from query.context_manager import ContextManager
from storage.context_store import MemoryContextStore, IntegratedContextStore
from util.logging import Logger, LogLevel

fn create_response(success: Bool, message: String, data: Optional[Object] = None) -> String:
    """
    Create a standard JSON response.
    
    Args:
        success: Whether the operation was successful
        message: Message describing the result
        data: Optional data to include in the response
        
    Returns:
        JSON string response
    """
    var response = Object()
    response["success"] = Value(success)
    response["message"] = Value(message)
    
    if not data.none():
        response["data"] = Value(data.value())
    
    return json_to_string(response)

fn vector_to_json[dtype: DType](vector: Vector[dtype]) -> Array:
    """
    Convert a vector to a JSON array.
    
    Args:
        vector: The vector to convert
        
    Returns:
        JSON array of vector values
    """
    var arr = Array()
    for i in range(vector.dim):
        arr[i] = Value(Float64(vector[i]))
    return arr

fn vector_record_to_json[dtype: DType](record: VectorRecord[dtype], include_vector: Bool, include_metadata: Bool) -> Object:
    """
    Convert a vector record to a JSON object.
    
    Args:
        record: The vector record to convert
        include_vector: Whether to include the vector data
        include_metadata: Whether to include metadata
        
    Returns:
        JSON object representation of the record
    """
    var obj = Object()
    obj["id"] = Value(record.id)
    
    if include_vector:
        obj["vector"] = Value(vector_to_json(record.vector))
    
    if include_metadata and record.metadata.size() > 0:
        var metadata = Object()
        for key in record.metadata.keys():
            metadata[key] = Value(record.metadata[key])
        obj["metadata"] = Value(metadata)
    
    return obj

@value
struct OmenDBRestService[dtype: DType]:
    """
    REST service implementation for OmenDB using Lightbug API.
    
    This struct handles the REST interface, transforming between JSON
    requests/responses and the internal data structures used by the API server.
    
    Attributes:
        server: The underlying API server
        logger: Logger for REST service operations
    """
    
    var server: OmenDBServer[dtype]
    var logger: Logger
    var app: App
    var context_endpoints: ContextEndpoints[dtype]
    
    fn __init__(inout self, server: OmenDBServer[dtype]):
        """
        Initialize the REST service with the API server.
        
        Args:
            server: The OmenDB API server
        """
        self.server = server
        self.logger = Logger("OmenDBRestService", LogLevel.INFO)
        self.app = App()
        
        # Initialize context system components
        var context_store = MemoryContextStore[dtype]()
        var context_manager = ContextManager[dtype](
            server.store, 
            context_store, 
            server.index
        )
        self.context_endpoints = ContextEndpoints[dtype](context_manager)
        
        self._register_routes()
    
    fn _register_routes(inout self):
        """Register all API routes."""
        # Vector API routes
        self.app.get("/v1/vectors", self.handle_list_vectors)
        self.app.get("/v1/vectors/:id", self.handle_get_vector)
        self.app.post("/v1/vectors", self.handle_insert_vector)
        self.app.put("/v1/vectors/:id", self.handle_update_vector)
        self.app.delete("/v1/vectors/:id", self.handle_delete_vector)
        self.app.post("/v1/vectors/search", self.handle_search_vectors)
        self.app.post("/v1/vectors/batch", self.handle_batch_insert)
        self.app.get("/v1/vectors/count", self.handle_count_vectors)
        self.app.get("/v1/health", self.handle_health_check)
        
        # Register context API routes
        self.context_endpoints.register_routes(self.app)
    
    fn start(self, host: String = "0.0.0.0", port: Int = 8080) raises:
        """
        Start the REST server.
        
        Args:
            host: The host to bind to
            port: The port to listen on
        """
        self.logger.info("Starting REST server on " + host + ":" + String(port))
        self.app.start_server(host + ":" + String(port))
    
    @always_inline
    fn handle_list_vectors(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/vectors."""
        self.logger.debug("Handling list vectors request")
        
        var limit = 100
        var offset = 0
        var include_vectors = False
        var include_metadata = True
        
        # Parse query parameters
        # In a more complete implementation, we would parse limit, offset, etc.
        
        try:
            var ids = self.server.list_vector_ids()
            var response_data = Object()
            var items = Array()
            
            var end = min(offset + limit, ids.size())
            for i in range(offset, end):
                var id = ids[i]
                var record = self.server.get_vector(
                    id, include_vectors, include_metadata
                )
                
                if record[0]:  # If success
                    var record_obj = Object()
                    record_obj["id"] = Value(record[2])  # id
                    
                    if include_metadata and not record[4].none():
                        var metadata = Object()
                        for key in record[4].value().keys():
                            metadata[key] = Value(record[4].value()[key])
                        record_obj["metadata"] = Value(metadata)
                    
                    items[i - offset] = Value(record_obj)
            
            response_data["items"] = Value(items)
            response_data["total"] = Value(ids.size())
            response_data["offset"] = Value(offset)
            response_data["limit"] = Value(limit)
            
            return OK(create_response(True, "Vectors retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error listing vectors: " + String(e))
            return BadRequest(create_response(False, "Error listing vectors: " + String(e)))
    
    @always_inline
    fn handle_get_vector(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/vectors/:id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 3:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling get vector request for ID: " + id)
        
        var include_vector = req.uri.query.contains("include_vector=true")
        var include_metadata = not req.uri.query.contains("include_metadata=false")
        
        try:
            var result = self.server.get_vector(id, include_vector, include_metadata)
            
            if not result[0]:
                return NotFound(create_response(False, result[1]))
            
            var response_data = Object()
            response_data["id"] = Value(result[2])
            
            if include_vector and not result[3].none():
                response_data["vector"] = Value(vector_to_json(result[3].value()))
            
            if include_metadata and not result[4].none():
                var metadata = Object()
                for key in result[4].value().keys():
                    metadata[key] = Value(result[4].value()[key])
                response_data["metadata"] = Value(metadata)
            
            return OK(create_response(True, "Vector retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error getting vector: " + String(e))
            return BadRequest(create_response(False, "Error getting vector: " + String(e)))
    
    @always_inline
    fn handle_insert_vector(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/vectors."""
        self.logger.debug("Handling insert vector request")
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("id") or not json_obj["id"].is_string():
                return BadRequest(create_response(False, "Missing or invalid 'id' field"))
            
            if not json_obj.contains("vector") or not json_obj["vector"].is_array():
                return BadRequest(create_response(False, "Missing or invalid 'vector' field"))
            
            var id = json_obj["id"].string()
            var vector_array = json_obj["vector"].array()
            var values = List[Float32]()
            
            for i in range(vector_array.size()):
                if vector_array[i].is_float() or vector_array[i].is_int():
                    values.append(Float32(vector_array[i].float()))
                else:
                    return BadRequest(create_response(False, "Vector values must be numbers"))
            
            var metadata = Dict[String, String]()
            if json_obj.contains("metadata") and json_obj["metadata"].is_object():
                var meta_obj = json_obj["metadata"].object()
                for key in meta_obj.to_dict().keys():
                    if meta_obj[key].is_string():
                        metadata[key] = meta_obj[key].string()
                    else:
                        metadata[key] = json_to_string(meta_obj[key])
            
            var vector = Vector[dtype].from_list(values)
            var result = self.server.insert_vector(id, vector, metadata)
            
            if result[0]:
                var response_data = Object()
                response_data["id"] = Value(id)
                return OK(create_response(True, "Vector inserted successfully", Optional[Object](response_data)))
            else:
                return BadRequest(create_response(False, result[1]))
        
        except e:
            self.logger.error("Error inserting vector: " + String(e))
            return BadRequest(create_response(False, "Error inserting vector: " + String(e)))
    
    @always_inline
    fn handle_update_vector(self, req: HTTPRequest) -> HTTPResponse:
        """Handle PUT /v1/vectors/:id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 3:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling update vector request for ID: " + id)
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            var vector_opt = None
            var metadata_opt = None
            
            if json_obj.contains("vector") and json_obj["vector"].is_array():
                var vector_array = json_obj["vector"].array()
                var values = List[Float32]()
                
                for i in range(vector_array.size()):
                    if vector_array[i].is_float() or vector_array[i].is_int():
                        values.append(Float32(vector_array[i].float()))
                    else:
                        return BadRequest(create_response(False, "Vector values must be numbers"))
                
                vector_opt = Optional[Vector[dtype]](Vector[dtype].from_list(values))
            
            if json_obj.contains("metadata") and json_obj["metadata"].is_object():
                var meta_obj = json_obj["metadata"].object()
                var metadata = Dict[String, String]()
                
                for key in meta_obj.to_dict().keys():
                    if meta_obj[key].is_string():
                        metadata[key] = meta_obj[key].string()
                    else:
                        metadata[key] = json_to_string(meta_obj[key])
                
                metadata_opt = Optional[Dict[String, String]](metadata)
            
            var result = self.server.update_vector(id, vector_opt, metadata_opt)
            
            if result[0]:
                var response_data = Object()
                response_data["id"] = Value(id)
                return OK(create_response(True, "Vector updated successfully", Optional[Object](response_data)))
            else:
                return BadRequest(create_response(False, result[1]))
        
        except e:
            self.logger.error("Error updating vector: " + String(e))
            return BadRequest(create_response(False, "Error updating vector: " + String(e)))
    
    @always_inline
    fn handle_delete_vector(self, req: HTTPRequest) -> HTTPResponse:
        """Handle DELETE /v1/vectors/:id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 3:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling delete vector request for ID: " + id)
        
        try:
            var result = self.server.delete_vector(id)
            
            if result[0]:
                var response_data = Object()
                response_data["id"] = Value(id)
                return OK(create_response(True, "Vector deleted successfully", Optional[Object](response_data)))
            else:
                return NotFound(create_response(False, result[1]))
        except e:
            self.logger.error("Error deleting vector: " + String(e))
            return BadRequest(create_response(False, "Error deleting vector: " + String(e)))
    
    @always_inline
    fn handle_search_vectors(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/vectors/search."""
        self.logger.debug("Handling search vectors request")
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("vector") or not json_obj["vector"].is_array():
                return BadRequest(create_response(False, "Missing or invalid 'vector' field"))
            
            var vector_array = json_obj["vector"].array()
            var values = List[Float32]()
            
            for i in range(vector_array.size()):
                if vector_array[i].is_float() or vector_array[i].is_int():
                    values.append(Float32(vector_array[i].float()))
                else:
                    return BadRequest(create_response(False, "Vector values must be numbers"))
            
            var k = 10
            if json_obj.contains("k") and json_obj["k"].is_int():
                k = json_obj["k"].int()
            
            var include_vectors = True
            if json_obj.contains("include_vectors") and json_obj["include_vectors"].is_bool():
                include_vectors = json_obj["include_vectors"].bool()
            
            var include_metadata = True
            if json_obj.contains("include_metadata") and json_obj["include_metadata"].is_bool():
                include_metadata = json_obj["include_metadata"].bool()
            
            var include_distances = True
            if json_obj.contains("include_distances") and json_obj["include_distances"].is_bool():
                include_distances = json_obj["include_distances"].bool()
            
            var filters_opt = None
            if json_obj.contains("filters") and json_obj["filters"].is_array():
                var filters_array = json_obj["filters"].array()
                var filters = List[QueryFilter]()
                
                for i in range(filters_array.size()):
                    if filters_array[i].is_object():
                        var filter_obj = filters_array[i].object()
                        if filter_obj.contains("field") and filter_obj.contains("op") and filter_obj.contains("value"):
                            var field = filter_obj["field"].string()
                            var op = filter_obj["op"].string()
                            var value = filter_obj["value"].string()
                            
                            var op_code = QueryFilter.OP_EQ
                            if op == "eq":
                                op_code = QueryFilter.OP_EQ
                            elif op == "neq":
                                op_code = QueryFilter.OP_NEQ
                            elif op == "gt":
                                op_code = QueryFilter.OP_GT
                            elif op == "gte":
                                op_code = QueryFilter.OP_GTE
                            elif op == "lt":
                                op_code = QueryFilter.OP_LT
                            elif op == "lte":
                                op_code = QueryFilter.OP_LTE
                            elif op == "contains":
                                op_code = QueryFilter.OP_CONTAINS
                            
                            filters.append(QueryFilter(field, op_code, value))
                
                filters_opt = Optional[List[QueryFilter]](filters)
            
            var query_vector = Vector[dtype].from_list(values)
            var result = self.server.search_vectors(
                query_vector, 
                k, 
                include_vectors, 
                include_metadata, 
                include_distances,
                filters_opt
            )
            
            if result[0]:
                var response_data = Object()
                var results_array = Array()
                
                for i in range(result[2].size()):
                    var record = result[2][i]
                    var result_obj = Object()
                    result_obj["id"] = Value(record.id)
                    
                    if include_distances and record.id in result[3]:
                        result_obj["distance"] = Value(Float64(result[3][record.id]))
                    
                    if include_vectors:
                        result_obj["vector"] = Value(vector_to_json(record.vector))
                    
                    if include_metadata:
                        var metadata = Object()
                        for key in record.metadata.keys():
                            metadata[key] = Value(record.metadata[key])
                        result_obj["metadata"] = Value(metadata)
                    
                    results_array[i] = Value(result_obj)
                
                response_data["results"] = Value(results_array)
                response_data["count"] = Value(result[2].size())
                
                return OK(create_response(True, "Search completed successfully", Optional[Object](response_data)))
            else:
                return BadRequest(create_response(False, result[1]))
        except e:
            self.logger.error("Error searching vectors: " + String(e))
            return BadRequest(create_response(False, "Error searching vectors: " + String(e)))
    
    @always_inline
    fn handle_batch_insert(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/vectors/batch."""
        self.logger.debug("Handling batch insert request")
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("vectors") or not json_obj["vectors"].is_array():
                return BadRequest(create_response(False, "Missing or invalid 'vectors' field"))
            
            var vectors_array = json_obj["vectors"].array()
            var records = List[VectorRecord[dtype]]()
            
            for i in range(vectors_array.size()):
                if not vectors_array[i].is_object():
                    return BadRequest(create_response(False, "Vector items must be objects"))
                
                var vector_obj = vectors_array[i].object()
                
                if not vector_obj.contains("id") or not vector_obj["id"].is_string():
                    return BadRequest(create_response(False, "Missing or invalid 'id' field in vector item"))
                
                if not vector_obj.contains("vector") or not vector_obj["vector"].is_array():
                    return BadRequest(create_response(False, "Missing or invalid 'vector' field in vector item"))
                
                var id = vector_obj["id"].string()
                var vector_array = vector_obj["vector"].array()
                var values = List[Float32]()
                
                for j in range(vector_array.size()):
                    if vector_array[j].is_float() or vector_array[j].is_int():
                        values.append(Float32(vector_array[j].float()))
                    else:
                        return BadRequest(create_response(False, "Vector values must be numbers"))
                
                var vector = Vector[dtype].from_list(values)
                var record = VectorRecord[dtype](id, vector)
                
                if vector_obj.contains("metadata") and vector_obj["metadata"].is_object():
                    var meta_obj = vector_obj["metadata"].object()
                    for key in meta_obj.to_dict().keys():
                        if meta_obj[key].is_string():
                            record.metadata[key] = meta_obj[key].string()
                        else:
                            record.metadata[key] = json_to_string(meta_obj[key])
                
                records.append(record)
            
            var result = self.server.batch_insert_vectors(records)
            
            var response_data = Object()
            response_data["successful_count"] = Value(result[0])
            response_data["failed_count"] = Value(result[1])
            
            if result[1] == 0:
                return OK(create_response(True, "All vectors inserted successfully", Optional[Object](response_data)))
            else:
                return OK(create_response(
                    True, 
                    String(result[0]) + " vectors inserted, " + String(result[1]) + " failed", 
                    Optional[Object](response_data)
                ))
        except e:
            self.logger.error("Error batch inserting vectors: " + String(e))
            return BadRequest(create_response(False, "Error batch inserting vectors: " + String(e)))
    
    @always_inline
    fn handle_count_vectors(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/vectors/count."""
        self.logger.debug("Handling count vectors request")
        
        try:
            var filters_opt = None
            # In a more complete implementation, we would parse filters from query params
            
            var count = self.server.count_vectors(filters_opt)
            
            var response_data = Object()
            response_data["count"] = Value(count)
            
            return OK(create_response(True, "Count retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error counting vectors: " + String(e))
            return BadRequest(create_response(False, "Error counting vectors: " + String(e)))
    
    @always_inline
    fn handle_health_check(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/health."""
        self.logger.debug("Handling health check request")
        
        var response_data = Object()
        response_data["status"] = Value("ok")
        response_data["version"] = Value("1.0.0")
        
        return OK(create_response(True, "OmenDB server is healthy", Optional[Object](response_data)))
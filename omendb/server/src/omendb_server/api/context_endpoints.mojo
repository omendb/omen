"""
OmenDB Context API Endpoints.

This module provides the REST API endpoints for the Vector Context System,
enabling efficient management of contextual information alongside vector embeddings.
"""

from collections import List, Dict, Optional
from lightbug_api import App
from lightbug_http import HTTPRequest, HTTPResponse, OK, NotFound, BadRequest
from emberjson import parse as parse_json, to_string as json_to_string, JSON, Value, Object, Array

from core.vector import Vector
from core.context import ContextSnippet, VectorContext, BudgetParams, ScoredSnippet
from storage.vector_store import QueryFilter
from query.context_manager import ContextManager
from util.logging import Logger, LogLevel

@value
struct ContextEndpoints[dtype: DType]:
    """
    REST API endpoints for the Vector Context System.
    
    This struct provides HTTP handlers for context-related operations.
    """
    
    var context_manager: ContextManager[dtype]
    var logger: Logger
    
    fn __init__(inout self, context_manager: ContextManager[dtype]):
        """
        Initialize context endpoints.
        
        Args:
            context_manager: The context manager implementation
        """
        self.context_manager = context_manager
        self.logger = Logger("ContextEndpoints", LogLevel.INFO)
    
    fn register_routes(inout self, app: App):
        """
        Register context-related routes with the API app.
        
        Args:
            app: The Lightbug API app
        """
        self.logger.debug("Registering context routes")
        
        # Context retrieval endpoints
        app.get("/v1/context/:vector_id", self.handle_get_context)
        app.post("/v1/context", self.handle_store_context)
        app.delete("/v1/context/:vector_id", self.handle_delete_context)
        
        # Snippet management endpoints
        app.post("/v1/context/:vector_id/snippets", self.handle_add_snippet)
        app.get("/v1/context/:vector_id/snippets", self.handle_list_snippets)
        app.get("/v1/context/:vector_id/snippets/:snippet_id", self.handle_get_snippet)
        app.put("/v1/context/:vector_id/snippets/:snippet_id", self.handle_update_snippet)
        app.delete("/v1/context/:vector_id/snippets/:snippet_id", self.handle_delete_snippet)
        
        # Context selection endpoints
        app.post("/v1/context/:vector_id/select", self.handle_select_context)
        app.post("/v1/context/search", self.handle_search_context)
        app.post("/v1/context/query", self.handle_query_context)
    
    @always_inline
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
    
    @always_inline
    fn handle_get_context(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/context/:vector_id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 3:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var vector_id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling get context request for vector ID: " + vector_id)
        
        try:
            var context_opt = self.context_manager.get_context_for_id(vector_id)
            
            if context_opt.none():
                return NotFound(create_response(False, "No context found for vector ID: " + vector_id))
            
            var context = context_opt.value()
            var response_data = Object()
            response_data["vector_id"] = Value(context.vector_id)
            
            var metadata = Object()
            for key in context.metadata.keys():
                metadata[key] = Value(context.metadata[key])
            response_data["metadata"] = Value(metadata)
            
            var snippets = Array()
            for i in range(context.snippets.size()):
                var snippet = context.snippets[i]
                var snippet_obj = Object()
                snippet_obj["id"] = Value(snippet.id)
                snippet_obj["text"] = Value(snippet.text)
                snippet_obj["token_count"] = Value(snippet.token_count)
                
                var snippet_meta = Object()
                for key in snippet.metadata.keys():
                    snippet_meta[key] = Value(snippet.metadata[key])
                snippet_obj["metadata"] = Value(snippet_meta)
                
                snippets[i] = Value(snippet_obj)
            
            response_data["snippets"] = Value(snippets)
            response_data["total_tokens"] = Value(context.total_token_count())
            
            return OK(create_response(True, "Context retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error getting context: " + String(e))
            return BadRequest(create_response(False, "Error getting context: " + String(e)))
    
    @always_inline
    fn handle_store_context(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/context."""
        self.logger.debug("Handling store context request")
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("vector_id") or not json_obj["vector_id"].is_string():
                return BadRequest(create_response(False, "Missing or invalid 'vector_id' field"))
            
            var vector_id = json_obj["vector_id"].string()
            var context = VectorContext[dtype](vector_id)
            
            if json_obj.contains("metadata") and json_obj["metadata"].is_object():
                var meta_obj = json_obj["metadata"].object()
                for key in meta_obj.keys():
                    if meta_obj[key].is_string():
                        context.metadata[key] = meta_obj[key].string()
                    else:
                        context.metadata[key] = json_to_string(meta_obj[key])
            
            if json_obj.contains("snippets") and json_obj["snippets"].is_array():
                var snippets_arr = json_obj["snippets"].array()
                
                for i in range(snippets_arr.size()):
                    if not snippets_arr[i].is_object():
                        return BadRequest(create_response(False, "Snippets must be objects"))
                    
                    var snippet_obj = snippets_arr[i].object()
                    
                    if not snippet_obj.contains("id") or not snippet_obj["id"].is_string():
                        return BadRequest(create_response(False, "Missing or invalid 'id' field in snippet"))
                    
                    if not snippet_obj.contains("text") or not snippet_obj["text"].is_string():
                        return BadRequest(create_response(False, "Missing or invalid 'text' field in snippet"))
                    
                    var snippet_id = snippet_obj["id"].string()
                    var text = snippet_obj["text"].string()
                    var token_count = 0
                    
                    if snippet_obj.contains("token_count") and snippet_obj["token_count"].is_int():
                        token_count = snippet_obj["token_count"].int()
                    
                    var snippet = ContextSnippet(snippet_id, text, token_count)
                    
                    if snippet_obj.contains("metadata") and snippet_obj["metadata"].is_object():
                        var snippet_meta = snippet_obj["metadata"].object()
                        for key in snippet_meta.keys():
                            if snippet_meta[key].is_string():
                                snippet.metadata[key] = snippet_meta[key].string()
                            else:
                                snippet.metadata[key] = json_to_string(snippet_meta[key])
                    
                    context.add_snippet(snippet)
            
            var success = self.context_manager.store_context(context)
            
            if success:
                var response_data = Object()
                response_data["vector_id"] = Value(vector_id)
                response_data["snippet_count"] = Value(context.snippets.size())
                return OK(create_response(True, "Context stored successfully", Optional[Object](response_data)))
            else:
                return BadRequest(create_response(False, "Failed to store context"))
        except e:
            self.logger.error("Error storing context: " + String(e))
            return BadRequest(create_response(False, "Error storing context: " + String(e)))
    
    @always_inline
    fn handle_delete_context(self, req: HTTPRequest) -> HTTPResponse:
        """Handle DELETE /v1/context/:vector_id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 3:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var vector_id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling delete context request for vector ID: " + vector_id)
        
        try:
            var success = self.context_manager.delete_context(vector_id)
            
            if success:
                var response_data = Object()
                response_data["vector_id"] = Value(vector_id)
                return OK(create_response(True, "Context deleted successfully", Optional[Object](response_data)))
            else:
                return NotFound(create_response(False, "No context found for vector ID: " + vector_id))
        except e:
            self.logger.error("Error deleting context: " + String(e))
            return BadRequest(create_response(False, "Error deleting context: " + String(e)))
    
    @always_inline
    fn handle_add_snippet(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/context/:vector_id/snippets."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 4:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var vector_id = path_parts[path_parts.size() - 2]
        self.logger.debug("Handling add snippet request for vector ID: " + vector_id)
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("id") or not json_obj["id"].is_string():
                return BadRequest(create_response(False, "Missing or invalid 'id' field"))
            
            if not json_obj.contains("text") or not json_obj["text"].is_string():
                return BadRequest(create_response(False, "Missing or invalid 'text' field"))
            
            var snippet_id = json_obj["id"].string()
            var text = json_obj["text"].string()
            var token_count = 0
            
            if json_obj.contains("token_count") and json_obj["token_count"].is_int():
                token_count = json_obj["token_count"].int()
            
            var snippet = ContextSnippet(snippet_id, text, token_count)
            
            if json_obj.contains("metadata") and json_obj["metadata"].is_object():
                var meta_obj = json_obj["metadata"].object()
                for key in meta_obj.keys():
                    if meta_obj[key].is_string():
                        snippet.metadata[key] = meta_obj[key].string()
                    else:
                        snippet.metadata[key] = json_to_string(meta_obj[key])
            
            var success = self.context_manager.add_snippet(vector_id, snippet)
            
            if success:
                var response_data = Object()
                response_data["vector_id"] = Value(vector_id)
                response_data["snippet_id"] = Value(snippet_id)
                return OK(create_response(True, "Snippet added successfully", Optional[Object](response_data)))
            else:
                return BadRequest(create_response(False, "Failed to add snippet"))
        except e:
            self.logger.error("Error adding snippet: " + String(e))
            return BadRequest(create_response(False, "Error adding snippet: " + String(e)))
    
    @always_inline
    fn handle_list_snippets(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/context/:vector_id/snippets."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 4:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var vector_id = path_parts[path_parts.size() - 2]
        self.logger.debug("Handling list snippets request for vector ID: " + vector_id)
        
        try:
            var snippets = self.context_manager.list_snippets(vector_id)
            
            var response_data = Object()
            response_data["vector_id"] = Value(vector_id)
            
            var snippets_arr = Array()
            for i in range(snippets.size()):
                var snippet = snippets[i]
                var snippet_obj = Object()
                snippet_obj["id"] = Value(snippet.id)
                snippet_obj["text"] = Value(snippet.text)
                snippet_obj["token_count"] = Value(snippet.token_count)
                
                var meta_obj = Object()
                for key in snippet.metadata.keys():
                    meta_obj[key] = Value(snippet.metadata[key])
                snippet_obj["metadata"] = Value(meta_obj)
                
                snippets_arr[i] = Value(snippet_obj)
            
            response_data["snippets"] = Value(snippets_arr)
            response_data["count"] = Value(snippets.size())
            
            return OK(create_response(True, "Snippets retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error listing snippets: " + String(e))
            return BadRequest(create_response(False, "Error listing snippets: " + String(e)))
    
    @always_inline
    fn handle_get_snippet(self, req: HTTPRequest) -> HTTPResponse:
        """Handle GET /v1/context/:vector_id/snippets/:snippet_id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 5:
            return BadRequest(create_response(False, "Invalid path format"))
        
        var vector_id = path_parts[path_parts.size() - 3]
        var snippet_id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling get snippet request for vector ID: " + vector_id + ", snippet ID: " + snippet_id)
        
        try:
            var snippet_opt = self.context_manager.get_snippet(vector_id, snippet_id)
            
            if snippet_opt.none():
                return NotFound(create_response(False, "Snippet not found"))
            
            var snippet = snippet_opt.value()
            var response_data = Object()
            response_data["id"] = Value(snippet.id)
            response_data["text"] = Value(snippet.text)
            response_data["token_count"] = Value(snippet.token_count)
            
            var meta_obj = Object()
            for key in snippet.metadata.keys():
                meta_obj[key] = Value(snippet.metadata[key])
            response_data["metadata"] = Value(meta_obj)
            
            return OK(create_response(True, "Snippet retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error getting snippet: " + String(e))
            return BadRequest(create_response(False, "Error getting snippet: " + String(e)))
    
    @always_inline
    fn handle_update_snippet(self, req: HTTPRequest) -> HTTPResponse:
        """Handle PUT /v1/context/:vector_id/snippets/:snippet_id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 5:
            return BadRequest(create_response(False, "Invalid path format"))
        
        var vector_id = path_parts[path_parts.size() - 3]
        var snippet_id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling update snippet request for vector ID: " + vector_id + ", snippet ID: " + snippet_id)
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            var text_opt = None
            var token_count_opt = None
            var metadata_opt = None
            
            if json_obj.contains("text") and json_obj["text"].is_string():
                text_opt = Optional[String](json_obj["text"].string())
            
            if json_obj.contains("token_count") and json_obj["token_count"].is_int():
                token_count_opt = Optional[Int](json_obj["token_count"].int())
            
            if json_obj.contains("metadata") and json_obj["metadata"].is_object():
                var meta_obj = json_obj["metadata"].object()
                var metadata = Dict[String, String]()
                
                for key in meta_obj.keys():
                    if meta_obj[key].is_string():
                        metadata[key] = meta_obj[key].string()
                    else:
                        metadata[key] = json_to_string(meta_obj[key])
                
                metadata_opt = Optional[Dict[String, String]](metadata)
            
            var success = self.context_manager.update_snippet(vector_id, snippet_id, text_opt, token_count_opt, metadata_opt)
            
            if success:
                var response_data = Object()
                response_data["vector_id"] = Value(vector_id)
                response_data["snippet_id"] = Value(snippet_id)
                return OK(create_response(True, "Snippet updated successfully", Optional[Object](response_data)))
            else:
                return NotFound(create_response(False, "Snippet not found"))
        except e:
            self.logger.error("Error updating snippet: " + String(e))
            return BadRequest(create_response(False, "Error updating snippet: " + String(e)))
    
    @always_inline
    fn handle_delete_snippet(self, req: HTTPRequest) -> HTTPResponse:
        """Handle DELETE /v1/context/:vector_id/snippets/:snippet_id."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 5:
            return BadRequest(create_response(False, "Invalid path format"))
        
        var vector_id = path_parts[path_parts.size() - 3]
        var snippet_id = path_parts[path_parts.size() - 1]
        self.logger.debug("Handling delete snippet request for vector ID: " + vector_id + ", snippet ID: " + snippet_id)
        
        try:
            var success = self.context_manager.delete_snippet(vector_id, snippet_id)
            
            if success:
                var response_data = Object()
                response_data["vector_id"] = Value(vector_id)
                response_data["snippet_id"] = Value(snippet_id)
                return OK(create_response(True, "Snippet deleted successfully", Optional[Object](response_data)))
            else:
                return NotFound(create_response(False, "Snippet not found"))
        except e:
            self.logger.error("Error deleting snippet: " + String(e))
            return BadRequest(create_response(False, "Error deleting snippet: " + String(e)))
    
    @always_inline
    fn handle_select_context(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/context/:vector_id/select."""
        var path_parts = req.uri.path.split("/")
        if path_parts.size() < 4:
            return BadRequest(create_response(False, "Invalid vector ID"))
        
        var vector_id = path_parts[path_parts.size() - 2]
        self.logger.debug("Handling select context request for vector ID: " + vector_id)
        
        try:
            var budget = BudgetParams()
            
            try:
                var json_data = parse_json(String(req.body_raw))
                if json_data.is_object():
                    var json_obj = json_data.object()
                    
                    if json_obj.contains("max_tokens") and json_obj["max_tokens"].is_int():
                        budget.max_tokens = json_obj["max_tokens"].int()
                    
                    if json_obj.contains("max_snippets") and json_obj["max_snippets"].is_int():
                        budget.max_snippets = json_obj["max_snippets"].int()
                    
                    if json_obj.contains("min_relevance") and (json_obj["min_relevance"].is_float() or json_obj["min_relevance"].is_int()):
                        budget.min_relevance = Float32(json_obj["min_relevance"].float())
            except:
                # Ignore parsing errors, use default budget
                pass
            
            var result = self.context_manager.select_context_for_vector(vector_id, budget)
            
            var response_data = Object()
            response_data["vector_id"] = Value(vector_id)
            response_data["total_tokens"] = Value(result.total_tokens)
            
            var snippets_arr = Array()
            for i in range(result.snippets.size()):
                var scored_snippet = result.snippets[i]
                var snippet_obj = Object()
                snippet_obj["id"] = Value(scored_snippet.snippet.id)
                snippet_obj["text"] = Value(scored_snippet.snippet.text)
                snippet_obj["token_count"] = Value(scored_snippet.snippet.token_count)
                snippet_obj["score"] = Value(Float64(scored_snippet.score))
                
                var meta_obj = Object()
                for key in scored_snippet.snippet.metadata.keys():
                    meta_obj[key] = Value(scored_snippet.snippet.metadata[key])
                snippet_obj["metadata"] = Value(meta_obj)
                
                snippets_arr[i] = Value(snippet_obj)
            
            response_data["snippets"] = Value(snippets_arr)
            response_data["count"] = Value(result.snippets.size())
            
            return OK(create_response(True, "Context selected successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error selecting context: " + String(e))
            return BadRequest(create_response(False, "Error selecting context: " + String(e)))
    
    @always_inline
    fn handle_search_context(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/context/search."""
        self.logger.debug("Handling search context request")
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("query") or not json_obj["query"].is_string():
                return BadRequest(create_response(False, "Missing or invalid 'query' field"))
            
            var query = json_obj["query"].string()
            var filters_opt = None
            
            if json_obj.contains("filters") and json_obj["filters"].is_array():
                var filters_arr = json_obj["filters"].array()
                var filters = List[QueryFilter]()
                
                for i in range(filters_arr.size()):
                    if filters_arr[i].is_object():
                        var filter_obj = filters_arr[i].object()
                        
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
                
                if filters.size() > 0:
                    filters_opt = Optional[List[QueryFilter]](filters)
            
            var results = self.context_manager.search_context(query, filters_opt)
            
            var response_data = Object()
            response_data["query"] = Value(query)
            
            var snippets_arr = Array()
            for i in range(results.size()):
                var scored_snippet = results[i]
                var snippet_obj = Object()
                snippet_obj["id"] = Value(scored_snippet.snippet.id)
                snippet_obj["text"] = Value(scored_snippet.snippet.text)
                snippet_obj["token_count"] = Value(scored_snippet.snippet.token_count)
                snippet_obj["score"] = Value(Float64(scored_snippet.score))
                
                var meta_obj = Object()
                for key in scored_snippet.snippet.metadata.keys():
                    meta_obj[key] = Value(scored_snippet.snippet.metadata[key])
                snippet_obj["metadata"] = Value(meta_obj)
                
                snippets_arr[i] = Value(snippet_obj)
            
            response_data["snippets"] = Value(snippets_arr)
            response_data["count"] = Value(results.size())
            
            return OK(create_response(True, "Search completed successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error searching context: " + String(e))
            return BadRequest(create_response(False, "Error searching context: " + String(e)))
    
    @always_inline
    fn handle_query_context(self, req: HTTPRequest) -> HTTPResponse:
        """Handle POST /v1/context/query."""
        self.logger.debug("Handling query context request")
        
        try:
            var json_data = parse_json(String(req.body_raw))
            if not json_data.is_object():
                return BadRequest(create_response(False, "Invalid JSON request body"))
            
            var json_obj = json_data.object()
            
            if not json_obj.contains("vector") or not json_obj["vector"].is_array():
                return BadRequest(create_response(False, "Missing or invalid 'vector' field"))
            
            var vector_arr = json_obj["vector"].array()
            var values = List[Float32]()
            
            for i in range(vector_arr.size()):
                if vector_arr[i].is_float() or vector_arr[i].is_int():
                    values.append(Float32(vector_arr[i].float()))
                else:
                    return BadRequest(create_response(False, "Vector values must be numbers"))
            
            var query_vector = Vector[dtype](len(values))
            for i in range(len(values)):
                query_vector.data[i] = values[i]
            
            var k = 5
            if json_obj.contains("k") and json_obj["k"].is_int():
                k = json_obj["k"].int()
            
            var budget = BudgetParams()
            if json_obj.contains("max_tokens") and json_obj["max_tokens"].is_int():
                budget.max_tokens = json_obj["max_tokens"].int()
            
            if json_obj.contains("max_snippets") and json_obj["max_snippets"].is_int():
                budget.max_snippets = json_obj["max_snippets"].int()
            
            if json_obj.contains("min_relevance") and (json_obj["min_relevance"].is_float() or json_obj["min_relevance"].is_int()):
                budget.min_relevance = Float32(json_obj["min_relevance"].float())
            
            var filters_opt = None
            if json_obj.contains("filters") and json_obj["filters"].is_array():
                var filters_arr = json_obj["filters"].array()
                var filters = List[QueryFilter]()
                
                for i in range(filters_arr.size()):
                    if filters_arr[i].is_object():
                        var filter_obj = filters_arr[i].object()
                        
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
                
                if filters.size() > 0:
                    filters_opt = Optional[List[QueryFilter]](filters)
            
            var result = self.context_manager.select_context_for_query(
                query_vector, 
                k,
                budget,
                filters_opt
            )
            
            var response_data = Object()
            response_data["total_tokens"] = Value(result.total_tokens)
            
            var snippets_arr = Array()
            for i in range(result.snippets.size()):
                var scored_snippet = result.snippets[i]
                var snippet_obj = Object()
                snippet_obj["id"] = Value(scored_snippet.snippet.id)
                snippet_obj["text"] = Value(scored_snippet.snippet.text)
                snippet_obj["token_count"] = Value(scored_snippet.snippet.token_count)
                snippet_obj["score"] = Value(Float64(scored_snippet.score))
                
                var meta_obj = Object()
                for key in scored_snippet.snippet.metadata.keys():
                    meta_obj[key] = Value(scored_snippet.snippet.metadata[key])
                snippet_obj["metadata"] = Value(meta_obj)
                
                snippets_arr[i] = Value(snippet_obj)
            
            response_data["snippets"] = Value(snippets_arr)
            response_data["count"] = Value(result.snippets.size())
            
            return OK(create_response(True, "Context retrieved successfully", Optional[Object](response_data)))
        except e:
            self.logger.error("Error querying context: " + String(e))
            return BadRequest(create_response(False, "Error querying context: " + String(e)))
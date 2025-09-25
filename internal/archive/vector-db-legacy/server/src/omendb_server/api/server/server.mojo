"""
OmenDB server initialization and configuration.

This module provides the main server entry point and configuration
for starting the OmenDB server with REST API endpoints.
"""

from collections import List, Dict, Optional
from core.vector import Vector
from storage.memory_store import MemoryVectorStore
from storage.file_store import FileVectorStore
from storage.mmap_store import MmapVectorStore
from index.hnsw_index import HnswIndex
from api.server.api_server import OmenDBServer
from api.server.rest_service import OmenDBRestService
from util.logging import Logger, LogLevel

@value
struct ServerConfig:
    """
    Configuration for the OmenDB server.
    
    Attributes:
        host: Server hostname or IP address
        port: Port for REST API
        vector_dimension: Dimensionality of vectors
        data_path: Path for data storage
        storage_type: Type of storage backend ("memory", "file", or "mmap")
        log_level: Logging level
    """
    
    var host: String
    var port: Int
    var vector_dimension: Int
    var data_path: String
    var storage_type: String
    var log_level: LogLevel
    
    fn __init__(
        inout self,
        host: String = "127.0.0.1",
        port: Int = 8080,
        vector_dimension: Int = 128,
        data_path: String = "./data",
        storage_type: String = "memory",
        log_level: LogLevel = LogLevel.INFO
    ):
        self.host = host
        self.port = port
        self.vector_dimension = vector_dimension
        self.data_path = data_path
        self.storage_type = storage_type
        self.log_level = log_level

fn create_server[dtype: DType](config: ServerConfig) raises -> OmenDBServer[dtype]:
    """
    Create and configure the OmenDB server.
    
    Args:
        config: Server configuration
        
    Returns:
        Configured OmenDB server instance
    """
    var logger = Logger("OmenDBServer", config.log_level)
    logger.info("Creating OmenDB server with " + String(config.vector_dimension) + "-dimensional vectors")
    logger.info("Storage type: " + config.storage_type)
    
    # Create the appropriate storage backend
    var store = None
    if config.storage_type == "memory":
        store = MemoryVectorStore[dtype]()
        logger.info("Using in-memory vector store")
    elif config.storage_type == "file":
        store = FileVectorStore[dtype](config.data_path)
        logger.info("Using file-based vector store at " + config.data_path)
    elif config.storage_type == "mmap":
        store = MmapVectorStore[dtype](config.data_path)
        logger.info("Using memory-mapped vector store at " + config.data_path)
    else:
        logger.error("Unknown storage type: " + config.storage_type)
        raise Error("Unknown storage type: " + config.storage_type)
    
    # Create the index
    var index = HnswIndex[dtype]()
    logger.info("Created HNSW index")
    
    # Create the server
    var server = OmenDBServer[dtype](store, index)
    logger.info("OmenDB server created successfully")
    
    return server

fn create_rest_service[dtype: DType](server: OmenDBServer[dtype]) -> OmenDBRestService[dtype]:
    """
    Create the REST service with the given server.
    
    Args:
        server: The OmenDB server
        
    Returns:
        Configured REST service
    """
    return OmenDBRestService[dtype](server)

fn start_server(config: ServerConfig) raises:
    """
    Start the OmenDB server with the specified configuration.
    
    Args:
        config: Server configuration
    """
    var logger = Logger("OmenDBServer", config.log_level)
    logger.info("Starting OmenDB server...")
    
    # Create server components
    var server = create_server[DType.float32](config)
    var rest_service = create_rest_service[DType.float32](server)
    
    logger.info("Server initialized")
    logger.info("REST API available at http://" + config.host + ":" + String(config.port) + "/v1")
    
    # Start the REST server using Lightbug API
    logger.info("Starting REST server...")
    try:
        rest_service.start(config.host, config.port)
    except e:
        logger.error("Failed to start REST server: " + String(e))
        raise Error("Failed to start REST server: " + String(e))
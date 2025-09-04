"""
OmenDB Server main entry point.

This module provides the main entry point for starting the OmenDB server
with command-line arguments for configuration.
"""

from api.server.server import ServerConfig, start_server
from util.logging import LogLevel

fn main() raises:
    """Start the OmenDB server with default configuration."""
    # In a real implementation, this would parse command-line arguments
    # For now, we'll use a default configuration
    var config = ServerConfig(
        host="127.0.0.1",
        port=8080,
        vector_dimension=128,
        data_path="./data",
        storage_type="memory",
        log_level=LogLevel.INFO
    )
    
    # Start the server
    start_server(config)
"""
CLI Commands for OmenDB embedded operations.

Implements dual-mode compatible CLI commands that work with both embedded
and server storage backends through the VectorStore trait interface.

Key Design Principles:
- Same interface for embedded and server modes
- Mode-agnostic command implementations  
- Shared configuration and error handling
- JSON-based data interchange format
"""

from collections import List, Dict, Optional
from core.vector import Vector
from core.record import VectorRecord
from core.metadata import Metadata
from storage.file_store import FileVectorStore
from util.logging import LogLevel
from algorithm import min


struct CLIConfig(Copyable, Movable):
    """Configuration for CLI operations - supports dual-mode backends."""
    var database_path: String
    var log_level: LogLevel
    var default_k: Int
    var json_pretty: Bool
    
    fn __init__(out self, database_path: String, log_level: LogLevel):
        self.database_path = database_path
        self.log_level = log_level
        self.default_k = 10
        self.json_pretty = True
    
    fn __copyinit__(out self, other: Self):
        self.database_path = other.database_path
        self.log_level = other.log_level
        self.default_k = other.default_k
        self.json_pretty = other.json_pretty


struct VectorData(Copyable, Movable):
    """Simple structure for JSON vector data."""
    var id: String
    var data: List[Float32]
    var metadata: Dict[String, String]
    
    fn __init__(out self, id: String, data: List[Float32]):
        self.id = id
        self.data = data
        self.metadata = Dict[String, String]()
    
    fn __copyinit__(out self, other: Self):
        self.id = other.id
        self.data = other.data
        self.metadata = other.metadata


struct CLICommands:
    """
    CLI command implementations for OmenDB.
    
    Designed to work with any VectorStore implementation following
    dual-mode compatibility requirements.
    """
    var config: CLIConfig
    
    fn __init__(out self, config: CLIConfig):
        self.config = config
    
    fn create_storage_backend(self) raises -> FileVectorStore:
        """
        Create appropriate storage backend based on database path.
        
        Future: Will support server backends when available.
        Currently focuses on embedded file storage.
        """
        # For embedded mode, use FileVectorStore  
        var db_path: String
        if self.config.database_path.endswith(".omen"):
            db_path = self.config.database_path
        else:
            # Default to current directory with .omen extension
            db_path = self.config.database_path + ".omen"
        
        return FileVectorStore(db_path, self.config.log_level.level)
    
    fn cmd_create(self) raises -> Int:
        """Create a new OmenDB database."""
        print("Creating OmenDB database: " + self.config.database_path)
        
        try:
            var store = self.create_storage_backend()
            
            # Verify creation by checking count (should be 0)
            var count = store.count()
            print("✅ Database created successfully")
            print("   Path: " + self.config.database_path)
            print("   Initial vector count: " + String(count))
            
            return 0
        except e:
            print("❌ Failed to create database: " + "error occurred")
            return 1
    
    fn cmd_insert(self, json_file: String) raises -> Int:
        """Insert vectors from JSON file into database."""
        print("Inserting vectors from: " + json_file)
        print("Into database: " + self.config.database_path)
        
        # Note: File existence check simplified for demonstration
        print("   Note: File existence check simplified for demonstration")
        
        try:
            var store = self.create_storage_backend()
            var vectors = self.parse_json_vectors(json_file)
            
            var inserted_count = 0
            var total_count = vectors.size
            
            print("Processing " + String(total_count) + " vectors...")
            
            for i in range(vectors.size):
                var vector_data = vectors[i]
                
                # Create Vector from data
                var vec = Vector[DType.float32](vector_data.data.size)
                for j in range(vector_data.data.size):
                    vec[j] = vector_data.data[j]
                
                # Create metadata (simplified)
                var metadata = Metadata()
                
                # Create record
                var record = VectorRecord[DType.float32](vector_data.id, vec, metadata)
                
                # Insert into store
                var success = store.insert(record)
                if success:
                    inserted_count += 1
                    
                # Progress indicator
                if (i + 1) % 100 == 0 or i == total_count - 1:
                    print("  Inserted " + String(i + 1) + "/" + String(total_count))
            
            print("✅ Insert completed")
            print("   Inserted: " + String(inserted_count) + "/" + String(total_count))
            print("   Total vectors in DB: " + String(store.count()))
            
            return 0
        except e:
            print("❌ Failed to insert vectors: " + "error occurred")
            return 1
    
    fn cmd_search(self, query_str: String, k: Int = -1) raises -> Int:
        """Search database with query vector."""
        var search_k = k if k > 0 else self.config.default_k
        
        print("Searching database: " + self.config.database_path)
        print("Query: " + query_str)
        print("Top-k: " + String(search_k))
        
        try:
            var store = self.create_storage_backend()
            var query_vector = self.parse_query_vector(query_str)
            
            # Perform search (simplified demonstration)
            # For demonstration, get all records and return first few
            var all_records = store.get_all()
            var results = List[VectorRecord[DType.float32]]()
            
            var max_results = min(search_k, all_records.size)
            for i in range(max_results):
                results.append(all_records[i])
            
            print("✅ Search completed")
            print("   Results found: " + String(results.size))
            
            # Display results
            if results.size > 0:
                print("\\nTop results:")
                for i in range(results.size):
                    var result = results[i]
                    print("  " + String(i + 1) + ". ID: " + result.id)
                    print("     Vector dim: " + String(result.vector.size))
                    print("     Metadata: present")
            else:
                print("\\nNo results found.")
            
            return 0
        except e:
            print("❌ Search failed: " + "error occurred")
            return 1
    
    fn cmd_info(self) raises -> Int:
        """Show database information and statistics."""
        print("Database information: " + self.config.database_path)
        
        try:
            var store = self.create_storage_backend()
            
            # Basic statistics
            var count = store.count()
            var all_ids = store.get_all_ids()
            
            print("✅ Database statistics:")
            print("   Path: " + self.config.database_path)
            print("   Vector count: " + String(count))
            print("   Storage type: Embedded File Store")
            
            # Sample vector info if available
            if count > 0:
                var sample_record = store.get(all_ids[0])
                if sample_record:
                    var record = sample_record.value()
                    print("   Vector dimension: " + String(record.vector.size))
                    print("   Sample vector ID: " + record.id)
                    print("   Metadata: present")
            
            return 0
        except e:
            print("❌ Failed to get database info: " + "error occurred")
            return 1
    
    fn parse_json_vectors(self, json_file: String) raises -> List[VectorData]:
        """Parse vectors from JSON file."""
        # Simplified JSON parsing - in production would use proper JSON library
        # For now, return empty list and show message
        print("   Note: JSON parsing simplified for demonstration")
        print("   Expected format: {\"vectors\": [{\"id\": \"...\", \"data\": [...], \"metadata\": {...}}]}")
        
        var vectors = List[VectorData]()
        
        # Demonstration: Create a few sample vectors
        var sample_data1 = List[Float32]()
        for i in range(128):  # 128-dimensional vector
            sample_data1.append(Float32(i) * 0.01)
        
        var sample1 = VectorData("demo_vector_1", sample_data1)
        sample1.metadata["source"] = "cli_demo"
        sample1.metadata["type"] = "demonstration"
        vectors.append(sample1)
        
        var sample_data2 = List[Float32]()
        for i in range(128):
            sample_data2.append(Float32(i) * 0.02)
        
        var sample2 = VectorData("demo_vector_2", sample_data2)
        sample2.metadata["source"] = "cli_demo"
        sample2.metadata["type"] = "demonstration"
        vectors.append(sample2)
        
        print("   Loaded " + String(vectors.size) + " demonstration vectors")
        return vectors
    
    fn parse_query_vector(self, query_str: String) raises -> Vector[DType.float32]:
        """Parse query vector from string."""
        # Simplified parsing - in production would support JSON arrays
        print("   Note: Query parsing simplified for demonstration")
        print("   Expected format: JSON array like [0.1, 0.2, 0.3, ...]")
        
        # Create demonstration query vector
        var query = Vector[DType.float32](128, 0.5)  # 128-dim vector with 0.5 values
        print("   Using demonstration query vector (128-dim, value=0.5)")
        
        return query
    
    fn print_help(self):
        """Print CLI help information."""
        print("OmenDB CLI - Embedded Vector Database")
        print("=====================================")
        print("")
        print("USAGE:")
        print("  omendb <command> [options]")
        print("")
        print("COMMANDS:")
        print("  create <database>           Create new embedded database")
        print("  insert <database> <file>    Insert vectors from JSON file")
        print("  search <database> <query>   Search with query vector")
        print("  info <database>             Show database statistics")
        print("  --help                      Show this help message")
        print("")
        print("EXAMPLES:")
        print("  omendb create my_vectors.omen")
        print("  omendb insert my_vectors.omen vectors.json")
        print("  omendb search my_vectors.omen '[0.1,0.2,0.3]'")
        print("  omendb info my_vectors.omen")
        print("")
        print("NOTES:")
        print("  - Database files use .omen extension")
        print("  - JSON format: {\"vectors\": [{\"id\": \"...\", \"data\": [...]}]}")
        print("  - Designed for dual-mode compatibility (embedded + server)")
"""
Storage engine interface and implementations for OmenDB.

Provides pluggable storage backends for persistence.
"""

from collections import List, Dict, Optional
from python import PythonObject, Python
from memory import UnsafePointer, memcpy

trait StorageEngine:
    """Minimal interface for storage backends."""
    
    fn save_vector(mut self, id: String, vector: List[Float32], metadata: Dict[String, String]) raises -> Bool:
        """Save a vector to storage."""
        ...
    
    fn load_vector(self, id: String) raises -> Optional[List[Float32]]:
        """Load a vector from storage."""
        ...
    
    fn delete_vector(mut self, id: String) raises -> Bool:
        """Delete a vector from storage."""
        ...
    
    fn checkpoint(mut self) raises -> Bool:
        """Force persistence to disk."""
        ...
    
    fn recover(mut self) raises -> Int:
        """Load from disk and return number of vectors recovered."""
        ...
    
    fn get_all_ids(self) -> List[String]:
        """Get all vector IDs in storage."""
        ...


struct StorageStats:
    """Statistics about storage recovery."""
    var recovered: Int
    var failed: Int
    var elapsed_ms: Float64
    
    fn __init__(out self, recovered: Int = 0, failed: Int = 0, elapsed_ms: Float64 = 0.0):
        self.recovered = recovered
        self.failed = failed
        self.elapsed_ms = elapsed_ms


struct InMemoryStorage(StorageEngine):
    """No persistence - for testing and benchmarks."""
    var vectors: Dict[String, List[Float32]]
    var metadata: Dict[String, Dict[String, String]]
    
    fn __init__(out self):
        self.vectors = Dict[String, List[Float32]]()
        self.metadata = Dict[String, Dict[String, String]]()
    
    fn __copyinit__(out self, existing: Self):
        self.vectors = existing.vectors
        self.metadata = existing.metadata
    
    fn __moveinit__(out self, owned existing: Self):
        self.vectors = existing.vectors^
        self.metadata = existing.metadata^
    
    fn save_vector(mut self, id: String, vector: List[Float32], metadata: Dict[String, String]) raises -> Bool:
        """Save vector to memory only."""
        self.vectors[id] = vector
        self.metadata[id] = metadata
        return True
    
    fn load_vector(self, id: String) raises -> Optional[List[Float32]]:
        """Load vector from memory."""
        if id in self.vectors:
            return Optional(self.vectors[id])
        return Optional[List[Float32]]()
    
    fn delete_vector(mut self, id: String) raises -> Bool:
        """Delete vector from memory."""
        if id in self.vectors:
            _ = self.vectors.pop(id)
            _ = self.metadata.pop(id)
            return True
        return False
    
    fn checkpoint(mut self) raises -> Bool:
        """No-op for in-memory storage."""
        return True
    
    fn recover(mut self) raises -> Int:
        """Nothing to recover for in-memory storage."""
        return 0
    
    fn get_all_ids(self) -> List[String]:
        """Get all vector IDs."""
        var ids = List[String]()
        for key in self.vectors.keys():
            ids.append(key)
        return ids


struct SnapshotStorage(StorageEngine, Copyable, Movable):
    """SQLite-style single file persistence."""
    var path: String
    var vectors: Dict[String, List[Float32]]
    var metadata: Dict[String, Dict[String, String]]
    var dirty_count: Int
    var checkpoint_interval: Int
    var dimension: Int
    
    fn __init__(out self, path: String, dimension: Int, checkpoint_interval: Int = 10000):
        self.path = path
        self.vectors = Dict[String, List[Float32]]()
        self.metadata = Dict[String, Dict[String, String]]()
        self.dirty_count = 0
        self.checkpoint_interval = checkpoint_interval
        self.dimension = dimension
    
    fn __copyinit__(out self, existing: Self):
        self.path = existing.path
        self.vectors = existing.vectors
        self.metadata = existing.metadata
        self.dirty_count = existing.dirty_count
        self.checkpoint_interval = existing.checkpoint_interval
        self.dimension = existing.dimension
    
    fn __moveinit__(out self, owned existing: Self):
        self.path = existing.path^
        self.vectors = existing.vectors^
        self.metadata = existing.metadata^
        self.dirty_count = existing.dirty_count
        self.checkpoint_interval = existing.checkpoint_interval
        self.dimension = existing.dimension
    
    fn save_vector(mut self, id: String, vector: List[Float32], metadata: Dict[String, String]) raises -> Bool:
        """Save vector to memory and schedule checkpoint."""
        self.vectors[id] = vector
        self.metadata[id] = metadata
        self.dirty_count += 1
        
        # Auto-checkpoint if needed
        if self.dirty_count >= self.checkpoint_interval:
            _ = self.checkpoint()
        
        return True
    
    fn load_vector(self, id: String) raises -> Optional[List[Float32]]:
        """Load vector from memory."""
        if id in self.vectors:
            return Optional(self.vectors[id])
        return Optional[List[Float32]]()
    
    fn delete_vector(mut self, id: String) raises -> Bool:
        """Delete vector and mark as dirty."""
        if id in self.vectors:
            _ = self.vectors.pop(id)
            _ = self.metadata.pop(id)
            self.dirty_count += 1
            return True
        return False
    
    fn checkpoint(mut self) raises -> Bool:
        """Write snapshot to disk atomically."""
        try:
            var python = Python.import_module("builtins")
            var os = Python.import_module("os")
            var struct_module = Python.import_module("struct")
            var json = Python.import_module("json")
            
            # Write to temp file
            var temp_path = self.path + ".tmp"
            var file = python.open(temp_path, "wb")
            
            # Write header
            # Magic: "OMEN" (4 bytes)
            var python_str = python.str("OMEN")
            var magic_bytes = python_str.encode("utf-8")
            file.write(struct_module.pack("4s", magic_bytes))
            
            # Version (4 bytes)
            file.write(struct_module.pack("I", 1))
            
            # Vector count (8 bytes)
            var count = len(self.vectors)
            file.write(struct_module.pack("Q", count))
            
            # Dimension (4 bytes)
            file.write(struct_module.pack("I", self.dimension))
            
            # Reserved (12 bytes)
            file.write(struct_module.pack("12x"))
            
            # Write vectors
            for id in self.vectors.keys():
                var vector = self.vectors[id]
                var meta = self.metadata.get(id, Dict[String, String]())
                
                # Write ID
                var id_str = python.str(id)
                var id_bytes = id_str.encode("utf-8")
                var id_len = python.len(id_bytes)
                file.write(struct_module.pack("I", id_len))
                file.write(id_bytes)
                
                # Write vector
                for val in vector:
                    file.write(struct_module.pack("f", val))
                
                # Write metadata
                var meta_dict = python.dict()
                for key in meta.keys():
                    meta_dict[key] = meta[key]
                var meta_json_str = json.dumps(meta_dict)
                var meta_json = meta_json_str.encode("utf-8")
                var meta_len = python.len(meta_json)
                file.write(struct_module.pack("I", meta_len))
                file.write(meta_json)
            
            file.close()
            
            # Atomic rename
            os.rename(temp_path, self.path)
            self.dirty_count = 0
            
            print("✅ Checkpoint completed:", count, "vectors saved to", self.path)
            return True
            
        except e:
            print("❌ Checkpoint failed:", e)
            return False
    
    fn recover(mut self) raises -> Int:
        """Load snapshot from disk."""
        try:
            var python = Python.import_module("builtins")
            var os = Python.import_module("os")
            var struct_module = Python.import_module("struct")
            var json = Python.import_module("json")
            
            # Check if file exists
            if not os.path.exists(self.path):
                print("📁 No snapshot file found at", self.path)
                return 0
            
            var file = python.open(self.path, "rb")
            
            # Read header
            var magic = file.read(4).decode()
            if magic != "OMEN":
                raise Error("Invalid snapshot file: wrong magic")
            
            var version = struct_module.unpack("I", file.read(4))[0]
            if version != 1:
                raise Error("Unsupported snapshot version: " + String(Int(version)))
            
            var count = Int(struct_module.unpack("Q", file.read(8))[0])
            var dimension = Int(struct_module.unpack("I", file.read(4))[0])
            
            if dimension != self.dimension:
                raise Error("Dimension mismatch: file has " + String(dimension) + 
                          " but expected " + String(self.dimension))
            
            # Skip reserved bytes
            _ = file.read(12)
            
            # Read vectors
            var recovered = 0
            for _ in range(count):
                # Read ID
                var id_len = Int(struct_module.unpack("I", file.read(4))[0])
                var id_bytes = file.read(id_len)
                var id = String(id_bytes.decode("utf-8"))
                
                # Read vector
                var vector = List[Float32]()
                for i in range(dimension):
                    var val_tuple = struct_module.unpack("f", file.read(4))
                    var val_float = Float64(val_tuple[0])
                    var val = Float32(val_float)
                    vector.append(val)
                
                # Read metadata
                var meta_len = Int(struct_module.unpack("I", file.read(4))[0])
                var meta_json = file.read(meta_len).decode()
                var meta_dict = json.loads(meta_json)
                
                var metadata = Dict[String, String]()
                for key in meta_dict.keys():
                    metadata[String(key)] = String(meta_dict[key])
                
                self.vectors[id] = vector
                self.metadata[id] = metadata
                recovered += 1
            
            file.close()
            
            print("✅ Recovery completed:", recovered, "vectors loaded from", self.path)
            return recovered
            
        except e:
            print("❌ Recovery failed:", e)
            return 0
    
    fn get_all_ids(self) -> List[String]:
        """Get all vector IDs."""
        var ids = List[String]()
        for key in self.vectors.keys():
            ids.append(key)
        return ids
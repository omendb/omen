"""Production configuration for OmenDB.

Core parameters for enterprise deployment.
"""

# Essential configuration - just what we actually need
alias DEFAULT_BUFFER_SIZE = 100        # Vectors before flush - reduced from 10K to eliminate 5MB pre-allocation
alias DEFAULT_MAX_MEMORY_MB = 1000       # Memory limit
alias MAX_VECTORS_IN_MEMORY = 50000      # Hard cap on vectors in memory (until disk persistence is fixed)
alias DEFAULT_CHECKPOINT_SEC = 60        # Checkpoint interval
alias DEFAULT_BEAM_WIDTH = 50            # Search beam width
alias DEFAULT_VALIDATE_CHECKSUMS = False # Checksums off by default for performance

@value
struct Config:
    """Runtime configuration for OmenDB.
    
    Only 5 essential fields that actually matter:
    1. buffer_size - How many vectors before flush
    2. max_memory_mb - Memory limit for the database
    3. checkpoint_interval_sec - How often to checkpoint
    4. beam_width - Search accuracy/speed tradeoff
    5. validate_checksums - Whether to validate on read
    """
    var buffer_size: Int
    var max_memory_mb: Int
    var checkpoint_interval_sec: Int
    var beam_width: Int
    var validate_checksums: Bool
    
    fn __init__(out self):
        """Initialize with sensible defaults."""
        self.buffer_size = DEFAULT_BUFFER_SIZE
        self.max_memory_mb = DEFAULT_MAX_MEMORY_MB
        self.checkpoint_interval_sec = DEFAULT_CHECKPOINT_SEC
        self.beam_width = DEFAULT_BEAM_WIDTH
        self.validate_checksums = DEFAULT_VALIDATE_CHECKSUMS
    
    fn with_buffer_size(mut self, size: Int) -> Self:
        """Set buffer size (builder pattern)."""
        self.buffer_size = size
        return self
    
    fn with_memory_limit(mut self, mb: Int) -> Self:
        """Set memory limit in MB."""
        self.max_memory_mb = mb
        return self
    
    fn with_checkpoint_interval(mut self, seconds: Int) -> Self:
        """Set checkpoint interval in seconds."""
        self.checkpoint_interval_sec = seconds
        return self
    
    fn with_beam_width(mut self, width: Int) -> Self:
        """Set search beam width."""
        self.beam_width = width
        return self
        
    fn with_validation(mut self, validate: Bool) -> Self:
        """Enable/disable checksum validation."""
        self.validate_checksums = validate
        return self
    
    fn validate(self) raises -> None:
        """Validate configuration values.
        
        Raises:
            Error if any value is invalid.
        """
        if self.buffer_size < 100 or self.buffer_size > 100000:
            raise Error("buffer_size must be between 100 and 100000")
        
        if self.max_memory_mb < 10 or self.max_memory_mb > 100000:
            raise Error("max_memory_mb must be between 10 and 100000")
        
        if self.checkpoint_interval_sec < 1:
            raise Error("checkpoint_interval_sec must be positive")
        
        if self.beam_width < 1 or self.beam_width > 200:
            raise Error("beam_width must be between 1 and 200")

# Global default configuration (can be modified before creating VectorStore)
var __default_config = Config()
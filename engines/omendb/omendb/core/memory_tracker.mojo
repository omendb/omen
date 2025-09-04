"""
Memory tracking utilities for OmenDB.
Provides idiomatic Mojo memory profiling and tracking.
"""

from collections import List
from sys import sizeof, info
from memory import UnsafePointer
from time import now

struct AllocationInfo(Copyable, Movable):
    """Information about a memory allocation."""
    var size: Int
    var timestamp: Int
    var name: String
    
    fn __init__(out self, size: Int, name: String):
        self.size = size
        self.timestamp = 0  # Simplified for now
        self.name = name

struct MemoryTracker(Copyable, Movable):
    """Track memory allocations and provide statistics."""
    
    var allocations: List[AllocationInfo]  # Simplified from Dict
    var total_allocated: Int
    var total_freed: Int
    var peak_usage: Int
    var enabled: Bool
    
    fn __init__(out self, enabled: Bool = True):
        """Initialize the memory tracker."""
        self.allocations = List[AllocationInfo]()
        self.total_allocated = 0
        self.total_freed = 0
        self.peak_usage = 0
        self.enabled = enabled
    
    fn track_allocation[T: AnyType](mut self, name: String, ptr: UnsafePointer[T], count: Int):
        """Track a memory allocation.
        
        Args:
            name: Name/description of the allocation
            ptr: Pointer to allocated memory
            count: Number of elements allocated
        """
        if not self.enabled:
            return
            
        var size = sizeof[T]() * count
        self.allocations.append(AllocationInfo(size, name))
        self.total_allocated += size
        
        var current_usage = self.total_allocated - self.total_freed
        if current_usage > self.peak_usage:
            self.peak_usage = current_usage
    
    fn track_free(mut self, name: String):
        """Track memory deallocation.
        
        Args:
            name: Name of the allocation being freed
        """
        if not self.enabled:
            return
            
        # Find and remove allocation by name
        for i in range(len(self.allocations)):
            if self.allocations[i].name == name:
                self.total_freed += self.allocations[i].size
                _ = self.allocations.pop(i)
                break
    
    fn get_current_usage(self) -> Int:
        """Get current memory usage in bytes."""
        return self.total_allocated - self.total_freed
    
    fn get_current_usage_mb(self) -> Float64:
        """Get current memory usage in MB."""
        return Float64(self.get_current_usage()) / (1024.0 * 1024.0)
    
    fn report(self):
        """Print a detailed memory report."""
        print("\n" + "="*60)
        print("Memory Usage Report")
        print("="*60)
        
        print(f"\nSummary:")
        print(f"  Total allocated:  {self.total_allocated / (1024*1024):.2f} MB")
        print(f"  Total freed:      {self.total_freed / (1024*1024):.2f} MB")
        print(f"  Current usage:    {self.get_current_usage_mb():.2f} MB")
        print(f"  Peak usage:       {self.peak_usage / (1024*1024):.2f} MB")
        
        if len(self.allocations) > 0:
            print(f"\nActive allocations:")
            # Print allocations (unsorted for simplicity)
            for i in range(len(self.allocations)):
                var info = self.allocations[i]
                print(f"  {info.name}: {Float64(info.size) / (1024.0*1024.0):.2f} MB")
    
    fn reset(mut self):
        """Reset all tracking data."""
        self.allocations.clear()
        self.total_allocated = 0
        self.total_freed = 0
        self.peak_usage = 0

# Global memory tracker instance
var __global_memory_tracker = MemoryTracker()

fn track_allocation[T: AnyType](name: String, ptr: UnsafePointer[T], count: Int):
    """Convenience function to track allocation globally."""
    __global_memory_tracker.track_allocation(name, ptr, count)

fn track_free(name: String):
    """Convenience function to track deallocation globally."""
    __global_memory_tracker.track_free(name)

fn memory_report():
    """Print global memory report."""
    __global_memory_tracker.report()

# Component-specific tracking
struct ComponentMemoryStats(Copyable, Movable):
    """Track memory usage by component."""
    var vectors_memory: Int
    var graph_memory: Int
    var metadata_memory: Int
    var buffer_memory: Int
    var index_memory: Int
    
    fn __init__(out self):
        self.vectors_memory = 0
        self.graph_memory = 0
        self.metadata_memory = 0
        self.buffer_memory = 0
        self.index_memory = 0
    
    fn total(self) -> Int:
        """Get total memory usage."""
        return (self.vectors_memory + self.graph_memory + 
                self.metadata_memory + self.buffer_memory + 
                self.index_memory)
    
    fn report(self):
        """Print component breakdown."""
        print("\nComponent Memory Breakdown:")
        print(f"  Vectors:    {self.vectors_memory / (1024*1024):8.2f} MB")
        print(f"  Graph:      {self.graph_memory / (1024*1024):8.2f} MB")
        print(f"  Metadata:   {self.metadata_memory / (1024*1024):8.2f} MB")
        print(f"  Buffer:     {self.buffer_memory / (1024*1024):8.2f} MB")
        print(f"  Index:      {self.index_memory / (1024*1024):8.2f} MB")
        print(f"  Total:      {self.total() / (1024*1024):8.2f} MB")
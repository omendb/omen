"""
Metadata implementation for OmenDB.

This module provides the metadata structures for storing additional information
alongside vectors in the database.
"""

from collections import List


struct Metadata(Copyable, Movable, Sized):
    """
    A container for storing metadata associated with vectors.
    Uses parallel arrays instead of Dict to avoid 8KB overhead per entry.

    Attributes:
        keys: List of metadata keys
        values: List of metadata values (parallel to keys)
    """

    var keys: List[String]
    var values: List[String]

    fn __init__(out self):
        """Initialize an empty metadata container."""
        self.keys = List[String]()
        self.values = List[String]()

    fn __init__(out self, init_keys: List[String], init_values: List[String]):
        """Initialize from lists of keys and values."""
        self.keys = List[String]()
        self.values = List[String]()
        var min_len = min(len(init_keys), len(init_values))
        for i in range(min_len):
            self.keys.append(init_keys[i])
            self.values.append(init_values[i])

    fn __len__(self) -> Int:
        """Return the number of metadata entries."""
        return len(self.keys)

    fn get(self, key: String) raises -> String:
        """Get the value for a key, or empty string if not found."""
        for i in range(len(self.keys)):
            if self.keys[i] == key:
                return self.values[i]
        return ""

    fn set(mut self, key: String, value: String) raises:
        """Set a key-value pair, updating if the key exists or adding if it doesn't.
        """
        # Check if key exists
        for i in range(len(self.keys)):
            if self.keys[i] == key:
                self.values[i] = value
                return
        # Add new key-value pair
        self.keys.append(key)
        self.values.append(value)

    fn contains(self, key: String) -> Bool:
        """Check if the metadata contains a specific key."""
        for i in range(len(self.keys)):
            if self.keys[i] == key:
                return True
        return False

    fn remove(mut self, key: String) raises -> Bool:
        """Remove a key-value pair. Returns True if removed, False if not found.
        """
        for i in range(len(self.keys)):
            if self.keys[i] == key:
                # Remove by creating new lists without this index
                var new_keys = List[String]()
                var new_values = List[String]()
                for j in range(len(self.keys)):
                    if j != i:
                        new_keys.append(self.keys[j])
                        new_values.append(self.values[j])
                self.keys = new_keys
                self.values = new_values
                return True
        return False

    fn clear(mut self):
        """Remove all metadata entries."""
        self.keys = List[String]()
        self.values = List[String]()

    fn get_all_keys(self) raises -> List[String]:
        """Get a copy of all keys."""
        var result = List[String]()
        for key in self.keys:
            result.append(key)
        return result

    fn get_all_values(self) raises -> List[String]:
        """Get a copy of all values."""
        var result = List[String]()
        for value in self.values:
            result.append(value)
        return result


    fn __str__(self) -> String:
        """Return a string representation of the metadata."""
        var result = String("{")
        for i in range(len(self.keys)):
            if i > 0:
                result = result + ", "
            result = result + self.keys[i] + ": " + self.values[i]
        result = result + "}"
        return result

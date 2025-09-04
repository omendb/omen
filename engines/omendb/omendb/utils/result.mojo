"""Production Result type for error handling.

Efficient discriminated union design.
"""

# Common error codes
alias OK = 0
alias ERR_DIMENSION = 1
alias ERR_NOT_FOUND = 2
alias ERR_IO = 3
alias ERR_MEMORY = 4
alias ERR_INVALID = 5

@value
struct Error:
    """Error with code and message."""
    var code: Int
    var message: String
    
    fn __init__(out self, code: Int = OK, message: String = ""):
        self.code = code
        self.message = message

@value
struct Result[T: AnyType]:
    """Result type using discriminated union pattern.
    
    Either contains a value OR an error, never both.
    No redundant Optional wrapper.
    """
    # Discriminated union - only one is valid
    var _is_ok: Bool
    var _value: T  # Only valid if _is_ok = True
    var _error: Error  # Only valid if _is_ok = False
    
    @staticmethod
    fn ok(value: T) -> Result[T]:
        """Create successful result."""
        var result = Result[T].__new__()
        result._is_ok = True
        result._value = value
        result._error = Error()  # Placeholder
        return result
    
    @staticmethod
    fn error(code: Int, message: String) -> Result[T]:
        """Create error result."""
        var result = Result[T].__new__()
        result._is_ok = False
        # Note: _value is uninitialized here - that's OK since we check _is_ok
        result._error = Error(code, message)
        return result
    
    fn is_ok(self) -> Bool:
        """Check if result is success."""
        return self._is_ok
    
    fn is_error(self) -> Bool:
        """Check if result is error."""
        return not self._is_ok
    
    fn value(self) raises -> T:
        """Get value or raise error."""
        if not self._is_ok:
            raise Error(self._error.message)
        return self._value
    
    fn value_or(self, default: T) -> T:
        """Get value or return default."""
        if self._is_ok:
            return self._value
        return default
    
    fn error(self) -> Error:
        """Get error (undefined if ok)."""
        return self._error
    
    fn and_then[U: AnyType](self, f: fn(T) raises -> Result[U]) raises -> Result[U]:
        """Chain operations that might fail."""
        if not self._is_ok:
            return Result[U].error(self._error.code, self._error.message)
        return f(self._value)
    
    fn or_else(self, default: T) -> Result[T]:
        """Return self if ok, otherwise new ok result with default."""
        if self._is_ok:
            return self
        return Result[T].ok(default)

# Helper functions
fn ok[T: AnyType](value: T) -> Result[T]:
    """Create successful result."""
    return Result[T].ok(value)

fn error[T: AnyType](code: Int, message: String) -> Result[T]:
    """Create error result."""
    return Result[T].error(code, message)
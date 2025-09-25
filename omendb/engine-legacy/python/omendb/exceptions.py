"""
Simplified exception classes for OmenDB embedded package.

Provides just 2 exception types for clean embedded API:
- DatabaseError: database operations, file I/O, native module issues
- ValidationError: input validation, invalid parameters
"""

from typing import Optional, Dict, Any


class OmenDBError(Exception):
    """Base exception class for all OmenDB errors."""

    def __init__(self, message: str, context: Optional[Dict[str, Any]] = None):
        super().__init__(message)
        self.context = context or {}

    def __str__(self) -> str:
        base_msg = super().__str__()
        if self.context:
            context_str = ", ".join(f"{k}={v}" for k, v in self.context.items())
            base_msg = f"{base_msg} (context: {context_str})"
        return base_msg


class DatabaseError(OmenDBError):
    """Database operation failed.

    Covers all database operations, file I/O, native module issues,
    connection problems, memory issues, and system errors.
    """

    pass


class ValidationError(OmenDBError):
    """Input validation failed.

    Covers invalid parameters, dimension mismatches, invalid vectors,
    malformed queries, and other user input errors.
    """

    pass


# Helper function for creating contextual errors
def create_context_error(
    base_error: Exception, operation: str, **context
) -> OmenDBError:
    """Create contextual error from base exception."""
    error_msg = f"Operation '{operation}' failed: {str(base_error)}"

    if isinstance(base_error, OmenDBError):
        # Preserve OmenDB error type and add context
        error_class = type(base_error)
        return error_class(
            error_msg, context={**getattr(base_error, "context", {}), **context}
        )
    else:
        # Wrap non-OmenDB errors as DatabaseError
        return DatabaseError(error_msg, context=context)

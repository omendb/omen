"""Safe Optional handling utilities for production stability.

Provides safe access patterns to eliminate panics from .value() calls.
"""

from sys import exit
from collections import Optional

fn optional_or[T: Copyable & Movable](opt: Optional[T], default: T) -> T:
    """Get value from Optional or return default."""
    if opt:
        return opt.value()  # Safe - we checked
    return default

fn optional_or_else[T: Copyable & Movable](opt: Optional[T], f: fn() -> T) -> T:
    """Get value from Optional or compute default."""
    if opt:
        return opt.value()  # Safe - we checked
    return f()

fn optional_or_raise[T: Copyable & Movable](opt: Optional[T], message: String) raises -> T:
    """Get value from Optional or raise with custom message."""
    if opt:
        return opt.value()  # Safe - we checked
    raise Error(message)

fn optional_map[T: Copyable, U: Copyable & Movable](opt: Optional[T], f: fn(T) -> U) -> Optional[U]:
    """Transform Optional value if present."""
    if opt:
        return Optional[U](f(opt.value()))  # Safe - we checked
    return Optional[U]()

fn optional_and_then[T: Copyable, U: Copyable & Movable](opt: Optional[T], f: fn(T) -> Optional[U]) -> Optional[U]:
    """Chain Optional operations."""
    if opt:
        return f(opt.value())  # Safe - we checked
    return Optional[U]()

fn optional_filter[T: Copyable & Movable](opt: Optional[T], predicate: fn(T) -> Bool) -> Optional[T]:
    """Filter Optional value with predicate."""
    if opt:
        var value = opt.value()  # Safe - we checked
        if predicate(value):
            return Optional[T](value)
    return Optional[T]()

# Safe access for production stability
fn safe_unwrap[T: Copyable & Movable](opt: Optional[T], context: String) raises -> T:
    """Safely unwrap Optional with context for debugging.
    
    Raises with context instead of panicking.
    """
    if opt:
        return opt.value()  # Safe - we checked
    
    # Raise with context for better debugging
    raise Error("Optional was None in context: " + context)

fn optional_exists[T: Copyable & Movable](opt: Optional[T]) -> Bool:
    """Check if Optional has a value."""
    return opt.__bool__()

fn optional_is_none[T: Copyable & Movable](opt: Optional[T]) -> Bool:
    """Check if Optional is None."""
    return not opt.__bool__()
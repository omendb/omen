"""
Main CLI entry point for OmenDB embedded operations.

Provides command-line interface for vector database operations with
dual-mode compatibility design.
"""

from collections import List
from cli.commands import CLICommands, CLIConfig
from util.logging import LogLevel


fn parse_args(args: List[String]) -> (String, List[String]):
    """Parse command line arguments."""
    if len(args) < 2:
        return ("help", List[String]())
    
    var command = args[1]
    var remaining_args = List[String]()
    
    for i in range(2, len(args)):
        remaining_args.append(args[i])
    
    return (command, remaining_args)


fn main() raises:
    """
    Main CLI entry point for OmenDB.
    
    Supports the following commands:
    - create <database>: Create new database
    - insert <database> <json_file>: Insert vectors from JSON
    - search <database> <query> [k]: Search with query vector
    - info <database>: Show database statistics
    - --help: Show help information
    """
    
    # For demonstration, simulate command line arguments
    # In production, would use proper argument parsing
    var args = List[String]()
    args.append("omendb")  # Program name
    args.append("--help")  # Default to help for demonstration
    
    var command: String
    var cmd_args: List[String]
    command, cmd_args = parse_args(args)
    
    # Handle help command
    if command == "--help" or command == "help" or command == "-h":
        var config = CLIConfig("", LogLevel(LogLevel.INFO))
        var cli = CLICommands(config)
        cli.print_help()
        return
    
    # Validate command and arguments
    if command == "create":
        if len(cmd_args) < 1:
            print("❌ Error: 'create' command requires database path")
            print("Usage: omendb create <database>")
            print("Try: omendb --help")
            return
        
        var database_path = cmd_args[0]
        var config = CLIConfig(database_path, LogLevel(LogLevel.INFO))
        var cli = CLICommands(config)
        var result = cli.cmd_create()
        
        if result != 0:
            print("Command failed with code: " + String(result))
    
    elif command == "insert":
        if len(cmd_args) < 2:
            print("❌ Error: 'insert' command requires database and JSON file")
            print("Usage: omendb insert <database> <json_file>")
            print("Try: omendb --help")
            return
        
        var database_path = cmd_args[0]
        var json_file = cmd_args[1]
        var config = CLIConfig(database_path, LogLevel(LogLevel.INFO))
        var cli = CLICommands(config)
        var result = cli.cmd_insert(json_file)
        
        if result != 0:
            print("Command failed with code: " + String(result))
    
    elif command == "search":
        if len(cmd_args) < 2:
            print("❌ Error: 'search' command requires database and query")
            print("Usage: omendb search <database> <query> [k]")
            print("Try: omendb --help")
            return
        
        var database_path = cmd_args[0]
        var query = cmd_args[1]
        var k = 10  # Default
        
        if len(cmd_args) >= 3:
            # Simple int parsing - in production would use proper parsing
            var k_str = cmd_args[2]
            if k_str == "5":
                k = 5
            elif k_str == "20":
                k = 20
            else:
                print("❌ Warning: Simplified parsing, using default k=10")
                k = 10
        
        var config = CLIConfig(database_path, LogLevel(LogLevel.INFO))
        var cli = CLICommands(config)
        var result = cli.cmd_search(query, k)
        
        if result != 0:
            print("Command failed with code: " + String(result))
    
    elif command == "info":
        if len(cmd_args) < 1:
            print("❌ Error: 'info' command requires database path")
            print("Usage: omendb info <database>")
            print("Try: omendb --help")
            return
        
        var database_path = cmd_args[0]
        var config = CLIConfig(database_path, LogLevel(LogLevel.INFO))
        var cli = CLICommands(config)
        var result = cli.cmd_info()
        
        if result != 0:
            print("Command failed with code: " + String(result))
    
    else:
        print("❌ Error: Unknown command '" + command + "'")
        print("Available commands: create, insert, search, info, --help")
        print("Try: omendb --help")
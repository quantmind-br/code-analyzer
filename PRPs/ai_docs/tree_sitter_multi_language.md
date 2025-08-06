# Tree-sitter Multi-Language AST Parsing Guide

## Overview

Tree-sitter is a parser generator tool and incremental parsing library that creates fast, efficient parsers for various programming languages. This document provides implementation guidance for using tree-sitter in Rust for multi-language code analysis.

## Core Architecture

### Parser Setup
```rust
use tree_sitter::{Language, Parser, Node};

// Create a parser instance
let mut parser = Parser::new();

// Set language grammar
parser.set_language(&tree_sitter_rust::LANGUAGE.into())
    .expect("Error loading Rust grammar");
```

### Multi-Language Support Pattern
```rust
// Language detection and parser configuration
fn get_language_parser(file_extension: &str) -> Option<Language> {
    match file_extension {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "js" | "jsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        "java" => Some(tree_sitter_java::LANGUAGE.into()),
        "cpp" | "cc" | "cxx" => Some(tree_sitter_cpp::LANGUAGE.into()),
        "c" | "h" => Some(tree_sitter_c::LANGUAGE.into()),
        "go" => Some(tree_sitter_go::LANGUAGE.into()),
        "ts" | "tsx" => Some(tree_sitter_typescript::language_typescript()),
        _ => None,
    }
}
```

## AST Traversal Patterns

### Function Detection
```rust
fn count_functions(node: &Node, source: &[u8]) -> usize {
    let mut count = 0;
    
    // Language-specific function node types
    match node.kind() {
        // Rust
        "function_item" => count += 1,
        // JavaScript/TypeScript  
        "function_declaration" | "function_expression" | "arrow_function" => count += 1,
        // Python
        "function_definition" => count += 1,
        // Java
        "method_declaration" | "constructor_declaration" => count += 1,
        // C/C++
        "function_definition" | "function_declarator" => count += 1,
        _ => {}
    }
    
    // Recursively traverse children
    for child in node.children(&mut node.walk()) {
        count += count_functions(&child, source);
    }
    
    count
}
```

### Class Detection
```rust
fn count_classes(node: &Node, source: &[u8]) -> usize {
    let mut count = 0;
    
    match node.kind() {
        // Rust
        "struct_item" | "enum_item" | "impl_item" => count += 1,
        // JavaScript/TypeScript
        "class_declaration" => count += 1,
        // Python  
        "class_definition" => count += 1,
        // Java
        "class_declaration" | "interface_declaration" | "enum_declaration" => count += 1,
        // C++
        "class_specifier" | "struct_specifier" => count += 1,
        _ => {}
    }
    
    for child in node.children(&mut node.walk()) {
        count += count_classes(&child, source);
    }
    
    count
}
```

## Error Handling

### Parse Error Recovery
```rust
fn parse_file_safely(parser: &mut Parser, source: &[u8]) -> Option<tree_sitter::Tree> {
    match parser.parse(source, None) {
        Some(tree) => {
            // Check for parse errors
            if tree.root_node().has_error() {
                eprintln!("Parse errors detected, results may be incomplete");
            }
            Some(tree)
        }
        None => {
            eprintln!("Failed to parse file");
            None
        }
    }
}
```

## Performance Considerations

### Incremental Parsing
```rust
// For large files, use incremental parsing
fn update_tree(parser: &mut Parser, old_tree: &tree_sitter::Tree, source: &[u8]) -> Option<tree_sitter::Tree> {
    parser.parse(source, Some(old_tree))
}
```

### Memory Management
- Tree-sitter trees are automatically memory-managed
- Reuse parser instances across files for better performance
- Language grammars are statically linked, no runtime loading overhead

## Language Grammar Dependencies

Add to Cargo.toml:
```toml
[dependencies]
tree-sitter = "0.20"
tree-sitter-rust = "0.20"
tree-sitter-javascript = "0.20"  
tree-sitter-python = "0.20"
tree-sitter-java = "0.20"
tree-sitter-c = "0.20"
tree-sitter-cpp = "0.20"
tree-sitter-go = "0.20"
tree-sitter-typescript = "0.20"
```

## Common Gotchas

1. **Different Node Types**: Each language has different node types for similar constructs
2. **Anonymous Nodes**: Some nodes are anonymous and need special handling
3. **Error Nodes**: Parse errors create error nodes that should be handled gracefully
4. **UTF-8 Assumptions**: Tree-sitter works with byte indices, ensure proper UTF-8 handling
5. **Language Loading**: Language grammars must be loaded once and can fail if incompatible

## Implementation Strategy

1. **Start Simple**: Begin with one language, expand gradually
2. **Test with Real Code**: Use actual codebases for testing, not just toy examples  
3. **Handle Errors Gracefully**: Never panic on parse errors, log and continue
4. **Profile Performance**: Large codebases can be memory-intensive
5. **Cache Results**: For repeated analysis, cache parsed trees when possible
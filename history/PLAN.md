# Code Analyzer - Plano de Melhorias

## Visão Geral

Este documento detalha o plano de implementação das melhorias sugeridas para o Code Analyzer, focando em:
1. Maior acurácia nas métricas de análise
2. Melhorias de performance e arquitetura
3. Melhor experiência do usuário (UX)
4. Robustez e qualidade de código

## Arquitetura Atual

```
src/
├── lib.rs              # API principal
├── main.rs             # Entry point CLI
├── cli.rs              # Parsing de argumentos (clap)
├── error.rs            # Tratamento de erros
├── analyzer/
│   ├── mod.rs          # AnalyzerEngine - orquestração
│   ├── language.rs     # LanguageManager, SupportedLanguage, NodeKindMapper
│   ├── parser.rs       # FileParser, FileAnalysis, contagem de métricas
│   └── walker.rs       # FileWalker - descoberta de arquivos
└── output/
    ├── mod.rs          # OutputManager
    ├── terminal.rs     # TerminalReporter - tabelas pretty
    └── json.rs         # JsonExporter - saída JSON
```

## Melhorias Propostas

### 1. Contagem de Comentários via AST (Prioridade: Alta)

**Problema**: A função `is_comment_line` usa heurística baseada em prefixos de string, que pode falhar com:
- Comentários de bloco multi-linha
- Strings que começam com `//` ou `#`
- Linguagens com sintaxe diferente

**Solução**: Usar o AST do tree-sitter para identificar nós de comentário.

**Implementação**:
1. Adicionar método `comment_node_kinds()` ao trait `NodeKindMapper`
2. Criar função `count_comment_lines_ast()` que percorre o AST
3. Calcular linhas de comentário pela posição dos nós no source

**Arquivos afetados**:
- `src/analyzer/language.rs` - adicionar `comment_node_kinds()`
- `src/analyzer/parser.rs` - nova função de contagem

### 2. Complexidade Ciclomática (Prioridade: Alta)

**Problema**: A métrica atual é simplista:
```rust
complexity = LOC/100 + 0.5*sqrt(functions) + 0.3*sqrt(classes)
```

**Solução**: Implementar Complexidade Ciclomática real, contando nós de controle de fluxo.

**Implementação**:
1. Adicionar método `control_flow_node_kinds()` ao trait `NodeKindMapper`
2. Criar função `calculate_cyclomatic_complexity()`
3. Adicionar campo `cyclomatic_complexity` ao `FileAnalysis`
4. Atualizar `complexity_score` para usar a nova métrica

**Nós de controle de fluxo por linguagem**:
- Rust: `if_expression`, `match_expression`, `for_expression`, `while_expression`, `loop_expression`
- JavaScript/TypeScript: `if_statement`, `for_statement`, `while_statement`, `switch_statement`, `ternary_expression`
- Python: `if_statement`, `for_statement`, `while_statement`, `try_statement`
- Java: `if_statement`, `for_statement`, `while_statement`, `switch_expression`, `try_statement`
- C/C++: `if_statement`, `for_statement`, `while_statement`, `switch_statement`
- Go: `if_statement`, `for_statement`, `switch_statement`, `select_statement`

### 3. Diferenciação de Funções/Métodos (Prioridade: Média)

**Problema**: Campo `functions` conta todas as declarações sem distinção.

**Solução**: Separar em `functions` (livres) e `methods` (associados a classes/structs).

**Implementação**:
1. Adicionar método `method_node_kinds()` ao trait `NodeKindMapper`
2. Atualizar `FileAnalysis` com novo campo `methods`
3. Criar função `count_methods()` separada
4. Atualizar output (terminal + JSON)

### 4. Travessia Iterativa do AST (Prioridade: Alta)

**Problema**: `count_functions` e `count_classes` usam recursão, risco de stack overflow.

**Solução**: Usar `TreeCursor` para travessia iterativa.

**Implementação**:
```rust
fn count_nodes_iterative(root: &Node, is_target: impl Fn(&str) -> bool) -> usize {
    let mut count = 0;
    let mut cursor = root.walk();
    loop {
        if is_target(cursor.node().kind()) {
            count += 1;
        }
        if cursor.goto_first_child() { continue; }
        loop {
            if cursor.goto_next_sibling() { break; }
            if !cursor.goto_parent() { return count; }
        }
    }
}
```

### 5. Tratamento de Erros Estruturado (Prioridade: Média)

**Problema**: `parse_file_safely` usa `eprintln!` para erros parciais.

**Solução**: Coletar erros de parse em estrutura dedicada.

**Implementação**:
1. Criar struct `ParseWarning` para erros não-fatais
2. Adicionar campo `warnings` ao `AnalysisReport`
3. Consolidar warnings no sumário final

### 6. Flag --compact para CLI (Prioridade: Baixa)

**Problema**: Não há forma fácil de obter saída mínima para CI/CD.

**Solução**: Adicionar flag `--compact` que mostra apenas métricas essenciais.

**Implementação**:
1. Adicionar `compact: bool` ao `CliArgs`
2. Adicionar variante `Compact` ao `OutputFormat`
3. Atualizar `TerminalReporter` para modo compacto

### 7. Otimização do LanguageManager (Prioridade: Baixa)

**Problema**: `LanguageManager` é criado 3 vezes em `from_cli_args`.

**Solução**: Criar uma vez e clonar para componentes que precisam.

**Implementação**: Refatorar `AnalyzerEngine::from_cli_args()` para reusar instância.

## Ordem de Implementação

1. **Fase 1 - Robustez** (fundação)
   - Travessia iterativa do AST
   - Tratamento de erros estruturado

2. **Fase 2 - Métricas** (valor principal)
   - Complexidade ciclomática
   - Contagem de comentários via AST
   - Diferenciação funções/métodos

3. **Fase 3 - UX** (polimento)
   - Flag --compact
   - Otimização LanguageManager

## Critérios de Sucesso

- [ ] Todos os testes existentes passam
- [ ] Novos testes cobrem funcionalidades adicionadas
- [ ] `cargo clippy -- -D warnings` sem erros
- [ ] `cargo fmt --check` sem erros
- [ ] Performance não degrada significativamente

## Notas Técnicas

### Compatibilidade
- Manter retrocompatibilidade na saída JSON
- Novos campos são opcionais ou têm valores default

### Testes
- Adicionar testes unitários para cada nova função
- Adicionar testes de integração para novas flags CLI
- Usar arquivos de teste existentes em `test_mixed_languages/`

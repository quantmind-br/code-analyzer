# Code Analyzer - Melhorias v2.0

## Briefing
Implementação de melhorias no Code Analyzer baseadas em análise de código:
- Maior acurácia nas métricas via AST
- Complexidade ciclomática real
- Melhor robustez e UX

## Tarefas

### Fase 1 - Robustez

- [ ] **1.1 Travessia Iterativa do AST**
  - [ ] Refatorar `count_functions()` para usar `TreeCursor`
  - [ ] Refatorar `count_classes()` para usar `TreeCursor`
  - [ ] Criar função genérica `count_nodes_iterative()`
  - [ ] Adicionar testes unitários

- [ ] **1.2 Tratamento de Erros Estruturado**
  - [ ] Criar struct `ParseWarning` em `error.rs`
  - [ ] Modificar `parse_file_safely()` para retornar warnings
  - [ ] Adicionar campo `warnings` ao `AnalysisReport`
  - [ ] Atualizar output para exibir warnings consolidados

### Fase 2 - Métricas

- [ ] **2.1 Complexidade Ciclomática**
  - [ ] Adicionar `control_flow_node_kinds()` ao trait `NodeKindMapper`
  - [ ] Implementar mapeamento para todas as 8 linguagens
  - [ ] Criar função `calculate_cyclomatic_complexity()`
  - [ ] Adicionar campo `cyclomatic_complexity` ao `FileAnalysis`
  - [ ] Atualizar `complexity_score` para usar nova métrica
  - [ ] Atualizar output (terminal + JSON)
  - [ ] Adicionar testes

- [ ] **2.2 Contagem de Comentários via AST**
  - [ ] Adicionar `comment_node_kinds()` ao trait `NodeKindMapper`
  - [ ] Implementar mapeamento para todas as 8 linguagens
  - [ ] Criar função `count_comment_lines_ast()`
  - [ ] Substituir heurística por contagem AST
  - [ ] Adicionar testes

- [ ] **2.3 Diferenciação Funções/Métodos**
  - [ ] Adicionar `method_node_kinds()` ao trait `NodeKindMapper`
  - [ ] Adicionar campo `methods` ao `FileAnalysis`
  - [ ] Criar função `count_methods()`
  - [ ] Atualizar `ProjectSummary` com `total_methods`
  - [ ] Atualizar output (terminal + JSON)
  - [ ] Adicionar testes

### Fase 3 - UX

- [ ] **3.1 Flag --compact**
  - [ ] Adicionar `compact: bool` ao `CliArgs`
  - [ ] Adicionar variante `Compact` ao `OutputFormat`
  - [ ] Implementar `display_compact_table()` em `TerminalReporter`
  - [ ] Atualizar documentação CLI
  - [ ] Adicionar testes de integração

- [ ] **3.2 Otimização LanguageManager**
  - [ ] Refatorar `AnalyzerEngine::from_cli_args()`
  - [ ] Criar instância única e clonar
  - [ ] Verificar que testes continuam passando

### Validação Final

- [ ] Executar `cargo fmt --check`
- [ ] Executar `cargo clippy -- -D warnings`
- [ ] Executar `cargo test`
- [ ] Executar `cargo build --release`
- [ ] Testar manualmente com projeto real

## Progresso

| Fase | Tarefa | Status |
|------|--------|--------|
| 1 | Travessia Iterativa | Pendente |
| 1 | Erros Estruturados | Pendente |
| 2 | Complexidade Ciclomática | Pendente |
| 2 | Comentários AST | Pendente |
| 2 | Funções/Métodos | Pendente |
| 3 | Flag --compact | Pendente |
| 3 | Otimização LanguageManager | Pendente |

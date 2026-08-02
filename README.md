# Code Analyzer

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/quantmind-br/code-analyzer/actions/workflows/ci.yml/badge.svg)](https://github.com/quantmind-br/code-analyzer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)

Uma ferramenta CLI poderosa para analisar bases de código e identificar candidatos a refatoração usando análise AST (Abstract Syntax Tree). Desenvolva código mais limpo e mantenha a qualidade técnica do seu projeto com métricas precisas e relatórios detalhados.

## 🚀 Funcionalidades

### 📊 Análise Precisa de Código
- **Parsing AST**: Utiliza tree-sitter para análise precisa de código fonte
- **Múltiplas Linguagens**: Suporte para Rust, JavaScript, Python, Java, C, C++, Go e TypeScript
- **Métricas Detalhadas**: Conta linhas de código, funções, classes e calcula pontuação de complexidade
- **Respeito ao .gitignore**: Ignora automaticamente arquivos especificados no .gitignore

### 🎯 Filtragem e Personalização
- **Filtros Flexíveis**: Por número de linhas, funções, classes e tamanho de arquivo
- **Ordenação Inteligente**: Por linhas, funções, classes ou complexidade
- **Seleção de Linguagens**: Analise apenas as linguagens que importam
- **Padrões de Exclusão**: Adicione padrões personalizados além do .gitignore

### 📈 Saída Dupla
- **Terminal Interativo**: Tabelas formatadas com cores e informações claras
- **Exportação JSON**: Dados estruturados para integração CI/CD e automação
- **Relatórios Customizáveis**: Limite resultados e personalize a saída

### ⚡ Performance
- **Processamento Paralelo**: Análise rápida usando todos os cores disponíveis
- **Barra de Progresso**: Acompanhe o progresso em tempo real
- **Otimizado para Grandes Projetos**: Eficiente mesmo em bases de código extensas

## 🛠️ Instalação

### Pré-requisitos
- Rust 1.70 ou superior
- Git (para clonagem do repositório)

### Instalação via Cargo (Recomendado)
```bash
# Clone o repositório
git clone https://github.com/quantmind-br/code-analyzer.git
cd code-analyzer

# Instale localmente
cargo install --path .
```

### Instalação via Crates.io (Em breve)
```bash
cargo install code-analyzer
```

### Build Manual
```bash
# Clone e compile
git clone https://github.com/quantmind-br/code-analyzer.git
cd code-analyzer
cargo build --release

# O executável estará em target/release/code-analyzer
```

## 📋 Uso

### Análise Básica
```bash
# Analisa o diretório atual
code-analyzer

# Analisa um diretório específico
code-analyzer /caminho/para/projeto

# Análise com saída detalhada
code-analyzer --verbose
```

### Filtragem e Personalização
```bash
# Apenas arquivos com mais de 100 linhas
code-analyzer --min-lines 100

# Filtrar por múltiplos critérios
code-analyzer --min-lines 50 --min-functions 5 --max-lines 1000

# Analisar apenas linguagens específicas
code-analyzer --languages rust,python,javascript

# Ordenar por complexidade
code-analyzer --sort complexity
```

### Opções de Saída
```bash
# Apenas saída JSON
code-analyzer --output json

# Salvar JSON em arquivo personalizado
code-analyzer --output-file meu-relatorio.json

# Limitar resultados exibidos no terminal
code-analyzer --limit 20

# Apenas arquivo JSON, sem saída no terminal
code-analyzer --json-only
```

### Exclusão de Arquivos
```bash
# Excluir padrões adicionais
code-analyzer --exclude "*.test.js,*.spec.py"

# Incluir arquivos ocultos
code-analyzer --include-hidden

# Limitar tamanho máximo de arquivo (em MB)
code-analyzer --max-file-size-mb 5
```

## 📊 Exemplo de Saída

### Terminal
```
┌─────────────────────────────┬──────────┬────────┬─────────┬───────────┬─────────────────┐
│ File                        │ Language │ Lines  │ Funcs   │ Classes   │ Complexity      │
├─────────────────────────────┼──────────┼────────┼─────────┼───────────┼─────────────────┤
│ src/analyzer/parser.rs      │ rust     │ 450    │ 25      │ 3         │ 5.2            │
│ src/cli.rs                  │ rust     │ 240    │ 15      │ 7         │ 5.1            │
│ src/lib.rs                  │ rust     │ 210    │ 18      │ 2         │ 4.8            │
│ PRPs/scripts/prp_runner.py  │ python   │ 210    │ 5       │ 0         │ 3.2            │
└─────────────────────────────┴──────────┴────────┴─────────┴───────────┴─────────────────┘

Analysis completed: 45 files analyzed
Total lines: 15,420 | Total functions: 234 | Total classes: 45
```

### JSON
```json
{
  "files": [
    {
      "path": "./src/analyzer/parser.rs",
      "language": "rust",
      "lines_of_code": 450,
      "blank_lines": 65,
      "comment_lines": 120,
      "functions": 25,
      "classes": 3,
      "complexity_score": 5.2
    }
  ],
  "summary": {
    "total_files": 45,
    "total_lines": 15420,
    "total_functions": 234,
    "total_classes": 45,
    "languages_found": ["rust", "python", "javascript"]
  }
}
```

## 🎯 Casos de Uso

### 👨‍💼 Para Arquitetos e Tech Leads
- **Identificação de Hot Spots**: Encontre arquivos complexos que precisam de refatoração
- **Planejamento de Sprint**: Priorize tarefas de melhoria baseadas em métricas objetivas
- **Code Review**: Use métricas para guiar revisões de código mais efetivas

### 🏢 Para Equipes de Desenvolvimento
- **Integração CI/CD**: Monitore qualidade de código automaticamente
- **Onboarding**: Novos desenvolvedores podem entender rapidamente a estrutura do projeto
- **Refatoração Orientada a Dados**: Tome decisões baseadas em evidências, não intuição

### 📈 Para Análise de Technical Debt
- **Baseline de Qualidade**: Estabeleça métricas iniciais para acompanhar melhorias
- **ROI de Refatoração**: Quantifique o impacto de melhorias no código
- **Relatórios para Stakeholders**: Dados objetivos sobre saúde do código

## 🔧 Desenvolvimento

### Configuração do Ambiente
```bash
# Clone o repositório
git clone https://github.com/example/code-analyzer.git
cd code-analyzer

# Instale as dependências e compile
cargo build
```

### Executando Testes
```bash
# Todos os testes
cargo test

# Apenas testes de integração
cargo test --test integration_tests

# Com saída detalhada
cargo test -- --nocapture
```

### Verificação de Qualidade
```bash
# Formatação
cargo fmt

# Linting
cargo clippy

# Verificação completa (para CI)
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

## 🤝 Contribuindo

Contribuições são bem-vindas! Veja nosso [guia de contribuição](CONTRIBUTING.md) para começar.

### Roadmap
- [ ] Suporte para mais linguagens (C#, PHP, Ruby)
- [ ] Interface web para visualização de métricas
- [ ] Integração com IDEs populares
- [ ] Análise histórica e tendências
- [ ] Suporte para métricas customizadas

## 📄 Licença

Este projeto está licenciado sob [MIT](LICENSE-MIT) ou [Apache-2.0](LICENSE-APACHE) - veja os arquivos de licença para detalhes.

## 📞 Suporte

- 🐛 **Issues**: [GitHub Issues](https://github.com/example/code-analyzer/issues)
- 📧 **Email**: suporte@code-analyzer.dev
- 💬 **Discussões**: [GitHub Discussions](https://github.com/example/code-analyzer/discussions)

---

<div align="center">

**Desenvolvido com ❤️ em Rust**

*Transforme análise de código em insights acionáveis*

</div>

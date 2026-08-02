# Contribuindo com o Code Analyzer

Correções, novas linguagens e melhorias nas métricas são bem-vindas. Prefira
pull requests pequenos e com um único objetivo.

## Ambiente de desenvolvimento

Use a versão estável do Rust. Antes de enviar uma contribuição, execute:

```bash
make quality
```

O comando verifica formatação, Clippy e toda a suíte de testes. Alterações no
schema JSON ou no comportamento da CLI devem incluir testes de integração e a
documentação correspondente.

Para adicionar uma linguagem, inclua a gramática tree-sitter, a estratégia de
detecção e casos de teste com arquivos pequenos e redistribuíveis.

Vulnerabilidades devem ser reportadas conforme [SECURITY.md](SECURITY.md), não
por uma issue pública.

# Visão Geral
O `code-analyzer` é uma ferramenta de linha de comando (CLI) que analisa bases de código para identificar potenciais candidatos a refatoração. Ela percorre recursivamente os diretórios, conta linhas, funções e classes em cada arquivo (ignorando os especificados no .gitignore), e gera tanto um relatório visual no terminal quanto um arquivo `refactor-candidates.json` para ajudar desenvolvedores a priorizar melhorias na manutenibilidade do código.

# Funcionalidades Principais

- **Análise de Código via AST (Abstract Syntax Tree)**
  - **O que ela faz:** Analisa o código-fonte de cada arquivo para contar o número de linhas, funções e classes.
  - **Por que é importante:** É o coração da ferramenta. Uma análise precisa é fundamental para gerar métricas confiáveis que ajudem a identificar os arquivos que realmente precisam de atenção.
  - **Como funciona em alto nível:** O `code-analyzer` utilizará a crate `tree-sitter` que converte o código-fonte em uma Árvore Sintática Abstrata (AST). Em seguida, ela percorre essa árvore para contar com precisão os nós que representam declarações de função e classe.

- **Leitura e Aplicação do `.gitignore`**
  - **O que ela faz:** Identifica e ignora arquivos e diretórios listados no arquivo `.gitignore` do projeto durante o escaneamento.
  - **Por que é importante:** Garante que a análise se concentre apenas no código-fonte relevante, excluindo arquivos de dependência, build e logs.
  - **Como funciona em alto nível:** O `code-analyzer` incluirá a crate `ignore`, que replica o comportamento do `git` para descobrir arquivos e aplicar regras de ignore de forma otimizada e paralela.

- **Geração de Relatórios (Terminal e JSON)**
  - **O que ela faz:** Apresenta os resultados da análise de duas formas: um relatório visual no terminal e um arquivo `refactor-candidates.json`.
  - **Por que é importante:** O relatório no terminal oferece feedback imediato. O arquivo JSON permite que os resultados sejam usados por outras ferramentas (ex: em pipelines de CI/CD).
  - **Como funciona em alto nível:** Após a análise, os dados serão agregados. Para o terminal, será usada a crate `prettytable-rs` para gerar uma tabela detalhada. Simultaneamente, os mesmos dados serão formatados com a `serde_json` e escritos em um arquivo `refactor-candidates.json`.

# Experiência do Usuário

- **Personas de usuário:**
  - **Público-alvo principal:** O Desenvolvedor Sênior / Arquiteto.
  - **Descrição:** Um desenvolvedor experiente que se preocupa com a "saúde" do código a longo prazo e usará a ferramenta para guiar os esforços de refatoração.
  - **Implicações:** A ferramenta deve ser precisa, confiável e oferecer opções de configuração para customizar a análise.

- **Fluxos de usuário chave:**
  - **Filosofia:** A ferramenta seguirá o princípio de "convenção sobre configuração". A execução básica é simples, mas permite customização através de flags opcionais.
  - **Interação Principal:**
    - `code-analyzer`: Executa a análise no diretório atual com as configurações padrão.
    - `code-analyzer /path/to/project`: Analisa um diretório específico.
    - `code-analyzer --min-lines 100`: Filtra os resultados para mostrar apenas arquivos com mais de 100 linhas.
    - Outras flags (`--sort`, etc.) podem ser adicionadas para maior controle.

# Stack de Tecnologia e Arquitetura de Alto Nível

- **Linguagem(ns) de Programação:** Rust
- **Serviços Chave (Bibliotecas/Crates):**
  - **Interface de Linha de Comando:** `clap`
  - **Análise de Código (AST):** `tree-sitter`
  - **Leitura de `.gitignore` e Travessia de Diretórios:** `ignore`
  - **Geração de Tabelas no Terminal:** `prettytable-rs`
  - **Manipulação de JSON:** `serde_json`
- **Banco(s) de Dados:** Não Aplicável (A ferramenta é stateless e opera localmente).
- **Provedor de Nuvem Principal:** Não Aplicável (A ferramenta é uma aplicação de terminal local).

# Estratégia de Implantação (Deployment)

- **Ambiente de Produção Alvo:** A ferramenta será publicada no registro `crates.io`. O método de instalação para o usuário final será através do comando `cargo install code-analyzer`.
- **Instalação em Desenvolvimento:** Durante o desenvolvimento, a ferramenta pode ser compilada e instalada localmente a partir do código-fonte usando o comando `cargo install --path .`. Isso permite testes rápidos sem a necessidade de publicar no `crates.io`.
- **Nível de Automação desejado:** Processo Totalmente Manual. Testes e a publicação de novas versões serão executados manualmente pelo desenvolvedor.
- **Abordagem para Monitoramento e Logs:** A ferramenta usará a saída de erro padrão (`stderr`) para reportar erros.
- **Principais Requisitos de Manutenção e Escalabilidade:** A manutenção consistirá em atualizar as dependências. A escalabilidade será garantida pela alta performance do Rust.

# Riscos e Mitigações

- **Desafios técnicos:**
  - **Risco:** O objetivo de suportar um grande número de linguagens de programação desde a primeira versão é tecnicamente desafiador e intensivo em tempo.
  - **Mitigação:** A abordagem escolhida é enfrentar este desafio diretamente. O sucesso dependerá de um esforço de desenvolvimento focado e intensivo. Recomenda-se priorizar um conjunto de linguagens-chave para garantir a qualidade.

- **Restrições de recursos:**
  - **Risco:** O grande escopo inicial pode levar a um ciclo de desenvolvimento muito longo, aumentando o risco de o projeto não ser concluído.
  - **Mitigação:** Não foi definida uma mitigação específica, aceitando-se o risco em favor de um lançamento inicial com mais funcionalidades.
# Architecture Decision Record (ADR)

Este documento registra as principais decisões arquiteturais do projeto **Game Manager**, explicando o contexto e os motivos por trás das escolhas técnicas.

---

## 1. Objetivo do Projeto

Criar uma aplicação desktop para gerenciamento de biblioteca de jogos, com foco em:

- uso pessoal real
- aprendizado de tecnologias modernas
- demonstração de habilidades full-stack em um projeto de portfólio

---

## 2. Plataforma e Arquitetura Geral

**Decisão:** Aplicação desktop multiplataforma usando Tauri.

**Motivação:**

- Melhor desempenho e menor consumo de memória comparado a Electron
- Uso de Rust no backend para aprendizado e segurança
- Integração natural com frontend web moderno

**Consequências:**

- Curva de aprendizado maior com Rust
- Build mais complexo, porém mais eficiente

---

## 3. Frontend

**Decisão:** React com TypeScript.

**Motivação:**

- Stack já conhecida
- Ecossistema maduro
- Facilidade de manutenção e escalabilidade

---

## 4. Backend Local

**Decisão:** Backend local em Rust (Tauri commands).

**Motivação:**

- Processamento local de dados
- Evitar dependência de serviços externos
- Melhor privacidade do usuário

---

## 5. Persistência de Dados

**Decisão:** Banco de dados local (ex: SQLite).

**Motivação:**

- Simplicidade
- Portabilidade
- Adequado para aplicação desktop

---

## 6. Estratégia de Recomendação de Jogos

### 6.1 Abordagem Inicial (MVP)

**Decisão:** Regras simples e filtros baseados em metadados.

Exemplos:

- Gêneros mais jogados
- Tags favoritas
- Tempo de jogo
- Avaliações do usuário

**Motivação:**

- Baixo custo computacional
- Resultados rápidos e explicáveis
- Sem necessidade de datasets externos

---

### 6.2 Abordagem com Machine Learning Clássico

**Decisão:** Modelos de ML supervisionados ou não supervisionados treinados localmente.

Exemplos:

- K-Means para clusterização de jogos
- Similaridade por cosseno
- KNN baseado em features dos jogos

**Motivação:**

- Aproveitar conhecimentos prévios em ciência de dados
- Custo computacional moderado
- Possibilidade de treinar com dados do próprio usuário

**Consequências:**

- Requer engenharia de features
- Necessita volume mínimo de dados

---

### 6.3 Uso de LLMs (Opcional / Experimental)

**Decisão:** Uso opcional de LLMs locais (ex: Ollama) ou APIs gratuitas apenas para explicação das recomendações.

**Motivação:**

- Melhor experiência do usuário
- Explicabilidade das sugestões
- Evitar dependência total de LLMs para inferência

**Consequências:**

- Dependência de hardware do usuário (modelo local)
- Possível latência
- Funcionalidade opcional, não obrigatória

---

## 7. Infraestrutura e DevOps

**Decisão:** Projeto local, sem dependência obrigatória de cloud.

**Motivação:**

- Aplicação desktop
- Reduz custos
- Simplicidade

**Observação:**
Experimentos futuros podem incluir serviços em cloud para:

- sincronização
- backup
- recomendações avançadas

---

## 8. Documentação e Open Source

**Decisão:** Documentação enxuta no GitHub (README, CONTRIBUTING, ADR).

**Motivação:**

- Foco em clareza
- Evitar sobrecarga de manutenção
- Demonstrar boas práticas de projetos open source

---

## 9. Status

Este ADR representa o estado atual das decisões arquiteturais e pode evoluir conforme o projeto crescer.

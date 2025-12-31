# Architecture Decision Record (ADR)

Este documento registra as principais decisões arquiteturais do projeto **Game Manager**, explicando o contexto e os motivos por trás das escolhas técnicas.

---

## 1. Objetivo do Projeto

Criar uma aplicação desktop para gerenciamento de biblioteca de jogos, com foco em:

- uso pessoal real
- aprendizado de tecnologias modernas
- demonstração de habilidades full-stack num projeto de portfólio

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

## 6. Segurança de Credenciais e Dados Sensíveis

**Decisão:** Armazenar credenciais de APIs (ex.: Steam API Key, Steam ID) em arquivo local **em texto plano**, de forma consciente, durante o estágio atual do projeto (MVP).

**Contexto:**
O projeto precisa persistir credenciais de acesso a APIs externas para funcionar corretamente. Durante o desenvolvimento, alternativas mais seguras foram avaliadas.

**Alternativas avaliadas:**

1. **Criptografia simétrica (AES-256) com derivação de chave (Argon2)**

   - Implementada experimentalmente.
   - Utilizou derivação deliberadamente lenta para mitigar força bruta.

2. **Keyring/credential store do sistema operacional**

   - Avaliado como solução mais adequada para aplicações desktop comerciais.
   - No ambiente atual, falhou por falta de assinatura de código e/ou reputação do app, levando o sistema a recusar o armazenamento.

**Motivação:**

- A derivação de chave (Argon2) introduziu latência perceptível (~3s) na leitura das credenciais, impactando negativamente a UX.
- O keyring do SO não funcionou de forma confiável no contexto atual de desenvolvimento (MVP sem assinatura).
- O aplicativo é:
  - *local-first*
  - de uso individual
  - sem dados sensíveis de terceiros
  - não exposto à internet como serviço

Dado esse contexto, o risco foi considerado aceitável para um **MVP de portfólio e aprendizado**.

**Consequências:**

- As credenciais ficam armazenadas em texto plano no ambiente local do usuário.
- O risco é parcialmente mitigado por:
  - execução local
  - ausência de sincronização automática
  - escopo limitado da aplicação

**Plano futuro:**

- Em uma versão comercial ou distribuída amplamente, tornar obrigatório o uso de keyring nativo do sistema operacional, preferencialmente com o aplicativo assinado.

Esta decisão é consciente, documentada e reversível, alinhada ao estágio atual do projeto.

## 7. Estratégia de Recomendação de Jogos

### 7.1 Abordagem Inicial (MVP)

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

### 7.2 Abordagem com Machine Learning Clássico (Futuro)

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

### 7.3 Uso de LLM Opcional (Futuro)

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

## 8. Infraestrutura e DevOps

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

### 8.1 Uso de Ferramentas de IA e Análises Automatizadas

**Decisão:** Utilizar ferramentas de IA (ex.: GitHub Copilot) como suporte para análise de código e identificação de melhorias, sem adoção automática das recomendações.

**Motivação:**

- Acelerar a identificação de possíveis problemas de segurança, performance e arquitetura.
- Expor o projeto a padrões utilizados na indústria.
- Exercitar análise crítica e tomada de decisão técnica.

**Abordagem adotada:**

As recomendações são classificadas em:

- **Aplicáveis imediatamente**
- **Relevantes apenas para versões comerciais ou em escala**
- **Conscientemente ignoradas** por não se adequarem ao contexto do projeto

Decisões de não implementação são consideradas tão importantes quanto as implementadas.

**Consequências:**

- Evita *overengineering* em um projeto de escopo reduzido.
- Mantém a base de código simples, legível e adequada ao uso real.
- Demonstra capacidade de avaliar *trade-offs* técnicos, em vez de seguir checklists genéricos.

---

## 9. Documentação e Open Source

**Decisão:** Documentação enxuta no GitHub (README, CONTRIBUTING, ADR).

**Motivação:**

- Foco em clareza
- Evitar sobrecarga de manutenção
- Demonstrar boas práticas de projetos open source

---

## 10. Status

Este ADR representa o estado atual das decisões arquiteturais e pode evoluir conforme o projeto crescer.

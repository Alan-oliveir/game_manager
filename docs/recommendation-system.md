# Sistema de Recomendação de Jogos

Este documento descreve o **design técnico do sistema de recomendação de jogos** do projeto **Game Manager**. O sistema segue uma abordagem **local-first**, utilizando **Machine Learning clássico** como motor principal de recomendação e **LLMs apenas como camada explicativa opcional**.

---

## 1. Objetivo

O sistema de recomendação tem como objetivo:

- Sugerir jogos relevantes com base no histórico do usuário
- Funcionar offline sem dependências externas
- Manter baixo consumo de recursos computacionais
- Preservar a privacidade dos dados do usuário
- Ser explicável e previsível em seus resultados

---

## 2. Visão Geral da Arquitetura

```text
┌─────────────┐
│  React UI   │
└──────┬──────┘
       │
┌──────▼──────────┐
│  Tauri Bridge   │
└──────┬──────────┘
       │
┌──────▼──────────┐
│   Rust Core     │
└──────┬──────────┘
       │
┌──────▼─────────────────┐
│ Recommendation Engine  │
├────────────────────────┤
│ • Rules Engine         │
│ • ML Engine            │
│ • LLM Explainer (opt)  │
└────────────────────────┘
```

### Fluxo de Processamento

1. O **ML Engine** determina quais jogos recomendar
2. O **Rules Engine** ajusta o ranking final aplicando regras de negócio
3. O **LLM Explainer** explica o resultado ao usuário (opcional)

---

## 3. Dados Utilizados

O sistema utiliza **exclusivamente dados locais** armazenados no dispositivo do usuário.

### 3.1 Features por Jogo

Cada jogo é representado por um conjunto de atributos convertidos em **vetores de características**:

| Feature | Tipo | Descrição |
|---------|------|-----------|
| Gêneros | Categórica | One-hot encoding dos gêneros |
| Tags | Categórica | Tags customizadas pelo usuário |
| Plataforma | Categórica | Steam, Epic, PlayStation, etc. |
| Tempo de jogo | Numérica | Horas jogadas |
| Avaliação | Numérica | Rating de 1-5 estrelas |
| Status | Categórica | Backlog, jogando, finalizado |
| Favorito | Booleana | Peso maior no cálculo |

### 3.2 Perfil do Usuário

O perfil é calculado como uma **média ponderada** dos vetores dos jogos com maior engajamento:

- Jogos marcados como favoritos
- Jogos com maior tempo de jogo
- Jogos com avaliação alta (4-5 estrelas)

---

## 4. Motor de Recomendação (ML Clássico)

### 4.1 Abordagem: Content-Based Filtering

A recomendação é baseada na **similaridade entre jogos** em relação ao perfil do usuário.

#### Algoritmos Implementados

Similaridade por Cosseno

- Calcula o ângulo entre vetores de características
- Fórmula: `similarity = (A · B) / (||A|| * ||B||)`
- Complexidade: O(n × m) onde n = jogos no catálogo, m = dimensões

KNN (k-Nearest Neighbors)

- Identifica os k jogos mais similares aos favoritos
- Valor típico: k = 5-10
- Vantagens: explicável, intuitivo, eficiente para datasets pequenos

### 4.2 Pipeline de Recomendação

```text
1. Extração de Features → Vetorização
2. Cálculo de Similaridade → Scoring
3. Aplicação de Regras → Ajuste de Ranking
4. Seleção Top-N → Resultado Final
```

---

## 5. Rules Engine

Após o cálculo de similaridade, **regras de negócio** refinam o ranking final.

### 5.1 Regras Implementadas

| Regra | Descrição | Impacto |
|-------|-----------|---------|
| Exclusão de finalizados | Remove jogos já concluídos | Opcional (configurável) |
| Penalização temporal | Reduz score de jogos recém-sugeridos | Evita repetição |
| Balanceamento de duração | Alterna entre jogos longos e curtos | Diversidade |
| Priorização de backlog | Aumenta score de jogos na lista de desejos | Engajamento |

### 5.2 Exemplo de Aplicação

```rust
// Pseudocódigo
fn apply_rules(games: Vec<Game>, user_profile: UserProfile) -> Vec<Game> {
    games
        .filter(|g| !g.finished || user_profile.include_finished)
        .map(|g| adjust_score_by_recency(g))
        .map(|g| adjust_score_by_duration(g))
        .map(|g| boost_if_in_backlog(g))
        .sort_by_score()
        .take(10)
}
```

---

## 6. LLM Explainer (Opcional)

### 6.1 Papel do LLM

O LLM **não participa da decisão de recomendação**. Seu papel é exclusivamente:

- Explicar por que um jogo foi sugerido
- Resumir padrões do histórico do usuário
- Humanizar a interface de recomendação

### 6.2 Estratégia de Execução

| Ambiente | Solução | Requisitos |
|----------|---------|------------|
| Desktop | Ollama local | Modelo instalado |
| Fallback | API externa gratuita | Conexão com internet |
| Modo offline | Sem explicação | Apenas ranking |

### 6.3 Exemplo de Prompt

```text
Você é um assistente que explica recomendações de jogos.

Jogos favoritos do usuário:
- The Witcher 3 (RPG, 120h jogadas, 5 estrelas)
- Dark Souls III (Action RPG, 80h jogadas, 5 estrelas)

Jogos recomendados:
1. Elden Ring (Action RPG, mundo aberto)
2. Bloodborne (Action RPG, atmosfera sombria)
3. Sekiro (Action, combate desafiador)

Explique em 2-3 frases por que esses jogos foram sugeridos.
```

### 6.4 Configuração

O uso de LLM é totalmente **opt-in**:

- Desabilitado por padrão
- Configurável em Settings
- Não bloqueia funcionalidade principal

---

## 7. Evoluções Futuras

### 7.1 Collaborative Filtering

- Integração com datasets públicos (Steam, Kaggle)
- Treinamento offline de modelos
- Recomendações baseadas em usuários similares

### 7.2 Backend Cloud (Opt-in)

- Sincronização entre dispositivos

### 7.3 Métricas e Avaliação

- Feedback loop do usuário (thumbs up/down)

---

## 8. Considerações de Implementação

- Privacidade
- Performance
- Escalabilidade

---

## 9. Stack Tecnológica

| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| Core Engine | Rust | Performance, segurança de tipos |
| ML Library | ndarray, smartcore | Álgebra linear eficiente |
| UI | React + TypeScript | Interface reativa |
| Bridge | Tauri | IPC seguro e eficiente |
| LLM (opcional) | Ollama / Grok | Flexibilidade de deployment |

---

## 10. Referências

- [Content-Based Recommender Systems](https://www.sciencedirect.com/topics/computer-science/content-based-recommender-system)
- [Cosine Similarity for Vector Space Models](https://en.wikipedia.org/wiki/Cosine_similarity)
- [Netflix Recommendation System](https://netflixtechblog.com/netflix-recommendations-beyond-the-5-stars-part-1-55838468f429)

---

*Documentação mantida por: Alan de Oliveira Gonçalves*  
*Última atualização: Dezembro/2025*  
*Versão: 1.0*

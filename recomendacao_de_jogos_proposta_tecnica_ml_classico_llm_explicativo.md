# 🎯 Proposta de Funcionalidade: Sistema de Recomendação de Jogos

## 1. Visão Geral

Esta proposta descreve a implementação de um **sistema de recomendação híbrido**, combinando **Machine Learning clássico (local-first)** para gerar recomendações eficientes e determinísticas, com **LLMs apenas como camada explicativa opcional**.

O objetivo é:
- Garantir **performance**, **baixo custo computacional** e **funcionamento offline**;
- Aproveitar conhecimentos prévios em **Ciência de Dados, ML clássico e Engenharia de Software**;
- Evitar dependência obrigatória de modelos grandes (LLMs);
- Criar uma arquitetura madura, alinhada com práticas usadas por empresas como **Steam, Netflix e Amazon**.

---

## 2. Princípios de Design

1. **Local-first**: recomendações funcionam 100% offline
2. **Determinismo**: resultados previsíveis e explicáveis
3. **Privacidade**: dados do usuário permanecem no dispositivo
4. **Performance**: latência mínima em desktop
5. **Extensibilidade**: fácil evolução para cloud ou mobile

---

## 3. Arquitetura Geral

```
[React UI]
     │
[Tauri Bridge]
     │
[Rust Core]
     │
[Recommendation Engine]
   ├── Rules Engine
   ├── ML Clássico (Content-Based / Similaridade)
   └── (Opcional) LLM Explainer
```

- **ML clássico** decide *o que* recomendar
- **LLM** explica *por que* foi recomendado

---

## 4. Motor Principal: Machine Learning Clássico

### 4.1 Abordagem Inicial: Content-Based Filtering

Não requer dataset externo. Utiliza apenas dados locais do usuário.

#### Features possíveis por jogo:
- Gêneros (one-hot encoding)
- Tags customizadas
- Plataforma
- Tempo médio de jogo
- Avaliação do usuário (1–5)
- Status (jogando / finalizado / backlog)
- Favorito (peso maior)

Cada jogo é representado como um **vetor de características**.

---

### 4.2 Perfil do Usuário

O perfil do usuário é calculado a partir de:
- Jogos favoritos
- Jogos mais jogados
- Jogos bem avaliados

Pode ser uma **média ponderada** dos vetores dos jogos preferidos.

---

### 4.3 Algoritmos Sugeridos

#### Opção A — Similaridade por Cosseno (Cosine Similarity)
- Simples
- Rápido
- Muito usado em produção

#### Opção B — KNN (k-Nearest Neighbors)
- Jogos similares aos favoritos
- Fácil de explicar
- Excelente para portfólio

📌 Implementável em:
- Rust (cálculo manual)
- Python (scikit-learn) como módulo isolado (opcional)

---

### 4.4 Regras de Negócio (Rules Engine)

Antes do ranking final:
- Excluir jogos já finalizados (opcional)
- Penalizar jogos recém-sugeridos
- Balancear jogos longos vs curtos
- Priorizar backlog

Isso aumenta muito a qualidade percebida.

---

## 5. Uso Opcional de LLM (Camada Explicativa)

### 5.1 O que o LLM **NÃO** faz

❌ Não calcula ranking
❌ Não decide recomendações
❌ Não é obrigatório

---

### 5.2 O que o LLM faz

✅ Explica recomendações
✅ Resume padrões
✅ Cria UX conversacional

Exemplo de prompt:

> "Explique por que os jogos abaixo foram recomendados, com base nos dados fornecidos."

Input:
- Lista de jogos sugeridos
- Jogos base (favoritos)
- Features relevantes

Output:
- Texto explicativo amigável

---

### 5.3 Estratégia de Execução

- **Desktop**: Ollama (se instalado)
- **Fallback**: API gratuita (opcional)
- **Configuração**: totalmente opt-in

---

## 6. Evolução Futura (Opcional)

### 6.1 Collaborative Filtering

- Usar datasets públicos (Steam, Kaggle)
- Treinar modelo offline
- Aplicar como modelo base

### 6.2 Backend Cloud

- Coleta anônima (opt-in)
- Treinamento periódico
- API de recomendação

📌 Integra perfeitamente com conhecimentos em **Cloud, DevOps e Backend**.

---

## 7. Diferenciais para Portfólio

- Decisão consciente de **não usar LLM como motor principal**
- Uso de ML clássico como em produtos reais
- Arquitetura híbrida bem definida
- Offline-first
- Performance + privacidade

---

## 8. Conclusão

Este sistema entrega:
- Recomendações rápidas e úteis
- Baixo consumo de recursos
- Excelente demonstração de maturidade técnica

A combinação **ML clássico + LLM explicativo** reflete práticas reais da indústria e fortalece o projeto como **case profissional de portfólio**.

---

*Documento criado para o projeto Game Manager*  
*Autor: Alan de Oliveira Gonçalves*  
*Data: Dezembro/2025*


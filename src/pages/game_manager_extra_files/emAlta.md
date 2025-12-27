# Sugestões - Página: Em Alta

### UX e Design: Inspiração na Steam

Hero Section (Cards Grandes): Usar layouts de destaque (carrossel ou grid assimétrico com um item grande) quebra a monotonia da grade padrão. Isso cria um "ponto focal" visualmente atraente.

Familiaridade: O usuário já sabe como navegar nesse tipo de layout.

Curadoria: Ao mostrar "O que está em alta", você resolve o problema da "paralisia de escolha" ajudando o usuário a descobrir novidades.

### A "Feature Matadora": Filtro de Jogos Já Adquiridos

A ideia de excluir os jogos que o usuário já possui é o grande diferencial.

Quantas vezes você entrou na loja da Steam, viu um banner gigante, clicou e descobriu que já tinha o jogo na biblioteca (talvez comprado num bundle anos atrás)?

A Solução: O seu app, por ser local e ter acesso ao banco SQLite, pode fazer esse filtro instantaneamente no Frontend.

Lógica: JogosEmAlta.filter(jogo => !meusJogos.some(meu => meu.name === jogo.name))

Valor: Isso transforma a página em uma ferramenta de Descoberta Pura. O usuário sabe que tudo que está ali é novidade para ele.

### Viabilidade Técnica (O Desafio da API)
Para realizar isso sem depender da Steam (que tem APIs de loja um pouco complexas e rate limits chatos), a estratégia será usar uma API Pública de Games.

Inicialmente será com a RAWG.io para essa etapa:

RAWG: É gratuita, fácil de usar e tem endpoints prontos para "Trending", "New Releases" e "Best of the Year".

Matching de Nomes: O maior desafio técnico será comparar os nomes.

Exemplo: Na Steam o jogo é "The Witcher 3: Wild Hunt". Na API externa pode ser apenas "The Witcher 3".

Solução: Implementar uma comparação de texto simplificada (normalizar strings, remover ":", "-", etc) para garantir que o filtro de "já possuo" funcione bem.

### Sugestões Adicionais para essa Ideia

Já que o foco é "Sugestão de Compra", aqui vão ideias:

- Botão de Wishlist (Lista de Desejos): o botão de ação do card poderia ser "Adicionar à Lista de Desejos" (criando uma nova aba ou filtro no seu banco local). Cria uma nova seção no sidebar futuramente

- Link Direto: Um botão "Ver na Loja" que abre o link da Steam/Epic no navegador padrão.

- Filtros na página Em Alta: Por plataforma (Steam, Epic, etc.), Por gênero e Por faixa de preço.

- Notificações/Badge: Mostrar um badge no sidebar "Em Alta" quando houver novos lançamentos interessantes

### Sugestão para as Seções Principais:

- Lançamentos Recentes

Jogos lançados nas últimas semanas/mês
Filtro importante: excluir jogos já na biblioteca
API sugerida: RAWG

- Mais Populares da Semana/Mês

Jogos com mais players/buzz
Baseado em dados de plataformas como Steam Charts
Também excluir os já possuídos


- Em Promoção (se usar APIs que fornecem isso)

Jogos com desconto no Steam/Epic/GOG
Mostrar % de desconto e preço
Ótimo call-to-action para compra


- Baseado nos Seus Gostos (personalizado)

Recomendações baseadas nos gêneros/plataformas mais jogados
Ex: "Você joga muito RPG, confira estes:"
Usa os dados da própria biblioteca para personalizar


- Para Você (algoritmo simples)

Jogos similares aos que estão na biblioteca
Pode usar tags/gêneros em comum
Ex: "Quem jogou The Witcher 3 também gostou de..."


- Tendências (opcional)

Jogos que estão crescendo em popularidade
"Em ascensão" ou "Todos estão jogando"

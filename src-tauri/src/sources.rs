//! Módulos utilizados para importar de jogos.
//!
//! Fornece funcionalidades para integrar e obter dados de diferentes plataformas de jogos.
//! Cada módulo encapsula a lógica necessária para comunicação com uma plataforma específica,
//! facilitando a manutenção e expansão do código.
//!
//! **Módulos:**
//!
//! - `amazon`: Importa jogos da Amazon Games.
//! - `battle_net`: Impora jogos da Battle.Net.
//! - `ea`: Importa jogos da EA Desktop (Electronic Arts), escaneando a pasta de instalação informada pelo usuário.
//! - `epic`: Importa jogos da Epic Games Store, conectando-se aos arquivos locais para obter a lista completa de jogos instalados.
//! - `gog`: Importa jogos do GOG Galaxy, com OAuth.
//! - `indiegala`: Importa jogos da IndieGala, conectando-se aos arquivos locais para obter a lista de jogos instalados ou biblioteca completa.
//! - `itch`: Importa jogos da plataforma Itch.io, lendo o banco de dados do app itch.
//! - `legacy`: Importa jogos da loja Legacy Games, utilizando métodos de leitura de arquivos para identificar jogos obtidos por essa plataforma.
//! - `providers`: Gerencia provedores de jogos, permitindo a integração com múltiplas plataformas de jogos.
//! - `scanner`: Escaneia pastas em busca de jogos instalados localmente.
//! - `steam`: Importa jogos da Steam.
//! - `ubisoft`: Importa jogos da Ubisoft Connect, conectando-se aos arquivos locais para obter a lista completa de jogos instalados.
//! - `xbox`: Importa jogos da Microsoft Store / Xbox App (Gaming Services), lendo os arquivos de configuração para identificar jogos instalados via essa plataforma.

pub mod amazon;
pub mod battle_net;
pub mod ea;
pub mod epic;
pub mod gog;
pub mod indiegala;
pub mod itch;
pub mod legacy;
pub mod providers;
pub mod scanner;
pub mod steam;
pub mod ubisoft;
pub mod xbox;

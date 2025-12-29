use crate::models::{Game, GenreScore, UserProfile};
use std::collections::HashMap;

// Pesos para o algoritmo (Configurável futuramente)
const WEIGHT_PLAYTIME_HOUR: f32 = 2.0;  // 2 pontos por hora jogada
const WEIGHT_FAVORITE: f32 = 50.0;      // 50 pontos se for favorito
const WEIGHT_RATING_STAR: f32 = 10.0;   // 10 pontos por estrela (ex: 5 estrelas = 50 pts)
const DECAY_FACTOR: f32 = 0.95;         // (Futuro) Para jogos muito antigos

pub fn calculate_user_profile(games: &[Game]) -> UserProfile {
    let mut genre_scores: HashMap<String, (f32, i32)> = HashMap::new();
    let mut total_playtime = 0;

    for game in games {
        total_playtime += game.playtime;

        // Calcular o Score Base do Jogo (Game Score)
        let mut game_score = 0.0;

        // Fator Tempo de Jogo (limitado a 100h para não distorcer demais)
        let hours = (game.playtime as f32 / 60.0).min(100.0);
        game_score += hours * WEIGHT_PLAYTIME_HOUR;

        // Fator Favorito
        if game.favorite {
            game_score += WEIGHT_FAVORITE;
        }

        // Fator Rating (Se tiver avaliação)
        if let Some(rating) = game.rating {
            game_score += (rating as f32) * WEIGHT_RATING_STAR;
        }

        // Distribuir o Score para os Gêneros do Jogo
        if let Some(genre_str) = &game.genre {
            // Se houver múltiplos gêneros separados por vírgula (ex: "RPG, Action")
            // Na Steam geralmente vem um principal, mas vamos preparar para split
            let genres: Vec<&str> = genre_str.split(',').map(|s| s.trim()).collect();

            for genre in genres {
                if genre.is_empty() || genre == "Desconhecido" { continue; }

                let entry = genre_scores.entry(genre.to_string()).or_insert((0.0, 0));
                entry.0 += game_score; // Soma pontuação
                entry.1 += 1;          // Conta ocorrência
            }
        }
    }

    // Converter HashMap para Vetor Ordenado
    let mut top_genres: Vec<GenreScore> = genre_scores
        .into_iter()
        .map(|(name, (score, count))| GenreScore { name, score, game_count: count })
        .collect();

    // Ordenar do maior score para o menor
    top_genres.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    UserProfile {
        top_genres,
        total_playtime,
        total_games: games.len() as i32,
    }
}

// Futuro: Função para cruzar perfil com novos jogos
// pub fn rank_games(profile: &UserProfile, candidates: Vec<Game>) -> Vec<Game> { ... }
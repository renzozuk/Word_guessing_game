use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

mod entities; 
use bincode::deserialize;
use clearscreen::clear;
use entities::{GameClient, GameConfig};

use crate::entities::{Difficulty, Game, Language, Player, Word};

fn main() {

    clear().unwrap();
    println!("Do you want to be the host? (y for yes, n for no) / Deseja ser o host ? (s para sim, n para não):");
   
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    if choice.trim().to_lowercase() == "s" || choice.trim().to_lowercase() == "y" {

        let (username, language, difficulty): (String, Language, Difficulty) = config_game();


        clear().unwrap();
        println!("Waiting for the second player... / Esperando o segundo jogador...");
        let listener = TcpListener::bind("0.0.0.0:6000").expect("Listener failed");

        if let Ok((mut stream, _addr)) = listener.accept() {
            let game_thread = thread::spawn(move || handle_client(&mut stream, &username, &language, &difficulty));
            let _ = game_thread.join();

        } else {
            eprintln!("Failed to accept connection / Erro ao aceitar a conexão.");
        }

    } else {
        
        clear().unwrap();
        println!("Choose your name / Escolhe seu nome:");
        let mut username = String::new();
        io::stdin().read_line(&mut username).unwrap();

        clear().unwrap();
        println!("Enter the IP address of the host / Digite o endreço IP do host:");
        let mut server_ip = String::new();
        io::stdin().read_line(&mut server_ip).unwrap();
        println!("{}", server_ip.trim());
        let server_address = format!("{}:6000", server_ip.trim());

        let mut client = TcpStream::connect(&server_address).expect("Failed to connect to the server / Falha ao conectar ao servidor");
        client.set_nonblocking(true).expect("Failed to set nonblocking mode / Falha ao definir modo não bloqueante");
        client.write_all(&username.trim().as_bytes()).expect("Failed to send player data / Falha ao enviar dados do jogador");

        let mut buffer = Vec::new();
        let mut temp_buffer = [0; 1024];

        loop {
            match client.read(&mut temp_buffer) {
                Ok(0) => break,
                Ok(_n) => {
                    buffer.extend_from_slice(&temp_buffer[.._n]);

                    clear().unwrap();

                    let game_config = deserialize::<GameConfig>(&buffer).unwrap();

                    let mut game = GameClient::new(&mut client, game_config);
                    game.start();
                    break;
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                },
                Err(e) => {
                   eprintln!("Error reading from host: {} / Erro ao ler do host: {}", e, e);
                    break;
                }
            }
        }


    }
}

fn handle_client(stream: &mut TcpStream, first_player_name: &str, language: &Language, difficulty: &Difficulty) {
    let mut buffer = Vec::new();

    loop {
        let mut temp_buffer = [0; 1024];
        match stream.read(&mut temp_buffer) {
            Ok(0) => {
                break;
            }
            Ok(n) => {
                buffer.extend_from_slice(&temp_buffer[..n]);

                if let Ok(second_player_username) = String::from_utf8(buffer.clone()) { 
                    buffer.clear();

                    let mut game = Game::new(first_player_name.trim(), &second_player_username, difficulty.clone(), language.clone());
                    game.start(stream);
                    break;
                } 
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(10)); 
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break; 
            }
        }
    }
}

fn config_game() -> (String, Language, Difficulty) {

    let language = get_language();
    let difficulty = get_difficulty(&language);

    clear().unwrap();
    match language {
        Language::English => println!("Choose your name:"),
        Language::Portuguese => println!("Escolha seu nome:"),
    }
    
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    (username, language, difficulty)

}

fn get_language() -> Language {

    clear().unwrap();
    loop {
            
        println!("Select the Language / Escolha o idioma: ");
        println!("1 - English/Inglês");
        println!("2 - Portuguese/Português");

        let mut language = String::new();
        io::stdin().read_line(&mut language).unwrap();

        let language = match language.trim().parse::<i32>() {
            Ok(lang) => lang,
            Err(_) => {
                print!("Invalid input. Please enter a valid number./");
                println!("Entrada inválida. Por favor, digite um número válido.");
                continue;
            }
        };

        let language = match language {
            1 => Language::English,
            2 => Language::Portuguese,
            _ => panic!("Invalid language/Idioma inválido"),
        };


        return language;
    }
}

fn get_difficulty(language: &Language) -> Difficulty {

    clear().unwrap();

    let select_difficulty_message;
    let invalid_input_message;
    let invalid_difficulty_message;

    match language {
        Language::English => {
            select_difficulty_message = String::from("Select the difficulty:\n1 - Easy\n2 - Normal\n3 - Hard");
            invalid_input_message = String::from("Invalid input. Please enter a valid number.");
            invalid_difficulty_message = String::from("Invalid difficulty.");
        },
        Language::Portuguese => {
            select_difficulty_message = String::from("Selecione a dificuldade:\n1 - Fácil\n2 - Normal\n3 - Difícil");
            invalid_input_message = String::from("Entrada inválida. Por favor, insira um número válido.");
            invalid_difficulty_message = String::from("Dificuldade inválida.");
        },
    }

    loop {

        println!("{}", select_difficulty_message);

        let mut difficulty = String::new();
        io::stdin().read_line(&mut difficulty).unwrap();

            let difficulty = match difficulty.trim().parse::<i32>() {
            Ok(diff) => diff,
            Err(_) => {
                print!("{}", invalid_input_message);
                continue;
            }
        };

        let difficulty = match difficulty {
            1 => Difficulty::Easy,
            2 => Difficulty::Normal,
            3 => Difficulty::Hard,
            _ => panic!("{}", invalid_difficulty_message),
        };


        return difficulty;
    }
}

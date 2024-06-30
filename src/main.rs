use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use bincode::{deserialize, serialize};

mod entities; 
use crate::entities::{Difficulty, Game, Language, Player, Word};

fn main() {
    println!("Username: ");
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();

    println!("Escolha (s para servidor, p para jogador):");
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    if choice.trim() == "s" {

        let listener = TcpListener::bind("127.0.0.1:6000").expect("Listener failed");

        if let Ok((mut stream, _addr)) = listener.accept() {
            thread::spawn(move || handle_client(&mut stream, &username));
        } else {
            eprintln!("Erro ao aceitar a conexão.");
        }

    } else if choice.trim() == "p" {
        let mut client = TcpStream::connect("127.0.0.1:6000").expect("Falha ao conectar ao servidor");
        client.set_nonblocking(true).expect("Falha ao definir modo não bloqueante");

        let player = Player::new(username.trim());
        let player_bytes = serialize(&player).expect("Falha ao serializar jogador");
        client.write_all(&player_bytes).expect("Falha ao enviar dados do jogador");

        loop {

        }
    } 
}

fn handle_client(stream: &mut TcpStream, first_player_name: &str) {
    let mut buffer = Vec::new();

    loop {
        let mut temp_buffer = [0; 1024];
        loop {

            match stream.read(&mut temp_buffer) {
                Ok(0) => {
                    break; 
                }
                Ok(n) => {
                    buffer.extend_from_slice(&temp_buffer[..n]);
                    if let Ok(second_player) = deserialize::<Player>(&buffer) {
                        println!("Player connected - {:?}", second_player.name);
                        buffer.clear();

                        let mut game = Game::new(first_player_name, &second_player.name, Difficulty::Normal, Language::English);

                        game.start();
                    }
                }
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
}


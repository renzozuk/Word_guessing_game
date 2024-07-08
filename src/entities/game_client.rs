use clearscreen::clear;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::stdin;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

use crate::entities::GameConfig;
use crate::Difficulty;
use crate::Language;
use crate::Word;

pub struct GameClient<'a> {
    client_stream: &'a mut TcpStream,
    game_config: GameConfig,
    round: usize,
    turn: bool,
    wordlist: HashSet<Word>,
}

impl<'a> GameClient<'a> {
    pub fn new(client_stream: &'a mut TcpStream, game_config: GameConfig) -> Self {

        let difficulty = game_config.difficulty.clone();
        let language = game_config.language.clone();

        let difficulty_number = get_difficulty_number(&difficulty);
        let language_name = get_language_name(&language);

        Self {
            client_stream,
            game_config,
            round: 1,
            turn: true,
            wordlist: get_wordlist(language_name, difficulty_number)
        }
    }

    pub fn start(&mut self) {

        let mut guess = Word::new("");

        self.show_welcome_message();

        while guess != self.game_config.selected_word {

            let mut user_input: String = Default::default();

            self.announce_player_turn();

            if self.turn {
                user_input = read_player_one_guess(self.client_stream).unwrap();
            }
            else {
                stdin().read_line(&mut user_input).expect("Failed to read the word.");
                let _ = self.client_stream.write_all(user_input.as_bytes());
            }

            guess = Word::new(&user_input.trim());

            if guess.length() != self.game_config.selected_word.length() {
                match self.game_config.language {
                    Language::English => {

                        println!("{} is an invalid word. Only {} letters long words are valid guesses.", guess.to_string(), get_difficulty_number(&self.game_config.difficulty));
                    },
                    Language::Portuguese => {
                        println!("{} é uma palavra inválida. O seu guess deve ter {} caracteres.", guess.to_string(), get_difficulty_number(&self.game_config.difficulty)); 
                    },
                }
            } else if self.game_config.difficulty != Difficulty::Hard && (self.game_config.first_player.has_guessed_word(&guess) || self.game_config.second_player.has_guessed_word(&guess)) {
                match self.game_config.language {
                    Language::English => println!("Repeat played words in not allowed."),
                    Language::Portuguese => println!("Repetir palavras já jogadas não é permitido."),
                }

            } else {
                self.check_word_in_wordlist(&guess);
            }

            self.add_guessed_word(Word::new(&user_input.trim_end()));
        }

    }

    fn show_welcome_message(&self) {
        clear().unwrap();
        match self.game_config.language {
            Language::English => println!("Welcome to word guessing game!\nLanguage: English\nDifficulty: {}\n\n{}\n\nRules:\nA {} letters long word was drawn.\nThe first player to guess correctly win the game.\nRepeat words is {}allowed.", match self.game_config.difficulty {
                Difficulty::Easy => "\x1b[32mEasy\x1b[0m",
                Difficulty::Normal => "\x1b[33mNormal\x1b[0m",
                Difficulty::Hard => "\x1b[31mHard\x1b[0m",
            }, self.game_config, get_difficulty_number(&self.game_config.difficulty), match self.game_config.difficulty {
                Difficulty::Hard => "",
                _ => "not ",
            }),
            Language::Portuguese => println!("Bem-vindo ao word guessing game!\nIdioma: Português\nDificuldade: {}\n\n{}\n\nRegras:\nUma palavra de {} caracteres foi sorteada.\nO primeiro jogador a adivinhar corretamente vence o jogo.\nRepetir palavras {}é permitido.", match self.game_config.difficulty {
                Difficulty::Easy => "\x1b[32mFácil\x1b[0m",
                Difficulty::Normal => "\x1b[33mNormal\x1b[0m",
                Difficulty::Hard => "\x1b[31mDifícil\x1b[0m",
            }, self.game_config, get_difficulty_number(&self.game_config.difficulty), match self.game_config.difficulty {
                Difficulty::Hard => "",
                _ => "não ",
            })
        }
    }

    fn announce_player_turn(&self) {
        match self.game_config.language {
            Language::English => {
                if self.turn {
                    println!("\n{}º round.\nIt's {}'s turn!", self.round, self.game_config.first_player);
                } else {
                    println!("\n{}º round.\nIt's {}'s turn!", self.round, self.game_config.second_player);
                }
            },
            Language::Portuguese => {
                if self.turn {
                    println!("\n{}ª rodada.\nÉ a vez de {}!", self.round, self.game_config.first_player);
                } else {
                    println!("\n{}ª rodada.\nÉ a vez de {}!", self.round, self.game_config.second_player);
                }
            },
        }
    }

    fn add_guessed_word(&mut self, word: Word) {
        if self.turn {
            self.game_config.first_player.guess_word(word);
        } else {
            self.game_config.second_player.guess_word(word);
        }
    }

    fn check_word_in_wordlist(&mut self, guess: &Word) {
        if self.wordlist.contains(guess) {
            if *guess == self.game_config.selected_word {
                self.end_game();
            } else {
                /* print!("{}", match self.game_config.language {
                    Language::English => "Last guess: ",
                    Language::Portuguese => "Última jogada: ",
                }); */

                for word in &self.wordlist {
                    if word == guess {
                        word.show_status(&self.game_config.selected_word);
                    }
                }

                self.next_play();
            }

        } else {
            println!("{} {}", guess, match self.game_config.language {
                Language::English => "is an invalid word or is not present in wordlist.",
                Language::Portuguese => "é uma palavra inválida ou não está presente na lista de palavras.",
            });
        }
    }

    fn next_play(&mut self) {
        self.turn = !self.turn;

        if self.turn {
            self.round += 1;
        }
    }

    fn end_game(&self) {
        println!("\x1b[32m{}\x1b[0m", self.game_config.selected_word);

        if self.turn {
            print!("\n{} {} ", self.game_config.first_player, match self.game_config.language {
                Language::English => "won the game after",
                Language::Portuguese => "venceu o jogo após",
            });
        } else {
            print!("\n{} {} ", self.game_config.second_player, match self.game_config.language {
                Language::English => "won the game after",
                Language::Portuguese => "venceu o jogo após",
            });
        }

        if self.round == 1 {
            println!("{}!", match self.game_config.language {
                Language::English => "only one try",
                Language::Portuguese => "uma única tentativa",
            })
        } else {
            println!("{} {}!", self.round, match self.game_config.language {
                Language::English => "tries",
                Language::Portuguese => "tentativas",
            })
        }
    }
}

fn get_difficulty_number(difficulty: &Difficulty) -> &str {
    match difficulty {
        Difficulty::Easy => "6",
        Difficulty::Normal => "7",
        Difficulty::Hard => "8",
    }
}

fn get_language_name(language: &Language) -> &str {
    match language {
        Language::English => "english",
        Language::Portuguese => "portuguese",
    }
}

fn get_wordlist(language_name: &str, difficulty_number: &str) -> HashSet<Word>{
    read_to_string(format!("resources/wordlist_{}_{}.txt", language_name, difficulty_number))
        .expect("Failed to read the file.")
        .lines()
        .map(|line| Word::new(line.trim()))
        .collect()
}

fn read_player_one_guess(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 1024];

    loop {
        match stream.read(&mut temp_buffer) {
            Ok(0) => {
                return Err(std::io::Error::new(ErrorKind::ConnectionAborted, "Connection closed"));
            }
            Ok(n) => {
                buffer.extend_from_slice(&temp_buffer[..n]);

                if let Ok(guess) = String::from_utf8(buffer.clone()) {
                    buffer.clear();
                    return Ok(guess);
                } 
            },
            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            },
            Err(e) => {
                return Err(e);
            }
        }
    }
}

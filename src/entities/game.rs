use bincode::serialize;
use clearscreen::clear;
use core::panic;
use std::collections::HashSet;
use std::fmt;
use std::fs::read_to_string;
use std::io::stdin;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

use rand::seq::IteratorRandom;
use rand::thread_rng;

use crate::entities::GameConfig;
use crate::Difficulty;
use crate::Language;
use crate::Player;
use crate::Word;

pub struct Game {
    first_player: Player,
    second_player: Player,
    difficulty: Difficulty,
    language: Language,
    round: usize,
    turn: bool,
    wordlist: HashSet<Word>,
    selected_word: Word,
}

impl Game {
    pub fn new(first_player_name: &str, second_player_name: &str, difficulty: Difficulty, language: Language) -> Self {
        let first_player = Player::new(first_player_name);
        let second_player = Player::new(second_player_name);

        let difficulty_number = get_difficulty_number(&difficulty);
        let language_name = get_language_name(&language);

        let words = get_wordlist(&language_name, &difficulty_number);
        let random_word = get_random_word(&words);

        Self {
            first_player,
            second_player,
            difficulty: difficulty.clone(),
            language: language.clone(),
            round: 1,
            turn: true,
            wordlist: words,
            selected_word: random_word
        }
    }

    pub fn start(&mut self, stream: &mut TcpStream) {
        let mut guess = Word::new("");

        let game_info_bytes = serialize(&GameConfig::new(&self.first_player.name,
                                                         &self.second_player.name,
                                                         self.language.clone(),
                                                         self.difficulty.clone(),
                                                         self.selected_word.get_word())).expect("Errot to serialize");
 

        self.show_welcome_message();
        let _ = stream.write_all(&game_info_bytes);

        while guess != self.selected_word {

            let mut user_input: String = Default::default();

            self.announce_player_turn();

            if self.turn {
                stdin().read_line(&mut user_input).expect("Failed to read the word.");
                
                let _ = stream.write_all(user_input.as_bytes());
            }
            else {
                user_input = read_player_two_guess(stream).unwrap();
            }

            guess = Word::new(&user_input.trim_end());


            if guess.length() != self.selected_word.length() {
                match self.language {
                    Language::English => { 
                        println!("{} is an invalid word. Only {} letters long words are valid guesses.", guess, get_difficulty_number(&self.difficulty));
                    },
                    Language::Portuguese => {
                        println!("{} é uma palavra inválida. O seu guess deve ter {} caracteres.", guess, get_difficulty_number(&self.difficulty)); 
                    },
                }
            } else if self.difficulty != Difficulty::Hard && (self.first_player.has_guessed_word(&guess) || self.second_player.has_guessed_word(&guess)) {
                match self.language {
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
        match self.language {
            Language::English => println!("Welcome to word guessing game!\nLanguage: English\nDifficulty: {}\n\n{}\n\nRules:\nA {} letters long word was drawn.\nThe first player to guess correctly win the game.\nRepeat words is {}allowed.", match self.difficulty {
                Difficulty::Easy => "\x1b[32mEasy\x1b[0m",
                Difficulty::Normal => "\x1b[33mNormal\x1b[0m",
                Difficulty::Hard => "\x1b[31mHard\x1b[0m",
            }, self, get_difficulty_number(&self.difficulty), match self.difficulty {
                Difficulty::Hard => "",
                _ => "not ",
            }),
            Language::Portuguese => println!("Bem-vindo ao word guessing game!\nIdioma: Português\nDificuldade: {}\n\n{}\n\nRegras:\nUma palavra de {} caracteres foi sorteada.\nO primeiro jogador a adivinhar corretamente vence o jogo.\nRepetir palavras {}é permitido.", match self.difficulty {
                Difficulty::Easy => "\x1b[32mFácil\x1b[0m",
                Difficulty::Normal => "\x1b[33mNormal\x1b[0m",
                Difficulty::Hard => "\x1b[31mDifícil\x1b[0m",
            }, self, get_difficulty_number(&self.difficulty), match self.difficulty {
                Difficulty::Hard => "",
                _ => "não ",
            }),
        }
    }

    fn announce_player_turn(&self) {
        match self.language {
            Language::English => {
                if self.turn {
                    println!("\n{}º round.\nIt's {}'s turn!", self.round, self.first_player);
                } else {
                    println!("\n{}º round.\nIt's {}'s turn!", self.round, self.second_player);
                }
            },
            Language::Portuguese => {
                if self.turn {
                    println!("\n{}ª rodada.\nÉ a vez de {}!", self.round, self.first_player);
                } else {
                    println!("\n{}ª rodada.\nÉ a vez de {}!", self.round, self.second_player);
                }
            },
        }
    }

    fn add_guessed_word(&mut self, word: Word) {
        if self.turn {
            self.first_player.guess_word(word);
        } else {
            self.second_player.guess_word(word);
        }
    }

    fn check_word_in_wordlist(&mut self, guess: &Word) {
        if self.wordlist.contains(guess) {
            if *guess == self.selected_word {
                self.end_game();
            } else {
                for word in &self.wordlist {
                    if word == guess {
                        word.show_status(&self.selected_word);
                    }
                }

                self.next_play();
            }

        } else {
            println!("{} {}", guess, match self.language {
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
        println!("\x1b[32m{}\x1b[0m", self.selected_word);

        if self.turn {
            print!("\n{} {} ", self.first_player, match self.language {
                Language::English => "won the game after",
                Language::Portuguese => "venceu o jogo após",
            });
        } else {
            print!("\n{} {} ", self.second_player, match self.language {
                Language::English => "won the game after",
                Language::Portuguese => "venceu o jogo após",
            });
        }

        if self.round == 1 {
            println!("{}!", match self.language {
                Language::English => "only one try",
                Language::Portuguese => "uma única tentativa",
            })
        } else {
            println!("{} {}!", self.round, match self.language {
                Language::English => "tries",
                Language::Portuguese => "tentativas",
            })
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.language {
            Language::English => write!(f, "[First player: {}] [Second player: {}]", self.first_player, self.second_player),
            Language::Portuguese => write!(f, "[Primeiro jogador: {}] [Segundo jogador: {}]", self.first_player, self.second_player),
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

fn get_random_word(wordlist: &HashSet<Word>) -> Word {
    let word = &wordlist.iter().choose(&mut thread_rng());

    match word {
        Some(word) => Word::new(&word.get_word()),
        None => panic!("Erro ao sortear palavra"),
    }
}

fn get_wordlist(language_name: &str, difficulty_number: &str) -> HashSet<Word>{
    read_to_string(format!("resources/wordlist_{}_{}.txt", language_name, difficulty_number))
        .expect("Failed to read the file.")
        .lines()
        .map(|line| Word::new(line.trim()))
        .collect()
}

fn read_player_two_guess(stream: &mut TcpStream) -> Result<String, std::io::Error> {
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

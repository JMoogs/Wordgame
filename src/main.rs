use std::collections::HashSet;

use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
fn main() {
    println!("Welcome to the word quiz game!");
    println!(
        "You will be presented with a word and four options. Your task is to select the option that is a synonym of the given word."
    );
    println!("The game will get harder as you get words right, or easier as you get them wrong.");
    println!(
        "If you ever want to stop, type 'quit', 'exit' or 'stop', and your score will be displayed."
    );
    println!("Good luck!");
    println!();

    // Set up random number generator
    let mut rng = rand::rng();

    // Keep a list of played words so we don't repeat them
    let mut played_questions: HashSet<Question> = HashSet::new();

    let mut difficulty: u32 = 100;
    let mut range = 50;

    let mut asked_questions = 0;
    let mut correct_answers = 0;

    let all_questions = load_words_from_file("words.json");

    // Filter words to anything within +- 50 of the current difficulty
    loop {
        // Filter questions based on difficulty and played questions
        let filtered_questions: Vec<Question> = all_questions
            .iter()
            .filter(|q| {
                q.difficulty >= difficulty.saturating_sub(range)
                    && q.difficulty <= difficulty.saturating_add(range)
                    && !played_questions.contains(q)
            })
            .cloned()
            .collect();

        // Expand our range if we don't find any questions until we do.
        if filtered_questions.is_empty() {
            range += 20;
            continue;
        }

        // Select a random question from the filtered list
        let question: &Question = filtered_questions.choose(&mut rng).unwrap();

        let (main_word, synonym, options) = question.build_question(&filtered_questions, &mut rng);

        println!("What is a synonym for the word '{}'?", main_word);
        for (i, option) in options.iter().enumerate() {
            println!("{}. {}", i + 1, option);
        }
        asked_questions += 1;

        // Get user input and validate their answer
        // Continue looping until we get valid input
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim().eq_ignore_ascii_case("quit")
                || input.trim().eq_ignore_ascii_case("stop")
                || input.trim().eq_ignore_ascii_case("exit")
            {
                println!("Thanks for playing!");
                // Subtract 1 from asked questions since we didn't answer this one
                println!(
                    "You answered {} out of {} questions correctly.",
                    correct_answers,
                    asked_questions - 1
                );
                println!(
                    "Your final difficulty level was: {}. The maximum difficulty is 1100.",
                    difficulty
                );
                return;
            }

            match input.trim().parse::<usize>() {
                Ok(choice) if choice >= 1 && choice <= options.len() => {
                    if options[choice - 1] == synonym {
                        println!("Correct!");
                        difficulty += 70;
                        correct_answers += 1;
                    } else {
                        println!("Wrong! The correct answer was: {}", synonym);
                        println!("Definition of '{}': {}", question.word1, question.def1);
                        println!("Definition of '{}': {}", question.word2, question.def2);
                        difficulty = difficulty.saturating_sub(90);
                    }
                    break;
                }
                _ => {
                    println!(
                        "Invalid input. Please enter a number between 1 and {}.",
                        options.len()
                    );
                    continue;
                }
            }
        }

        println!();

        // Add question to played questions to prevent repetition
        played_questions.insert(question.clone());
    }
}

// A question consists of a word and a synonym where the word and synonym are of a similar difficulty.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
struct Question {
    // Difficulty of the question
    difficulty: u32,
    // First word of the synonym pair
    word1: String,
    def1: String,
    // Second word of the synonym pair
    word2: String,
    def2: String,
}

impl Question {
    // Build a question using word1 or word2 as the main word and the other as the synonym.
    // Return (main_word, synonym, all options)
    fn build_question(
        &self,
        words: &Vec<Question>,
        rng: &mut impl rand::Rng,
    ) -> (String, String, Vec<String>) {
        // Randomly choose which word is the prompt and which is the correct answer
        let (main_word, synonym) = if rand::random() {
            (self.word1.clone(), self.word2.clone())
        } else {
            (self.word2.clone(), self.word1.clone())
        };

        // Create options array with the correct answer
        let mut options = vec![synonym.clone()];

        // Select candidate words for wrong options
        let random_words = words.choose_multiple(rng, 10).cloned().collect::<Vec<_>>();

        // Add 3 more options that are not the same as the prompt word
        let mut added = 0;
        for word in random_words {
            if added >= 3 {
                break;
            }

            let candidate = if rand::random() {
                word.word1.clone()
            } else {
                word.word2.clone()
            };

            // Skip if the candidate is the same as the main word (prompt) or already in options
            if candidate != main_word && !options.contains(&candidate) {
                options.push(candidate);
                added += 1;
            }
        }

        // If we couldn't find enough unique words, fill with placeholders
        while options.len() < 4 {
            let filler = format!("option{}", options.len());
            if !options.contains(&filler) && filler != main_word {
                options.push(filler);
            }
        }

        // Shuffle options to randomize answer position
        use rand::seq::SliceRandom;
        options.shuffle(rng);

        (main_word, synonym, options)
    }
}

fn load_words_from_file(file_path: &str) -> Vec<Question> {
    let file_content = std::fs::read_to_string(file_path).expect("Failed to read file");
    let questions: Vec<Question> =
        serde_json::from_str(&file_content).expect("Failed to parse JSON");
    questions
}

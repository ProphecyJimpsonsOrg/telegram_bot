use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use teloxide::prelude::*;
use teloxide::types::{Message, InputFile, MessageId};
use teloxide::utils::command::BotCommands;
use regex::Regex;
use rand::seq::SliceRandom;
use dotenv::dotenv;
use std::env;

// List of vulgar words (example)
const VULGAR_WORDS: [&str; 77] = ["fuck", "shit", "cunt", "bitch", "asshole", "dickhead", "motherfucker", "cocksucker", "pussy", "tits", "nipples", "dick", "pussy", "whore", "jerk off", "fuck buddy", "handjob", "blowjob", "slut", "nigger", "chink", "spic", "kike", "retard", "fag", "tranny", "pussy-whipped", "whore", "bimbos", "gold digger", "fag", "queer", "tranny", "dyke", "shitcoin", "ruggers", "scammer", "ponzi", "dump", "pump and dump", "scam", "shiller", "idiot", "loser", "moron", "dumbass", "retard", "cuck", "fatass", "snowflake", "degenerate", "neckbeard", "kill yourself", "go die", "burn in hell", "eat shit", "get fucked", "weed", "cocaine", "meth", "heroin", "acid", "ecstasy", "pothead", "crackhead", "I'll kill you", "I'm going to hurt you", "cut you", "stab you", "shoot you", "bomb", "1000x gains", "mooning", "fake news", "shilling", "guaranteed profits", "get rich quick"];

// Verification questions and answers
const VERIFICATION_QUESTIONS: [(&str, &str, &str); 11] = [
    ("How many fingers are there 0,1,2,3,...?", "10", "https://ipfs.filebase.io/ipfs/Qmdz85tAFwVLwJThZCd1QQ3bAbmU2NfUhNyymmiQWSQira"),
    ("How many fingers are there 0,1,2,3,...?", "9", "https://ipfs.filebase.io/ipfs/QmZRRWBsBpTR4AqJgAjjY2GhyAHhwgxT9G7qipzcJfdCvw"),
    ("How many fingers are there 0,1,2,3,...?", "8", "https://ipfs.filebase.io/ipfs/QmTpu9Vcs7iRvkuBSqTm5eae1bKJXH1mbW7DXU6pcwkAEE"),
    ("How many fingers are there 0,1,2,3,...?", "7", "https://ipfs.filebase.io/ipfs/QmRuUgNssRA3fdXqUBinWTzFsQacfyNWnRkzkQthkwfYXe"),
    ("How many fingers are there 0,1,2,3,...?", "6", "https://ipfs.filebase.io/ipfs/QmVURxnnC4gdz7wUsYhtfcEXQXrrxAhcDBcbry9H7tGAsm"),
    ("How many fingers are there 0,1,2,3,...?", "5", "https://ipfs.filebase.io/ipfs/QmZkzhyaCmsZp65xFM7qMNcPxZz7gF6XRBRtM5XpoW4nib"),
    ("How many fingers are there 0,1,2,3,...?", "4", "https://ipfs.filebase.io/ipfs/QmQn9f9FvayUuBFyDVU6dFF5TDZCaQqcYkZWhQWEv6FwfR"),
    ("How many fingers are there 0,1,2,3,...?", "3", "https://ipfs.filebase.io/ipfs/Qmc6FjP98w4ttTf8SHx9dcVbumLDcxsGH2vMX85c7vXKDi"),
    ("How many fingers are there 0,1,2,3,...?", "2", "https://ipfs.filebase.io/ipfs/QmQwZAxAmBr9PFsBvPMDFPa3i15zhHpiWfcnr3xTgTLoRT"),
    ("How many fingers are there 0,1,2,3,...?", "1", "https://ipfs.filebase.io/ipfs/QmSU8Mjt6WzJ6DPKEGvpeZnrxEtVsMtZvsW9n3Pr63UYGN"),
    ("How many fingers are there 0,1,2,3,...?", "0", "https://ipfs.filebase.io/ipfs/QmP2GQbBNurnKiYHvrn5ejT2y1akEGiU9famxbrt9vFELG"),
];

// Struct to hold user verification state
struct UserState {
    verified: bool,
    current_question: Option<(String, String, String)>,
    message_ids: Vec<MessageId>,
    stored_messages: Vec<(MessageId, String)>,
    verify_command_message_id: Option<MessageId>,
}

// Function to check for vulgar words
fn contains_vulgar_word(text: &str) -> bool {
    let re = Regex::new(&format!(r"(?i)\b({})\b", VULGAR_WORDS.join("|"))).unwrap();
    let result = re.is_match(text);
    log::info!("Checking text: '{}', Contains vulgar word: {}", text, result);
    result
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Start the verification process")]
    Verify,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok(); // Load .env file
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot_token = env::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN must be set");
    let bot = Bot::new(bot_token);

    let user_states = Arc::new(Mutex::new(HashMap::<u64, UserState>::new()));

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(command_handler),
        )
        .branch(dptree::endpoint(message_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![user_states])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn command_handler(
    bot: Bot,
    msg: Message,
    cmd: Command,
    user_states: Arc<Mutex<HashMap<u64, UserState>>>,
) -> ResponseResult<()> {
    match cmd {
        Command::Verify => {
            let user_id = msg.from().unwrap().id.0;
            let mut states = user_states.lock().await;
            let state = states.entry(user_id).or_insert(UserState {
                verified: false,
                current_question: None,
                message_ids: Vec::new(),
                stored_messages: Vec::new(),
                verify_command_message_id: None,
            });

            if !state.verified {
                // Send a loading message first
                let loading_message = bot.send_message(msg.chat.id, "Please wait, fetching verification image...").await?;
                
                // Store the loading message ID
                state.message_ids.push(loading_message.id);

                let (question, answer, image_url) = VERIFICATION_QUESTIONS.choose(&mut rand::thread_rng()).unwrap();
                state.current_question = Some((question.to_string(), answer.to_string(), image_url.to_string()));
                state.verify_command_message_id = Some(msg.id); // Store the /verify command message ID

                // Now attempt to fetch and send the image
                match bot.send_photo(msg.chat.id, InputFile::url(image_url.parse().unwrap()))
                    .caption(format!("Please answer this question: {}", question))
                    .await {
                        Ok(sent_msg) => {
                            state.message_ids.push(sent_msg.id);

                            // Edit the loading message to display the question
                            bot.edit_message_text(msg.chat.id, loading_message.id, "Image fetched successfully, please answer the question above.").await?;
                        },
                        Err(err) => {
                            log::error!("Failed to send photo: {:?}", err);

                            // If there is an error, edit the loading message with an error message
                            bot.edit_message_text(msg.chat.id, loading_message.id, "Failed to fetch the verification image. Please try again.").await?;
                        }
                    }
            } else {
                bot.send_message(msg.chat.id, "You are already verified!").await?;
            }
        }
    }
    Ok(())
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    user_states: Arc<Mutex<HashMap<u64, UserState>>>,
) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        log::info!("Received message: '{}'", text);

        let user_id = msg.from().unwrap().id.0;
        let mut states = user_states.lock().await;
        let state = states.entry(user_id).or_insert(UserState {
            verified: false,
            current_question: None,
            message_ids: Vec::new(),
            stored_messages: Vec::new(),
            verify_command_message_id: None,
        });

        if !state.verified {
            // Store the message ID and content
            state.stored_messages.push((msg.id, text.to_string()));

            if let Some((_question, answer, _)) = &state.current_question {
                if text.to_lowercase() == answer.to_lowercase() {
                    state.verified = true;
                    state.current_question = None;

                    // Delete all stored messages
                    for (msg_id, _) in state.stored_messages.drain(..) {
                        if let Err(err) = bot.delete_message(msg.chat.id, msg_id).await {
                            log::warn!("Failed to delete message {}: {:?}", msg_id, err);
                        }
                    }

                    // Delete all verification-related messages (images, questions, and the loading message)
                    for msg_id in state.message_ids.drain(..) {
                        if let Err(err) = bot.delete_message(msg.chat.id, msg_id).await {
                            log::warn!("Failed to delete verification message {}: {:?}", msg_id, err);
                        }
                    }

                    // Delete the /verify command message
                    if let Some(verify_msg_id) = state.verify_command_message_id.take() {
                        if let Err(err) = bot.delete_message(msg.chat.id, verify_msg_id).await {
                            log::warn!("Failed to delete /verify command message {}: {:?}", verify_msg_id, err);
                        }
                    }

                    // Send a welcome message after successful verification
                    let welcome_message = format!(
                        "🎉 Congratulations, {}! You have been verified successfully. Welcome to the community!",
                        msg.from().unwrap().first_name
                    );
                    bot.send_message(msg.chat.id, welcome_message).await?;

                } else {
                    let sent_msg = bot.send_message(msg.chat.id, "Incorrect answer. Please try again or use /verify to get a new question.").await?;
                    state.message_ids.push(sent_msg.id);
                }
            } else {
                let sent_msg = bot.send_message(msg.chat.id, "Please use /verify to start the verification process.").await?;
                state.message_ids.push(sent_msg.id);
            }

            // Delete the user's message
            if let Err(err) = bot.delete_message(msg.chat.id, msg.id).await {
                log::warn!("Failed to delete message {}: {:?}", msg.id, err);
            }
        } else if contains_vulgar_word(text) {
            if let Err(err) = bot.delete_message(msg.chat.id, msg.id).await {
                log::warn!("Failed to delete vulgar message {}: {:?}", msg.id, err);
            }
            bot.send_message(msg.chat.id, "A message containing inappropriate content has been deleted.")
                .await?;
        }
    }
    Ok(())
}
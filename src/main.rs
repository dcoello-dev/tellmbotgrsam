use teloxide::{dispatching::UpdateFilterExt, prelude::*, types::{ MediaKind, MessageKind}, utils::command::BotCommands};
use clap::Parser;
use ollama_rs::generation::options::GenerationOptions;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use std::rc::Rc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    model: String,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description= "These commands are supported:")]
enum Command {
    #[command(description = "display help")]
    Help,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?,
    };
    Ok(())
}

async fn generate_response(prompt: &str) -> Option<String> {
    let ollama = Ollama::default();
    let model = "llama3.1".to_string();
    
    let options = GenerationOptions::default()
        .temperature(0.2)
        .repeat_penalty(1.5)
        .top_k(25)
        .top_p(0.25);
    
    let res = ollama.generate(GenerationRequest::new(model, prompt.to_string()).options(options)).await;
    
    if let Ok(res) = res {
        return Some(res.response);
    }
    None
}

async fn dispatch_payload(payload : MediaKind) -> Option<String> {
    match payload {
        MediaKind::Text(text) => {
            return generate_response(&text.text).await;
        }
        _other => (),
    };
    None
}

async fn message_handler(bot: Bot, msg: Message) -> ResponseResult<()> {
    match msg.kind {
        MessageKind::Common(payload) => {
            let res = dispatch_payload(payload.media_kind).await;
            if let Some(response) = res {
                bot.send_message(msg.chat.id, response)
                    .await?;
            }
        }
        _other => println!("other message"),
    }

    Ok(())
}

thread_local! {
    pub static ARGS: Rc<Args> = Rc::new(Args::parse());
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bot = Bot::from_env();

    Dispatcher::builder(bot, 
        dptree::entry()
            .branch(Update::filter_message().filter_command::<Command>().endpoint(move |bot: Bot, msg: Message, cmd:Command| { answer(bot, msg, cmd)}))
            .branch(Update::filter_message().endpoint(message_handler))
    ).enable_ctrlc_handler().build().dispatch().await;
}

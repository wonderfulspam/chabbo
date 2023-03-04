use markov::Chain;
use tracing::debug;

pub mod backends;

pub fn get_chain_from_text(text: &str) -> Chain<String> {
    debug!("loaded markov corpus of {} bytes", &text.len());
    let mut chain = Chain::of_order(2);
    for line in text.lines() {
        chain.feed_str(&line.to_lowercase());
    }
    debug!(
        "fed {} lines of text into markov chain",
        text.lines().count()
    );

    chain
}

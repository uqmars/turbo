use rand::prelude::*;

/// Commands which has text-based responses

/// Banter Command 
///
/// Responds with a bit of banter
const BANTER_REPLY_1: &str = "
Bant her? I only just met her!
";

const BANTER_REPLY_2: &str = "
I hardly know her!
";

const BANTER_REPLY_3: &str = "
Only if she'll let ya!
";

const BANTER_REPLY_0: &str = "
You may say, it is impossible for a man to become like the Machine. And I would reply, that only the smallest mind strives to comprehend its limits.
";

pub fn banter() -> String {
    let response: i32 = rand::thread_rng().gen_range(0..3);

    match response {
        0 => return BANTER_REPLY_0.to_string(),
        1 => return BANTER_REPLY_1.to_string(),
        2 => return BANTER_REPLY_2.to_string(),
        3 => return BANTER_REPLY_3.to_string(),
        _ => return "SPAGETTI".to_string(),
    }
}



/// Roll Command
///
/// RNG that defaults as a 1d20 with 1 as the lowest number.
/// Uses:
///     !roll
///     !roll [max]
///     !roll [min] [max]

pub fn roll(max: Option<i32>, min: Option<i32>, range: Option<i32>) -> String {
    let max: i32 = max.unwrap_or(20);
    let min: i32 = min.unwrap_or(1);
    let range: i32 = range.unwrap_or(1);
    let mut collection: Vec<i32> = (1..range).collect();
    let mut number = rand::thread_rng().gen_range(min..max);

    // Range selection not working due to type errors.
    let selection: String = collection.iter()
        .map(|&collection| collection.to_string() + " ")
        .collect();

    return number.to_string();
}

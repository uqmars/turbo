use super::{Leaderboard, Member, Star, db::get_discord_from_aoc};
use std::{
    collections::{BinaryHeap, HashMap},
    u8,
};

const TWO_STAR: &str = "\x1b[1;38;2;255;255;0m*";
const ONE_STAR: &str = "\x1b[1;38;2;150;150;150m^";
const NO_STAR: &str = "\x1b[1;38;2;50;50;50m.";

fn reduce_member(member: Member, num_days: u8) -> Option<(u64, Vec<u8>, String)> {
    if member.completion_day_level.len() == 0 {
        return None;
    }
    let mut days: Vec<u8> = Vec::with_capacity(num_days as usize);
    for day in 0..num_days {
        let day_stars: Option<&HashMap<u8, Star>> = member.completion_day_level.get(&(day + 1));
        let stars: u8 = match day_stars {
            Some(x) => x.keys().fold(0u8, |acc, i| acc | (1 << (i - 1))),
            None => 0u8,
        };
        days.push(stars);
    }
    Some((member.local_score, days, member.name))
}

pub fn render_lb(
    lb: Leaderboard,
    max_slots: u8,
    dc_id_to_name: impl Fn(Option<u64>) -> Option<String>,
) -> Option<String> {
    let mut contestants: BinaryHeap<(u64, Vec<u8>, String)> =
        BinaryHeap::with_capacity(lb.members.len());
    for (_, member) in lb.members {
        match reduce_member(member, lb.num_days) {
            Some(x) => contestants.push(x),
            None => (),
        }
    }

    if contestants.len() == 0 {
        return None;
    }

    let mut lb_str = String::from("\x1b[1m");
    // Add title lines
    let mut days: [String; 2] = [String::new(), String::new()];
    for day in 1..(lb.num_days + 1) {
        let day_size = (day.ilog10() + 1) as u8;
        days[1].push_str(&match day_size {
            1 => format!("{}", " "),
            _ => format!("{}", day / 10),
        });
        days[0].push_str(&format!("{}", day % 10));
    }
    lb_str.push_str(&format!(
        "{} {} {} {}\r\n",
        " ".repeat("Order".len()),
        " ".repeat("Score".len()),
        days[1],
        ""
    ));
    lb_str.push_str(&format!(
        "{} {} {} {}\r\n",
        "Order", "Score", days[0], "Name"
    ));
    lb_str.push_str("\x1b[0m");

    // Add body
    let rows: u8 = match contestants.len() as u8 > max_slots {
        true => max_slots,
        false => contestants.len() as u8,
    };
    for i in 0..rows {
        let contestant = contestants.pop().unwrap();
        let position = format!(
            "{}{})",
            " ".repeat(4 - ((i + 1).ilog10() + 1) as usize),
            i + 1
        );
        let score = format!(
            "{}{}",
            " ".repeat(5 - (contestant.0.ilog10() + 1) as usize),
            contestant.0
        );
        let mut stars = String::new();
        for i in 0..lb.num_days {
            if i > 0 && contestant.1.get(i as usize) == contestant.1.get((i - 1) as usize) {
                stars.push_str(match contestant.1.get(i as usize) {
                    Some(x) => match x {
                        0 => ".",
                        1 => "^",
                        _ => "*",
                    },
                    None => ".",
                });
            } else {
                stars.push_str(match contestant.1.get(i as usize) {
                    Some(x) => match x {
                        0 => NO_STAR,
                        1 => ONE_STAR,
                        _ => TWO_STAR,
                    },
                    None => NO_STAR,
                });
            }
        }
        stars.push_str("\x1b[0m");
        let name = match get_discord_from_aoc(contestant.2.clone()) {
            Ok(x) => match dc_id_to_name(x) {
                Some(y) => format!("{} ({})", &contestant.2, y),
                None => contestant.2.clone(),
            },
            Err(_) => String::new(),
        };
        lb_str.push_str(&format!("{} {} {} {}\r\n", position, score, stars, name));
    }

    Some(lb_str)
}

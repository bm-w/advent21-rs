// Copyright (c) 2022 Bastiaan Marinus van de Weerd

mod day01;
mod day02;
mod day03;

fn main() {
	println!("Day 1, part 1: {}", day01::part1(day01::input_nums()));
	println!("Day 1, part 2: {}", day01::part2(day01::input_nums()));
	println!("Day 2, part 1: {}", day02::part1(day02::input_commands()));
	println!("Day 2, part 2: {}", day02::part2(day02::input_commands()));
	println!("Day 3, part 1: {}", day03::part1(day03::input_strs()));
	println!("Day 3, part 2: {}", day03::part2(day03::input_strs()));
}

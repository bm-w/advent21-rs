// Copyright (c) 2022 Bastiaan Marinus van de Weerd

mod day01;
mod day02;

fn main() {
	println!("Day 1, part 1: {}", day01::part1(day01::input_nums()));
	println!("Day 1, part 2: {}", day01::part2(day01::input_nums()));
	println!("Day 2, part 1: {}", day02::part1(day02::input_commands()));
	println!("Day 2, part 2: {}", day02::part2(day02::input_commands()));
}

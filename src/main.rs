use std::{
    collections::HashMap, 
    fmt::Debug, 
    fs::{remove_file, File, OpenOptions}, 
    io::{self, stdout, Write}, 
    path::Path, time::Instant
};
use clap::Parser;

fn gen_years(from: u16, to: u16) -> Result<Vec<u16>, io::Error> {
    if from > to {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, 
            format!("{} cannot be bigger than {}", from, to)
        ))
    }

    let mut years: Vec<u16> = Vec::new();

    for year in from..=to {
        years.push(year);
    }

    Ok(years)
}

fn special_chars() -> Vec<char> {
    vec!['!', '?', ',', ';', ',', '-', '_']
}

fn human_alternatives() -> HashMap<char, char> {
    let mut alternatives = HashMap::new();
    let char_map = [
        ('@', 'a'),
        ('4', 'A'),
        ('1', 'I'), 
        ('!', '1'),
        ('0', 'o'),
        ('5', 'S'),
        ('3', 'E'),
        ('7', 'T'),
        ('$', 'S'),
        ('2', 'Z'),
        ('8', 'B'),
    ];

    for &(key, value) in &char_map {
        alternatives.insert(key, value);
    }

    alternatives
}

fn if_contains_replace(keyword: &mut String, to_check: &str, replace_if: &str) {
    if keyword.contains(to_check) {
        *keyword = keyword.replace(to_check, replace_if);
    }
}

fn add_elements<T: std::fmt::Display>(
    collection: &Vec<T>, 
    password_pool: &mut Vec<String>, 
    variation: &mut String
) {
    for element in collection {
        password_pool.push(format!("{}{}", variation, element));

        for n in 0..variation.len() {
            password_pool.push(format!("{}{}{}", &variation[..n], element, &variation[n..]));
        }
    }
}

fn append_password(file: &mut File, password: &str) {
    file.write_all(format!("{}\n", password).as_bytes())
        .expect("Unable to append data");
    file.flush().unwrap();
}

fn generate_passwords(keywords: Vec<String>, filename: &str, year_from: u16, year_to: u16) {
    let years = gen_years(year_from, year_to).unwrap();
    let specials = special_chars();
    let alt_chars = human_alternatives();
    let mut generated_passwords = Vec::new();
    let original = keywords[0].clone();
    let mut count = 0;

    if Path::new(filename).exists() {
        remove_file(filename).expect(&format!("Unable to remove: {}", filename));
    }

    let mut file = OpenOptions::new()
        .create_new(true)
        .append(true)
        .open(filename)
        .expect("Unable to open file");

    for mut keyword in keywords {
        // Replace characters based on alternatives
        for (alt, classic) in &alt_chars {
            if_contains_replace(&mut keyword, "i", "1");
            if_contains_replace(&mut keyword, "e", "3");
            if_contains_replace(&mut keyword, &classic.to_string(), &alt.to_string());
        }

        // Generate case variations
        let keyword_chars: Vec<char> = keyword.chars().collect();
        let keyword_length = keyword_chars.len();
        let variations_count = 1 << keyword_length; // 2^length for all combinations

        for current in 0..variations_count {
            let mut variation = String::new();

            // Consider case variations
            for (idx, &c) in keyword_chars.iter().enumerate() {
                if (current & (1 << idx)) != 0 {
                    variation.push(c.to_uppercase().next().unwrap());
                } else {
                    variation.push(c);
                }
            }

            // Include years and special chars to each element in password pool
            add_elements(&years, &mut generated_passwords, &mut variation);
            add_elements(&specials, &mut generated_passwords, &mut variation);
        }

        // Append original keyword
        append_password(&mut file, &original);

        // Append recently generated passwords for each keyword
        for password in &generated_passwords {
            append_password(&mut file, password);
            count += 1;

            print!(
                "\r{} passwords generated,current: {}{}", 
                count, password, " ".repeat(20)
            );
            stdout().flush().unwrap();
        }
    }

    print!("\r");
    println!("\rDone, generated {} passwords in total!{}", count, " ".repeat(10));
    println!(" ==> Output saved to `{}`", filename)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    keywords: String,
    #[arg(short, long)]
    output_filename: String,
    #[arg(short, long, default_value_t=1990)]
    from_year: u16,
    #[arg(short, long, default_value_t=2025)]
    to_year: u16
}

fn request_continue() -> bool {
    let stdin = io::stdin(); 

    loop {
        print!("Continue? (y/N): ");
        io::stdout().flush().unwrap();

        let mut buffer = String::new();
        stdin.read_line(&mut buffer).unwrap();
        let input = buffer.trim().to_lowercase();

        match input.as_str() {
            "y"  => return true,
            "n" | "" => return false, 
            _ => println!("Invalid input: {}", input),
        }
    }
}

fn main() {
    let start_time = Instant::now();

    {
        let args = Args::parse();
        let keywords = args.keywords;
        let filename = args.output_filename;
        let year_from = args.from_year;
        let year_to = args.to_year;

        println!();
        println!(" +++ Humanizer - Generator for Realistic Passwords +++");
        println!();
        println!("######################################################");
        println!("  + Keywords:    {}", keywords);
        println!("  + Output File: {}", filename);
        println!("  + Year From:   {}", year_from);
        println!("  + Year To:     {}", year_to);
        println!("######################################################");

        if !request_continue() {
            println!("Quitting..");
            return;
        }

        let tok_keywords = keywords.split(',')
            .map(|s| s.trim().to_string()) 
            .collect();

        generate_passwords(tok_keywords, &filename, year_from, year_to);
    }

    let end_time = Instant::now();
    println!("Time needed: {:#.2?}", end_time - start_time);
}
//! CLI tool for credit card validation.
//!
//! # Usage
//!
//! ```bash
//! # Validate a card number
//! ccvalidator validate 4111111111111111
//!
//! # Generate test card numbers
//! ccvalidator generate --brand visa --count 5
//!
//! # Format a card number
//! ccvalidator format 4111111111111111
//!
//! # Validate CVV
//! ccvalidator cvv 123 --brand visa
//!
//! # Validate expiry
//! ccvalidator expiry 12/25
//! ```

use cc_validator::{cvv, expiry, format, generate, is_valid, mask, validate, CardBrand};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "ccvalidator")]
#[command(
    author,
    version,
    about = "Enterprise-grade credit card validation tool"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a credit card number
    Validate {
        /// Card number to validate (spaces and dashes allowed)
        card_number: String,

        /// Output format
        #[arg(short, long, default_value = "text")]
        output: OutputFormat,
    },

    /// Generate test card numbers (for testing only)
    Generate {
        /// Card brand to generate
        #[arg(short, long, default_value = "visa")]
        brand: BrandArg,

        /// Number of cards to generate
        #[arg(short, long, default_value = "1")]
        count: usize,

        /// Output formatted (with spaces)
        #[arg(short, long)]
        formatted: bool,
    },

    /// Format a card number
    Format {
        /// Card number to format
        card_number: String,

        /// Separator to use
        #[arg(short, long, default_value = " ")]
        separator: String,
    },

    /// Validate a CVV/CVC
    Cvv {
        /// CVV to validate
        cvv: String,

        /// Card brand (affects valid length)
        #[arg(short, long)]
        brand: Option<BrandArg>,
    },

    /// Validate an expiry date
    Expiry {
        /// Expiry date (MM/YY, MM/YYYY, etc.)
        date: String,

        /// Maximum years in future to accept
        #[arg(short, long)]
        max_years: Option<u16>,
    },

    /// Mask a card number (PCI-DSS compliant)
    Mask {
        /// Card number to mask
        card_number: String,

        /// Include BIN (first 6 digits)
        #[arg(short, long)]
        with_bin: bool,
    },

    /// Check if a card passes Luhn algorithm
    Luhn {
        /// Card number to check
        card_number: String,
    },

    /// Detect card brand from number
    Detect {
        /// Card number (or partial number)
        card_number: String,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum BrandArg {
    Visa,
    Mastercard,
    Amex,
    Discover,
    DinersClub,
    Jcb,
    UnionPay,
    Maestro,
    Mir,
    Rupay,
    Verve,
    Elo,
    Troy,
    BcCard,
}

impl From<BrandArg> for CardBrand {
    fn from(arg: BrandArg) -> Self {
        match arg {
            BrandArg::Visa => CardBrand::Visa,
            BrandArg::Mastercard => CardBrand::Mastercard,
            BrandArg::Amex => CardBrand::Amex,
            BrandArg::Discover => CardBrand::Discover,
            BrandArg::DinersClub => CardBrand::DinersClub,
            BrandArg::Jcb => CardBrand::Jcb,
            BrandArg::UnionPay => CardBrand::UnionPay,
            BrandArg::Maestro => CardBrand::Maestro,
            BrandArg::Mir => CardBrand::Mir,
            BrandArg::Rupay => CardBrand::RuPay,
            BrandArg::Verve => CardBrand::Verve,
            BrandArg::Elo => CardBrand::Elo,
            BrandArg::Troy => CardBrand::Troy,
            BrandArg::BcCard => CardBrand::BcCard,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate {
            card_number,
            output,
        } => {
            cmd_validate(&card_number, output);
        }
        Commands::Generate {
            brand,
            count,
            formatted,
        } => {
            cmd_generate(brand.into(), count, formatted);
        }
        Commands::Format {
            card_number,
            separator,
        } => {
            cmd_format(&card_number, &separator);
        }
        Commands::Cvv {
            cvv: cvv_input,
            brand,
        } => {
            cmd_cvv(&cvv_input, brand.map(|b| b.into()));
        }
        Commands::Expiry { date, max_years } => {
            cmd_expiry(&date, max_years);
        }
        Commands::Mask {
            card_number,
            with_bin,
        } => {
            cmd_mask(&card_number, with_bin);
        }
        Commands::Luhn { card_number } => {
            cmd_luhn(&card_number);
        }
        Commands::Detect { card_number } => {
            cmd_detect(&card_number);
        }
    }
}

fn cmd_validate(card_number: &str, output: OutputFormat) {
    match validate(card_number) {
        Ok(card) => {
            match output {
                OutputFormat::Text => {
                    println!("Valid: yes");
                    println!("Brand: {}", card.brand().name());
                    println!("Last Four: {}", card.last_four());
                    println!("Masked: {}", card.masked());
                }
                OutputFormat::Json => {
                    println!("{{");
                    println!("  \"valid\": true,");
                    println!("  \"brand\": \"{}\",", card.brand().name());
                    println!("  \"last_four\": \"{}\",", card.last_four());
                    println!("  \"masked\": \"{}\"", card.masked());
                    println!("}}");
                }
            }
            std::process::exit(0);
        }
        Err(e) => {
            match output {
                OutputFormat::Text => {
                    println!("Valid: no");
                    println!("Error: {}", e);
                }
                OutputFormat::Json => {
                    println!("{{");
                    println!("  \"valid\": false,");
                    println!("  \"error\": \"{}\"", e);
                    println!("}}");
                }
            }
            std::process::exit(1);
        }
    }
}

fn cmd_generate(brand: CardBrand, count: usize, formatted: bool) {
    for _ in 0..count {
        let card = generate::generate_card(brand);
        if formatted {
            println!("{}", format::format_card_number(&card));
        } else {
            println!("{}", card);
        }
    }
}

fn cmd_format(card_number: &str, separator: &str) {
    let formatted = format::format_with_separator(card_number, separator);
    println!("{}", formatted);
}

fn cmd_cvv(cvv_input: &str, brand: Option<CardBrand>) {
    let result = match brand {
        Some(b) => cvv::validate_cvv_for_brand(cvv_input, b),
        None => cvv::validate_cvv(cvv_input),
    };

    match result {
        Ok(validated) => {
            println!("Valid: yes");
            println!("Length: {} digits", validated.length());
            std::process::exit(0);
        }
        Err(e) => {
            println!("Valid: no");
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_expiry(date: &str, max_years: Option<u16>) {
    let result = match max_years {
        Some(years) => expiry::validate_expiry_with_options(date, true, Some(years)),
        None => expiry::validate_expiry(date),
    };

    match result {
        Ok(exp) => {
            println!("Valid: yes");
            println!("Month: {:02}", exp.month());
            println!("Year: {}", exp.year());
            println!("Formatted: {}", exp.format_short());
            if exp.is_expired() {
                println!("Status: Expired");
            } else {
                println!("Months Until Expiry: {}", exp.months_until_expiry());
            }
            std::process::exit(0);
        }
        Err(e) => {
            println!("Valid: no");
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_mask(card_number: &str, with_bin: bool) {
    if with_bin {
        match validate(card_number) {
            Ok(card) => {
                println!("{}", mask::mask_with_bin(&card));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Simple masking without full validation
        let digits: String = card_number.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            let masked = format!(
                "{}{}",
                "*".repeat(digits.len() - 4),
                &digits[digits.len() - 4..]
            );
            println!("{}", masked);
        } else {
            eprintln!("Error: Card number too short");
            std::process::exit(1);
        }
    }
}

fn cmd_luhn(card_number: &str) {
    if is_valid(card_number) {
        println!("Luhn check: PASS");
        std::process::exit(0);
    } else {
        println!("Luhn check: FAIL");
        std::process::exit(1);
    }
}

fn cmd_detect(card_number: &str) {
    let digits: Vec<u8> = card_number
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c as u8 - b'0')
        .collect();

    if digits.is_empty() {
        eprintln!("Error: No digits provided");
        std::process::exit(1);
    }

    let brand = cc_validator::detect::detect_brand(&digits);
    match brand {
        Some(b) => {
            println!("Detected Brand: {}", b.name());
            println!("Valid Lengths: {:?}", b.valid_lengths());
        }
        None => {
            println!("Detected Brand: Unknown");
        }
    }
}

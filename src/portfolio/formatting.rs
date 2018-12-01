use std::fmt::Write;

use ansi_term::{Style, Color, ANSIString};
use num_traits::{ToPrimitive, Zero};
use separator::Separatable;

use types::Decimal;
use util;

use super::asset_allocation::{Portfolio, AssetAllocation, Holding};

// FIXME: flat mode
pub fn print_portfolio(portfolio: &Portfolio) {
    let expected_assets_value = portfolio.total_value - portfolio.min_free_assets;

    for asset in &portfolio.assets {
        print_asset(asset, expected_assets_value, &portfolio.currency, 0);
    }

    println!();
    println!("{}: {}", colorify_name("Total value"), format_cash(&portfolio.currency, portfolio.total_value));
    println!("{}: {}", colorify_name("Free assets"), format_cash(&portfolio.currency, portfolio.free_assets));
}

fn print_asset(asset: &AssetAllocation, expected_total_value: Decimal, currency: &str, depth: usize) {
    let expected_value = expected_total_value * asset.expected_weight;

    let mut buffer = String::new();

    write!(&mut buffer, "{bullet:>indent$} {name}",
           bullet='•', indent=depth * 2 + 1, name=colorify_name(&asset.full_name())).unwrap();

    if asset.buy_blocked {
        write!(&mut buffer, " {}", colorify_restriction("[buy blocked]")).unwrap();
    }
    if asset.sell_blocked {
        write!(&mut buffer, " {}", colorify_restriction("[sell blocked]")).unwrap();
    }

    write!(&mut buffer, " -").unwrap();

    if let Holding::Stock(ref holding) = asset.holding {
        write!(&mut buffer, " {}",
               format_shares(holding.current_shares.to_i32().unwrap(), false)).unwrap();
    }

    write!(&mut buffer, " {current_weight} ({current_value})",
           current_weight=format_weight(get_weight(asset.current_value, expected_total_value)),
           current_value=format_cash(currency, asset.current_value)).unwrap();

    if asset.target_value != asset.current_value {
        if let Holding::Stock(ref holding) = asset.holding {
            let colorify_func = if holding.target_shares > holding.current_shares {
                colorify_buy
            } else {
                colorify_sell
            };

            let shares_change =
                holding.target_shares.to_i32().unwrap() - holding.current_shares.to_i32().unwrap();
            let value_change = asset.target_value - asset.current_value;

            let changes = format!(
                "{shares_change} ({value_change})",
                shares_change=format_shares(shares_change, true),
                value_change=format_cash(currency, value_change.abs()));

            write!(&mut buffer, " {}", colorify_func(&changes)).unwrap();
        }

        write!(&mut buffer, " → {target_weight} ({target_value})",
               target_weight=format_weight(get_weight(asset.target_value, expected_total_value)),
               target_value=format_cash(currency, asset.target_value)).unwrap();
    }

    write!(&mut buffer, " / {expected_weight} ({expected_value})",
           expected_weight=format_weight(asset.expected_weight),
           expected_value=format_cash(currency, expected_value)).unwrap();

    if let Holding::Group(ref sub_assets) = asset.holding {
        write!(&mut buffer, ":").unwrap();
        println!("{}", buffer);

        for sub_asset in sub_assets {
            print_asset(sub_asset, expected_value, currency, depth + 1);
        }
    } else {
        println!("{}", buffer);
    }
}

fn format_cash(currency: &str, amount: Decimal) -> String {
    let mut buffer = String::new();

    if currency == "USD" {
        write!(&mut buffer, "$").unwrap();
    }

    write!(&mut buffer, "{}", util::round_to(amount, 0).to_i64().unwrap().separated_string()).unwrap();

    match currency {
        "USD" => (),
        "RUB" => write!(&mut buffer, "₽").unwrap(),
        _ => write!(&mut buffer, " {}", currency).unwrap(),
    };

    buffer
}

fn format_shares(shares: i32, with_sign: bool) -> String {
    let symbol = 's';

    if with_sign {
        format!("{:+}{}", shares, symbol)
    } else {
        format!("{}{}", shares, symbol)
    }
}

fn get_weight(asset_value: Decimal, expected_total_value: Decimal) -> Decimal {
    if expected_total_value.is_zero() {
        Decimal::max_value()
    } else {
        asset_value / expected_total_value
    }
}

fn format_weight(weight: Decimal) -> String {
    if weight == Decimal::max_value() {
        "∞".to_owned()
    } else {
        format!("{}%", util::round_to(weight * dec!(100), 1))
    }
}

fn colorify_name(name: &str) -> ANSIString {
    Style::new().bold().paint(name)
}

fn colorify_restriction(message: &str) -> ANSIString {
    Color::Blue.paint(message)
}

fn colorify_buy(message: &str) -> ANSIString {
    Color::Green.paint(message)
}

fn colorify_sell(message: &str) -> ANSIString {
    Color::Red.paint(message)
}
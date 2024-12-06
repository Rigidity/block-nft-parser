use std::fs;

use chia::{
    clvm_traits::ToClvm,
    protocol::{Coin, CoinSpend, Program},
    puzzles::nft::NftMetadata,
};
use chia_wallet_sdk::{HashedPtr, Layer, NftInfo, NftStateLayer, Puzzle, SingletonLayer};
use clvmr::Allocator;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct File {
    block_spends: Vec<BlockSpend>,
}

#[derive(Debug, Deserialize)]
struct BlockSpend {
    coin: CoinJson,
    puzzle_reveal: String,
    solution: String,
}

#[derive(Debug, Deserialize)]
struct CoinJson {
    parent_coin_info: String,
    puzzle_hash: String,
    amount: u64,
}

fn strip_hex(source: &str) -> &str {
    if let Some(source) = source.strip_prefix("0x") {
        source
    } else {
        source
    }
}

fn main() -> anyhow::Result<()> {
    let source = fs::read_to_string("output.json")?;
    let file: File = serde_json::from_str(&source)?;

    for block_spend in file.block_spends {
        let coin_spend = CoinSpend {
            coin: Coin {
                parent_coin_info: hex::decode(strip_hex(&block_spend.coin.parent_coin_info))?
                    .try_into()?,
                puzzle_hash: hex::decode(strip_hex(&block_spend.coin.puzzle_hash))?.try_into()?,
                amount: block_spend.coin.amount,
            },
            puzzle_reveal: Program::from(hex::decode(strip_hex(&block_spend.puzzle_reveal))?),
            solution: Program::from(hex::decode(strip_hex(&block_spend.solution))?),
        };

        let mut allocator = Allocator::new();
        let puzzle = coin_spend.puzzle_reveal.to_clvm(&mut allocator)?;
        let puzzle = Puzzle::parse(&allocator, puzzle);

        let Some((parsed, _)) = NftInfo::<HashedPtr>::parse(&allocator, puzzle)? else {
            let Some(parsed) = SingletonLayer::<NftStateLayer<NftMetadata, Puzzle>>::parse_puzzle(
                &allocator, puzzle,
            )?
            else {
                continue;
            };
            println!("\nNFT0: {parsed:?}");
            continue;
        };

        println!("\nNFT1: {parsed:?}");
    }

    Ok(())
}

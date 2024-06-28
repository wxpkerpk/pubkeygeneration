use mysql::{params, prelude::Queryable, OptsBuilder, Pool};
use rand::Rng;
use rayon::prelude::*;
use solana_program::{config::program, program_error::ProgramError, pubkey::Pubkey};
use std::sync::atomic::{AtomicUsize, Ordering};
extern crate mysql;

const BASE58_PROGRAM_ID: &str = "C6L4yyXXCc44SVXvUSnijjMgmhqxAStM69qfd7yummZM";
// 解码 Base58 编码的程序 ID
fn decode_base58_program_id() -> Pubkey {
    let decoded_bytes = bs58::decode(BASE58_PROGRAM_ID).into_vec().expect("msg");
    let program_id = Pubkey::new(&decoded_bytes);
    (program_id)
}


fn main() {
    let opts = OptsBuilder::new()
        .ip_or_hostname(Some("taproot-mysql.cd2ui68waqtj.ap-southeast-1.rds.amazonaws.com"))
        .user(Some("taproot_mysql"))
        .pass(Some("taproot{gAme}88"))
        .db_name(Some("memecoin"))
        .tcp_port(3306);
    // 通过数据库参数，建立一个数据库连接池Pool
    let pool = match Pool::new(opts) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error {}", e);
            std::process::exit(1);
        }
    };

    // 在连接池中建立一个新的连接

    let program_id = decode_base58_program_id();
    (0..6).into_par_iter().for_each(|_| {
        let mut rng = rand::thread_rng();
        let mut b: [u8; 32] = [0u8; 32];

        loop {
            let seed: [u8; 32] = rng.gen();

            let seeds: &[&[u8]] = &[&seed];

            let pda_address = Pubkey::find_program_address(seeds, &program_id);

            let pubkey_str = pda_address.0.to_string();

            if pubkey_str.to_lowercase().ends_with("meme") {
                println!("Solana 公钥: {}", pubkey_str);
                match pool.get_conn() {
                    Ok(mut conn) => {
                        // 使用conn执行数据库操作
                        match conn.exec_drop(
                            "insert into mint_address (address) values (:address)",
                            params! {
                                "address" => pubkey_str
                            },
                        ) {
                            Ok(_) => println!("Insert successful"),
                            Err(e) => eprintln!("Error inserting data: {}", e),
                        }
                    }
                    Err(e) => {
                        eprintln!("Error getting connection from pool: {}", e);
                    }
                }
            }
        }
    });
}

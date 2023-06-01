use std::{path::{Path, PathBuf}, fs::remove_file, io::Write};

use serde::{Serialize, Deserialize};
use sqlx::mysql::MySqlPoolOptions;



#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect("mysql://root:123456@localhost:3306/ord")
        .await?;
    
    let rows = sqlx::query_as::<_, WalletInfo>(r#"select s_id, wallet_id, receive_address, create_time from wallet_info"#)
        .fetch_all(&pool).await?;

    println!("query wallet_info size: {}", rows.len());

    let rows: Vec<OrdDomain> = sqlx::query_as(r#"select wallet_id, dom_name, dom_state, inscribe_id, expire_time, create_time from ord_domain order by s_id asc"#)
        .fetch_all(&pool).await?;

    println!("query OrdDomain size: {}", rows.len());


    let inscriptions: Vec<DomainInscriptionInfo> = sqlx::query_as(r#"select * from domain_inscription_info order by inscribe_num asc limit 2100"#)
        .fetch_all(&pool).await?;

    println!("query og size: {}", inscriptions.len());

    let mut magic_res = vec![];
    let mut collect_res = vec![];
    let mut ow_res = vec![];
    for row in rows {
        let dom_name = row.dom_name;
        let img_url = format!("https://btcdomains.io/images/domain/{}.jpeg", &dom_name[0..dom_name.len() - 4]);
        if row.dom_state == 0 || row.dom_state == 5 || row.dom_state == 6 {
            let data = Data{
                id: row.inscribe_id.clone(),
                meta: Meta { 
                    name: dom_name.clone(), 
                    high_res_img_url: img_url, 
                    attributes: [
                        Attr{
                            trait_type: "register_date".to_string(),
                            value: row.create_time.to_string()
                        },
                        Attr{
                            trait_type: "expire_date".to_string(),
                            value: row.expire_time.to_string()
                        }
                    ].to_vec()
                }
            };
            magic_res.push(data);

            let coll = Data {
                id: row.inscribe_id.clone(),
                meta: Meta2 {
                    name: dom_name.clone()
                }
            };
            collect_res.push(coll);
            
            
            let check_og = check_og_fn(dom_name.clone(), inscriptions.clone());
            let (status, rank) = if check_og {
                (format!("OG"), get_len(&dom_name, true))
            }else {
                (format!("NonOG"), get_len(&dom_name, false))
            };
            let data = Data {
                id: row.inscribe_id.clone(),
                meta: MetaOw { 
                    name: dom_name.clone(), 
                    status, 
                    rank,
                    attributes: vec![],
                }
            };
            ow_res.push(data);
        }
    }

    let default_path = "/home/free/data/ord.json";
    if PathBuf::from(&default_path).exists() {
        let _ = remove_file(default_path);
    }
    let mut file = std::fs::File::create(default_path).unwrap();
    let file_data = serde_json::to_vec(&magic_res).unwrap();
    let _ = file.write(&file_data);

    println!("[magic]finish!! total size: {}", magic_res.len());

    let default_path = "/home/free/data/inscriptions.json";
    if PathBuf::from(&default_path).exists() {
        let _ = remove_file(default_path);
    }
    let mut file = std::fs::File::create(default_path).unwrap();
    let file_data = serde_json::to_vec(&collect_res).unwrap();
    let _ = file.write(&file_data);

    println!("[collect]finish!! total size: {}", collect_res.len());


    let default_path = "/home/free/data/ow/inscriptions.json";
    if PathBuf::from(&default_path).exists() {
        let _ = remove_file(default_path);
    }
    let mut file = std::fs::File::create(default_path).unwrap();
    let file_data = serde_json::to_vec(&ow_res).unwrap();
    let _ = file.write(&file_data);

    println!("[ow]finish!! total size: {}", ow_res.len());

    Ok(())
}

fn check_og_fn(name: String, inscriptions: Vec<DomainInscriptionInfo>) -> bool {
    for info in inscriptions {
        if info.domain_name == name {
            return true;
        }
    }
    return false;
}

fn get_len(name: &str, og: bool) -> i32{
    match name.len() {
        8 => {
            if og {
                10
            }else {
                11
            }
        }
        9 => {
            if og {
                20
            }else {
                21
            }
        }
        _ => {
            if og {
                30
            }else {
                31
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Data<T> {
    pub id: String,
    pub meta: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Meta {
    pub name: String,
    pub high_res_img_url: String,
    pub attributes: Vec<Attr>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Attr {
    pub trait_type: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct MetaOw {
    pub name: String,
    pub status: String,
    pub rank: i32,
    pub attributes: Vec<Attr>,
}

// #[derive(Serialize, Debug, Clone, sqlx::FromRow)]
// pub struct OrdDomain {
//     pub s_id: Option<i64>,
//     pub wallet_id: Option<String>,
//     pub dom_name: Option<String>,
//     pub dom_type: Option<String>,
//     pub dom_state: Option<i64>,
//     pub inscribe_id: Option<String>,
//     pub tx_hash: Option<String>,
//     pub img_url: Option<String>,
//     pub expire_time: Option<i64>,
//     pub cost_fee: Option<f64>,
//     pub out_wallet: Option<String>,
//     pub fee_rate: Option<i64>,
//     pub create_time: Option<i64>,
//     pub update_time: Option<i64>,
// }

#[derive(Serialize, Debug, Clone, sqlx::FromRow)]
pub struct OrdDomain {
    pub wallet_id: String,
    pub dom_name: String,
    pub dom_state: i32,
    pub inscribe_id: String,
    pub expire_time: i64,
    pub create_time: i64,
}


#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct WalletInfo {
    pub s_id: i64,
    pub wallet_id: String,
    pub receive_address: String,
    pub create_time: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Meta2 {
    pub name: String
}


#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, Clone)]

pub struct DomainInscriptionInfo {
    pub id: i64,
    pub inscribe_num: i64,
    pub inscribe_id: String,
    pub sat: i64,
    pub domain_name: String,
    pub address: String,
    pub create_time: i64,
    pub update_time: i64,
    pub expire_date: i64,
    pub register_date: i64,
}
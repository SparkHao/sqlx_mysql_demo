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

    let mut magic_res = vec![];
    let mut collect_res = vec![];
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
                id: row.inscribe_id,
                meta: Meta2 {
                    name: dom_name
                }
            };

            collect_res.push(coll);
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

    Ok(())
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
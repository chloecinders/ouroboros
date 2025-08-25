use serenity::all::Timestamp;
use tokio::sync::Mutex;

static COUNTER: Mutex<u64> = Mutex::const_new(0);

pub async fn random() -> u64 {
    let mut c = COUNTER.lock().await;

    if *c == 0 {
        *c = Timestamp::now().timestamp() as u64;
    }

    *c = c.wrapping_mul(54329072133).wrapping_add(9081523890);
    *c
}

const CHAR_MAP: &str = "ABCDEFGHJKLMNPRSTUVWXYZabcdefghjkmnpqrstuvwxyz123456789";

pub async fn tinyid() -> String {
    let mut res = String::new();

    for _ in 1..=6 {
        let rand = (random().await % CHAR_MAP.len() as u64) as usize;
        res.push(CHAR_MAP.chars().nth(rand).unwrap());
    }

    res
}

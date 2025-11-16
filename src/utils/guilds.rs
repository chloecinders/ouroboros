use serenity::all::{CacheHttp, GuildInfo, GuildPagination};

pub async fn get_all_guilds(http: impl CacheHttp) -> Vec<GuildInfo> {
    let mut result: Vec<GuildInfo> = Vec::new();

    loop {
        let last_page = result.last().map(|g| GuildPagination::After(g.id));
        let guilds = http.http().get_guilds(last_page, None).await;

        if let Ok(guilds) = guilds {
            if guilds.len() == 0 {
                break;
            }

            for guild in guilds {
                result.push(guild);
            }
        } else {
            break;
        }
    }

    result
}

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use linkify::{LinkFinder, LinkKind};
use mirakc_client::models::MirakurunProgramGenresInner;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use typeshare::typeshare;

// Meilisearchのドキュメントの型
#[derive(Clone, Debug, Deserialize, Serialize)]
#[typeshare]
pub struct ProgramDocument {
    #[serde(rename = "タイトル")]
    pub title: String,

    #[serde(rename = "番組情報")]
    pub description: String,

    #[serde(rename = "その他情報")]
    pub extended: String,

    #[serde(rename = "放送局")]
    pub channel: String,

    #[serde(rename = "ジャンル")]
    pub genres: Vec<String>,

    #[serde(rename = "開始時刻")]
    #[typeshare(serialized_as = "string")]
    pub start_at: DateTime<Utc>,

    #[serde(rename = "終了時刻")]
    #[typeshare(serialized_as = "string")]
    pub end_at: DateTime<Utc>,

    #[serde(rename = "放送曜日")]
    pub day_of_week: String,

    // 放送時間は分単位にする
    #[serde(rename = "放送時間")]
    #[typeshare(serialized_as = "number")]
    pub duration: i64,

    #[serde(rename = "公式サイト等")]
    pub urls: Vec<String>,

    pub ogp_url: Option<String>,
    pub ogp_url_hash: Option<String>,

    // 元の形式
    #[typeshare(serialized_as = "number")]
    pub program_id: i64,
    #[typeshare(serialized_as = "number")]
    pub service_id: i64,
}

impl ProgramDocument {
    pub fn from_program_with_service(
        program: mirakc_client::models::MirakurunProgram,
        service: mirakc_client::models::MirakurunService,
    ) -> Self {
        let start_at = unix_millis_to_datetime(program.start_at);
        let end_at = unix_millis_to_datetime(program.start_at + program.duration);
        let duration = Duration::milliseconds(program.duration);
        let extended = match program.extended.unwrap_or_default().as_object() {
            Some(ext) => ext
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v.as_str().unwrap_or_default()))
                .collect::<Vec<String>>()
                .join("\n"),
            None => "".to_string(),
        };
        let mut finder = LinkFinder::new();
        finder.url_must_have_scheme(false);
        finder.kinds(&[LinkKind::Url]);
        let urls = finder
            .links(&extended.replace("　", " "))
            .filter(|link| link.kind() == &LinkKind::Url)
            .filter(|link| {
                let link_with_scheme = if link.as_str().starts_with("http") {
                    link.as_str().to_string()
                } else {
                    format!("https://{}", link.as_str())
                };
                let url = match url::Url::parse(link_with_scheme.as_str()) {
                    Ok(url) => url,
                    Err(_) => return false,
                };
                let host = url.host_str().unwrap_or_default();
                // SNSはOGP持ってないので除外する
                // TODO: アイコンとかトップイメージで代用出来ないか？
                if [
                    "x.com",
                    "twitter.com",
                    "tiktok.com",
                    "instagram.com",
                    "www.instagram.com",
                    "facebook.com",
                ]
                .contains(&host)
                {
                    return false;
                }
                true
            })
            .map(|link| link.as_str().to_string())
            .collect::<Vec<_>>();

        let ogp_url = if urls.is_empty() {
            None
        } else {
            Some(urls[0].clone())
        };
        let ogp_url_hash = match ogp_url.clone() {
            Some(ogp_url) => {
                let mut hasher = Sha1::new();
                hasher.update(ogp_url.as_bytes());
                Some(format!("{:x}", hasher.finalize()))
            }
            None => None,
        };
        let day_of_weeks = ["月", "火", "水", "木", "金", "土", "日"];
        let day_of_week = day_of_weeks[start_at
            .with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap())
            .weekday() as usize];

        Self {
            title: program
                .name
                .unwrap_or_else(|| Some("＜不明な番組＞".to_string()))
                .unwrap_or_else(|| "＜不明な番組＞".to_string()),
            description: program
                .description
                .unwrap_or_else(|| Some("＜不明＞".to_string()))
                .unwrap_or_else(|| "＜不明＞".to_string()),
            extended,
            channel: service.name,
            genres: program
                .genres
                .unwrap_or_else(|| Some(vec![]))
                .unwrap_or_default()
                .iter()
                .map(genre_code_to_string)
                .collect(),
            start_at,
            end_at,
            day_of_week: day_of_week.to_string(),
            duration: duration.num_minutes(),
            ogp_url,
            ogp_url_hash,
            urls,
            program_id: program.id,
            service_id: service.id,
        }
    }
}

fn unix_millis_to_datetime(millis: i64) -> DateTime<Utc> {
    // ミリ秒からナノ秒へ変換
    let nanos = millis * 1_000_000;
    // UNIX時間ナノ秒を DateTime<Utc> に変換
    Utc.timestamp_nanos(nanos)
}

fn genre_code_to_string(genre: &MirakurunProgramGenresInner) -> String {
    get_subgenre(genre.lv1, genre.lv2).to_string()
}

const GENRE: [&str; 16] = [
    "ニュース・報道",
    "スポーツ",
    "情報・ワイドショー",
    "ドラマ",
    "音楽",
    "バラエティ",
    "映画",
    "アニメ・特撮",
    "ドキュメンタリー・教養",
    "劇場・公演",
    "趣味・教育",
    "福祉",
    "予備",
    "予備",
    "拡張",
    "その他",
];

pub fn get_genre(genre: u8) -> &'static str {
    GENRE[genre as usize]
}

const SUB_GENRE: [&str; 256] = [
    "ニュース・報道／定時・総合",                   // 0x00
    "ニュース・報道／天気",                         // 0x01
    "ニュース・報道／特集・ドキュメント",           // 0x02
    "ニュース・報道／政治・国会",                   // 0x03
    "ニュース・報道／経済・市況",                   // 0x04
    "ニュース・報道／海外・国際",                   // 0x05
    "ニュース・報道／解説",                         // 0x06
    "ニュース・報道／討論・会談",                   // 0x07
    "ニュース・報道／報道特番",                     // 0x08
    "ニュース・報道／ローカル・地域",               // 0x09
    "ニュース・報道／交通",                         // 0x0A
    "ニュース・報道／？",                           // 0x0B
    "ニュース・報道／？",                           // 0x0C
    "ニュース・報道／？",                           // 0x0D
    "ニュース・報道／？",                           // 0x0E
    "ニュース・報道／その他",                       // 0x0F
    "スポーツ／スポーツニュース",                   // 0x10
    "スポーツ／野球",                               // 0x11
    "スポーツ／サッカー",                           // 0x12
    "スポーツ／ゴルフ",                             // 0x13
    "スポーツ／その他の球技",                       // 0x14
    "スポーツ／相撲・格闘技",                       // 0x15
    "スポーツ／オリンピック・国際大会",             // 0x16
    "スポーツ／マラソン・陸上・水泳",               // 0x17
    "スポーツ／モータースポーツ",                   // 0x18
    "スポーツ／マリン・ウィンタースポーツ",         // 0x19
    "スポーツ／競馬・公営競技",                     // 0x1A
    "スポーツ／？",                                 // 0x1B
    "スポーツ／？",                                 // 0x1C
    "スポーツ／？",                                 // 0x1D
    "スポーツ／？",                                 // 0x1E
    "スポーツ／その他",                             // 0x1F
    "情報・ワイドショー／芸能・ワイドショー",       // 0x20
    "情報・ワイドショー／ファッション",             // 0x21
    "情報・ワイドショー／暮らし・住まい",           // 0x22
    "情報・ワイドショー／健康・医療",               // 0x23
    "情報・ワイドショー／ショッピング・通販",       // 0x24
    "情報・ワイドショー／グルメ・料理",             // 0x25
    "情報・ワイドショー／イベント",                 // 0x26
    "情報・ワイドショー／番組紹介・お知らせ",       // 0x27
    "情報・ワイドショー／？",                       // 0x28
    "情報・ワイドショー／？",                       // 0x29
    "情報・ワイドショー／？",                       // 0x2A
    "情報・ワイドショー／？",                       // 0x2B
    "情報・ワイドショー／？",                       // 0x2C
    "情報・ワイドショー／？",                       // 0x2D
    "情報・ワイドショー／？",                       // 0x2E
    "情報・ワイドショー／その他",                   // 0x2F
    "ドラマ／国内ドラマ",                           // 0x30
    "ドラマ／海外ドラマ",                           // 0x31
    "ドラマ／時代劇",                               // 0x32
    "ドラマ／サスペンス・ミステリー",               // 0x33 あってる？
    "ドラマ／ファンタジー",                         // 0x34 あってる？
    "ドラマ／SF",                                   // 0x35 あってる？
    "ドラマ／アクション",                           // 0x36 あってる？
    "ドラマ／アドベンチャー",                       // 0x37 あってる？
    "ドラマ／コメディー",                           // 0x38 あってる？
    "ドラマ／ラブストーリー",                       // 0x39 あってる？
    "ドラマ／ファミリー",                           // 0x3A あってる？
    "ドラマ／？",                                   // 0x3B
    "ドラマ／？",                                   // 0x3C
    "ドラマ／？",                                   // 0x3D
    "ドラマ／？",                                   // 0x3E
    "ドラマ／その他",                               // 0x3F
    "音楽／国内ロック・ポップス",                   // 0x40
    "音楽／海外ロック・ポップス",                   // 0x41
    "音楽／クラシック・オペラ",                     // 0x42
    "音楽／ジャズ・フュージョン",                   // 0x43
    "音楽／歌謡曲・演歌",                           // 0x44
    "音楽／ライブ・コンサート",                     // 0x45
    "音楽／ランキング・リクエスト",                 // 0x46
    "音楽／カラオケ・のど自慢",                     // 0x47
    "音楽／民謡・邦楽",                             // 0x48
    "音楽／童謡・キッズ",                           // 0x49
    "音楽／民族音楽・ワールドミュージック",         // 0x4A
    "音楽／？",                                     // 0x4B
    "音楽／？",                                     // 0x4C
    "音楽／？",                                     // 0x4D
    "音楽／？",                                     // 0x4E
    "音楽／その他",                                 // 0x4F
    "バラエティ／クイズ",                           // 0x50
    "バラエティ／ゲーム",                           // 0x51
    "バラエティ／トークバラエティ",                 // 0x52
    "バラエティ／お笑い・コメディ",                 // 0x53
    "バラエティ／音楽バラエティ",                   // 0x54
    "バラエティ／旅バラエティ",                     // 0x55
    "バラエティ／料理バラエティ",                   // 0x56
    "バラエティ／？",                               // 0x57
    "バラエティ／？",                               // 0x58
    "バラエティ／？",                               // 0x59
    "バラエティ／？",                               // 0x5A
    "バラエティ／？",                               // 0x5B
    "バラエティ／？",                               // 0x5C
    "バラエティ／？",                               // 0x5D
    "バラエティ／？",                               // 0x5E
    "バラエティ／その他",                           // 0x5F
    "映画／洋画",                                   // 0x60
    "映画／邦画",                                   // 0x61
    "映画／アニメ",                                 // 0x62
    "映画／映画（その他）",                         // 0x63 あってる？
    "映画／？",                                     // 0x64
    "映画／？",                                     // 0x65
    "映画／？",                                     // 0x66
    "映画／？",                                     // 0x67
    "映画／？",                                     // 0x68
    "映画／？",                                     // 0x69
    "映画／？",                                     // 0x6A
    "映画／？",                                     // 0x6B
    "映画／？",                                     // 0x6C
    "映画／？",                                     // 0x6D
    "映画／？",                                     // 0x6E
    "映画／その他",                                 // 0x6F
    "アニメ・特撮／国内アニメ",                     // 0x70
    "アニメ・特撮／海外アニメ",                     // 0x71
    "アニメ・特撮／特撮",                           // 0x72
    "アニメ・特撮／？",                             // 0x73
    "アニメ・特撮／？",                             // 0x74
    "アニメ・特撮／？",                             // 0x75
    "アニメ・特撮／？",                             // 0x76
    "アニメ・特撮／？",                             // 0x77
    "アニメ・特撮／？",                             // 0x78
    "アニメ・特撮／？",                             // 0x79
    "アニメ・特撮／？",                             // 0x7A
    "アニメ・特撮／？",                             // 0x7B
    "アニメ・特撮／？",                             // 0x7C
    "アニメ・特撮／？",                             // 0x7D
    "アニメ・特撮／？",                             // 0x7E
    "アニメ・特撮／その他",                         // 0x7F
    "ドキュメンタリー・教養／社会・時事",           // 0x80
    "ドキュメンタリー・教養／歴史・紀行",           // 0x81
    "ドキュメンタリー・教養／自然・動物・環境",     // 0x82
    "ドキュメンタリー・教養／宇宙・科学・医学",     // 0x83
    "ドキュメンタリー・教養／カルチャー・伝統文化", // 0x84
    "ドキュメンタリー・教養／文学・文芸",           // 0x85
    "ドキュメンタリー・教養／スポーツ",             // 0x86
    "ドキュメンタリー・教養／ドキュメンタリー全般", // 0x87
    "ドキュメンタリー・教養／インタビュー・討論",   // 0x88
    "ドキュメンタリー・教養／？",                   // 0x89
    "ドキュメンタリー・教養／？",                   // 0x8A
    "ドキュメンタリー・教養／？",                   // 0x8B
    "ドキュメンタリー・教養／？",                   // 0x8C
    "ドキュメンタリー・教養／？",                   // 0x8D
    "ドキュメンタリー・教養／？",                   // 0x8E
    "ドキュメンタリー・教養／その他",               // 0x8F
    "劇場・公演／現代劇・新劇",                     // 0x90
    "劇場・公演／ミュージカル",                     // 0x91
    "劇場・公演／ダンス・バレエ",                   // 0x92
    "劇場・公演／落語・演芸",                       // 0x93
    "劇場・公演／歌舞伎・古典",                     // 0x94
    "劇場・公演／サーカス・パフォーマンス",         // 0x95 あってる？
    "劇場・公演／？",                               // 0x96
    "劇場・公演／？",                               // 0x97
    "劇場・公演／？",                               // 0x98
    "劇場・公演／？",                               // 0x99
    "劇場・公演／？",                               // 0x9A
    "劇場・公演／？",                               // 0x9B
    "劇場・公演／？",                               // 0x9C
    "劇場・公演／？",                               // 0x9D
    "劇場・公演／？",                               // 0x9E
    "劇場・公演／その他",                           // 0x9F
    "趣味・教育／旅・釣り・アウトドア",             // 0xA0
    "趣味・教育／園芸・ペット・手芸",               // 0xA1
    "趣味・教育／音楽・美術・工芸",                 // 0xA2
    "趣味・教育／囲碁・将棋",                       // 0xA3
    "趣味・教育／麻雀・パチンコ",                   // 0xA4
    "趣味・教育／車・オートバイ",                   // 0xA5
    "趣味・教育／コンピュータ・ＴＶゲーム",         // 0xA6
    "趣味・教育／会話・語学",                       // 0xA7
    "趣味・教育／幼児・小学生",                     // 0xA8
    "趣味・教育／中学生・高校生",                   // 0xA9
    "趣味・教育／大学生・受験",                     // 0xAA
    "趣味・教育／生涯教育・資格",                   // 0xAB
    "趣味・教育／教育問題",                         // 0xAC
    "趣味・教育／？",                               // 0xAD
    "趣味・教育／？",                               // 0xAE
    "趣味・教育／その他",                           // 0xAF
    "福祉／高齢者",                                 // 0xB0
    "福祉／障害者",                                 // 0xB1
    "福祉／社会福祉",                               // 0xB2
    "福祉／ボランティア",                           // 0xB3
    "福祉／手話",                                   // 0xB4
    "福祉／文字（字幕）",                           // 0xB5
    "福祉／音声解説",                               // 0xB6
    "福祉／その他",                                 // 0xB7 あってる？
    "福祉／？",                                     // 0xB8
    "福祉／？",                                     // 0xB9
    "福祉／？",                                     // 0xBA
    "福祉／？",                                     // 0xBB
    "福祉／？",                                     // 0xBC
    "福祉／？",                                     // 0xBD
    "福祉／？",                                     // 0xBE
    "福祉／その他",                                 // 0xBF
    "予備／？",                                     // 0xC0
    "予備／？",                                     // 0xC1
    "予備／？",                                     // 0xC2
    "予備／？",                                     // 0xC3
    "予備／？",                                     // 0xC4
    "予備／？",                                     // 0xC5
    "予備／？",                                     // 0xC6
    "予備／？",                                     // 0xC7
    "予備／？",                                     // 0xC8
    "予備／？",                                     // 0xC9
    "予備／？",                                     // 0xCA
    "予備／？",                                     // 0xCB
    "予備／？",                                     // 0xCC
    "予備／？",                                     // 0xCD
    "予備／？",                                     // 0xCE
    "予備／その他",                                 // 0xCF
    "予備／？",                                     // 0xD0
    "予備／？",                                     // 0xD1
    "予備／？",                                     // 0xD2
    "予備／？",                                     // 0xD3
    "予備／？",                                     // 0xD4
    "予備／？",                                     // 0xD5
    "予備／？",                                     // 0xD6
    "予備／？",                                     // 0xD7
    "予備／？",                                     // 0xD8
    "予備／？",                                     // 0xD9
    "予備／？",                                     // 0xDA
    "予備／？",                                     // 0xDB
    "予備／？",                                     // 0xDC
    "予備／？",                                     // 0xDD
    "予備／？",                                     // 0xDE
    "予備／その他",                                 // 0xDF
    "拡張／BS/地上デジタル放送用番組付属情報",      // 0xE0
    "拡張／広帯域 CS デジタル放送用拡張",           // 0xE1
    "拡張／？",                                     // 0xE2
    "拡張／サーバー型番組付属情報",                 // 0xE3
    "拡張／IP 放送用番組付属情報",                  // 0xE4
    "拡張／？",                                     // 0xE5
    "拡張／？",                                     // 0xE6
    "拡張／？",                                     // 0xE7
    "拡張／？",                                     // 0xE8
    "拡張／？",                                     // 0xE9
    "拡張／？",                                     // 0xEA
    "拡張／？",                                     // 0xEB
    "拡張／？",                                     // 0xEC
    "拡張／？",                                     // 0xED
    "拡張／？",                                     // 0xEE
    "拡張／その他",                                 // 0xEF
    "その他",                                       // 0xF0
    "その他",                                       // 0xF1
    "その他",                                       // 0xF2
    "その他",                                       // 0xF3
    "その他",                                       // 0xF4
    "その他",                                       // 0xF5
    "その他",                                       // 0xF6
    "その他",                                       // 0xF7
    "その他",                                       // 0xF8
    "その他",                                       // 0xF9
    "その他",                                       // 0xFA
    "その他",                                       // 0xFB
    "その他",                                       // 0xFC
    "その他",                                       // 0xFD
    "その他",                                       // 0xFE
    "",                                             // 0xFF
];

pub fn get_subgenre(genre: i32, subgenre: i32) -> &'static str {
    SUB_GENRE[((genre as usize & 0x0f) << 4) | (subgenre as usize & 0x0f)]
}

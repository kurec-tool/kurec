use anyhow::Result;
use chrono::{TimeZone, Utc};
use domain::events::kurec_events::EpgStoredEvent;
use domain::events::mirakc_events::EpgProgramsUpdatedEvent;
use domain::models::epg::{AudioType, KurecProgram, KurecSeriesInfo, VideoType};
// use domain::ports::mirakc_api::MirakcApi; // 未使用なので削除
use domain::ports::notifiers::EpgNotifier;
use domain::ports::repositories::kurec_program_repository::KurecProgramRepository;
use infra_mirakc::factory::MirakcClientFactory;
use mirakc_client; // mirakc_client をインポート
use shared_core::error_handling::{ClassifyError, ErrorAction};
use shared_macros::stream_worker;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument, warn};

// エラー型
#[derive(Debug, thiserror::Error)]
pub enum EpgUpdaterError {
    #[error("Mirakc API error: {0}")]
    MirakcApiError(String),
    #[error("KVS storage error: {0}")]
    KvsStorageError(String),
    #[error("Notification error: {0}")]
    NotificationError(String),
    #[error("Invalid program data: {0}")]
    InvalidProgramData(String),
}

// エラー分類の実装
impl ClassifyError for EpgUpdaterError {
    fn error_action(&self) -> ErrorAction {
        match self {
            // API エラーは再試行する
            EpgUpdaterError::MirakcApiError(_) => ErrorAction::Retry,
            // KVS ストレージエラーは再試行する
            EpgUpdaterError::KvsStorageError(_) => ErrorAction::Retry,
            // 通知エラーは再試行する
            EpgUpdaterError::NotificationError(_) => ErrorAction::Retry,
            // 無効なプログラムデータは無視する
            EpgUpdaterError::InvalidProgramData(_) => ErrorAction::Ignore,
        }
    }
}

// ジャンルコード変換用のマップ
struct GenreMapper {
    lv1_map: HashMap<u32, String>,
    lv2_map: HashMap<(u32, u32), String>,
}

impl GenreMapper {
    fn new() -> Self {
        let mut lv1_map = HashMap::new();
        lv1_map.insert(0x0, "ニュース・報道".to_string());
        lv1_map.insert(0x1, "スポーツ".to_string());
        lv1_map.insert(0x2, "情報・ワイドショー".to_string());
        lv1_map.insert(0x3, "ドラマ".to_string());
        lv1_map.insert(0x4, "音楽".to_string());
        lv1_map.insert(0x5, "バラエティ".to_string());
        lv1_map.insert(0x6, "映画".to_string());
        lv1_map.insert(0x7, "アニメ・特撮".to_string());
        lv1_map.insert(0x8, "ドキュメンタリー・教養".to_string());
        lv1_map.insert(0x9, "劇場・公演".to_string());
        lv1_map.insert(0xa, "趣味・教育".to_string());
        lv1_map.insert(0xb, "福祉".to_string());
        lv1_map.insert(0xf, "その他".to_string());

        let mut lv2_map = HashMap::new();
        // ニュース・報道
        lv2_map.insert((0x0, 0x0), "定時・総合".to_string());
        lv2_map.insert((0x0, 0x1), "天気".to_string());
        lv2_map.insert((0x0, 0x2), "特集・ドキュメント".to_string());
        lv2_map.insert((0x0, 0x3), "政治・国会".to_string());
        lv2_map.insert((0x0, 0x4), "経済・市況".to_string());
        lv2_map.insert((0x0, 0x5), "海外・国際".to_string());
        lv2_map.insert((0x0, 0x6), "解説".to_string());
        lv2_map.insert((0x0, 0x7), "討論・会談".to_string());
        lv2_map.insert((0x0, 0x8), "報道特番".to_string());
        lv2_map.insert((0x0, 0x9), "ローカル・地域".to_string());
        lv2_map.insert((0x0, 0xa), "交通".to_string());
        lv2_map.insert((0x0, 0xf), "その他".to_string());

        // スポーツ
        lv2_map.insert((0x1, 0x0), "スポーツニュース".to_string());
        lv2_map.insert((0x1, 0x1), "野球".to_string());
        lv2_map.insert((0x1, 0x2), "サッカー".to_string());
        lv2_map.insert((0x1, 0x3), "ゴルフ".to_string());
        lv2_map.insert((0x1, 0x4), "その他の球技".to_string());
        lv2_map.insert((0x1, 0x5), "相撲・格闘技".to_string());
        lv2_map.insert((0x1, 0x6), "オリンピック・国際大会".to_string());
        lv2_map.insert((0x1, 0x7), "マラソン・陸上・水泳".to_string());
        lv2_map.insert((0x1, 0x8), "モータースポーツ".to_string());
        lv2_map.insert((0x1, 0x9), "マリン・ウィンタースポーツ".to_string());
        lv2_map.insert((0x1, 0xa), "競馬・公営競技".to_string());
        lv2_map.insert((0x1, 0xf), "その他".to_string());

        // 情報・ワイドショー
        lv2_map.insert((0x2, 0x0), "芸能・ワイドショー".to_string());
        lv2_map.insert((0x2, 0x1), "ファッション".to_string());
        lv2_map.insert((0x2, 0x2), "暮らし・住まい".to_string());
        lv2_map.insert((0x2, 0x3), "健康・医療".to_string());
        lv2_map.insert((0x2, 0x4), "ショッピング・通販".to_string());
        lv2_map.insert((0x2, 0x5), "グルメ・料理".to_string());
        lv2_map.insert((0x2, 0x6), "イベント".to_string());
        lv2_map.insert((0x2, 0x7), "番組紹介・お知らせ".to_string());
        lv2_map.insert((0x2, 0xf), "その他".to_string());

        // ドラマ
        lv2_map.insert((0x3, 0x0), "国内ドラマ".to_string());
        lv2_map.insert((0x3, 0x1), "海外ドラマ".to_string());
        lv2_map.insert((0x3, 0x2), "時代劇".to_string());
        lv2_map.insert((0x3, 0xf), "その他".to_string());

        // 音楽
        lv2_map.insert((0x4, 0x0), "国内ロック・ポップス".to_string());
        lv2_map.insert((0x4, 0x1), "海外ロック・ポップス".to_string());
        lv2_map.insert((0x4, 0x2), "クラシック・オペラ".to_string());
        lv2_map.insert((0x4, 0x3), "ジャズ・フュージョン".to_string());
        lv2_map.insert((0x4, 0x4), "歌謡曲・演歌".to_string());
        lv2_map.insert((0x4, 0x5), "ライブ・コンサート".to_string());
        lv2_map.insert((0x4, 0x6), "ランキング・リクエスト".to_string());
        lv2_map.insert((0x4, 0x7), "カラオケ・のど自慢".to_string());
        lv2_map.insert((0x4, 0x8), "民謡・邦楽".to_string());
        lv2_map.insert((0x4, 0x9), "童謡・キッズ".to_string());
        lv2_map.insert((0x4, 0xa), "民族音楽・ワールドミュージック".to_string());
        lv2_map.insert((0x4, 0xf), "その他".to_string());

        // バラエティ
        lv2_map.insert((0x5, 0x0), "クイズ".to_string());
        lv2_map.insert((0x5, 0x1), "ゲーム".to_string());
        lv2_map.insert((0x5, 0x2), "トークバラエティ".to_string());
        lv2_map.insert((0x5, 0x3), "お笑い・コメディ".to_string());
        lv2_map.insert((0x5, 0x4), "音楽バラエティ".to_string());
        lv2_map.insert((0x5, 0x5), "旅バラエティ".to_string());
        lv2_map.insert((0x5, 0x6), "料理バラエティ".to_string());
        lv2_map.insert((0x5, 0xf), "その他".to_string());

        // 映画
        lv2_map.insert((0x6, 0x0), "洋画".to_string());
        lv2_map.insert((0x6, 0x1), "邦画".to_string());
        lv2_map.insert((0x6, 0x2), "アニメ".to_string());
        lv2_map.insert((0x6, 0xf), "その他".to_string());

        // アニメ・特撮
        lv2_map.insert((0x7, 0x0), "国内アニメ".to_string());
        lv2_map.insert((0x7, 0x1), "海外アニメ".to_string());
        lv2_map.insert((0x7, 0x2), "特撮".to_string());
        lv2_map.insert((0x7, 0xf), "その他".to_string());

        // ドキュメンタリー・教養
        lv2_map.insert((0x8, 0x0), "社会・時事".to_string());
        lv2_map.insert((0x8, 0x1), "歴史・紀行".to_string());
        lv2_map.insert((0x8, 0x2), "自然・動物・環境".to_string());
        lv2_map.insert((0x8, 0x3), "宇宙・科学・医学".to_string());
        lv2_map.insert((0x8, 0x4), "カルチャー・伝統文化".to_string());
        lv2_map.insert((0x8, 0x5), "文学・文芸".to_string());
        lv2_map.insert((0x8, 0x6), "スポーツ".to_string());
        lv2_map.insert((0x8, 0x7), "ドキュメンタリー全般".to_string());
        lv2_map.insert((0x8, 0x8), "インタビュー・討論".to_string());
        lv2_map.insert((0x8, 0xf), "その他".to_string());

        // 劇場・公演
        lv2_map.insert((0x9, 0x0), "現代劇・新劇".to_string());
        lv2_map.insert((0x9, 0x1), "ミュージカル".to_string());
        lv2_map.insert((0x9, 0x2), "ダンス・バレエ".to_string());
        lv2_map.insert((0x9, 0x3), "落語・演芸".to_string());
        lv2_map.insert((0x9, 0x4), "歌舞伎・古典".to_string());
        lv2_map.insert((0x9, 0xf), "その他".to_string());

        // 趣味・教育
        lv2_map.insert((0xa, 0x0), "旅・釣り・アウトドア".to_string());
        lv2_map.insert((0xa, 0x1), "園芸・ペット・手芸".to_string());
        lv2_map.insert((0xa, 0x2), "音楽・美術・工芸".to_string());
        lv2_map.insert((0xa, 0x3), "囲碁・将棋".to_string());
        lv2_map.insert((0xa, 0x4), "麻雀・パチンコ".to_string());
        lv2_map.insert((0xa, 0x5), "車・オートバイ".to_string());
        lv2_map.insert((0xa, 0x6), "コンピュータ・ＴＶゲーム".to_string());
        lv2_map.insert((0xa, 0x7), "会話・語学".to_string());
        lv2_map.insert((0xa, 0x8), "幼児・小学生".to_string());
        lv2_map.insert((0xa, 0x9), "中学生・高校生".to_string());
        lv2_map.insert((0xa, 0xa), "大学生・受験".to_string());
        lv2_map.insert((0xa, 0xb), "生涯教育・資格".to_string());
        lv2_map.insert((0xa, 0xc), "教育問題".to_string());
        lv2_map.insert((0xa, 0xf), "その他".to_string());

        // 福祉
        lv2_map.insert((0xb, 0x0), "高齢者".to_string());
        lv2_map.insert((0xb, 0x1), "障害者".to_string());
        lv2_map.insert((0xb, 0x2), "社会福祉".to_string());
        lv2_map.insert((0xb, 0x3), "ボランティア".to_string());
        lv2_map.insert((0xb, 0x4), "手話".to_string());
        lv2_map.insert((0xb, 0x5), "文字(字幕)".to_string());
        lv2_map.insert((0xb, 0x6), "音声解説".to_string());
        lv2_map.insert((0xb, 0xf), "その他".to_string());

        // その他
        lv2_map.insert((0xf, 0xf), "その他".to_string());

        Self { lv1_map, lv2_map }
    }

    fn get_genre_text(&self, lv1: u32, lv2: u32) -> String {
        let lv1_text = self
            .lv1_map
            .get(&lv1)
            .unwrap_or(&"不明".to_string())
            .clone();
        let lv2_text = self
            .lv2_map
            .get(&(lv1, lv2))
            .unwrap_or(&"不明".to_string())
            .clone();

        if lv2_text.is_empty() || lv2_text == "不明" {
            lv1_text
        } else {
            format!("{}/{}", lv1_text, lv2_text)
        }
    }
}

// ビデオコンポーネントタイプの変換
fn map_video_component_type(component_type: u32) -> VideoType {
    match component_type {
        0x01 => VideoType::SD_4_3,
        0x02 => VideoType::SD_16_9_PanVector,
        0x03 => VideoType::SD_16_9,
        0x04 => VideoType::SD_Over16_9,
        0x83 => VideoType::UHD_16_9,
        0x91 => VideoType::HD_4_3,
        0x92 => VideoType::HD_16_9_PanVector,
        0x93 => VideoType::HD_16_9,
        0x94 => VideoType::HD_Over16_9,
        0xa1 => VideoType::SD_4_3,
        0xa2 => VideoType::SD_16_9_PanVector,
        0xa3 => VideoType::SD_16_9,
        0xa4 => VideoType::SD_Over16_9,
        0xb1 => VideoType::HD_4_3,
        0xb2 => VideoType::HD_16_9_PanVector,
        0xb3 => VideoType::HD_16_9,
        0xb4 => VideoType::HD_Over16_9,
        0xc1 => VideoType::HD_4_3,
        0xc2 => VideoType::HD_16_9_PanVector,
        0xc3 => VideoType::HD_16_9,
        0xc4 => VideoType::HD_Over16_9,
        0xd1 => VideoType::SD_4_3,
        0xd2 => VideoType::SD_16_9_PanVector,
        0xd3 => VideoType::SD_16_9,
        0xd4 => VideoType::SD_Over16_9,
        0xe1 => VideoType::HD_4_3,
        0xe2 => VideoType::HD_16_9_PanVector,
        0xe3 => VideoType::HD_16_9,
        0xe4 => VideoType::HD_Over16_9,
        0xf1 => VideoType::SD_4_3,
        0xf2 => VideoType::SD_16_9_PanVector,
        0xf3 => VideoType::SD_16_9,
        0xf4 => VideoType::SD_Over16_9,
        _ => VideoType::Unknown,
    }
}

// オーディオコンポーネントタイプの変換
fn map_audio_component_type(component_type: u32) -> AudioType {
    match component_type {
        0b00001 => AudioType::Mono,
        0b00010 => AudioType::DualMono,
        0b00011 => AudioType::Stereo,
        0b00100 => AudioType::Mode2_1,
        0b00101 => AudioType::Mode3_0,
        0b00110 => AudioType::Mode2_2,
        0b00111 => AudioType::Mode3_1,
        0b01000 => AudioType::Mode3_2,
        0b01001 => AudioType::Mode3_2_LFE,
        _ => AudioType::Unknown,
    }
}

// オーディオサンプリングレートの変換
fn map_audio_sampling_rate(rate: u32) -> String {
    match rate {
        16000 => "16kHz".to_string(),
        22050 => "22.05kHz".to_string(),
        24000 => "24kHz".to_string(),
        32000 => "32kHz".to_string(),
        44100 => "44.1kHz".to_string(),
        48000 => "48kHz".to_string(),
        _ => format!("{}Hz", rate),
    }
}

// 言語コードの変換
fn map_language_code(code: &str) -> String {
    match code {
        "jpn" => "日本語".to_string(),
        "eng" => "英語".to_string(),
        "zho" => "中国語".to_string(),
        "kor" => "韓国語".to_string(),
        "fra" => "フランス語".to_string(),
        "deu" => "ドイツ語".to_string(),
        "ita" => "イタリア語".to_string(),
        "rus" => "ロシア語".to_string(),
        "spa" => "スペイン語".to_string(),
        "por" => "ポルトガル語".to_string(),
        _ => code.to_string(),
    }
}

// EPG更新ワーカー
pub struct EpgUpdaterWorker {
    program_repository: Arc<dyn KurecProgramRepository>,
    epg_notifier: Arc<dyn EpgNotifier>,
    mirakc_api_factory: Arc<dyn MirakcClientFactory>, // Factory を保持
    genre_mapper: GenreMapper,
}

impl EpgUpdaterWorker {
    pub fn new(
        program_repository: Arc<dyn KurecProgramRepository>,
        epg_notifier: Arc<dyn EpgNotifier>,
        mirakc_api_factory: Arc<dyn MirakcClientFactory>, // Factory を受け取る
    ) -> Self {
        Self {
            program_repository,
            epg_notifier,
            mirakc_api_factory,
            genre_mapper: GenreMapper::new(),
        }
    }

    #[instrument(skip(self, event), fields(service_id = %event.data.service_id, mirakc_url = %event.mirakc_url))]
    pub async fn process_epg_programs_updated(
        &self,
        event: EpgProgramsUpdatedEvent,
    ) -> Result<EpgStoredEvent, EpgUpdaterError> {
        info!(
            "Processing EPG programs updated event for service_id={} from {}",
            event.data.service_id, event.mirakc_url
        );

        // Factory から MirakcApi クライアントを取得
        let client = self.mirakc_api_factory.create_client();

        // サービス情報を取得
        let service_value = client
            .get_service(&event.mirakc_url, event.data.service_id) // キャスト削除
            .await
            .map_err(|e| {
                EpgUpdaterError::MirakcApiError(format!("Failed to get service: {}", e))
            })?;
        // Value から MirakurunService にデシリアライズ
        let service: mirakc_client::models::MirakurunService =
            serde_json::from_value(service_value).map_err(|e| {
                EpgUpdaterError::InvalidProgramData(format!("Failed to deserialize service: {}", e))
            })?;

        // サービスのプログラム一覧を取得
        let programs_value = client
            .get_programs_of_service(&event.mirakc_url, event.data.service_id) // キャスト削除
            .await
            .map_err(|e| {
                EpgUpdaterError::MirakcApiError(format!("Failed to get programs: {}", e))
            })?;
        // Vec<Value> から Vec<MirakurunProgram> にデシリアライズ
        let programs: Vec<mirakc_client::models::MirakurunProgram> = programs_value
            .into_iter()
            .map(|v| serde_json::from_value(v))
            .collect::<Result<_, _>>()
            .map_err(|e| {
                EpgUpdaterError::InvalidProgramData(format!(
                    "Failed to deserialize programs: {}",
                    e
                ))
            })?;

        debug!(
            "Retrieved {} programs for service_id={}",
            programs.len(),
            event.data.service_id
        );

        // プログラムをKurecProgramに変換
        let mut kurec_programs = Vec::new();
        for program in programs {
            // 必須フィールドのチェック
            if program.name.is_none() {
                warn!("Program has no name, skipping: {:?}", program.id);
                continue;
            }

            let mut genres = Vec::new();
            if let Some(Some(genre_list)) = &program.genres {
                for genre in genre_list {
                    let genre_text = self
                        .genre_mapper
                        .get_genre_text(genre.lv1 as u32, genre.lv2 as u32);
                    genres.push(genre_text);
                }
            }

            // ビデオ情報の変換
            let video_type = if let Some(Some(video)) = &program.video {
                map_video_component_type(video.component_type as u32)
            } else {
                VideoType::Unknown
            };

            // ビデオ情報文字列の生成
            let video_info = match video_type {
                VideoType::Unknown => None,
                _ => Some(format!("{:?}", video_type)),
            };

            // オーディオ情報の変換
            let mut audio_types = Vec::new();
            let mut audio_langs = Vec::new();

            if let Some(Some(audio)) = &program.audio {
                let audio_type = map_audio_component_type(audio.component_type as u32);
                audio_types.push(audio_type);

                let _sampling_rate = map_audio_sampling_rate(audio.sampling_rate as u32);

                for lang in &audio.langs {
                    let lang_text = map_language_code(lang);
                    audio_langs.push(lang_text);
                }
            }

            if let Some(audios) = &program.audios {
                for audio in audios {
                    let audio_type = map_audio_component_type(audio.component_type as u32);
                    if !audio_types.contains(&audio_type) {
                        audio_types.push(audio_type);
                    }

                    let _sampling_rate = map_audio_sampling_rate(audio.sampling_rate as u32);

                    for lang in &audio.langs {
                        let lang_text = map_language_code(lang);
                        if !audio_langs.contains(&lang_text) {
                            audio_langs.push(lang_text);
                        }
                    }
                }
            }

            // シリーズ情報の変換
            let series_info = if let Some(Some(series)) = &program.series {
                let expire_at = if series.expire_at > 0 {
                    Some(Utc.timestamp_millis_opt(series.expire_at).unwrap())
                } else {
                    None
                };

                Some(KurecSeriesInfo {
                    id: series.id as i64,           // i32 -> i64
                    repeat: series.repeat as i64,   // i32 -> i64
                    pattern: series.pattern as i64, // i32 -> i64
                    expire_at,
                    episode: series.episode as i64, // i32 -> i64
                    last_episode: series.last_episode as i64, // i32 -> i64
                    name: series.name.clone(),
                })
            } else {
                None
            };

            // KurecProgramの作成
            let kurec_program = KurecProgram {
                id: program.id,
                mirakc_url: event.mirakc_url.clone(),
                service_id: program.service_id as i64, // i32 -> i64
                network_id: program.network_id as i64, // i32 -> i64
                event_id: program.event_id as i64,     // i32 -> i64
                channel_name: service.name.clone(),
                channel_type: service.channel.r#type.to_string(),
                channel: service.channel.channel.clone(),
                name: program.name.and_then(|n| n),
                description: program.description.and_then(|d| d),
                extended: program.extended,
                start_at: Utc.timestamp_millis_opt(program.start_at).unwrap(),
                duration_millis: program.duration,
                is_free: program.is_free,
                genres,
                video_info,
                audio_infos: audio_langs,
                series_info,
            };

            kurec_programs.push(kurec_program);
        }

        let program_count = kurec_programs.len();

        info!(
            "Converted {} programs for service_id={}",
            program_count, event.data.service_id
        );

        // KVSに保存
        self.program_repository
            .save_service_programs(
                &event.mirakc_url,
                event.data.service_id, // キャスト不要
                kurec_programs,
            )
            .await
            .map_err(|e| {
                EpgUpdaterError::KvsStorageError(format!("Failed to save programs: {}", e))
            })?;

        info!(
            "Saved {} programs for service_id={} to KVS",
            program_count, event.data.service_id
        );

        // 通知イベントの作成
        let stored_event = EpgStoredEvent {
            mirakc_url: event.mirakc_url.clone(),
            service_id: event.data.service_id, // キャスト不要
        };

        // 通知
        self.epg_notifier
            .notify_epg_stored(stored_event.clone())
            .await
            .map_err(|e| {
                EpgUpdaterError::NotificationError(format!("Failed to notify EPG stored: {}", e))
            })?;

        info!(
            "Notified EPG stored event for service_id={}",
            event.data.service_id
        );

        Ok(stored_event)
    }
}

// ストリームワーカーの実装
#[stream_worker]
pub async fn process_epg_programs_updated_event(
    _event: EpgProgramsUpdatedEvent,
) -> Result<EpgStoredEvent, EpgUpdaterError> {
    // ワーカーの実装
    // 実際のワーカーはDIコンテナから取得する
    // ここではコンパイルを通すための仮実装
    Err(EpgUpdaterError::MirakcApiError(
        "Worker implementation will be injected at runtime".to_string(),
    ))
}

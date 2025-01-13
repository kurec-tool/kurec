/*
 Generated by typeshare 1.13.2
*/


export interface MeilisearchRule {
	query: string;
	filter: string;
}

export interface MeilisearchRules {
	rules: MeilisearchRule[];
}

export interface ProgramDocument {
	タイトル: string;
	番組情報: string;
	その他情報: string;
	放送局: string;
	ジャンル: string[];
	開始時刻: string;
	終了時刻: string;
	放送曜日: string;
	放送時間: number;
	公式サイト等: string[];
	ogp_url?: string;
	ogp_url_hash?: string;
	program_id: number;
	service_id: number;
}


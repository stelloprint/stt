import { invoke } from "@tauri-apps/api/core";

export type SessionMode = "hold" | "toggle" | "record";

export interface Session {
	app_name: string | null;
	chars_count: number;
	ended_at: number | null;
	id: string;
	language: string | null;
	mode: SessionMode;
	model_profile: string;
	started_at: number;
	translated: boolean;
	words_count: number;
}

export interface SessionCreate {
	app_name: string | null;
	id: string;
	language: string | null;
	mode: SessionMode;
	model_profile: string;
	started_at: number;
	translated: boolean;
}

export interface Entry {
	ended_at: number;
	id: string;
	session_id: string;
	source: SessionMode;
	started_at: number;
	text: string;
	typed: boolean;
}

export interface EntryCreate {
	ended_at: number;
	id: string;
	session_id: string;
	source: SessionMode;
	started_at: number;
	text: string;
	typed: boolean;
}

export type ActivationMode = "hold" | "toggle";
export type SilenceRms = "low" | "medium" | "high";
export type ModelProfile =
	| "small.en"
	| "multilingual-small"
	| "multilingual-medium";

export interface Hotkeys {
	left_chord: boolean;
	right_chord: boolean;
}

export interface TypingPrefs {
	newline_at_end: boolean;
	throttle_ms: number;
}

export interface VoiceCommandMap {
	backtick: string;
	close_quote: string;
	code_block: string;
	colon: string;
	comma: string;
	new_paragraph: string;
	newline: string;
	open_quote: string;
	period: string;
	semicolon: string;
	tab: string;
}

export interface VoiceCommands {
	enabled: boolean;
	map: VoiceCommandMap;
}

export interface RecordPrefs {
	chunk_seconds: number;
	max_file_gb: number;
	max_hours: number;
}

export interface Preferences {
	hotkeys: Hotkeys;
	mode: ActivationMode;
	model_profile: ModelProfile;
	record: RecordPrefs;
	silence_rms: SilenceRms;
	silence_seconds: number;
	translate_to_english: boolean;
	typing: TypingPrefs;
	voice_commands: VoiceCommands;
}

export const api = {
	preferences: {
		get: () => invoke<Preferences>("get_preferences"),
		update: (prefs: Preferences) =>
			invoke<void>("update_preferences", { prefs }),
	},

	dirs: {
		config: () => invoke<string>("get_config_dir"),
		data: () => invoke<string>("get_data_dir"),
		models: () => invoke<string>("get_models_dir"),
	},

	sessions: {
		create: (session: SessionCreate) =>
			invoke<Session>("create_session", { session }),
		get: (id: string) => invoke<Session | null>("get_session", { id }),
		getAll: () => invoke<Session[]>("get_all_sessions"),
		update: (
			id: string,
			endedAt?: number,
			charsCount?: number,
			wordsCount?: number
		) =>
			invoke<Session | null>("update_session", {
				id,
				ended_at: endedAt,
				chars_count: charsCount,
				words_count: wordsCount,
			}),
		delete: (id: string) => invoke<boolean>("delete_session", { id }),
		start: (mode: string) => invoke<Session>("start_session", { mode }),
		end: () => invoke<Session | null>("end_session"),
	},

	entries: {
		create: (entry: EntryCreate) => invoke<Entry>("create_entry", { entry }),
		get: (id: string) => invoke<Entry | null>("get_entry", { id }),
		getBySession: (sessionId: string) =>
			invoke<Entry[]>("get_entries_by_session", { session_id: sessionId }),
		getAll: () => invoke<Entry[]>("get_all_entries"),
		update: (id: string, text?: string, typed?: boolean) =>
			invoke<Entry | null>("update_entry", { id, text, typed }),
		delete: (id: string) => invoke<boolean>("delete_entry", { id }),
		search: (query: string) => invoke<Entry[]>("search_entries", { query }),
		addTyped: (text: string) => invoke<Entry>("add_typed_entry", { text }),
		addUntyped: (text: string) => invoke<Entry>("add_untyped_entry", { text }),
	},
};

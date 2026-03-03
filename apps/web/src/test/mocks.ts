import { vi } from "vitest";
import type {
	ActivationMode,
	Entry,
	ModelProfile,
	Preferences,
	Session,
	SilenceRms,
} from "@/lib/api";

export const mockPreferences: Preferences = {
	mode: "hold" as ActivationMode,
	silence_seconds: 3.0,
	silence_rms: "medium" as SilenceRms,
	model_profile: "small.en" as ModelProfile,
	translate_to_english: false,
	hotkeys: {
		left_chord: true,
		right_chord: true,
	},
	typing: {
		newline_at_end: true,
		throttle_ms: 0,
	},
	voice_commands: {
		enabled: true,
		map: {
			newline: "Enter",
			new_paragraph: "Enter Enter",
			tab: "Tab",
			period: ".",
			comma: ",",
			colon: ":",
			semicolon: ";",
			open_quote: '"',
			close_quote: '"',
			backtick: "`",
			code_block: "```",
		},
	},
	record: {
		chunk_seconds: 60,
		max_hours: 8,
		max_file_gb: 4,
	},
};

export const mockSession: Session = {
	id: "test-session-123",
	mode: "hold",
	started_at: Date.now() - 60_000,
	ended_at: Date.now(),
	words_count: 10,
	chars_count: 50,
	language: "en",
	model_profile: "small.en",
	translated: false,
	app_name: "Terminal",
};

export const mockEntry: Entry = {
	id: "test-entry-456",
	session_id: "test-session-123",
	source: "hold",
	started_at: Date.now() - 60_000,
	ended_at: Date.now() - 55_000,
	text: "Hello world",
	typed: true,
};

export const createMockApi = () => ({
	preferences: {
		get: vi.fn().mockResolvedValue(mockPreferences),
		update: vi.fn().mockResolvedValue(undefined),
	},
	sessions: {
		getAll: vi.fn().mockResolvedValue([mockSession]),
		create: vi.fn().mockResolvedValue(mockSession),
		update: vi.fn().mockResolvedValue(mockSession),
		get: vi.fn().mockResolvedValue(mockSession),
		delete: vi.fn().mockResolvedValue(true),
	},
	entries: {
		getAll: vi.fn().mockResolvedValue([mockEntry]),
		getBySession: vi.fn().mockResolvedValue([mockEntry]),
		create: vi.fn().mockResolvedValue(mockEntry),
		update: vi.fn().mockResolvedValue(mockEntry),
		get: vi.fn().mockResolvedValue(mockEntry),
		delete: vi.fn().mockResolvedValue(true),
		search: vi.fn().mockResolvedValue([mockEntry]),
	},
	dirs: {
		config: vi.fn().mockResolvedValue("/tmp/config"),
		data: vi.fn().mockResolvedValue("/tmp/data"),
		models: vi.fn().mockResolvedValue("/tmp/models"),
	},
});

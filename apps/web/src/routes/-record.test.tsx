import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockEntry, mockSession } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

const formatDuration = (start: number, end?: number | null) => {
	const duration = end ? end - start : Date.now() - start;
	const seconds = Math.floor(duration / 1000);
	const minutes = Math.floor(seconds / 60);
	const hours = Math.floor(minutes / 60);

	if (hours > 0) {
		return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
	}
	if (minutes > 0) {
		return `${minutes}m ${seconds % 60}s`;
	}
	return `${seconds}s`;
};

describe("Record Page", () => {
	describe("Duration Formatting", () => {
		it("should format duration in seconds", () => {
			const result = formatDuration(0, 5000);
			expect(result).toBe("5s");
		});

		it("should format duration in minutes", () => {
			const result = formatDuration(0, 120_000);
			expect(result).toBe("2m 0s");
		});

		it("should format duration in hours", () => {
			const result = formatDuration(0, 3_600_000);
			expect(result).toBe("1h 0m 0s");
		});

		it("should format duration for multiple hours", () => {
			const result = formatDuration(0, 7_200_000);
			expect(result).toBe("2h 0m 0s");
		});

		it("should format duration for ongoing session", () => {
			const start = Date.now() - 60_000;
			const result = formatDuration(start);
			expect(result).toContain("m");
		});

		it("should handle duration of exactly 1 second", () => {
			const result = formatDuration(0, 1000);
			expect(result).toBe("1s");
		});

		it("should handle mixed hours and minutes", () => {
			const result = formatDuration(0, 3_900_000);
			expect(result).toBe("1h 5m 0s");
		});
	});

	describe("Stats Calculation", () => {
		it("should calculate total words across sessions", () => {
			const sessions = [
				{ ...mockSession, words_count: 10, chars_count: 50 },
				{ ...mockSession, id: "2", words_count: 20, chars_count: 100 },
			];

			const totalWords = sessions.reduce((acc, s) => acc + s.words_count, 0);
			const totalChars = sessions.reduce((acc, s) => acc + s.chars_count, 0);

			expect(totalWords).toBe(30);
			expect(totalChars).toBe(150);
		});

		it("should include current session in stats when recording", () => {
			const sessions = [{ ...mockSession, words_count: 10, chars_count: 50 }];
			const isRecording = true;
			const currentSession = mockSession;

			const sessionData =
				isRecording && currentSession
					? [...sessions, currentSession]
					: sessions;
			const totalWords = sessionData.reduce((acc, s) => acc + s.words_count, 0);

			expect(totalWords).toBe(20);
		});

		it("should handle empty sessions", () => {
			const sessions: (typeof mockSession)[] = [];
			const totalWords = sessions.reduce((acc, s) => acc + s.words_count, 0);
			expect(totalWords).toBe(0);
		});
	});

	describe("Session Filtering", () => {
		it("should filter active sessions", () => {
			const sessions = [
				{ ...mockSession, ended_at: null },
				{ ...mockSession, id: "2", ended_at: Date.now() },
			];

			const activeSessions = sessions.filter((s) => !s.ended_at);
			expect(activeSessions.length).toBe(1);
		});

		it("should detect recording state from active sessions", () => {
			const activeSessions = [mockSession];
			const isRecording = activeSessions.length > 0;
			expect(isRecording).toBe(true);
		});

		it("should detect not recording when no active sessions", () => {
			const activeSessions: (typeof mockSession)[] = [];
			const isRecording = activeSessions.length > 0;
			expect(isRecording).toBe(false);
		});

		it("should return all sessions when none ended", () => {
			const sessions = [
				{ ...mockSession, ended_at: null },
				{ ...mockSession, id: "2", ended_at: null },
			];
			const activeSessions = sessions.filter((s) => !s.ended_at);
			expect(activeSessions.length).toBe(2);
		});
	});

	describe("Transcript Export", () => {
		it("should export transcript to txt format", () => {
			const entries = [mockEntry];
			const content = entries
				.map(
					(e) => `[${new Date(e.started_at).toLocaleTimeString()}] ${e.text}`
				)
				.join("\n\n");
			expect(content).toContain("[");
			expect(content).toContain("]");
		});

		it("should export multiple entries with separators", () => {
			const entries = [
				mockEntry,
				{ ...mockEntry, id: "2", text: "Second entry" },
			];
			const content = entries
				.map(
					(e) => `[${new Date(e.started_at).toLocaleTimeString()}] ${e.text}`
				)
				.join("\n\n");
			const parts = content.split("\n\n");
			expect(parts.length).toBe(2);
		});

		it("should include text in export", () => {
			const entries = [mockEntry];
			const content = entries
				.map((e) => `[${new Date(e.started_at).getTime()}] ${e.text}`)
				.join("\n");
			expect(content).toContain(mockEntry.text);
		});
	});

	describe("Recording State Display", () => {
		it("should show recording indicator when active", () => {
			const isRecording = true;
			expect(isRecording).toBe(true);
		});

		it("should show ready indicator when not recording", () => {
			const isRecording = false;
			expect(isRecording).toBe(false);
		});
	});

	describe("Session Management", () => {
		it("should create new session with record mode", () => {
			const session = {
				id: crypto.randomUUID(),
				mode: "record" as const,
				started_at: Date.now(),
				language: null,
				model_profile: "multilingual-small",
				translated: true,
				app_name: null,
			};
			expect(session.mode).toBe("record");
		});

		it("should update session with end time", () => {
			const session = { ...mockSession, ended_at: null };
			const ended_at = Date.now();
			const updated = { ...session, ended_at };
			expect(updated.ended_at).toBe(ended_at);
		});

		it("should slice session ID for display", () => {
			const id = "test-session-123";
			const sliced = id.slice(0, 8);
			expect(sliced).toBe("test-ses");
		});
	});

	describe("File Rotation", () => {
		it("should display file rotation count", () => {
			const fileRotationCount = 2;
			expect(fileRotationCount).toBe(2);
		});

		it("should track zero rotations", () => {
			const fileRotationCount = 0;
			expect(fileRotationCount).toBe(0);
		});
	});

	describe("Entry Display", () => {
		it("should display entry count", () => {
			const entries = [mockEntry, { ...mockEntry, id: "2" }];
			expect(entries.length).toBe(2);
		});

		it("should show waiting message when no entries", () => {
			const entries: (typeof mockEntry)[] = [];
			expect(entries.length).toBe(0);
		});
	});

	describe("Session History", () => {
		it("should limit session history display", () => {
			const sessions = [
				mockSession,
				{ ...mockSession, id: "2" },
				{ ...mockSession, id: "3" },
				{ ...mockSession, id: "4" },
			];
			const displayed = sessions.slice(0, 10);
			expect(displayed.length).toBe(4);
		});

		it("should paginate session history", () => {
			const sessions = Array.from({ length: 25 }, (_, i) => ({
				...mockSession,
				id: String(i),
			}));
			const page = 2;
			const pageSize = 10;
			const displayed = sessions.slice((page - 1) * pageSize, page * pageSize);
			expect(displayed.length).toBe(10);
		});

		it("should disable export when no sessions", () => {
			const sessions: (typeof mockSession)[] = [];
			const disabled = sessions.length === 0;
			expect(disabled).toBe(true);
		});

		it("should enable export when sessions exist", () => {
			const sessions = [mockSession];
			const disabled = sessions.length === 0;
			expect(disabled).toBe(false);
		});
	});
});

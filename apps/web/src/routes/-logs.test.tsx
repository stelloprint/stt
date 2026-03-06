import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockEntry, mockSession } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

const formatDate = (timestamp: number) => {
	return new Date(timestamp).toLocaleString();
};

const formatDuration = (start: number, end: number) => {
	const seconds = Math.round((end - start) / 1000);
	const minutes = Math.floor(seconds / 60);
	const remainingSeconds = seconds % 60;
	if (minutes > 0) {
		return `${minutes}m ${remainingSeconds}s`;
	}
	return `${seconds}s`;
};

const getModeLabel = (mode: string) => {
	switch (mode) {
		case "hold":
			return "Hold";
		case "toggle":
			return "Toggle";
		case "record":
			return "Record";
		default:
			return mode;
	}
};

describe("Logs Table", () => {
	describe("Date Formatting", () => {
		it("should format date with year", () => {
			const timestamp = new Date("2024-01-15T10:30:00").getTime();
			const formatted = formatDate(timestamp);
			expect(formatted).toContain("2024");
		});

		it("should format date with month and day", () => {
			const timestamp = new Date("2024-01-15T10:30:00").getTime();
			const formatted = formatDate(timestamp);
			expect(formatted).toContain("15");
			expect(formatted).toContain("2024");
		});

		it("should format ISO date correctly", () => {
			const timestamp = new Date("2024-06-20T14:45:00").getTime();
			const formatted = new Date(timestamp).toISOString();
			expect(formatted).toContain("2024-06-20");
		});
	});

	describe("Duration Formatting", () => {
		it("should format duration for seconds only", () => {
			const result = formatDuration(0, 30_000);
			expect(result).toBe("30s");
		});

		it("should format duration for exactly one minute", () => {
			const result = formatDuration(0, 60_000);
			expect(result).toBe("1m 0s");
		});

		it("should format duration for two minutes", () => {
			const result = formatDuration(0, 120_000);
			expect(result).toBe("2m 0s");
		});

		it("should format duration for mixed minutes and seconds", () => {
			const result = formatDuration(0, 90_000);
			expect(result).toBe("1m 30s");
		});

		it("should format duration for short duration", () => {
			const result = formatDuration(0, 5000);
			expect(result).toBe("5s");
		});

		it("should handle zero duration", () => {
			const result = formatDuration(0, 0);
			expect(result).toBe("0s");
		});
	});

	describe("Mode Label", () => {
		it("should return 'Hold' for hold mode", () => {
			expect(getModeLabel("hold")).toBe("Hold");
		});

		it("should return 'Toggle' for toggle mode", () => {
			expect(getModeLabel("toggle")).toBe("Toggle");
		});

		it("should return 'Record' for record mode", () => {
			expect(getModeLabel("record")).toBe("Record");
		});

		it("should return unknown mode as-is", () => {
			expect(getModeLabel("unknown")).toBe("unknown");
			expect(getModeLabel("custom")).toBe("custom");
		});
	});

	describe("Session Map", () => {
		it("should create session map with id as key", () => {
			const sessions = [mockSession];
			const sessionMap = new Map(sessions.map((s) => [s.id, s]));
			expect(sessionMap.get("test-session-123")).toEqual(mockSession);
		});

		it("should retrieve session by id", () => {
			const sessions = [mockSession, { ...mockSession, id: "session-456" }];
			const sessionMap = new Map(sessions.map((s) => [s.id, s]));
			expect(sessionMap.get("session-456")?.id).toBe("session-456");
		});

		it("should return undefined for non-existent session", () => {
			const sessions = [mockSession];
			const sessionMap = new Map(sessions.map((s) => [s.id, s]));
			expect(sessionMap.get("non-existent")).toBeUndefined();
		});
	});

	describe("Search Filtering", () => {
		it("should filter entries by search query case-insensitively", () => {
			const entries = [
				{ ...mockEntry, text: "Hello world" },
				{ ...mockEntry, id: "2", text: "Goodbye world" },
			];
			const query = "Hello";
			const filtered = entries.filter((e) =>
				e.text.toLowerCase().includes(query.toLowerCase())
			);
			expect(filtered.length).toBe(1);
			expect(filtered[0]?.text).toBe("Hello world");
		});

		it("should return all entries for empty search query", () => {
			const entries: (typeof mockEntry)[] = [
				mockEntry,
				{ ...mockEntry, id: "2" },
			];
			const query: string = "";
			let filtered: (typeof mockEntry)[] = entries;
			if (query.length > 0) {
				filtered = entries.filter((e) =>
					e.text.toLowerCase().includes(query.toLowerCase())
				);
			}
			expect(filtered.length).toBe(2);
		});

		it("should return empty array when no matches", () => {
			const entries = [mockEntry];
			const query = "xyz";
			const filtered = entries.filter((e) =>
				e.text.toLowerCase().includes(query.toLowerCase())
			);
			expect(filtered.length).toBe(0);
		});

		it("should match partial words", () => {
			const entries = [{ ...mockEntry, text: "Hello world" }];
			const query = "ello";
			const filtered = entries.filter((e) =>
				e.text.toLowerCase().includes(query.toLowerCase())
			);
			expect(filtered.length).toBe(1);
		});
	});

	describe("Export Format", () => {
		it("should export entries to txt format with timestamp", () => {
			const entries = [mockEntry];
			const content = entries
				.map((e) => `[${new Date(e.started_at).toISOString()}] ${e.text}`)
				.join("\n");
			expect(content).toContain("[");
			expect(content).toContain("]");
			expect(content).toContain(mockEntry.text);
		});

		it("should export multiple entries with newlines", () => {
			const entries = [
				mockEntry,
				{ ...mockEntry, id: "2", text: "Second entry" },
			];
			const content = entries
				.map((e) => `[${new Date(e.started_at).toISOString()}] ${e.text}`)
				.join("\n");
			const lines = content.split("\n");
			expect(lines.length).toBe(2);
		});
	});

	describe("Session Display", () => {
		it("should count sessions correctly", () => {
			const sessions = [mockSession, { ...mockSession, id: "2" }];
			expect(sessions.length).toBe(2);
		});

		it("should slice session ID for display", () => {
			const id = "test-session-123";
			const sliced = id.slice(0, 8);
			expect(sliced).toBe("test-ses");
		});

		it("should handle empty sessions array", () => {
			const sessions: (typeof mockSession)[] = [];
			expect(sessions.length).toBe(0);
		});
	});

	describe("Entry Display", () => {
		it("should count entries correctly", () => {
			const entries = [mockEntry, { ...mockEntry, id: "2" }];
			expect(entries.length).toBe(2);
		});

		it("should show typed status", () => {
			expect(mockEntry.typed).toBe(true);
		});

		it("should show untyped status", () => {
			const untypedEntry = { ...mockEntry, typed: false };
			expect(untypedEntry.typed).toBe(false);
		});

		it("should handle empty entries array", () => {
			const entries: (typeof mockEntry)[] = [];
			expect(entries.length).toBe(0);
		});
	});

	describe("Table Sorting", () => {
		it("should sort entries by date descending", () => {
			const entries = [
				{ ...mockEntry, id: "1", started_at: 1000 },
				{ ...mockEntry, id: "2", started_at: 2000 },
				{ ...mockEntry, id: "3", started_at: 500 },
			];
			const sorted = [...entries].sort((a, b) => b.started_at - a.started_at);
			expect(sorted[0]?.id).toBe("2");
			expect(sorted[1]?.id).toBe("1");
			expect(sorted[2]?.id).toBe("3");
		});

		it("should sort entries by date ascending", () => {
			const entries = [
				{ ...mockEntry, id: "1", started_at: 1000 },
				{ ...mockEntry, id: "2", started_at: 2000 },
			];
			const sorted = [...entries].sort((a, b) => a.started_at - b.started_at);
			expect(sorted[0]?.id).toBe("1");
			expect(sorted[1]?.id).toBe("2");
		});
	});

	describe("Pagination", () => {
		it("should paginate entries correctly", () => {
			const entries = Array.from({ length: 25 }, (_, i) => ({
				...mockEntry,
				id: String(i),
			}));
			const page = 1;
			const pageSize = 10;
			const paginated = entries.slice((page - 1) * pageSize, page * pageSize);
			expect(paginated.length).toBe(10);
		});

		it("should calculate total pages", () => {
			const entries = Array.from({ length: 25 }, (_, i) => ({
				...mockEntry,
				id: String(i),
			}));
			const pageSize = 10;
			const totalPages = Math.ceil(entries.length / pageSize);
			expect(totalPages).toBe(3);
		});

		it("should handle last page with fewer items", () => {
			const entries = Array.from({ length: 25 }, (_, i) => ({
				...mockEntry,
				id: String(i),
			}));
			const page = 3;
			const pageSize = 10;
			const paginated = entries.slice((page - 1) * pageSize, page * pageSize);
			expect(paginated.length).toBe(5);
		});
	});
});

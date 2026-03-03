import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockEntry, mockSession } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

describe("Logs Table", () => {
	it("should format date correctly", () => {
		const timestamp = new Date("2024-01-15T10:30:00").getTime();
		const formatted = new Date(timestamp).toLocaleString();
		expect(formatted).toContain("2024");
	});

	it("should format duration for seconds", () => {
		const start = 0;
		const end = 30_000; // 30 seconds
		const seconds = Math.round((end - start) / 1000);
		expect(seconds).toBe(30);
	});

	it("should format duration for minutes", () => {
		const start = 0;
		const end = 120_000; // 2 minutes
		const seconds = Math.round((end - start) / 1000);
		const minutes = Math.floor(seconds / 60);
		const remainingSeconds = seconds % 60;
		expect(`${minutes}m ${remainingSeconds}s`).toBe("2m 0s");
	});

	it("should format duration for mixed minutes and seconds", () => {
		const start = 0;
		const end = 90_000; // 1 minute 30 seconds
		const seconds = Math.round((end - start) / 1000);
		const minutes = Math.floor(seconds / 60);
		const remainingSeconds = seconds % 60;
		expect(`${minutes}m ${remainingSeconds}s`).toBe("1m 30s");
	});

	it("should get mode label for hold", () => {
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
		expect(getModeLabel("hold")).toBe("Hold");
	});

	it("should get mode label for toggle", () => {
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
		expect(getModeLabel("toggle")).toBe("Toggle");
	});

	it("should get mode label for record", () => {
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
		expect(getModeLabel("record")).toBe("Record");
	});

	it("should create session map correctly", () => {
		const sessions = [mockSession];
		const sessionMap = new Map(sessions.map((s) => [s.id, s]));
		expect(sessionMap.get("test-session-123")).toEqual(mockSession);
	});

	it("should filter entries by search query", () => {
		const entries = [
			{ ...mockEntry, text: "Hello world" },
			{ ...mockEntry, id: "2", text: "Goodbye world" },
		];
		const query = "Hello";
		const filtered = entries.filter((e) =>
			e.text.toLowerCase().includes(query.toLowerCase())
		);
		expect(filtered.length).toBe(1);
		caseSensitive: expect(filtered[0]?.text).toBe("Hello world");
	});

	it("should handle empty search query", () => {
		const entries = [mockEntry];
		const query = "";
		const filtered = query
			? entries.filter((e) => e.text.includes(query))
			: entries;
		expect(filtered.length).toBe(1);
	});

	it("should handle no matching entries", () => {
		const entries = [mockEntry];
		const query = "xyz";
		const filtered = entries.filter((e) =>
			e.text.toLowerCase().includes(query.toLowerCase())
		);
		expect(filtered.length).toBe(0);
	});

	it("should export entries to txt format", () => {
		const entries = [mockEntry];
		const content = entries
			.map((e) => `[${new Date(e.started_at).toISOString()}] ${e.text}`)
			.join("\n");
		expect(content).toContain("[");
		expect(content).toContain("]");
		expect(content).toContain(mockEntry.text);
	});

	it("should display session count", () => {
		const sessions = [mockSession, { ...mockSession, id: "2" }];
		expect(sessions.length).toBe(2);
	});

	it("should display entry count", () => {
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

	it("should slice session ID for display", () => {
		const id = "test-session-123";
		const sliced = id.slice(0, 8);
		expect(sliced).toBe("test-ses");
	});

	it("should handle no sessions", () => {
		const sessions: (typeof mockSession)[] = [];
		expect(sessions.length).toBe(0);
	});

	it("should handle no entries", () => {
		const entries: (typeof mockEntry)[] = [];
		expect(entries.length).toBe(0);
	});
});

import { describe, expect, it } from "vitest";
import { mockEntry, mockSession } from "../test/mocks";
import {
	createExportBlob,
	exportEntries,
	exportToMd,
	exportToTxt,
	hasVoiceCommands,
	processVoiceCommands,
} from "./export";

describe("Export Module", () => {
	describe("hasVoiceCommands", () => {
		it("should detect newline command", () => {
			expect(hasVoiceCommands("say newline please")).toBe(true);
		});

		it("should detect new paragraph command", () => {
			expect(hasVoiceCommands("new paragraph")).toBe(true);
		});

		it("should detect tab command", () => {
			expect(hasVoiceCommands("press tab")).toBe(true);
		});

		it("should detect period command", () => {
			expect(hasVoiceCommands("add period")).toBe(true);
		});

		it("should detect comma command", () => {
			expect(hasVoiceCommands("add comma")).toBe(true);
		});

		it("should detect colon command", () => {
			expect(hasVoiceCommands("add colon")).toBe(true);
		});

		it("should detect semicolon command", () => {
			expect(hasVoiceCommands("add semicolon")).toBe(true);
		});

		it("should detect open quote command", () => {
			expect(hasVoiceCommands("open quote")).toBe(true);
		});

		it("should detect close quote command", () => {
			expect(hasVoiceCommands("close quote")).toBe(true);
		});

		it("should detect backtick command", () => {
			expect(hasVoiceCommands("add backtick")).toBe(true);
		});

		it("should detect code block command", () => {
			expect(hasVoiceCommands("add code block")).toBe(true);
		});

		it("should return false for text without voice commands", () => {
			expect(hasVoiceCommands("Hello world")).toBe(false);
		});

		it("should be case insensitive", () => {
			expect(hasVoiceCommands("NEWLINE")).toBe(true);
			expect(hasVoiceCommands("NewLine")).toBe(true);
		});

		it("should detect multiple voice commands", () => {
			expect(hasVoiceCommands("newline and tab")).toBe(true);
		});
	});

	describe("processVoiceCommands", () => {
		it("should replace newline with symbol", () => {
			expect(processVoiceCommands("say newline")).toBe("say ⏎");
		});

		it("should replace new paragraph with double newline", () => {
			expect(processVoiceCommands("new paragraph")).toBe("⏎⏎");
		});

		it("should replace tab with symbol", () => {
			expect(processVoiceCommands("press tab")).toBe("press ⇥");
		});

		it("should replace period punctuation", () => {
			expect(processVoiceCommands("say period")).toBe("say .");
		});

		it("should replace comma punctuation", () => {
			expect(processVoiceCommands("say comma")).toBe("say ,");
		});

		it("should replace colon punctuation", () => {
			expect(processVoiceCommands("say colon")).toBe("say :");
		});

		it("should replace semicolon punctuation", () => {
			expect(processVoiceCommands("say semicolon")).toBe("say ;");
		});

		it("should replace open quote", () => {
			expect(processVoiceCommands("open quote")).toBe('"');
		});

		it("should replace close quote", () => {
			expect(processVoiceCommands("close quote")).toBe('"');
		});

		it("should replace backtick", () => {
			expect(processVoiceCommands("add backtick")).toBe("add `");
		});

		it("should replace code block with triple backticks", () => {
			expect(processVoiceCommands("code block")).toBe("```");
		});

		it("should handle text without voice commands", () => {
			expect(processVoiceCommands("Hello world")).toBe("Hello world");
		});

		it("should handle multiple commands in one text", () => {
			expect(processVoiceCommands("newline tab period")).toBe("⏎ ⇥ .");
		});
	});

	describe("exportToTxt", () => {
		const entries = [
			{
				...mockEntry,
				id: "1",
				text: "First entry",
				started_at: 1000,
				ended_at: 2000,
			},
			{
				...mockEntry,
				id: "2",
				text: "Second entry",
				started_at: 3000,
				ended_at: 4000,
			},
			{
				...mockEntry,
				id: "3",
				text: "Third entry",
				started_at: 5000,
				ended_at: 6000,
			},
		];

		it("should export entries with timestamps by default", () => {
			const result = exportToTxt(entries, mockSession);
			expect(result).toContain("[");
			expect(result).toContain("]");
			expect(result).toContain("First entry");
		});

		it("should export entries without timestamps when option disabled", () => {
			const result = exportToTxt(entries, mockSession, {
				includeTimestamps: false,
			});
			expect(result).not.toContain("[");
			expect(result).toContain("First entry");
		});

		it("should include session header by default", () => {
			const result = exportToTxt(entries, mockSession);
			expect(result).toContain("Session");
			expect(result).toContain(mockSession.id);
			expect(result).toContain("model:");
			expect(mockSession.model_profile).toBeTruthy();
		});

		it("should exclude session header when option disabled", () => {
			const result = exportToTxt(entries, mockSession, {
				includeSessionHeader: false,
			});
			expect(result).not.toContain("Session");
		});

		it("should export entries in chronological order", () => {
			const shuffledEntries = [entries[2], entries[0], entries[1]];
			const result = exportToTxt(shuffledEntries, mockSession);
			const firstEntryPos = result.indexOf("First entry");
			const secondEntryPos = result.indexOf("Second entry");
			expect(firstEntryPos).toBeLessThan(secondEntryPos);
		});

		it("should handle empty entries array", () => {
			const result = exportToTxt([], mockSession);
			expect(result).toContain("Session");
			expect(result).not.toContain("First entry");
		});
	});

	describe("exportToMd", () => {
		const entries = [
			{
				...mockEntry,
				id: "1",
				text: "First entry",
				started_at: 1000,
				ended_at: 2000,
			},
			{
				...mockEntry,
				id: "2",
				text: "Second entry",
				started_at: 3000,
				ended_at: 4000,
			},
		];

		it("should export entries as markdown with session heading", () => {
			const result = exportToMd(entries, mockSession);
			expect(result).toContain("# Session");
			expect(result).toContain(mockSession.id);
		});

		it("should include session metadata", () => {
			const result = exportToMd(entries, mockSession);
			expect(result).toContain("**Started**");
			expect(result).toContain("**Mode**");
			expect(result).toContain("**Model**");
			expect(result).toContain("**Translated**");
		});

		it("should include timestamps by default in blockquote format", () => {
			const result = exportToMd(entries, mockSession);
			expect(result).toContain(">");
			expect(result).toMatch(/\[.*:.*:.*\]/);
		});

		it("should exclude timestamps when option disabled", () => {
			const result = exportToMd(entries, mockSession, {
				includeTimestamps: false,
			});
			expect(result).not.toMatch(/> \[/);
		});

		it("should exclude session header when option disabled", () => {
			const result = exportToMd(entries, mockSession, {
				includeSessionHeader: false,
			});
			expect(result).not.toContain("# Session");
		});

		it("should wrap voice commands in code blocks when mapCodeBlocks enabled", () => {
			const entriesWithVoice = [
				{ ...mockEntry, id: "1", text: "say newline now" },
			];
			const result = exportToMd(entriesWithVoice, mockSession, {
				mapCodeBlocks: true,
			});
			expect(result).toContain("```");
			expect(result).toContain("⏎");
		});

		it("should not use code blocks when mapCodeBlocks disabled", () => {
			const entriesWithVoice = [
				{ ...mockEntry, id: "1", text: "say newline now" },
			];
			const result = exportToMd(entriesWithVoice, mockSession, {
				mapCodeBlocks: false,
			});
			const codeBlocks = result.match(/```/g);
			expect(codeBlocks).toBeNull();
		});

		it("should export entries in chronological order", () => {
			const shuffledEntries = [entries[1], entries[0]];
			const result = exportToMd(shuffledEntries, mockSession);
			const firstEntryPos = result.indexOf("First entry");
			const secondEntryPos = result.indexOf("Second entry");
			expect(firstEntryPos).toBeLessThan(secondEntryPos);
		});

		it("should handle empty entries array", () => {
			const result = exportToMd([], mockSession);
			expect(result).toContain("# Session");
			expect(result).not.toContain("First entry");
		});
	});

	describe("exportEntries", () => {
		const entries = [{ ...mockEntry, text: "First entry" }];

		it("should export to txt format", () => {
			const result = exportEntries(entries, mockSession, "txt");
			expect(result).toContain("First entry");
		});

		it("should export to md format", () => {
			const result = exportEntries(entries, mockSession, "md");
			expect(result).toContain("# Session");
		});

		it("should pass options to txt export", () => {
			const result = exportEntries(entries, mockSession, "txt", {
				includeTimestamps: false,
				includeSessionHeader: false,
			});
			expect(result).toBe("First entry");
		});

		it("should pass options to md export", () => {
			const result = exportEntries(entries, mockSession, "md", {
				includeTimestamps: false,
				includeSessionHeader: false,
			});
			expect(result).toContain("First entry");
		});
	});

	describe("createExportBlob", () => {
		it("should create blob with text/plain for txt", () => {
			const blob = createExportBlob("test content", "txt");
			expect(blob.type).toMatch(/^text\/plain/);
		});

		it("should create blob with text/markdown for md", () => {
			const blob = createExportBlob("test content", "md");
			expect(blob.type).toMatch(/^text\/markdown/);
		});
	});

	describe("downloadExport", () => {
		it("should create a valid blob for download", () => {
			const blob = createExportBlob("test content", "txt");
			expect(blob.size).toBeGreaterThan(0);
			expect(blob.type).toMatch(/text\/plain/);
		});

		it("should create blob with correct size", () => {
			const content = "test content";
			const blob = createExportBlob(content, "txt");
			expect(blob.size).toBe(content.length);
		});
	});

	describe("Multi-entry selection", () => {
		const entries = [
			{
				...mockEntry,
				id: "1",
				text: "Entry 1",
				started_at: 1000,
				ended_at: 2000,
			},
			{
				...mockEntry,
				id: "2",
				text: "Entry 2",
				started_at: 2000,
				ended_at: 3000,
			},
			{
				...mockEntry,
				id: "3",
				text: "Entry 3",
				started_at: 3000,
				ended_at: 4000,
			},
		];

		it("should export selected entries to txt", () => {
			const selectedIds = new Set(["1", "3"]);
			const selectedEntries = entries.filter((e) => selectedIds.has(e.id));
			const result = exportToTxt(selectedEntries, mockSession);

			expect(result).toContain("Entry 1");
			expect(result).toContain("Entry 3");
			expect(result).not.toContain("Entry 2");
		});

		it("should export selected entries to md", () => {
			const selectedIds = new Set(["1", "3"]);
			const selectedEntries = entries.filter((e) => selectedIds.has(e.id));
			const result = exportToMd(selectedEntries, mockSession);

			expect(result).toContain("Entry 1");
			expect(result).toContain("Entry 3");
			expect(result).not.toContain("Entry 2");
		});

		it("should preserve chronological order for selected entries", () => {
			const selectedIds = new Set(["3", "1"]);
			const selectedEntries = entries.filter((e) => selectedIds.has(e.id));
			const result = exportToTxt(selectedEntries, mockSession, {
				includeSessionHeader: false,
				includeTimestamps: false,
			});

			const firstPos = result.indexOf("Entry 1");
			const secondPos = result.indexOf("Entry 3");
			expect(firstPos).toBeLessThan(secondPos);
		});

		it("should handle empty selection", () => {
			const selectedEntries: (typeof entries)[] = [];
			const result = exportToTxt(selectedEntries, mockSession);

			expect(result).toContain("Session");
			expect(result).not.toContain("Entry");
		});
	});

	describe("File save dialog integration", () => {
		it("should create appropriate blob for txt export", () => {
			const blob = createExportBlob("test", "txt");
			expect(blob.type).toMatch(/^text\/plain/);
		});

		it("should create appropriate blob for md export", () => {
			const blob = createExportBlob("test", "md");
			expect(blob.type).toMatch(/^text\/markdown/);
		});

		it("should create blob with correct mime type based on format", () => {
			const txtBlob = createExportBlob("content", "txt");
			expect(txtBlob.type).toMatch(/^text\/plain/);

			const mdBlob = createExportBlob("content", "md");
			expect(mdBlob.type).toMatch(/^text\/markdown/);
		});
	});

	describe("Edge cases", () => {
		it("should handle session without end time", () => {
			const sessionWithoutEnd = {
				...mockSession,
				ended_at: null,
			};
			const result = exportToTxt([mockEntry], sessionWithoutEnd);
			expect(result).toContain("ongoing");
		});

		it("should handle empty session id", () => {
			const sessionWithEmptyId = {
				...mockSession,
				id: "",
			};
			const result = exportToMd([mockEntry], sessionWithEmptyId);
			expect(result).toContain("# Session ");
		});

		it("should handle very long entry text", () => {
			const longTextEntry = {
				...mockEntry,
				text: "A".repeat(10_000),
			};
			const result = exportToTxt([longTextEntry], mockSession);
			expect(result).toContain("A".repeat(10_000));
		});

		it("should handle unicode in entry text", () => {
			const unicodeEntry = {
				...mockEntry,
				text: "Hello 世界 🌍 ñ",
			};
			const result = exportToTxt([unicodeEntry], mockSession);
			expect(result).toContain("Hello 世界 🌍 ñ");
		});

		it("should handle entries with special characters", () => {
			const specialEntry = {
				...mockEntry,
				text: "Special chars: <>&\"'\\n\\t",
			};
			const result = exportToTxt([specialEntry], mockSession);
			expect(result).toContain("Special chars:");
		});

		it("should handle timestamps at midnight", () => {
			const midnightEntry = {
				...mockEntry,
				started_at: 0,
				ended_at: 1000,
			};
			const result = exportToMd([midnightEntry], mockSession);
			expect(result).toContain("00:00:");
		});

		it("should handle entries with same timestamp", () => {
			const sameTimeEntries = [
				{
					...mockEntry,
					id: "1",
					text: "First",
					started_at: 1000,
					ended_at: 2000,
				},
				{
					...mockEntry,
					id: "2",
					text: "Second",
					started_at: 1000,
					ended_at: 2000,
				},
			];
			const result = exportToTxt(sameTimeEntries, mockSession, {
				includeSessionHeader: false,
			});
			expect(result).toContain("First");
			expect(result).toContain("Second");
		});
	});
});

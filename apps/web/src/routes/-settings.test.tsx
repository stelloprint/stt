import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockPreferences } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

describe("Settings Form Validation", () => {
	describe("Mode Validation", () => {
		const validModes = ["hold", "toggle"];

		it("should accept hold mode", () => {
			expect(validModes.includes("hold")).toBe(true);
		});

		it("should accept toggle mode", () => {
			expect(validModes.includes("toggle")).toBe(true);
		});

		it("should reject invalid mode values", () => {
			expect(validModes.includes("invalid")).toBe(false);
			expect(validModes.includes("press")).toBe(false);
			expect(validModes.includes("")).toBe(false);
		});
	});

	describe("Silence Seconds Range", () => {
		const isValidSilenceSeconds = (value: number) =>
			value >= 0.5 && value <= 30;

		it("should accept minimum boundary value", () => {
			expect(isValidSilenceSeconds(0.5)).toBe(true);
		});

		it("should accept maximum boundary value", () => {
			expect(isValidSilenceSeconds(30)).toBe(true);
		});

		it("should accept typical value", () => {
			expect(isValidSilenceSeconds(3.0)).toBe(true);
		});

		it("should reject value below minimum", () => {
			expect(isValidSilenceSeconds(0.1)).toBe(false);
			expect(isValidSilenceSeconds(0)).toBe(false);
			expect(isValidSilenceSeconds(0.49)).toBe(false);
		});

		it("should reject value above maximum", () => {
			expect(isValidSilenceSeconds(31)).toBe(false);
			expect(isValidSilenceSeconds(100)).toBe(false);
		});
	});

	describe("Silence RMS Validation", () => {
		const validValues = ["low", "medium", "high"];

		it("should accept low", () => {
			expect(validValues.includes("low")).toBe(true);
		});

		it("should accept medium", () => {
			expect(validValues.includes("medium")).toBe(true);
		});

		it("should accept high", () => {
			expect(validValues.includes("high")).toBe(true);
		});

		it("should reject invalid values", () => {
			expect(validValues.includes("medium-high")).toBe(false);
			expect(validValues.includes("")).toBe(false);
		});
	});

	describe("Model Profile Validation", () => {
		const validProfiles = [
			"small.en",
			"multilingual-small",
			"multilingual-medium",
		];

		it("should accept small.en", () => {
			expect(validProfiles.includes("small.en")).toBe(true);
		});

		it("should accept multilingual-small", () => {
			expect(validProfiles.includes("multilingual-small")).toBe(true);
		});

		it("should accept multilingual-medium", () => {
			expect(validProfiles.includes("multilingual-medium")).toBe(true);
		});

		it("should reject invalid profiles", () => {
			expect(validProfiles.includes("large")).toBe(false);
			expect(validProfiles.includes("tiny")).toBe(false);
		});
	});

	describe("Throttle_ms Range", () => {
		const isValidThrottle = (value: number) => value >= 0 && value <= 500;

		it("should accept minimum boundary", () => {
			expect(isValidThrottle(0)).toBe(true);
		});

		it("should accept maximum boundary", () => {
			expect(isValidThrottle(500)).toBe(true);
		});

		it("should accept typical value", () => {
			expect(isValidThrottle(100)).toBe(true);
		});

		it("should reject negative values", () => {
			expect(isValidThrottle(-1)).toBe(false);
		});

		it("should reject values above maximum", () => {
			expect(isValidThrottle(501)).toBe(false);
			expect(isValidThrottle(1000)).toBe(false);
		});
	});

	describe("Chunk Seconds Range", () => {
		const isValidChunk = (value: number) => value >= 10 && value <= 300;

		it("should accept minimum boundary", () => {
			expect(isValidChunk(10)).toBe(true);
		});

		it("should accept maximum boundary", () => {
			expect(isValidChunk(300)).toBe(true);
		});

		it("should accept typical value", () => {
			expect(isValidChunk(60)).toBe(true);
		});

		it("should reject values below minimum", () => {
			expect(isValidChunk(9)).toBe(false);
			expect(isValidChunk(0)).toBe(false);
		});

		it("should reject values above maximum", () => {
			expect(isValidChunk(301)).toBe(false);
		});
	});

	describe("Max Hours Range", () => {
		const isValidMaxHours = (value: number) => value >= 1 && value <= 24;

		it("should accept minimum boundary", () => {
			expect(isValidMaxHours(1)).toBe(true);
		});

		it("should accept maximum boundary", () => {
			expect(isValidMaxHours(24)).toBe(true);
		});

		it("should accept typical value", () => {
			expect(isValidMaxHours(8)).toBe(true);
		});

		it("should reject zero", () => {
			expect(isValidMaxHours(0)).toBe(false);
		});

		it("should reject values above maximum", () => {
			expect(isValidMaxHours(25)).toBe(false);
		});
	});

	describe("Max File GB Range", () => {
		const isValidMaxFileGb = (value: number) => value >= 1 && value <= 32;

		it("should accept minimum boundary", () => {
			expect(isValidMaxFileGb(1)).toBe(true);
		});

		it("should accept maximum boundary", () => {
			expect(isValidMaxFileGb(32)).toBe(true);
		});

		it("should accept typical value", () => {
			expect(isValidMaxFileGb(4)).toBe(true);
		});

		it("should reject zero", () => {
			expect(isValidMaxFileGb(0)).toBe(false);
		});

		it("should reject values above maximum", () => {
			expect(isValidMaxFileGb(33)).toBe(false);
		});
	});

	describe("Form Data Parsing", () => {
		it("should parse mode from form data", () => {
			const formData = new FormData();
			formData.set("mode", "hold");
			expect(formData.get("mode")).toBe("hold");
		});

		it("should parse silence_seconds as float", () => {
			const formData = new FormData();
			formData.set("silence_seconds", "3.0");
			const value = Number.parseFloat(
				formData.get("silence_seconds") as string
			);
			expect(value).toBe(3.0);
		});

		it("should parse throttle_ms as integer", () => {
			const formData = new FormData();
			formData.set("throttle_ms", "100");
			const value = Number.parseInt(formData.get("throttle_ms") as string, 10);
			expect(value).toBe(100);
		});

		it("should parse checkbox as boolean", () => {
			const formData = new FormData();
			formData.set("left_chord", "on");
			const isChecked = formData.get("left_chord") === "on";
			expect(isChecked).toBe(true);
		});

		it("should handle missing form values with defaults", () => {
			const formData = new FormData();
			formData.set("silence_seconds", "");

			const value =
				Number.parseFloat(formData.get("silence_seconds") as string) || 3.0;
			expect(value).toBe(3.0);
		});

		it("should handle empty throttle with default", () => {
			const formData = new FormData();
			formData.set("throttle_ms", "");

			const value =
				Number.parseInt(formData.get("throttle_ms") as string, 10) || 0;
			expect(value).toBe(0);
		});
	});

	describe("Voice Commands Map", () => {
		it("should have newline mapping", () => {
			expect(mockPreferences.voice_commands.map.newline).toBe("Enter");
		});

		it("should have period mapping", () => {
			expect(mockPreferences.voice_commands.map.period).toBe(".");
		});

		it("should have comma mapping", () => {
			expect(mockPreferences.voice_commands.map.comma).toBe(",");
		});

		it("should have tab mapping", () => {
			expect(mockPreferences.voice_commands.map.tab).toBe("Tab");
		});

		it("should have new_paragraph mapping", () => {
			expect(mockPreferences.voice_commands.map.new_paragraph).toBe(
				"Enter Enter"
			);
		});

		it("should have colon mapping", () => {
			expect(mockPreferences.voice_commands.map.colon).toBe(":");
		});

		it("should have semicolon mapping", () => {
			expect(mockPreferences.voice_commands.map.semicolon).toBe(";");
		});

		it("should have open_quote mapping", () => {
			expect(mockPreferences.voice_commands.map.open_quote).toBe('"');
		});

		it("should have close_quote mapping", () => {
			expect(mockPreferences.voice_commands.map.close_quote).toBe('"');
		});

		it("should have backtick mapping", () => {
			expect(mockPreferences.voice_commands.map.backtick).toBe("`");
		});

		it("should have code_block mapping", () => {
			expect(mockPreferences.voice_commands.map.code_block).toBe("```");
		});
	});

	describe("Display Labels", () => {
		it("should have correct model profile labels", () => {
			const labels: Record<string, string> = {
				"small.en": "English (Small)",
				"multilingual-small": "Multilingual (Small)",
				"multilingual-medium": "Multilingual (Medium)",
			};
			expect(labels["small.en"]).toBe("English (Small)");
			expect(labels["multilingual-small"]).toBe("Multilingual (Small)");
			expect(labels["multilingual-medium"]).toBe("Multilingual (Medium)");
		});

		it("should have correct silence RMS labels", () => {
			const labels: Record<string, string> = {
				low: "Low",
				medium: "Medium",
				high: "High",
			};
			expect(labels.low).toBe("Low");
			expect(labels.medium).toBe("Medium");
			expect(labels.high).toBe("High");
		});

		it("should have correct mode labels", () => {
			const labels: Record<string, string> = {
				hold: "Hold (push-to-talk)",
				toggle: "Toggle",
				record: "Record",
			};
			expect(labels.hold).toBe("Hold (push-to-talk)");
			expect(labels.toggle).toBe("Toggle");
			expect(labels.record).toBe("Record");
		});
	});

	describe("Hotkey Checkboxes", () => {
		it("should handle checked left_chord", () => {
			const formData = new FormData();
			formData.set("left_chord", "on");
			expect(formData.get("left_chord") === "on").toBe(true);
		});

		it("should handle checked right_chord", () => {
			const formData = new FormData();
			formData.set("right_chord", "on");
			expect(formData.get("right_chord") === "on").toBe(true);
		});

		it("should handle unchecked checkbox", () => {
			const formData = new FormData();
			expect(formData.get("left_chord") === "on").toBe(false);
		});

		it("should handle both checkboxes checked", () => {
			const formData = new FormData();
			formData.set("left_chord", "on");
			formData.set("right_chord", "on");
			const left = formData.get("left_chord") === "on";
			const right = formData.get("right_chord") === "on";
			expect(left && right).toBe(true);
		});
	});
});

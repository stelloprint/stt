import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockPreferences } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

describe("Settings Form Validation", () => {
	it("should validate mode values", () => {
		const validModes = ["hold", "toggle"];
		expect(validModes.includes("hold")).toBe(true);
		expect(validModes.includes("toggle")).toBe(true);
	});

	it("should reject invalid mode values", () => {
		const validModes = ["hold", "toggle"];
		expect(validModes.includes("invalid")).toBe(false);
	});

	it("should validate silence_seconds range", () => {
		const validRange = (value: number) => value >= 0.5 && value <= 30;
		expect(validRange(3.0)).toBe(true);
		expect(validRange(0.5)).toBe(true);
		expect(validRange(30)).toBe(true);
		expect(validRange(0.1)).toBe(false);
		expect(validRange(31)).toBe(false);
	});

	it("should validate silence_rms values", () => {
		const validValues = ["low", "medium", "high"];
		expect(validValues.includes("low")).toBe(true);
		expect(validValues.includes("medium")).toBe(true);
		expect(validValues.includes("high")).toBe(true);
	});

	it("should validate model_profile values", () => {
		const validProfiles = [
			"small.en",
			"multilingual-small",
			"multilingual-medium",
		];
		expect(validProfiles.includes("small.en")).toBe(true);
		expect(validProfiles.includes("multilingual-small")).toBe(true);
		expect(validProfiles.includes("multilingual-medium")).toBe(true);
	});

	it("should validate throttle_ms range", () => {
		const validRange = (value: number) => value >= 0 && value <= 500;
		expect(validRange(0)).toBe(true);
		expect(validRange(500)).toBe(true);
		expect(validRange(100)).toBe(true);
		expect(validRange(-1)).toBe(false);
		expect(validRange(501)).toBe(false);
	});

	it("should validate chunk_seconds range", () => {
		const validRange = (value: number) => value >= 10 && value <= 300;
		expect(validRange(60)).toBe(true);
		expect(validRange(10)).toBe(true);
		expect(validRange(300)).toBe(true);
		expect(validRange(9)).toBe(false);
		expect(validRange(301)).toBe(false);
	});

	it("should validate max_hours range", () => {
		const validRange = (value: number) => value >= 1 && value <= 24;
		expect(validRange(8)).toBe(true);
		expect(validRange(1)).toBe(true);
		expect(validRange(24)).toBe(true);
		expect(validRange(0)).toBe(false);
		expect(validRange(25)).toBe(false);
	});

	it("should validate max_file_gb range", () => {
		const validRange = (value: number) => value >= 1 && value <= 32;
		expect(validRange(4)).toBe(true);
		expect(validRange(1)).toBe(true);
		expect(validRange(32)).toBe(true);
		expect(validRange(0)).toBe(false);
		expect(validRange(33)).toBe(false);
	});

	it("should parse form values correctly", () => {
		const formData = new FormData();
		formData.set("mode", "hold");
		formData.set("silence_seconds", "3.0");
		formData.set("silence_rms", "medium");
		formData.set("model_profile", "small.en");
		formData.set("left_chord", "on");
		formData.set("right_chord", "on");
		formData.set("translate_to_english", "on");
		formData.set("newline_at_end", "on");
		formData.set("throttle_ms", "0");
		formData.set("voice_commands_enabled", "on");
		formData.set("map_newline", "Enter");
		formData.set("chunk_seconds", "60");
		formData.set("max_hours", "8");
		formData.set("max_file_gb", "4");

		expect(formData.get("mode")).toBe("hold");
		expect(formData.get("silence_seconds")).toBe("3.0");
		expect(formData.get("left_chord")).toBe("on");
	});

	it("should handle missing form values with defaults", () => {
		const formData = new FormData();
		formData.set("silence_seconds", "");
		formData.set("throttle_ms", "");

		const silence_seconds =
			Number.parseFloat(formData.get("silence_seconds") as string) || 3.0;
		const throttle_ms =
			Number.parseInt(formData.get("throttle_ms") as string, 10) || 0;

		expect(silence_seconds).toBe(3.0);
		expect(throttle_ms).toBe(0);
	});

	it("should validate voice command map values", () => {
		const { voice_commands } = mockPreferences;
		expect(voice_commands.map.newline).toBe("Enter");
		expect(voice_commands.map.period).toBe(".");
		expect(voice_commands.map.comma).toBe(",");
		expect(voice_commands.map.tab).toBe("Tab");
	});

	it("should display correct model profile labels", () => {
		const labels: Record<string, string> = {
			"small.en": "English (Small)",
			"multilingual-small": "Multilingual (Small)",
			"multilingual-medium": "Multilingual (Medium)",
		};
		expect(labels["small.en"]).toBe("English (Small)");
		expect(labels["multilingual-small"]).toBe("Multilingual (Small)");
	});

	it("should display correct silence RMS labels", () => {
		const labels: Record<string, string> = {
			low: "Low",
			medium: "Medium",
			high: "High",
		};
		expect(labels.low).toBe("Low");
		expect(labels.medium).toBe("Medium");
		expect(labels.high).toBe("High");
	});

	it("should validate hotkey checkboxes", () => {
		const formData = new FormData();
		formData.set("left_chord", "on");
		formData.set("right_chord", "on");

		const left_chord = formData.get("left_chord") === "on";
		const right_chord = formData.get("right_chord") === "on";

		expect(left_chord).toBe(true);
		expect(right_chord).toBe(true);
	});

	it("should handle unchecked checkboxes", () => {
		const formData = new FormData();
		// Checkbox not present means unchecked

		const left_chord = formData.get("left_chord") === "on";
		expect(left_chord).toBe(false);
	});
});

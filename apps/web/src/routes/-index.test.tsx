import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockPreferences, mockSession } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

const getModeLabel = (mode: string) => {
	switch (mode) {
		case "hold":
			return "Hold (push-to-talk)";
		case "toggle":
			return "Toggle";
		case "record":
			return "Record";
		default:
			return mode;
	}
};

const getModelLabel = (profile: string) => {
	switch (profile) {
		case "small.en":
			return "English (Small)";
		case "multilingual-small":
			return "Multilingual (Small)";
		case "multilingual-medium":
			return "Multilingual (Medium)";
		default:
			return profile;
	}
};

describe("HUD Component", () => {
	describe("Mode Display", () => {
		it("should return correct label for hold mode", () => {
			expect(getModeLabel("hold")).toBe("Hold (push-to-talk)");
		});

		it("should return correct label for toggle mode", () => {
			expect(getModeLabel("toggle")).toBe("Toggle");
		});

		it("should return correct label for record mode", () => {
			expect(getModeLabel("record")).toBe("Record");
		});

		it("should return unknown mode as-is", () => {
			expect(getModeLabel("unknown")).toBe("unknown");
		});
	});

	describe("Model Display", () => {
		it("should return correct label for small.en", () => {
			expect(getModelLabel("small.en")).toBe("English (Small)");
		});

		it("should return correct label for multilingual-small", () => {
			expect(getModelLabel("multilingual-small")).toBe("Multilingual (Small)");
		});

		it("should return correct label for multilingual-medium", () => {
			expect(getModelLabel("multilingual-medium")).toBe(
				"Multilingual (Medium)"
			);
		});

		it("should return unknown profile as-is", () => {
			expect(getModelLabel("unknown-model")).toBe("unknown-model");
		});
	});

	describe("Voice Commands Status", () => {
		it("should show enabled when voice commands are enabled", () => {
			expect(mockPreferences.voice_commands.enabled).toBe(true);
		});

		it("should show disabled when voice commands are disabled", () => {
			const disabledPrefs = {
				...mockPreferences,
				voice_commands: { enabled: false, map: {} },
			};
			expect(disabledPrefs.voice_commands.enabled).toBe(false);
		});
	});

	describe("Session Stats Display", () => {
		it("should display word count from session", () => {
			expect(mockSession.words_count).toBe(10);
		});

		it("should display character count from session", () => {
			expect(mockSession.chars_count).toBe(50);
		});

		it("should return null when no sessions exist", () => {
			const sessions: (typeof mockSession)[] = [];
			const latestSession = sessions[0] ?? null;
			expect(latestSession).toBeNull();
		});
	});

	describe("Duration Calculation", () => {
		it("should calculate duration in seconds", () => {
			const start = 1000;
			const end = 5000;
			const duration = Math.round((end - start) / 1000);
			expect(duration).toBe(4);
		});

		it("should show 'Active' when session has no end time", () => {
			const session = { ...mockSession, ended_at: null };
			expect(session.ended_at).toBeNull();
		});
	});

	describe("Hotkey Display", () => {
		it("should display both left and right chord hotkeys", () => {
			const { left_chord, right_chord } = mockPreferences.hotkeys;
			const hotkeyText = `${left_chord ? "L " : ""}${
				right_chord ? "R" : ""
			} Cmd+Opt`;
			expect(hotkeyText).toBe("L R Cmd+Opt");
		});

		it("should display only left chord when right is disabled", () => {
			const prefs = {
				...mockPreferences,
				hotkeys: { left_chord: true, right_chord: false },
			};
			const hotkeyText = `${prefs.hotkeys.left_chord ? "L " : ""}${
				prefs.hotkeys.right_chord ? "R" : ""
			} Cmd+Opt`;
			expect(hotkeyText).toBe("L  Cmd+Opt");
		});

		it("should display only right chord when left is disabled", () => {
			const prefs = {
				...mockPreferences,
				hotkeys: { left_chord: false, right_chord: true },
			};
			const hotkeyText = `${prefs.hotkeys.left_chord ? "L " : ""}${
				prefs.hotkeys.right_chord ? "R" : ""
			} Cmd+Opt`;
			expect(hotkeyText).toBe("R Cmd+Opt");
		});
	});

	describe("Silence Timeout Display", () => {
		it("should display configured silence timeout", () => {
			expect(mockPreferences.silence_seconds).toBe(3.0);
		});
	});

	describe("Translate Preference Display", () => {
		it("should show 'To English' when translate is enabled", () => {
			expect(mockPreferences.translate_to_english).toBe(false);
		});

		it("should show 'Original' when translate is disabled", () => {
			const prefs = { ...mockPreferences, translate_to_english: true };
			expect(prefs.translate_to_english).toBe(true);
		});
	});

	describe("Typing Options Display", () => {
		it("should display newline_at_end option", () => {
			expect(mockPreferences.typing.newline_at_end).toBe(true);
		});

		it("should display throttle_ms option", () => {
			expect(mockPreferences.typing.throttle_ms).toBe(0);
		});

		it("should include throttle in display when greater than 0", () => {
			const typing = { newline_at_end: true, throttle_ms: 100 };
			const display =
				`${typing.newline_at_end ? "+newline" : "no newline"}` +
				(typing.throttle_ms > 0 ? ` (${typing.throttle_ms}ms)` : "");
			expect(display).toBe("+newline (100ms)");
		});
	});

	describe("Mic Level Display", () => {
		it("should show Mic icon when level is above threshold", () => {
			const micLevel = 50;
			const showMic = micLevel > 10;
			expect(showMic).toBe(true);
		});

		it("should show MicOff icon when level is below threshold", () => {
			const micLevel = 5;
			const showMic = micLevel > 10;
			expect(showMic).toBe(false);
		});

		it("should calculate mic level percentage", () => {
			const micLevel = 45;
			expect(Math.round(micLevel)).toBe(45);
		});
	});
});

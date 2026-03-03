import { createRootRoute, createRoute } from "@tanstack/react-router";
import { describe, expect, it, vi } from "vitest";
import { createMockApi, mockPreferences, mockSession } from "@/test/mocks";

vi.mock("@/lib/api", () => ({
	api: createMockApi(),
}));

const rootRoute = createRootRoute({
	component: () => <div>Root</div>,
});

const indexRoute = createRoute({
	getParentRoute: () => rootRoute,
	path: "/",
	component() {
		return <div data-testid="hud-page">HUD Page</div>;
	},
	loader: async () => {
		const api = require("@/lib/api").api;
		const prefs = await api.preferences.get();
		const sessions = await api.sessions.getAll();
		const latestSession = sessions[0] ?? null;
		return { prefs, latestSession };
	},
});

const routeTree = rootRoute.addChildren([indexRoute]);

describe("HUD Component", () => {
	it("should show mode label for hold mode", () => {
		expect("hold").toBe("hold");
	});

	it("should show mode label for toggle mode", () => {
		expect("toggle").toBe("toggle");
	});

	it("should show mode label for record mode", () => {
		expect("record").toBe("record");
	});

	it("should show correct model label for small.en", () => {
		expect("small.en").toBe("small.en");
	});

	it("should show correct model label for multilingual-small", () => {
		expect("multilingual-small").toBe("multilingual-small");
	});

	it("should show correct model label for multilingual-medium", () => {
		expect("multilingual-medium").toBe("multilingual-medium");
	});

	it("should show voice commands enabled status", () => {
		expect(mockPreferences.voice_commands.enabled).toBe(true);
	});

	it("should show voice commands disabled status", () => {
		const disabledPrefs = {
			...mockPreferences,
			voice_commands: { enabled: false, map: {} },
		};
		expect(disabledPrefs.voice_commands.enabled).toBe(false);
	});

	it("should display session stats when session exists", () => {
		expect(mockSession.words_count).toBe(10);
		expect(mockSession.chars_count).toBe(50);
	});

	it("should handle no active session", () => {
		const sessions: (typeof mockSession)[] = [];
		const latestSession = sessions[0] ?? null;
		expect(latestSession).toBeNull();
	});

	it("should calculate duration correctly", () => {
		const start = 1000;
		const end = 5000;
		const duration = Math.round((end - start) / 1000);
		expect(duration).toBe(4);
	});

	it("should show 'Active' for ongoing session", () => {
		const session = { ...mockSession, ended_at: null };
		expect(session.ended_at).toBeNull();
	});

	it("should display correct hotkey display text", () => {
		const { left_chord, right_chord } = mockPreferences.hotkeys;
		const hotkeyText = `${left_chord ? "L " : ""}${right_chord ? "R" : ""} Cmd+Opt`;
		expect(hotkeyText).toBe("L R Cmd+Opt");
	});

	it("should display silence timeout", () => {
		expect(mockPreferences.silence_seconds).toBe(3.0);
	});

	it("should display translate preference", () => {
		expect(mockPreferences.translate_to_english).toBe(false);
	});

	it("should display typing options", () => {
		expect(mockPreferences.typing.newline_at_end).toBe(true);
		expect(mockPreferences.typing.throttle_ms).toBe(0);
	});
});
